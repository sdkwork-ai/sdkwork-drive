use async_trait::async_trait;

use crate::domain::resource_resolution::{DriveResourceScopeKind, ResolvedDriveResource};
use crate::DriveServiceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveDriveResource {
    pub tenant_id: String,
    pub scope_kind: DriveResourceScopeKind,
    pub scope_uuid: String,
    pub relative_path: String,
    pub pinned_generation: Option<i64>,
    pub pinned_node_version_id: Option<String>,
}

#[async_trait]
pub trait DriveResourceResolutionStore: Send + Sync {
    async fn resolve(
        &self,
        request: &ResolveDriveResource,
    ) -> Result<ResolvedDriveResource, DriveServiceError>;
}
