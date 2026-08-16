//! Entity definitions for sdkwork-database-repository integration.
//!
//! This module contains entity definitions that implement the
//! sdkwork-database-repository Entity trait for use with the
//! unified repository pattern.

pub mod drive_audit_event;
pub mod drive_node;
pub mod drive_node_version;
pub mod drive_space;
pub mod drive_storage_provider;
pub mod drive_storage_provider_kind;
pub mod drive_upload_session;

// Re-export entities
pub use drive_audit_event::DriveAuditEventEntity;
pub use drive_node::DriveNodeEntity;
pub use drive_node_version::DriveNodeVersionEntity;
pub use drive_space::DriveSpaceEntity;
pub use drive_storage_provider::DriveStorageProviderEntity;
pub use drive_storage_provider_kind::DriveStorageProviderKindEntity;
pub use drive_upload_session::DriveUploadSessionEntity;
