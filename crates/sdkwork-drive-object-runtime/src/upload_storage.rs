//! `DriveUploadMultipartStorage` adapter over the shared object-store runtime.
//!
//! The workspace service's upload-session state machine must not depend on
//! provider credentials or the S3 SDK. This adapter is the in-process half: it
//! resolves each provider through [`DriveObjectStoreRuntime`] - including the
//! account-center credential path and the adapter cache - enforces that the
//! provider is `active` and still addresses the recorded bucket, and maps
//! provider errors onto the service error kinds.
//!
//! A sibling module that finalises Drive uploads in-process builds one of these
//! over the host pool and passes it to
//! `sdkwork_drive_workspace_service::application::upload_session_service::DriveUploadSessionService`:
//!
//! ```ignore
//! let service = DriveUploadSessionService::new(
//!     pool.clone(),
//!     DriveObjectRuntimeUploadStorage::new(pool.clone()),
//! );
//! ```

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use sdkwork_drive_storage_contract::{
    AbortMultipartUploadRequest, CompleteMultipartUploadRequest, CompletedMultipartPart,
    CreateMultipartUploadRequest, DeleteObjectRequest, DriveObjectLocator, DriveObjectStore,
};
use sdkwork_drive_workspace_service::ports::upload_multipart_storage::{
    map_object_store_error, AbortDriveMultipartUploadCommand, CompleteDriveMultipartUploadCommand,
    CreateDriveMultipartUploadCommand, DeleteDriveStoredObjectCommand, DriveUploadMultipartStorage,
};
use sdkwork_drive_workspace_service::DriveServiceError;

use crate::provider_store::ProviderAccessIntent;
use crate::runtime::DriveObjectStoreRuntime;

/// Multipart-storage adapter backed by the shared provider runtime.
///
/// Every call resolves the provider with write intent, so an inactive or deleted
/// provider fails closed instead of writing bytes whose credentials no longer
/// exist.
#[derive(Clone, Debug)]
pub struct DriveObjectRuntimeUploadStorage {
    runtime: DriveObjectStoreRuntime,
}

impl DriveObjectRuntimeUploadStorage {
    /// Build the adapter over the host pool.
    pub fn new(pool: PgPool) -> Self {
        Self {
            runtime: DriveObjectStoreRuntime::new(pool),
        }
    }

    /// Build the adapter over an existing runtime (shares its adapter cache).
    pub fn with_runtime(runtime: DriveObjectStoreRuntime) -> Self {
        Self { runtime }
    }

    /// Resolve the store for one write, requiring the live provider to still
    /// address `bucket`.
    ///
    /// # Errors
    ///
    /// `NotFound` when the provider or its credential source is gone,
    /// `Conflict` when the provider is not active or no longer addresses the
    /// bucket, `PermissionDenied` when the credential is rejected, and
    /// `Internal` for transport/build failures.
    async fn store_for(
        &self,
        storage_provider_id: &str,
        bucket: &str,
    ) -> Result<Arc<dyn DriveObjectStore>, DriveServiceError> {
        let provider = self
            .runtime
            .factory()
            .load_provider_for(storage_provider_id, ProviderAccessIntent::Write)
            .await
            .map_err(map_object_store_error)?;
        self.runtime
            .factory()
            .resolve_for_locator(&provider, bucket, ProviderAccessIntent::Write)
            .await
            .map_err(map_object_store_error)
    }
}

#[async_trait]
impl DriveUploadMultipartStorage for DriveObjectRuntimeUploadStorage {
    async fn create_multipart_upload(
        &self,
        command: CreateDriveMultipartUploadCommand,
    ) -> Result<String, DriveServiceError> {
        let store = self
            .store_for(&command.storage_provider_id, &command.bucket)
            .await?;
        let created = store
            .create_multipart_upload(CreateMultipartUploadRequest {
                locator: DriveObjectLocator {
                    bucket: command.bucket,
                    object_key: command.object_key,
                },
                content_type: None,
                metadata: std::collections::BTreeMap::new(),
                checksum_sha256_hex: None,
            })
            .await
            .map_err(map_object_store_error)?;
        Ok(created.upload_id)
    }

    async fn complete_multipart_upload(
        &self,
        command: CompleteDriveMultipartUploadCommand,
    ) -> Result<(), DriveServiceError> {
        let store = self
            .store_for(&command.storage_provider_id, &command.bucket)
            .await?;
        store
            .complete_multipart_upload(CompleteMultipartUploadRequest {
                locator: DriveObjectLocator {
                    bucket: command.bucket,
                    object_key: command.object_key,
                },
                upload_id: command.storage_upload_id,
                parts: command
                    .parts
                    .into_iter()
                    .map(|part| CompletedMultipartPart {
                        part_number: part.part_no,
                        etag: part.etag,
                    })
                    .collect(),
            })
            .await
            .map(|_| ())
            .map_err(map_object_store_error)
    }

    async fn abort_multipart_upload(
        &self,
        command: AbortDriveMultipartUploadCommand,
    ) -> Result<(), DriveServiceError> {
        let store = self
            .store_for(&command.storage_provider_id, &command.bucket)
            .await?;
        store
            .abort_multipart_upload(AbortMultipartUploadRequest {
                locator: DriveObjectLocator {
                    bucket: command.bucket,
                    object_key: command.object_key,
                },
                upload_id: command.storage_upload_id,
            })
            .await
            .map_err(map_object_store_error)
    }

    async fn delete_object(
        &self,
        command: DeleteDriveStoredObjectCommand,
    ) -> Result<bool, DriveServiceError> {
        let store = self
            .store_for(&command.storage_provider_id, &command.bucket)
            .await?;
        store
            .delete_object(DeleteObjectRequest {
                locator: DriveObjectLocator {
                    bucket: command.bucket,
                    object_key: command.object_key,
                },
            })
            .await
            .map_err(map_object_store_error)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_is_send_and_sync_so_the_service_stays_spawnable() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DriveObjectRuntimeUploadStorage>();
    }

    #[tokio::test]
    async fn missing_provider_fails_closed_before_touching_storage() {
        // A lazy pool never connects: the provider lookup itself is what fails,
        // which is exactly the contract (no bucket-only fallback, no write).
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(std::time::Duration::from_millis(50))
            .connect_lazy("postgres://nobody:nobody@127.0.0.1:1/absent")
            .expect("lazy pool");
        let adapter = DriveObjectRuntimeUploadStorage::new(pool);
        let result = adapter
            .create_multipart_upload(CreateDriveMultipartUploadCommand {
                storage_provider_id: "absent-provider".to_string(),
                bucket: "bucket-1".to_string(),
                object_key: "objects/a".to_string(),
            })
            .await;
        assert!(
            matches!(result, Err(DriveServiceError::Internal(_))),
            "an unreachable provider lookup must surface as Internal, got {result:?}"
        );
    }
}
