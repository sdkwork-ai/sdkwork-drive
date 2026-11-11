use async_trait::async_trait;

use sdkwork_drive_storage_contract::{DriveObjectStoreError, DriveObjectStoreErrorKind};

use crate::DriveServiceError;

/// Map a storage-contract error onto the service error kinds every caller
/// already understands.
///
/// The upload paths must not turn a "provider is gone" into a 500 or a transport
/// failure into a 400, so the kind mapping is shared by the HTTP surface and by
/// any in-process adapter instead of being re-derived per call site.
pub fn map_object_store_error(error: DriveObjectStoreError) -> DriveServiceError {
    match error.kind {
        DriveObjectStoreErrorKind::NotFound => DriveServiceError::NotFound(error.message),
        DriveObjectStoreErrorKind::InvalidRequest => DriveServiceError::Validation(error.message),
        DriveObjectStoreErrorKind::Conflict => DriveServiceError::Conflict(error.message),
        DriveObjectStoreErrorKind::PermissionDenied => {
            DriveServiceError::PermissionDenied(error.message)
        }
        _ => DriveServiceError::Internal(error.message),
    }
}

/// One part the client already uploaded, as the storage vendor numbered it.
///
/// `part_no` is the 1-based multipart part number the client presigned; `etag`
/// is the vendor-assigned entity tag the client reported back. The service
/// validates both before they reach a provider, so a provider implementation
/// may assume they are non-empty and sorted ascending with unique numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveMultipartPart {
    pub part_no: u16,
    pub etag: String,
}

/// Where a multipart upload writes, plus which provider serves it.
///
/// The provider id is carried separately from the bucket because buckets are a
/// storage-level namespace: two tenants may legitimately use the same bucket on
/// different providers, so a locator without a provider is not an identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveMultipartUploadTarget {
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
}

/// Command that opens a multipart upload for one object locator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateDriveMultipartUploadCommand {
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
}

/// Command that finalises a multipart upload into one object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteDriveMultipartUploadCommand {
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
    pub storage_upload_id: String,
    pub parts: Vec<DriveMultipartPart>,
}

/// Command that cancels a multipart upload before it produced an object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbortDriveMultipartUploadCommand {
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
    pub storage_upload_id: String,
}

/// Command that removes an object that was written but never committed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteDriveStoredObjectCommand {
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
}

/// Storage-side operations the upload-session lifecycle needs.
///
/// The session state machine, part accounting, idempotency and node-version
/// activation belong to
/// [`crate::application::upload_session_service::DriveUploadSessionService`];
/// this port only moves bytes at the provider. Implementations live in the
/// surface crate that owns the shared object-store runtime, so the workspace
/// service never depends on provider credentials or the S3 SDK.
///
/// Every method is tenant-agnostic on purpose: the caller has already resolved
/// the target through the tenant-scoped provider bindings, and the target it
/// passes is the exact locator recorded on the upload session. Implementations
/// MUST NOT widen the tenant boundary by resolving a provider from the bucket
/// alone.
#[async_trait]
pub trait DriveUploadMultipartStorage: Send + Sync {
    /// Open a multipart upload and return the provider's upload id.
    ///
    /// # Errors
    ///
    /// Returns a provider-mapped error: `NotFound` when the provider or bucket
    /// no longer exists, `Conflict` when the provider is not active, and
    /// `Internal` for transport or vendor failures. The caller compensates by
    /// aborting whatever upload id was returned before the call failed; a
    /// failed call that produced no upload id leaves no storage state behind.
    async fn create_multipart_upload(
        &self,
        command: CreateDriveMultipartUploadCommand,
    ) -> Result<String, DriveServiceError>;

    /// Finalise the multipart upload into a single object.
    ///
    /// # Errors
    ///
    /// Returns a provider-mapped error when the vendor rejects the part list or
    /// the upload id (typically `Conflict`/`Validation`) or when the transport
    /// fails (`Internal`). The caller releases its completion claim so the
    /// client can retry.
    async fn complete_multipart_upload(
        &self,
        command: CompleteDriveMultipartUploadCommand,
    ) -> Result<(), DriveServiceError>;

    /// Abort a multipart upload, discarding its uploaded parts.
    ///
    /// # Errors
    ///
    /// Returns a provider-mapped error when the abort cannot be performed
    /// (missing provider, inactive provider, vendor rejection, transport
    /// failure). Cancelling a session surfaces that error instead of pretending
    /// the parts are gone.
    async fn abort_multipart_upload(
        &self,
        command: AbortDriveMultipartUploadCommand,
    ) -> Result<(), DriveServiceError>;

    /// Delete one stored object.
    ///
    /// # Returns
    ///
    /// `Ok(true)` when the object was deleted, `Ok(false)` when the provider
    /// kind has no object-delete capability (the object is then left for the
    /// maintenance sweeper). This is used only for best-effort compensation
    /// after a database commit failed, so a `false` or `Err` result is logged
    /// and never fails the business operation.
    ///
    /// # Errors
    ///
    /// Returns a provider-mapped error; callers on the compensation path log it
    /// and continue.
    async fn delete_object(
        &self,
        command: DeleteDriveStoredObjectCommand,
    ) -> Result<bool, DriveServiceError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    /// Minimal in-memory adapter proving the trait is implementable outside the
    /// surface crate and that commands round-trip without provider types.
    #[derive(Debug, Default)]
    struct RecordingStorage {
        created: Mutex<Vec<String>>,
        completed: Mutex<BTreeMap<String, Vec<u16>>>,
        aborted: Mutex<Vec<String>>,
        deleted: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl DriveUploadMultipartStorage for RecordingStorage {
        async fn create_multipart_upload(
            &self,
            command: CreateDriveMultipartUploadCommand,
        ) -> Result<String, DriveServiceError> {
            self.created
                .lock()
                .map_err(|_| DriveServiceError::Internal("poisoned".to_string()))?
                .push(command.object_key.clone());
            Ok(format!("upload:{}", command.object_key))
        }

        async fn complete_multipart_upload(
            &self,
            command: CompleteDriveMultipartUploadCommand,
        ) -> Result<(), DriveServiceError> {
            self.completed
                .lock()
                .map_err(|_| DriveServiceError::Internal("poisoned".to_string()))?
                .insert(
                    command.storage_upload_id,
                    command.parts.iter().map(|part| part.part_no).collect(),
                );
            Ok(())
        }

        async fn abort_multipart_upload(
            &self,
            command: AbortDriveMultipartUploadCommand,
        ) -> Result<(), DriveServiceError> {
            self.aborted
                .lock()
                .map_err(|_| DriveServiceError::Internal("poisoned".to_string()))?
                .push(command.storage_upload_id);
            Ok(())
        }

        async fn delete_object(
            &self,
            command: DeleteDriveStoredObjectCommand,
        ) -> Result<bool, DriveServiceError> {
            self.deleted
                .lock()
                .map_err(|_| DriveServiceError::Internal("poisoned".to_string()))?
                .push(command.object_key);
            Ok(true)
        }
    }

    #[tokio::test]
    async fn recording_storage_reports_part_numbers_in_upload_order() {
        let storage = RecordingStorage::default();
        let upload_id = storage
            .create_multipart_upload(CreateDriveMultipartUploadCommand {
                storage_provider_id: "provider-1".to_string(),
                bucket: "bucket-1".to_string(),
                object_key: "objects/a".to_string(),
            })
            .await
            .expect("create multipart upload");
        assert_eq!(upload_id, "upload:objects/a");

        storage
            .complete_multipart_upload(CompleteDriveMultipartUploadCommand {
                storage_provider_id: "provider-1".to_string(),
                bucket: "bucket-1".to_string(),
                object_key: "objects/a".to_string(),
                storage_upload_id: upload_id.clone(),
                parts: vec![
                    DriveMultipartPart {
                        part_no: 1,
                        etag: "etag-1".to_string(),
                    },
                    DriveMultipartPart {
                        part_no: 2,
                        etag: "etag-2".to_string(),
                    },
                ],
            })
            .await
            .expect("complete multipart upload");

        assert_eq!(
            storage
                .completed
                .lock()
                .expect("completed lock")
                .get(&upload_id),
            Some(&vec![1, 2])
        );
    }
}
