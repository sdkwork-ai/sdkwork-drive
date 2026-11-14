use async_trait::async_trait;

use crate::DriveServiceError;

/// One tenant-scoped drive node projected for the backend admin file list.
///
/// This projection is metadata-only: object keys, storage provider ids, and
/// credentials stay out of it, so an operator can audit the workspace without
/// receiving anything the storage plane uses to reach a bucket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveAdminFileRecord {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub space_name: String,
    pub space_type: String,
    pub owner_subject_type: String,
    pub owner_subject_id: String,
    pub parent_node_id: Option<String>,
    pub node_type: String,
    pub node_name: String,
    pub file_path: String,
    pub file_extension: Option<String>,
    pub content_type: Option<String>,
    pub content_type_group: Option<String>,
    pub content_length: i64,
    pub lifecycle_status: String,
    pub version: i64,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Store-level filters for the admin file list.
///
/// Every optional field is already normalized (trimmed, empty removed) by the
/// application service; the store binds them directly instead of re-parsing.
#[derive(Debug, Clone)]
pub struct ListAdminFilesQuery {
    pub tenant_id: String,
    pub space_id: Option<String>,
    pub space_type: Option<String>,
    pub node_type: Option<String>,
    pub lifecycle_status: Option<String>,
    pub content_type_group: Option<String>,
    pub keyword: Option<String>,
    pub owner_subject_id: Option<String>,
    pub offset: i64,
    pub limit: i64,
}

#[async_trait]
pub trait DriveAdminFileStore: Send + Sync {
    async fn list_admin_files(
        &self,
        query: &ListAdminFilesQuery,
    ) -> Result<Vec<DriveAdminFileRecord>, DriveServiceError>;
}
