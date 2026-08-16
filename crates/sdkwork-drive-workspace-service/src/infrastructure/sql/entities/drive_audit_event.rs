//! DriveAuditEvent entity definition for sdkwork-database-repository.
//!
//! This module defines the DriveAuditEvent entity that can be used with
//! the sdkwork-database-repository framework.

use chrono::NaiveDateTime;
use sdkwork_database_repository::impl_entity;
use serde::{Deserialize, Serialize};

/// Drive audit event entity for tracking operations in the drive system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveAuditEventEntity {
    /// Unique event identifier (snowflake ID)
    pub id: i64,

    /// Tenant identifier
    pub tenant_id: String,

    /// Action performed (create, update, delete, move, copy, etc.)
    pub action: String,

    /// Resource type (node, space, version, etc.)
    pub resource_type: String,

    /// Resource identifier
    pub resource_id: String,

    /// Operator who performed the action
    pub operator_id: String,

    /// Request identifier for tracing
    pub request_id: Option<String>,

    /// Trace identifier for distributed tracing
    pub trace_id: Option<String>,

    /// Event timestamp
    pub created_at: NaiveDateTime,
}

// Implement Entity trait for DriveAuditEventEntity
impl_entity!(
    DriveAuditEventEntity,
    "dr_drive_audit_event",
    id,
    [
        id,
        tenant_id,
        action,
        resource_type,
        resource_id,
        operator_id,
        request_id,
        trace_id,
        created_at
    ]
);

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_database_repository::Entity;

    #[test]
    fn test_drive_audit_event_entity() {
        assert_eq!(DriveAuditEventEntity::table_name(), "dr_drive_audit_event");
        assert_eq!(DriveAuditEventEntity::column_count(), 9);
        assert!(DriveAuditEventEntity::has_column("id"));
        assert!(DriveAuditEventEntity::has_column("tenant_id"));
        assert!(DriveAuditEventEntity::has_column("action"));
        assert!(DriveAuditEventEntity::has_column("resource_type"));
        assert!(DriveAuditEventEntity::has_column("operator_id"));
    }
}
