use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DownloadPackageItem {
    #[serde(rename = "nodeId")]
    pub node_id: String,

    #[serde(rename = "nodeName")]
    pub node_name: String,

    #[serde(rename = "archivePath")]
    pub archive_path: String,

    pub bucket: String,

    #[serde(rename = "objectKey")]
    pub object_key: String,

    #[serde(rename = "contentType")]
    pub content_type: String,

    #[serde(rename = "contentLength")]
    pub content_length: i64,

    #[serde(rename = "checksumSha256Hex")]
    pub checksum_sha256_hex: String,
}
