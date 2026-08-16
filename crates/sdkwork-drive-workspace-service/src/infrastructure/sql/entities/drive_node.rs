//! DriveNode entity definition for sdkwork-database-repository.
//!
//! This module defines the DriveNode entity that can be used with
//! the sdkwork-database-repository framework.

use chrono::NaiveDateTime;
use sdkwork_database_repository::impl_entity_string_pk;
use serde::{Deserialize, Serialize};

/// Drive node entity representing files, folders, and shortcuts in the drive system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveNodeEntity {
    /// Unique node identifier (snowflake ID as string)
    pub id: String,

    /// Tenant identifier
    pub tenant_id: String,

    /// Space identifier
    pub space_id: String,

    /// Space type (personal, shared, organization)
    pub space_type: String,

    /// Parent node ID (null for root nodes)
    pub parent_node_id: Option<String>,

    /// Target node ID for shortcuts
    pub shortcut_target_node_id: Option<String>,

    /// Node type: file, folder, shortcut, virtual_reference
    pub node_type: String,

    /// Display name of the node
    pub node_name: String,

    /// Lifecycle status: active, trashed, deleted
    pub lifecycle_status: String,

    /// Version number for optimistic locking
    pub version: i64,

    /// Creation timestamp
    pub created_at: NaiveDateTime,

    /// Last update timestamp
    pub updated_at: NaiveDateTime,

    /// Deletion timestamp (soft delete)
    pub deleted_at: Option<NaiveDateTime>,
}

// Implement Entity trait for DriveNodeEntity
impl_entity_string_pk!(
    DriveNodeEntity,
    "dr_drive_node",
    id,
    [
        id,
        tenant_id,
        space_id,
        space_type,
        parent_node_id,
        shortcut_target_node_id,
        node_type,
        node_name,
        lifecycle_status,
        version,
        created_at,
        updated_at,
        deleted_at
    ]
);

impl DriveNodeEntity {
    /// Generate a new unique ID using the SDKWork ID generator.
    ///
    /// Uses a Snowflake ID generator with node_id=31 (drive service).
    pub fn generate_id() -> String {
        use sdkwork_database_id::SnowflakeIdGenerator;
        static GENERATOR: std::sync::OnceLock<SnowflakeIdGenerator> = std::sync::OnceLock::new();
        let gen = GENERATOR.get_or_init(|| {
            SnowflakeIdGenerator::new(31).expect("valid node_id for drive service")
        });
        match gen.generate() {
            Ok(id) => id.to_string(),
            Err(_) => {
                // Fallback to UUID if Snowflake fails
                use sdkwork_database_id::uuid_v4;
                uuid_v4()
            }
        }
    }

    /// Create a new DriveNodeEntity with an auto-generated ID.
    pub fn new(
        tenant_id: String,
        space_id: String,
        space_type: String,
        node_type: String,
        node_name: String,
    ) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id: Self::generate_id(),
            tenant_id,
            space_id,
            space_type,
            parent_node_id: None,
            shortcut_target_node_id: None,
            node_type,
            node_name,
            lifecycle_status: "active".to_string(),
            version: 1,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_database_repository::Entity;

    #[test]
    fn test_drive_node_entity() {
        assert_eq!(DriveNodeEntity::table_name(), "dr_drive_node");
        assert_eq!(DriveNodeEntity::column_count(), 13);
        assert!(DriveNodeEntity::has_column("id"));
        assert!(DriveNodeEntity::has_column("tenant_id"));
        assert!(DriveNodeEntity::has_column("node_name"));
        assert!(!DriveNodeEntity::has_column("nonexistent"));
    }
}
