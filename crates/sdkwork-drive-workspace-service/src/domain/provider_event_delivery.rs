#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveProviderEventDelivery {
    pub channel_id: String,
    pub provider_resource_uuid: String,
    pub address: String,
    pub expiration_epoch_ms: i64,
    pub lifecycle_status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}
