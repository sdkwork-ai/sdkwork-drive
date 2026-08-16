use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FileVersion {
    pub id: String,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    #[serde(rename = "storageObjectId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_object_id: Option<String>,

    #[serde(rename = "versionNo")]
    pub version_no: i64,

    #[serde(rename = "contentType")]
    pub content_type: String,

    #[serde(rename = "contentLength")]
    pub content_length: i64,

    #[serde(rename = "checksumSha256Hex")]
    pub checksum_sha256_hex: String,

    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,
}
