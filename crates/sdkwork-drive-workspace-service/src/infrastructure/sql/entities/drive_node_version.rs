//! DriveNodeVersion entity definition for sdkwork-database-repository.
//!
//! This module defines the DriveNodeVersion entity that can be used with
//! the sdkwork-database-repository framework.

use chrono::NaiveDateTime;
use sdkwork_database_repository::impl_entity_string_pk;
use serde::{Deserialize, Serialize};

/// Drive node version entity for tracking file versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveNodeVersionEntity {
    /// Unique version identifier
    pub id: String,

    /// Tenant identifier
    pub tenant_id: String,

    /// Space identifier
    pub space_id: String,

    /// Node identifier
    pub node_id: String,

    /// Version number (sequential)
    pub version_no: i64,

    /// Storage object identifier
    pub storage_object_id: Option<String>,

    /// Content MIME type
    pub content_type: String,

    /// Content length in bytes
    pub content_length: i64,

    /// SHA-256 checksum in hex
    pub checksum_sha256_hex: String,

    /// Version kind: auto, manual, restore, import, ai_generated, system
    pub version_kind: String,

    /// Optional version label
    pub version_label: Option<String>,

    /// Change source: app_api, backend_api, uploader, sync, ai, import, restore, system
    pub change_source: String,

    /// Optional change summary
    pub change_summary: Option<String>,

    /// Source version ID for restores
    pub restored_from_version_id: Option<String>,

    /// Application identifier
    pub app_id: Option<String>,

    /// Application resource type
    pub app_resource_type: Option<String>,

    /// Application resource identifier
    pub app_resource_id: Option<String>,

    /// Scene identifier
    pub scene: Option<String>,

    /// Source identifier
    pub source: Option<String>,

    /// Lifecycle status: active, deleted
    pub lifecycle_status: String,

    /// User who created the version
    pub created_by: String,

    /// User who last updated the version
    pub updated_by: String,

    /// Creation timestamp
    pub created_at: NaiveDateTime,

    /// Last update timestamp
    pub updated_at: NaiveDateTime,
}

// Implement Entity trait for DriveNodeVersionEntity
impl_entity_string_pk!(
    DriveNodeVersionEntity,
    "dr_drive_node_version",
    id,
    [
        id,
        tenant_id,
        space_id,
        node_id,
        version_no,
        storage_object_id,
        content_type,
        content_length,
        checksum_sha256_hex,
        version_kind,
        version_label,
        change_source,
        change_summary,
        restored_from_version_id,
        app_id,
        app_resource_type,
        app_resource_id,
        scene,
        source,
        lifecycle_status,
        created_by,
        updated_by,
        created_at,
        updated_at
    ]
);

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_database_repository::Entity;

    #[test]
    fn test_drive_node_version_entity() {
        assert_eq!(
            DriveNodeVersionEntity::table_name(),
            "dr_drive_node_version"
        );
        assert_eq!(DriveNodeVersionEntity::column_count(), 24);
        assert!(DriveNodeVersionEntity::has_column("id"));
        assert!(DriveNodeVersionEntity::has_column("tenant_id"));
        assert!(DriveNodeVersionEntity::has_column("node_id"));
        assert!(DriveNodeVersionEntity::has_column("version_no"));
        assert!(DriveNodeVersionEntity::has_column("content_type"));
    }
}
