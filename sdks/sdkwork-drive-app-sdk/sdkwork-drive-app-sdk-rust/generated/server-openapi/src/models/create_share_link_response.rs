use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateShareLinkResponse {
    pub id: String,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    pub role: String,

    #[serde(rename = "expiresAtEpochMs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_epoch_ms: Option<i64>,

    #[serde(rename = "downloadLimit")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_limit: Option<i64>,

    #[serde(rename = "downloadCount")]
    pub download_count: i64,

    #[serde(rename = "accessCodeRequired")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_code_required: Option<bool>,

    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: String,

    pub version: i64,

    /// Raw share token returned only once when the share link is created.
    pub token: String,

    /// Extraction code returned only once when configured at create time.
    #[serde(rename = "accessCode")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_code: Option<String>,
}
