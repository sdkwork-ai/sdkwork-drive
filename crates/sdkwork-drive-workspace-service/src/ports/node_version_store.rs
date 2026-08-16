use async_trait::async_trait;

use crate::domain::node_version::{CreateDriveNodeVersionCommand, DriveNodeVersion};
use crate::DriveServiceError;

#[async_trait]
pub trait DriveNodeVersionStore: Send + Sync {
    async fn create(
        &self,
        command: CreateDriveNodeVersionCommand,
    ) -> Result<DriveNodeVersion, DriveServiceError>;

    async fn find_by_id(
        &self,
        tenant_id: &str,
        node_id: &str,
        version_id: &str,
    ) -> Result<Option<DriveNodeVersion>, DriveServiceError>;

    async fn list_by_node(
        &self,
        tenant_id: &str,
        node_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DriveNodeVersion>, DriveServiceError>;
}
