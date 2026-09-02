use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OpenDownloadUrlResponse {
    #[serde(rename = "downloadUrl")]
    pub download_url: String,

    #[serde(rename = "expiresAtEpochMs")]
    pub expires_at_epoch_ms: String,

    pub method: String,
}
