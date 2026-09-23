//! Same-origin Drive node content read for the app-api authority.
//!
//! The download URL and download grant operations hand a client a short-lived
//! cross-origin presigned provider URL. That is the right shape for a browser
//! download, a CDN handoff, or an auditable grant, and it remains the required
//! path for large artifacts. It is the wrong shape when an authenticated
//! first-party client must *inspect* content on its own origin: it forces a
//! second origin into the request and it cannot be replayed without a signing
//! round trip.
//!
//! This module owns the same-origin counterpart, `nodes.content.retrieve`. It
//! reuses exactly the same reader authorization as the download URL operation
//! and then reads the object itself. Storage provider id, bucket, object key,
//! credentials, and presigned URLs never cross this boundary.
//!
//! ## Why the response is enveloped and bounded
//!
//! `API_SPEC.md` section 4.5.1 and `SDK_SPEC.md` section 4.2 make
//! `SdkWorkApiResponse` the canonical wire envelope for SDKWork-owned `app-api`
//! operations, and the SDK generator enforces that under
//! `--standard-profile sdkwork-v3`: a non-JSON success body is rejected on this
//! surface. Only the non-enveloped `custom` SDK surfaces (internal-api,
//! vendor-compatibility open-api) may return raw binary.
//!
//! So this operation returns the envelope, carrying content with an explicit
//! encoding and a platform-bounded payload, exactly like
//! `sandboxFileContents.retrieve`. Content larger than the ceiling is served by
//! the download URL / download grant / download package operations, or read in
//! successive `byteRangeStart` + `byteRangeLength` windows.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use axum::Json;
use sdkwork_drive_storage_contract::{DriveByteRange, DriveObjectStore, ReadObjectRangeRequest};
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::Row;

use crate::acl;
use crate::app_context::DriveRequestContext;
use crate::dto::DriveNodeContentQuery;
use crate::error::{
    map_object_store_route_error, map_service_error, not_found_problem, payload_too_large_problem,
    validation_problem, ProblemDetail,
};
use crate::node_repository::find_active_node;
use crate::object_store::{
    build_s3_object_store_for_provider, find_storage_provider_by_id,
    require_active_storage_provider,
};
use crate::response::success_resource;
use crate::state::AppState;

/// Platform ceiling for one enveloped content payload.
///
/// Matches `DriveSandboxFileContent.content.maxLength` and the generated
/// `DriveNodeContent.content.maxLength` (5592408 = 4 MiB of raw bytes once
/// base64 overhead is accounted for). Reads beyond this must page or use a
/// download operation.
const MAX_CONTENT_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentEncoding {
    Utf8,
    Base64,
}

impl ContentEncoding {
    fn parse(value: Option<&str>) -> Result<Self, (StatusCode, Json<ProblemDetail>)> {
        // Drive content is arbitrary binary, so base64 is the default here even
        // though the sandbox reader defaults to utf8.
        match value.unwrap_or("base64") {
            "utf8" => Ok(Self::Utf8),
            "base64" => Ok(Self::Base64),
            _ => Err(validation_problem("encoding must be utf8 or base64")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Utf8 => "utf8",
            Self::Base64 => "base64",
        }
    }

    fn encode(self, bytes: &[u8]) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
        match self {
            Self::Utf8 => String::from_utf8(bytes.to_vec()).map_err(|_| {
                validation_problem("Drive content is not valid UTF-8; request base64 encoding")
            }),
            Self::Base64 => Ok(sdkwork_utils_rust::base64_encode(bytes)),
        }
    }
}

/// Payload for `nodes.content.retrieve`.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DriveNodeContentResponse {
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    pub encoding: String,
    pub content: String,
    pub size_bytes: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_sha256: Option<String>,
    pub returned_range_start: String,
    pub returned_range_length: String,
    pub has_more: bool,
}

struct ResolvedContent {
    object_store: Box<dyn DriveObjectStore>,
    bucket: String,
    object_key: String,
    content_type: Option<String>,
    content_length: u64,
    checksum_sha256: Option<String>,
    file_name: Option<String>,
}

/// Read the active content of one Drive node on the same origin.
pub(crate) async fn retrieve_node_content(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(node_id): Path<String>,
    Query(query): Query<DriveNodeContentQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    if node.node_type != "file" {
        return Err(not_found_problem(
            "node content is only available for file nodes",
        ));
    }
    // `reader` is the same requirement as `nodes.downloadUrls.retrieve`.
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "reader").await?;

    let encoding = ContentEncoding::parse(query.encoding.as_deref())?;
    let resolved = resolve_content(&state, &tenant_id, &node_id).await?;

    let max_bytes = match query.max_bytes {
        Some(value) => u64::try_from(value)
            .map_err(|_| validation_problem("maxBytes must be a positive integer"))?
            .min(MAX_CONTENT_BYTES),
        None => MAX_CONTENT_BYTES,
    };

    let start = match query.byte_range_start {
        Some(value) => u64::try_from(value)
            .map_err(|_| validation_problem("byteRangeStart must be zero or greater"))?,
        None => 0,
    };
    if start > resolved.content_length {
        return Err(validation_problem(
            "byteRangeStart is beyond the end of the active content",
        ));
    }

    let available = resolved.content_length - start;
    let requested_length = match query.byte_range_length {
        Some(value) => u64::try_from(value)
            .map_err(|_| validation_problem("byteRangeLength must be a positive integer"))?,
        None => available,
    };

    // A read that cannot be satisfied within the bounded ceiling must be paged
    // or moved to a download operation rather than silently truncated.
    if requested_length > max_bytes && available > max_bytes {
        return Err(payload_too_large_problem(format!(
            "requested Drive content window exceeds the {max_bytes} byte bounded read ceiling; \
             page with byteRangeStart and byteRangeLength or use a download operation"
        )));
    }

    let read_length = requested_length.min(max_bytes).min(available);
    let end_inclusive = if read_length == 0 {
        start
    } else {
        start + read_length - 1
    };

    let bytes = if read_length == 0 {
        Vec::new()
    } else {
        read_range(&resolved, start, end_inclusive).await?
    };
    let returned_length = bytes.len() as u64;
    let has_more = start + returned_length < resolved.content_length;
    let encoded = encoding.encode(&bytes)?;
    let checksum_sha256 = resolved
        .checksum_sha256
        .clone()
        .filter(|value| value.len() == 64);

    Ok(success_resource(DriveNodeContentResponse {
        node_id: node_id.clone(),
        file_name: resolved.file_name,
        content_type: resolved.content_type,
        encoding: encoding.as_str().to_string(),
        content: encoded,
        size_bytes: resolved.content_length.to_string(),
        checksum_sha256,
        returned_range_start: start.to_string(),
        returned_range_length: returned_length.to_string(),
        has_more,
    }))
}

async fn resolve_content(
    state: &AppState,
    tenant_id: &str,
    node_id: &str,
) -> Result<ResolvedContent, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT o.storage_provider_id, o.bucket, o.object_key, o.content_type, o.content_length,
                o.checksum_sha256_hex, o.original_file_name
         FROM dr_drive_storage_object o
         INNER JOIN dr_drive_node n
            ON n.tenant_id=o.tenant_id
           AND n.id=o.node_id
           AND n.lifecycle_status='active'
           AND n.node_type='file'
         WHERE o.tenant_id=$1
           AND o.node_id=$2
           AND o.lifecycle_status='active'
         ORDER BY o.version_no DESC
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(node_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "query active dr_drive_storage_object failed: {error}"
        )))
    })?;

    let Some(row) = row else {
        return Err(not_found_problem(
            "active storage object for node was not found",
        ));
    };

    let storage_provider_id: String = row.get("storage_provider_id");
    let bucket: String = row.get("bucket");
    let object_key: String = row.get("object_key");
    let content_type: Option<String> = row.get("content_type");
    let stored_length: i64 = row.get("content_length");
    let checksum_sha256: Option<String> = row.get("checksum_sha256_hex");
    let file_name: Option<String> = row.try_get("original_file_name").ok();

    let content_length = u64::try_from(stored_length).map_err(|_| {
        map_service_error(DriveServiceError::Internal(
            "resolved Drive node content length is invalid".to_string(),
        ))
    })?;

    let provider = find_storage_provider_by_id(&state.pool, &storage_provider_id)
        .await
        .map_err(map_service_error)?
        .ok_or_else(|| {
            map_service_error(DriveServiceError::Conflict(format!(
                "active storage provider is required for bucket {bucket} to read object content"
            )))
        })?;
    let provider = require_active_storage_provider(provider, &bucket).map_err(map_service_error)?;
    let object_store = build_s3_object_store_for_provider(&provider)
        .await
        .map_err(map_service_error)?
        .ok_or_else(|| {
            map_service_error(DriveServiceError::Conflict(format!(
            "active storage provider for bucket {bucket} does not support object store content reads"
        )))
        })?;

    Ok(ResolvedContent {
        object_store: Box::new(object_store),
        bucket,
        object_key,
        content_type: content_type.filter(|value| !value.trim().is_empty()),
        content_length,
        checksum_sha256,
        file_name: file_name.filter(|value| !value.trim().is_empty()),
    })
}

async fn read_range(
    resolved: &ResolvedContent,
    start_inclusive: u64,
    end_inclusive: u64,
) -> Result<Vec<u8>, (StatusCode, Json<ProblemDetail>)> {
    let expected = end_inclusive - start_inclusive + 1;
    let (read, mut chunks) = resolved
        .object_store
        .read_object_range(ReadObjectRangeRequest {
            locator: sdkwork_drive_storage_contract::DriveObjectLocator {
                bucket: resolved.bucket.clone(),
                object_key: resolved.object_key.clone(),
            },
            range: DriveByteRange {
                start_inclusive,
                end_inclusive,
            },
        })
        .await
        .map_err(map_object_store_route_error)?;
    if read.content_length != expected {
        return Err(map_object_store_route_error(
            sdkwork_drive_storage_contract::DriveObjectStoreError::new(
                sdkwork_drive_storage_contract::DriveObjectStoreErrorKind::IntegrityFailed,
                "Drive content range length does not match committed metadata",
            ),
        ));
    }
    let mut bytes = Vec::with_capacity(expected as usize);
    while let Some(chunk) = chunks
        .next_chunk()
        .await
        .map_err(map_object_store_route_error)?
    {
        bytes.extend_from_slice(&chunk);
    }
    if bytes.len() as u64 != expected {
        return Err(map_object_store_route_error(
            sdkwork_drive_storage_contract::DriveObjectStoreError::new(
                sdkwork_drive_storage_contract::DriveObjectStoreErrorKind::IntegrityFailed,
                "Drive content stream length does not match committed range length",
            ),
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoding_parser_defaults_to_base64_and_rejects_unknown_values() {
        assert_eq!(
            ContentEncoding::parse(None).expect("default encoding"),
            ContentEncoding::Base64
        );
        assert_eq!(
            ContentEncoding::parse(Some("utf8")).expect("utf8"),
            ContentEncoding::Utf8
        );
        assert!(ContentEncoding::parse(Some("hex")).is_err());
    }

    #[test]
    fn utf8_encoding_refuses_non_utf8_drive_content() {
        let err = ContentEncoding::Utf8
            .encode(&[0xff, 0xfe, 0x00])
            .expect_err("invalid utf8 must be rejected");
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn base64_encoding_round_trips_arbitrary_bytes() {
        let encoded = ContentEncoding::Base64
            .encode(&[0x00, 0xff, 0x10])
            .expect("base64 encoding");
        assert_eq!(encoded, "AP8Q");
    }
}
