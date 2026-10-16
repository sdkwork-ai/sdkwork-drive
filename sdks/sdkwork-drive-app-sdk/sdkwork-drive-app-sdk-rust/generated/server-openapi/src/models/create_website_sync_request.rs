use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateWebsiteSyncRequest {
    #[serde(rename = "expectedRootVersion")]
    pub expected_root_version: String,

    #[serde(rename = "expectedGeneration")]
    pub expected_generation: String,

    #[serde(rename = "manifestSha256")]
    pub manifest_sha256: String,

    #[serde(rename = "manifestFileCount")]
    pub manifest_file_count: String,

    #[serde(rename = "manifestTotalBytes")]
    pub manifest_total_bytes: String,

    #[serde(rename = "expiresAt")]
    pub expires_at: String,
}
