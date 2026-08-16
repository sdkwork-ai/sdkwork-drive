use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateWatchChannelRequest {
    pub id: String,

    #[serde(rename = "spaceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,

    pub address: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    #[serde(rename = "channelType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<String>,

    #[serde(rename = "expirationEpochMs")]
    pub expiration_epoch_ms: i64,
}
