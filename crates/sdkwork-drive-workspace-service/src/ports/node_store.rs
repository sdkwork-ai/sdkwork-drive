use async_trait::async_trait;

use crate::domain::node::DriveNode;
use crate::domain::space::DriveSpaceType;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct NewDriveNode {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub space_type: String,
    pub parent_node_id: Option<String>,
    pub shortcut_target_node_id: Option<String>,
    pub node_type: String,
    pub node_name: String,
    pub lifecycle_status: String,
    pub created_by: String,
    pub updated_by: String,
}

#[async_trait]
pub trait DriveNodeStore: Send + Sync {
    async fn find_active_space_type(
        &self,
        tenant_id: &str,
        space_id: &str,
    ) -> Result<Option<DriveSpaceType>, DriveServiceError>;

    async fn find_active_node_in_space(
        &self,
        tenant_id: &str,
        space_id: &str,
        node_id: &str,
    ) -> Result<Option<DriveNode>, DriveServiceError>;

    async fn exists_live_name_in_parent(
        &self,
        tenant_id: &str,
        space_id: &str,
        parent_node_id: Option<&str>,
        node_name: &str,
    ) -> Result<bool, DriveServiceError>;

    async fn insert_node(&self, new_node: &NewDriveNode) -> Result<DriveNode, DriveServiceError>;
}
