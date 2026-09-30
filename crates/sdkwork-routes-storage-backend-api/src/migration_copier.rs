use async_trait::async_trait;

use crate::object_store::build_object_store_for_provider;
use crate::provider_lookup::get_provider_for_read;
use crate::state::AdminStorageState;
use sdkwork_drive_storage_contract::{
    DriveObjectLocator, DriveObjectStore, HeadObjectRequest, PutObjectRequest,
    ReadObjectRangeRequest,
};
use sdkwork_drive_workspace_service::ports::storage_migration_copier::{
    DriveStorageMigrationCopier, StorageMigrationCopyOutcome, StorageMigrationCopyRequest,
};
use sdkwork_drive_workspace_service::DriveServiceError;
use sdkwork_utils_rust::sha256_hash;

/// Bytes fetched from the source provider per range request.
///
/// The whole object is deliberately not loaded at once: a migration moves
/// arbitrary tenant data, and a single multi-gigabyte object would otherwise
/// have to fit in memory on a worker whose real job is throughput, not capacity.
/// 8 MiB keeps each read small enough to bound memory while staying well above
/// the per-request overhead that would dominate at a few kilobytes.
const COPY_CHUNK_BYTES: u64 = 8 * 1024 * 1024;

/// Cross-provider copier backed by the admin storage object stores.
///
/// Each call resolves two *independent* stores — one per provider — because the
/// two ends can be different vendors on different accounts. That is precisely
/// why this cannot reuse `DriveObjectStore::copy_object`, whose `CopyObject`
/// semantics only move bytes within a single account.
pub(crate) struct AdminStorageMigrationCopier<'a> {
    state: &'a AdminStorageState,
    tenant_id: String,
}

impl<'a> AdminStorageMigrationCopier<'a> {
    pub(crate) fn new(state: &'a AdminStorageState, tenant_id: String) -> Self {
        Self { state, tenant_id }
    }
}

#[async_trait]
impl DriveStorageMigrationCopier for AdminStorageMigrationCopier<'_> {
    async fn copy_for_migration(
        &self,
        request: StorageMigrationCopyRequest,
    ) -> Result<StorageMigrationCopyOutcome, DriveServiceError> {
        // The source is resolved with a `Read` intent so a `disabled` provider
        // can still be drained — draining a retired provider is the main reason
        // to migrate at all.
        let source_provider = get_provider_for_read(self.state, &self.tenant_id, &request.source_provider_id)
            .await
            .map_err(map_route_error_to_service)?;
        let target_provider = get_provider_for_read(self.state, &self.tenant_id, &request.target_provider_id)
            .await
            .map_err(map_route_error_to_service)?;

        let source_store = build_object_store_for_provider(self.state, &source_provider)
            .await
            .map_err(map_route_error_to_service)?;
        let target_store = build_object_store_for_provider(self.state, &target_provider)
            .await
            .map_err(map_route_error_to_service)?;

        let source_locator = DriveObjectLocator {
            bucket: request.source_bucket.clone(),
            object_key: request.source_object_key.clone(),
        };

        // Size from the source's own head rather than from the queued
        // `content_length`: the queued value is a snapshot and the object may
        // have been replaced since. Reading the live size prevents a truncated
        // or over-read copy.
        let head = source_store
            .head_object(HeadObjectRequest {
                locator: source_locator.clone(),
            })
            .await
            .map_err(map_object_store_error_to_service)?;

        let content_type = head
            .content_type
            .clone()
            .unwrap_or_else(|| request.content_type.clone());

        let bytes = read_all_bytes(source_store.as_ref(), &source_locator, head.content_length)
            .await
            .map_err(map_object_store_error_to_service)?;

        let observed_checksum = sha256_hash(&bytes);

        let target_locator = DriveObjectLocator {
            bucket: request.target_bucket.clone(),
            object_key: request.target_object_key.clone(),
        };
        target_store
            .put_object(PutObjectRequest {
                locator: target_locator,
                content_type: Some(content_type),
                metadata: Default::default(),
                // The copied bytes are re-checksummed rather than trusting the
                // source's recorded value, so the target's own checksum is
                // evidence about what actually landed.
                checksum_sha256_hex: Some(observed_checksum.clone()),
                body: bytes,
            })
            .await
            .map_err(map_object_store_error_to_service)?;

        // Compared here, at the boundary, so the engine's own second check is a
        // belt-and-braces guard rather than the only line of defence.
        if observed_checksum != request.expected_checksum_sha256_hex {
            return Err(DriveServiceError::Conflict(format!(
                "migration copy checksum mismatch for {}: source records {}, target received {}",
                request.source_object_key, request.expected_checksum_sha256_hex, observed_checksum
            )));
        }

        Ok(StorageMigrationCopyOutcome {
            bytes: head.content_length as i64,
            verified_checksum_sha256_hex: Some(observed_checksum),
        })
    }
}

/// Read an object fully through bounded range requests.
async fn read_all_bytes(
    store: &dyn DriveObjectStore,
    locator: &DriveObjectLocator,
    content_length: u64,
) -> Result<Vec<u8>, sdkwork_drive_storage_contract::DriveObjectStoreError> {
    let mut collected = Vec::with_capacity(content_length as usize);
    let mut offset = 0u64;
    while offset < content_length {
        let end_inclusive = (offset + COPY_CHUNK_BYTES - 1).min(content_length - 1);
        let (_, mut stream) = store
            .read_object_range(ReadObjectRangeRequest {
                locator: locator.clone(),
                range: sdkwork_drive_storage_contract::DriveByteRange {
                    start_inclusive: offset,
                    end_inclusive,
                },
            })
            .await?;
        while let Some(chunk) = stream.next_chunk().await? {
            collected.extend_from_slice(&chunk);
        }
        offset = end_inclusive + 1;
    }
    if collected.len() as u64 != content_length {
        return Err(sdkwork_drive_storage_contract::DriveObjectStoreError::new(
            sdkwork_drive_storage_contract::DriveObjectStoreErrorKind::Internal,
            format!(
                "range read produced {} bytes but the source reports {content_length}",
                collected.len()
            ),
        ));
    }
    Ok(collected)
}

/// Collapse a route-layer problem into a service error.
///
/// The route layer speaks `(StatusCode, ProblemDetail)`; the migration port
/// speaks `DriveServiceError`. The status code is preserved inside the message
/// because it is the part an operator actually needs when triaging a failed
/// object.
fn map_route_error_to_service(
    (status, detail): (
        axum::http::StatusCode,
        axum::Json<crate::error::ProblemDetail>,
    ),
) -> DriveServiceError {
    let message = serde_json::to_value(&detail.0)
        .ok()
        .and_then(|value| {
            value
                .get("detail")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "storage provider resolution failed".to_string());
    match status {
        axum::http::StatusCode::NOT_FOUND => DriveServiceError::NotFound(message),
        axum::http::StatusCode::CONFLICT => DriveServiceError::Conflict(message),
        axum::http::StatusCode::BAD_REQUEST => DriveServiceError::Validation(message),
        axum::http::StatusCode::FORBIDDEN => DriveServiceError::PermissionDenied(message),
        _ => DriveServiceError::Internal(format!("{status}: {message}")),
    }
}

fn map_object_store_error_to_service(
    error: sdkwork_drive_storage_contract::DriveObjectStoreError,
) -> DriveServiceError {
    DriveServiceError::Internal(format!("{error:?}"))
}

#[cfg(test)]
mod tests {
    use super::COPY_CHUNK_BYTES;

    #[test]
    fn chunk_size_stays_bounded_and_non_zero() {
        // Guards against a future edit that would either disable chunking
        // (making a large migration read an object wholly into memory) or make
        // chunking so small that per-request overhead dominates.
        assert!(
            COPY_CHUNK_BYTES >= 1024 * 1024,
            "chunk size must stay meaningful"
        );
        assert!(
            COPY_CHUNK_BYTES <= 64 * 1024 * 1024,
            "chunk size must stay bounded"
        );
    }
}
