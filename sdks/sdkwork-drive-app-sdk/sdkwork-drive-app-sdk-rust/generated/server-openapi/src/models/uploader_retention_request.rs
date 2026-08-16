use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UploaderRetentionRequest {
    pub mode: String,

    #[serde(rename = "ttlSeconds")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<i64>,

    #[serde(rename = "cleanupAction")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cleanup_action: Option<String>,

    #[serde(rename = "hardDeleteAfterSeconds")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hard_delete_after_seconds: Option<i64>,
}
