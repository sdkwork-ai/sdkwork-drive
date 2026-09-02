use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RootScopeEventDelivery {
    #[serde(rename = "channelId")]
    pub channel_id: String,

    #[serde(rename = "subscriptionUuid")]
    pub subscription_uuid: String,

    pub address: String,

    #[serde(rename = "expirationEpochMs")]
    pub expiration_epoch_ms: String,

    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: String,

    pub version: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
