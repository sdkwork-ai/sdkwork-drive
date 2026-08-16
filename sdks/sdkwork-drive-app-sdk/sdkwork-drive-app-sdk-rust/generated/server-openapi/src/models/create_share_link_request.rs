use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateShareLinkRequest {
    pub id: String,

    /// Optional client-supplied token. When omitted, the server generates a high-entropy token and returns it once in the create response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    #[serde(rename = "expiresAtEpochMs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_epoch_ms: Option<i64>,

    #[serde(rename = "downloadLimit")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_limit: Option<i64>,

    /// Optional extraction code required by recipients before resolving or downloading the shared link.
    #[serde(rename = "accessCode")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_code: Option<String>,
}
