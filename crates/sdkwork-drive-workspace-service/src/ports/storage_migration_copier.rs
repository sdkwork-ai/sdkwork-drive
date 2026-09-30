use async_trait::async_trait;

use crate::DriveServiceError;

/// One object copy the migration engine wants performed.
#[derive(Debug, Clone)]
pub struct StorageMigrationCopyRequest {
    /// Provider whose bytes are read. Resolved as a `Read` intent so a
    /// `disabled` provider can still be drained.
    pub source_provider_id: String,
    pub source_bucket: String,
    pub source_object_key: String,
    /// Provider the bytes are written to. Must be `active`.
    pub target_provider_id: String,
    pub target_bucket: String,
    pub target_object_key: String,
    pub content_type: String,
    /// Checksum recorded on `dr_drive_storage_object`. The implementation must
    /// compare the copied bytes against this and fail the item on mismatch.
    pub expected_checksum_sha256_hex: String,
}

/// Result of a copy attempt.
#[derive(Debug, Clone)]
pub struct StorageMigrationCopyOutcome {
    pub bytes: i64,
    /// SHA-256 observed on the target. Present on success.
    pub verified_checksum_sha256_hex: Option<String>,
}

/// Port for the byte movement of a migration.
///
/// Deliberately separate from `DriveObjectStore`: a migration is *not* a
/// single-store operation. `DriveObjectStore::copy_object` moves bytes within
/// one account, whereas migration reads from one provider's account and writes
/// to another's, so it needs two independently resolved stores and has to
/// verify the result itself. Keeping it a distinct port lets the engine be
/// tested against a fake without standing up two object stores.
#[async_trait]
pub trait DriveStorageMigrationCopier: Send + Sync {
    async fn copy_for_migration(
        &self,
        request: StorageMigrationCopyRequest,
    ) -> Result<StorageMigrationCopyOutcome, DriveServiceError>;
}
