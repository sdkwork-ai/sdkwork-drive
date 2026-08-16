use async_trait::async_trait;

use crate::DriveServiceError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebsiteTreeCleanupKind {
    TerminalSync,
    ExpiredGeneration,
}

impl WebsiteTreeCleanupKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TerminalSync => "terminal_sync",
            Self::ExpiredGeneration => "expired_generation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebsiteTreeCleanupCandidate {
    pub kind: WebsiteTreeCleanupKind,
    pub resource_id: String,
    pub tenant_id: String,
    pub root_node_id: String,
    pub delete_tree: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebsiteTreeStorageObject {
    pub id: String,
    pub storage_provider_id: String,
    pub storage_provider_version: i64,
    pub bucket: String,
    pub object_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WebsitePublishingMaintenanceResult {
    pub expired_syncs: u64,
    pub completed_candidates: u64,
    pub deleted_objects: u64,
    pub deleted_nodes: u64,
}

#[async_trait]
pub trait DriveWebsitePublishingMaintenanceStore: Send + Sync {
    async fn expire_stale_syncs(
        &self,
        limit: i64,
        operator_id: &str,
    ) -> Result<u64, DriveServiceError>;

    async fn claim_next_cleanup_candidate(
        &self,
        operator_id: &str,
    ) -> Result<Option<WebsiteTreeCleanupCandidate>, DriveServiceError>;

    async fn list_candidate_storage_objects(
        &self,
        candidate: &WebsiteTreeCleanupCandidate,
        limit: i64,
    ) -> Result<Vec<WebsiteTreeStorageObject>, DriveServiceError>;

    async fn mark_storage_object_deleted(
        &self,
        candidate: &WebsiteTreeCleanupCandidate,
        storage_object_id: &str,
        operator_id: &str,
    ) -> Result<bool, DriveServiceError>;

    async fn complete_cleanup_candidate(
        &self,
        candidate: &WebsiteTreeCleanupCandidate,
        operator_id: &str,
    ) -> Result<u64, DriveServiceError>;
}
