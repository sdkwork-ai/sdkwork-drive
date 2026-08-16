//! DriveStorageProviderKind entity definition for sdkwork-database-repository.
//!
//! This module defines the DriveStorageProviderKind entity that can be used
//! with the sdkwork-database-repository framework.

use chrono::NaiveDateTime;
use sdkwork_database_repository::impl_entity_string_pk;
use serde::{Deserialize, Serialize};

/// Built-in storage provider kind registry row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveStorageProviderKindEntity {
    /// Provider kind key: local_filesystem, s3_compatible, ...
    pub provider_kind: String,

    /// Human-readable provider brand name.
    pub display_name: String,

    /// Whether the provider kind is enabled for new/active configurations.
    pub enabled: bool,

    /// Display ordering for admin surfaces.
    pub sort_order: i32,

    /// Version number for optimistic locking.
    pub version: i64,

    /// Creation timestamp.
    pub created_at: NaiveDateTime,

    /// Last update timestamp.
    pub updated_at: NaiveDateTime,
}

impl_entity_string_pk!(
    DriveStorageProviderKindEntity,
    "dr_drive_storage_provider_kind",
    provider_kind,
    [
        provider_kind,
        display_name,
        enabled,
        sort_order,
        version,
        created_at,
        updated_at
    ]
);
