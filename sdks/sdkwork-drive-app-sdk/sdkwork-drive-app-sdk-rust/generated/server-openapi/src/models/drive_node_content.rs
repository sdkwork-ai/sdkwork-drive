use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveNodeContent {
    #[serde(rename = "nodeId")]
    pub node_id: String,

    #[serde(rename = "fileName")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,

    #[serde(rename = "contentType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    pub encoding: String,

    pub content: String,

    #[serde(rename = "sizeBytes")]
    pub size_bytes: String,

    #[serde(rename = "checksumSha256")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum_sha256: Option<String>,

    #[serde(rename = "returnedRangeStart")]
    pub returned_range_start: String,

    #[serde(rename = "returnedRangeLength")]
    pub returned_range_length: String,

    #[serde(rename = "hasMore")]
    pub has_more: bool,
}
