use async_trait::async_trait;

use crate::domain::provider_event_delivery::DriveProviderEventDelivery;
use crate::DriveServiceError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveProviderEventResourceKind {
    KnowledgebaseRawScope,
    WebsiteRoot,
}

#[derive(Debug, Clone)]
pub struct EnsureDriveProviderEventDelivery {
    pub tenant_id: String,
    pub resource_kind: DriveProviderEventResourceKind,
    pub provider_resource_uuid: String,
    pub channel_id: String,
    pub address: String,
    pub signing_key_sha256: String,
    pub expiration_epoch_ms: i64,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct EnsureDriveProviderEventDeliveryResult {
    pub delivery: DriveProviderEventDelivery,
    pub created: bool,
}

#[async_trait]
pub trait DriveProviderEventDeliveryStore: Send + Sync {
    async fn ensure_delivery(
        &self,
        command: &EnsureDriveProviderEventDelivery,
    ) -> Result<EnsureDriveProviderEventDeliveryResult, DriveServiceError>;
}
