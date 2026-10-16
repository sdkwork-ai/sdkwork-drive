use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageMigrationItem {
    pub id: String,

    #[serde(rename = "storageObjectId")]
    pub storage_object_id: String,

    #[serde(rename = "sourceProviderId")]
    pub source_provider_id: String,

    #[serde(rename = "sourceBucket")]
    pub source_bucket: String,

    #[serde(rename = "sourceObjectKey")]
    pub source_object_key: String,

    #[serde(rename = "targetProviderId")]
    pub target_provider_id: String,

    #[serde(rename = "targetBucket")]
    pub target_bucket: String,

    #[serde(rename = "targetObjectKey")]
    pub target_object_key: String,

    #[serde(rename = "contentType")]
    pub content_type: String,

    #[serde(rename = "contentLength")]
    pub content_length: String,

    /// SHA-256 recorded on the source registry row, in lowercase hex.
    #[serde(rename = "checksumSha256Hex")]
    pub checksum_sha256_hex: String,

    pub status: String,

    /// SHA-256 observed on the target after the copy, present only on success.
    #[serde(rename = "verifiedChecksumSha256Hex")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_checksum_sha256_hex: Option<String>,

    #[serde(rename = "failureMessage")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_message: Option<String>,
}
