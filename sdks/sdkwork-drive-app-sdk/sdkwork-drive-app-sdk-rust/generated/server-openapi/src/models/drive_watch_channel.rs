use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveWatchChannel {
    pub id: String,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "spaceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,

    #[serde(rename = "nodeId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,

    #[serde(rename = "resourceType")]
    pub resource_type: String,

    #[serde(rename = "resourceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,

    #[serde(rename = "channelType")]
    pub channel_type: String,

    pub address: String,

    #[serde(rename = "expirationEpochMs")]
    pub expiration_epoch_ms: i64,

    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: String,

    pub version: i64,
}
