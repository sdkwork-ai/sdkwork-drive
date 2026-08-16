use async_trait::async_trait;

use crate::domain::website_root::{
    DriveWebsiteContentMode, DriveWebsiteRoot, DriveWebsiteSourceRootMode,
};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct CreateDriveWebsiteRoot {
    pub tenant_id: String,
    pub space_id: String,
    pub root_key: String,
    pub display_name: String,
    pub source_root_mode: DriveWebsiteSourceRootMode,
    pub selected_folder_node_id: Option<String>,
    pub content_mode: DriveWebsiteContentMode,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct CreateDriveWebsiteRootResult {
    pub root: DriveWebsiteRoot,
    pub created: bool,
}

#[async_trait]
pub trait DriveWebsiteRootStore: Send + Sync {
    async fn create_or_get(
        &self,
        root: &CreateDriveWebsiteRoot,
    ) -> Result<CreateDriveWebsiteRootResult, DriveServiceError>;

    async fn get_by_uuid(
        &self,
        tenant_id: &str,
        root_uuid: &str,
    ) -> Result<DriveWebsiteRoot, DriveServiceError>;

    async fn list_by_space(
        &self,
        tenant_id: &str,
        space_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DriveWebsiteRoot>, DriveServiceError>;
}
