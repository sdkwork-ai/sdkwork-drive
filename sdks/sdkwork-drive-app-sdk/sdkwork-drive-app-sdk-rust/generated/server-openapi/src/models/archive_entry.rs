use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ArchiveEntry {
    pub path: String,

    pub name: String,

    #[serde(rename = "isDirectory")]
    pub is_directory: bool,

    #[serde(rename = "uncompressedSizeBytes")]
    pub uncompressed_size_bytes: i64,

    #[serde(rename = "compressedSizeBytes")]
    pub compressed_size_bytes: i64,

    #[serde(rename = "contentType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}
