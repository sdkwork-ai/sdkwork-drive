//! DriveStorageProvider entity definition for sdkwork-database-repository.
//!
//! This module defines the DriveStorageProvider entity that can be used with
//! the sdkwork-database-repository framework.

use chrono::NaiveDateTime;
use sdkwork_database_repository::impl_entity_string_pk;
use serde::{Deserialize, Serialize};

/// Drive storage provider entity for managing storage backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveStorageProviderEntity {
    /// Unique provider identifier
    pub id: String,

    /// Provider kind: local, s3, opendal, etc.
    pub provider_kind: String,

    /// Display name of the provider
    pub name: String,

    /// Storage endpoint URL
    pub endpoint_url: String,

    /// Storage region (for cloud providers)
    pub region: Option<String>,

    /// Storage bucket name
    pub bucket: String,

    /// Use path-style access (for S3-compatible storage)
    pub path_style: bool,

    /// Require strict TLS verification
    pub strict_tls: bool,

    /// Reference to credential store
    pub credential_ref: Option<String>,

    /// Server-side encryption mode
    pub server_side_encryption_mode: Option<String>,

    /// Default storage class
    pub default_storage_class: Option<String>,

    /// Provider status: active, disabled, error
    pub status: String,

    /// Version number for optimistic locking
    pub version: i64,

    /// Creation timestamp
    pub created_at: NaiveDateTime,

    /// Last update timestamp
    pub updated_at: NaiveDateTime,
}

// Implement Entity trait for DriveStorageProviderEntity
impl_entity_string_pk!(
    DriveStorageProviderEntity,
    "dr_drive_storage_provider",
    id,
    [
        id,
        provider_kind,
        name,
        endpoint_url,
        region,
        bucket,
        path_style,
        strict_tls,
        credential_ref,
        server_side_encryption_mode,
        default_storage_class,
        status,
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
    fn test_drive_storage_provider_entity() {
        assert_eq!(
            DriveStorageProviderEntity::table_name(),
            "dr_drive_storage_provider"
        );
        assert_eq!(DriveStorageProviderEntity::column_count(), 15);
        assert!(DriveStorageProviderEntity::has_column("id"));
        assert!(DriveStorageProviderEntity::has_column("provider_kind"));
        assert!(DriveStorageProviderEntity::has_column("name"));
        assert!(DriveStorageProviderEntity::has_column("endpoint_url"));
    }
}
