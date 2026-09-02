use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateShareLinkRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    #[serde(rename = "expiresAtEpochMs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_epoch_ms: Option<String>,

    #[serde(rename = "downloadLimit")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_limit: Option<String>,
}
