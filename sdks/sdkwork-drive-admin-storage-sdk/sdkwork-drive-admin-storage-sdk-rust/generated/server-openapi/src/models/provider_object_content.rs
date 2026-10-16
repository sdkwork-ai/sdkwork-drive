use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProviderObjectContent {
    #[serde(rename = "providerId")]
    pub provider_id: String,

    pub bucket: String,

    #[serde(rename = "objectKey")]
    pub object_key: String,

    #[serde(rename = "contentType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    #[serde(rename = "sizeBytes")]
    pub size_bytes: String,

    pub encoding: String,

    /// Base64-encoded object bytes.
    pub content: String,

    /// SHA-256 hex digest of the object bytes.
    #[serde(rename = "checksumSha256")]
    pub checksum_sha256: String,
}
