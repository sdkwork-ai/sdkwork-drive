//! DriveSpace entity definition for sdkwork-database-repository.
//!
//! This module defines the DriveSpace entity that can be used with
//! the sdkwork-database-repository framework.

use chrono::NaiveDateTime;
use sdkwork_database_repository::impl_entity_string_pk;
use serde::{Deserialize, Serialize};

/// Drive space entity representing storage spaces in the drive system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveSpaceEntity {
    /// Unique space identifier (snowflake ID as string)
    pub id: String,

    /// Tenant identifier
    pub tenant_id: String,

    /// Owner subject type (user, organization, system)
    pub owner_subject_type: String,

    /// Owner subject ID
    pub owner_subject_id: String,

    /// Display name of the space
    pub display_name: String,

    /// Space type: personal, team, knowledge_base, etc.
    pub space_type: String,

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

// Implement Entity trait for DriveSpaceEntity
impl_entity_string_pk!(
    DriveSpaceEntity,
    "dr_drive_space",
    id,
    [
        id,
        tenant_id,
        owner_subject_type,
        owner_subject_id,
        display_name,
        space_type,
        lifecycle_status,
        version,
        created_at,
        updated_at,
        deleted_at
    ]
);

impl DriveSpaceEntity {
    /// Generate a new unique ID using the SDKWork ID generator.
    pub fn generate_id() -> String {
        use sdkwork_database_id::SnowflakeIdGenerator;
        static GENERATOR: std::sync::OnceLock<SnowflakeIdGenerator> = std::sync::OnceLock::new();
        let gen = GENERATOR.get_or_init(|| {
            SnowflakeIdGenerator::new(31).expect("valid node_id for drive service")
        });
        match gen.generate() {
            Ok(id) => id.to_string(),
            Err(_) => {
                use sdkwork_database_id::uuid_v4;
                uuid_v4()
            }
        }
    }

    /// Create a new DriveSpaceEntity with an auto-generated ID.
    pub fn new(
        tenant_id: String,
        owner_subject_type: String,
        owner_subject_id: String,
        display_name: String,
        space_type: String,
    ) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id: Self::generate_id(),
            tenant_id,
            owner_subject_type,
            owner_subject_id,
            display_name,
            space_type,
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
    fn test_drive_space_entity() {
        assert_eq!(DriveSpaceEntity::table_name(), "dr_drive_space");
        assert_eq!(DriveSpaceEntity::column_count(), 11);
        assert!(DriveSpaceEntity::has_column("id"));
        assert!(DriveSpaceEntity::has_column("tenant_id"));
        assert!(DriveSpaceEntity::has_column("display_name"));
    }
}
