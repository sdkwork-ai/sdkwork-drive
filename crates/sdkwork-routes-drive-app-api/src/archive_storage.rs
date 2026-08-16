use crate::archive::{is_supported_archive_content, validate_archive_source_node};
use crate::constants::ARCHIVE_MAX_COMPRESSED_BYTES;
use crate::dto::ActiveStorageObjectRef;
use crate::error::{
    internal_sql_error, map_object_store_route_error, map_service_error, not_found_problem,
    problem, ProblemDetail, SdkWorkResultCode,
};
use crate::node_repository::find_node;
use crate::object_store::{
    build_s3_object_store_for_provider, find_storage_provider_by_id,
    missing_signing_provider_error, require_active_storage_provider,
    unsupported_signing_provider_error,
};
use crate::state::AppState;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_storage_contract::{
    DriveByteRange, DriveObjectLocator, DriveObjectStore, ReadObjectRangeRequest,
};
use sdkwork_drive_storage_s3::S3DriveObjectStore;
use sqlx::PgPool;
use sqlx::Row;

pub(crate) async fn read_archive_node_bytes(
    state: &AppState,
    tenant_id: &str,
    node_id: &str,
) -> Result<Vec<u8>, (StatusCode, Json<ProblemDetail>)> {
    let node = find_node(&state.pool, tenant_id, node_id).await?;
    validate_archive_source_node(&node)?;
    let object_ref = find_active_storage_object_ref(&state.pool, tenant_id, node_id).await?;
    if !is_supported_archive_content(&node.node_name, &object_ref.content_type) {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "archiveEntries can only be used with ZIP archive files",
            SdkWorkResultCode::ValidationError,
        ));
    }
    let provider = find_storage_provider_by_id(&state.pool, &object_ref.storage_provider_id)
        .await
        .map_err(map_service_error)?
        .ok_or_else(|| map_service_error(missing_signing_provider_error(&object_ref.bucket)))?;
    let provider =
        require_active_storage_provider(provider, &object_ref.bucket).map_err(map_service_error)?;
    let object_store = build_s3_object_store_for_provider(&provider)
        .await
        .map_err(map_service_error)?
        .ok_or_else(|| map_service_error(unsupported_signing_provider_error(&object_ref.bucket)))?;
    read_full_storage_object(&object_store, &object_ref).await
}

async fn find_active_storage_object_ref(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
) -> Result<ActiveStorageObjectRef, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT storage_provider_id, bucket, object_key, content_type, content_length
         FROM dr_drive_storage_object
         WHERE tenant_id=$1
           AND node_id=$2
           AND lifecycle_status='active'
         ORDER BY version_no DESC
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(node_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error(
        "read active dr_drive_storage_object failed",
    ))?;
    let Some(row) = row else {
        return Err(not_found_problem(
            "storage object for archive node is not found or inactive",
        ));
    };
    Ok(ActiveStorageObjectRef {
        storage_provider_id: row.get("storage_provider_id"),
        bucket: row.get("bucket"),
        object_key: row.get("object_key"),
        content_type: row.get("content_type"),
        content_length: row.get("content_length"),
    })
}

async fn read_full_storage_object(
    object_store: &S3DriveObjectStore,
    object_ref: &ActiveStorageObjectRef,
) -> Result<Vec<u8>, (StatusCode, Json<ProblemDetail>)> {
    if object_ref.content_length < 0 {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "contentLength must not be negative",
            SdkWorkResultCode::ValidationError,
        ));
    }
    if object_ref.content_length == 0 {
        return Ok(Vec::new());
    }
    if object_ref.content_length > ARCHIVE_MAX_COMPRESSED_BYTES {
        return Err(problem(
            StatusCode::PAYLOAD_TOO_LARGE,
            "validation failed",
            format!("archive compressed size must be at most {ARCHIVE_MAX_COMPRESSED_BYTES} bytes"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    let (_response, mut stream) = object_store
        .read_object_range(ReadObjectRangeRequest {
            locator: DriveObjectLocator {
                bucket: object_ref.bucket.clone(),
                object_key: object_ref.object_key.clone(),
            },
            range: DriveByteRange {
                start_inclusive: 0,
                end_inclusive: object_ref.content_length as u64 - 1,
            },
        })
        .await
        .map_err(map_object_store_route_error)?;
    let mut bytes = Vec::with_capacity(object_ref.content_length as usize);
    while let Some(chunk) = stream
        .next_chunk()
        .await
        .map_err(map_object_store_route_error)?
    {
        if bytes.len() + chunk.len() > ARCHIVE_MAX_COMPRESSED_BYTES as usize {
            return Err(problem(
                StatusCode::PAYLOAD_TOO_LARGE,
                "validation failed",
                format!(
                    "archive compressed size must be at most {ARCHIVE_MAX_COMPRESSED_BYTES} bytes"
                ),
                SdkWorkResultCode::ValidationError,
            ));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
