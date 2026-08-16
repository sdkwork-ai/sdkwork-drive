use async_trait::async_trait;

use crate::DriveServiceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveWorkspaceNodeRecord {
    pub id: String,
    pub parent_node_id: Option<String>,
    pub node_type: String,
    pub node_name: String,
    pub updated_at: String,
    pub content_type: Option<String>,
    pub content_length: Option<i64>,
    pub children_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewDriveWorkspaceNodeRecord {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub parent_node_id: Option<String>,
    pub node_type: String,
    pub node_name: String,
    pub content_state: String,
    pub operator_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewDriveWorkspaceObjectRecord {
    pub id: String,
    pub tenant_id: String,
    pub node_id: String,
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
    pub operator_id: String,
}

#[async_trait]
pub trait DriveWorkspaceStore: Send + Sync {
    async fn find_child_node(
        &self,
        tenant_id: &str,
        space_id: &str,
        parent_node_id: Option<&str>,
        node_name: &str,
    ) -> Result<Option<DriveWorkspaceNodeRecord>, DriveServiceError>;

    async fn ensure_node(
        &self,
        record: NewDriveWorkspaceNodeRecord,
    ) -> Result<DriveWorkspaceNodeRecord, DriveServiceError>;

    async fn ensure_object_ref(
        &self,
        record: NewDriveWorkspaceObjectRecord,
    ) -> Result<(), DriveServiceError>;

    async fn mark_node_content_ready(
        &self,
        tenant_id: &str,
        node_id: &str,
        operator_id: &str,
    ) -> Result<(), DriveServiceError>;

    async fn list_children(
        &self,
        tenant_id: &str,
        space_id: &str,
        parent_node_id: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DriveWorkspaceNodeRecord>, DriveServiceError>;

    async fn find_node(
        &self,
        tenant_id: &str,
        space_id: &str,
        node_id: &str,
    ) -> Result<Option<DriveWorkspaceNodeRecord>, DriveServiceError>;
}
