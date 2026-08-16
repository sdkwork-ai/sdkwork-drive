//! DriveUploadSession entity definition for sdkwork-database-repository.
//!
//! This module defines the DriveUploadSession entity that can be used with
//! the sdkwork-database-repository framework.

use chrono::NaiveDateTime;
use sdkwork_database_repository::impl_entity_string_pk;
use serde::{Deserialize, Serialize};

/// Drive upload session entity for tracking file uploads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveUploadSessionEntity {
    /// Unique session identifier
    pub id: String,

    /// Tenant identifier
    pub tenant_id: String,

    /// Target space identifier
    pub space_id: String,

    /// Target node identifier
    pub node_id: String,

    /// Storage bucket name
    pub bucket: String,

    /// Storage object key
    pub object_key: String,

    /// Idempotency key for deduplication
    pub idempotency_key: String,

    /// Storage provider identifier
    pub storage_provider_id: String,

    /// Storage upload identifier (multipart upload ID)
    pub storage_upload_id: String,

    /// Session state: created, uploading, completing, completed, aborted, expired
    pub state: String,

    /// Expiration timestamp in epoch milliseconds
    pub expires_at_epoch_ms: i64,

    /// Version number for optimistic locking
    pub version: i64,

    /// Creation timestamp
    pub created_at: NaiveDateTime,

    /// Last update timestamp
    pub updated_at: NaiveDateTime,
}

// Implement Entity trait for DriveUploadSessionEntity
impl_entity_string_pk!(
    DriveUploadSessionEntity,
    "dr_drive_upload_session",
    id,
    [
        id,
        tenant_id,
        space_id,
        node_id,
        bucket,
        object_key,
        idempotency_key,
        storage_provider_id,
        storage_upload_id,
        state,
        expires_at_epoch_ms,
        version,
        created_at,
        updated_at
    ]
);

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_database_repository::Entity;

    #[test]
    fn test_drive_upload_session_entity() {
        assert_eq!(
            DriveUploadSessionEntity::table_name(),
            "dr_drive_upload_session"
        );
        assert_eq!(DriveUploadSessionEntity::column_count(), 14);
        assert!(DriveUploadSessionEntity::has_column("id"));
        assert!(DriveUploadSessionEntity::has_column("tenant_id"));
        assert!(DriveUploadSessionEntity::has_column("state"));
        assert!(DriveUploadSessionEntity::has_column("idempotency_key"));
    }
}
