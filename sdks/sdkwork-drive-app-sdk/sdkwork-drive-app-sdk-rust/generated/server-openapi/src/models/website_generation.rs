use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WebsiteGeneration {
    pub generation: String,

    #[serde(rename = "rootNodeId")]
    pub root_node_id: String,

    #[serde(rename = "manifestSha256")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_sha256: Option<String>,

    #[serde(rename = "fileCount")]
    pub file_count: String,

    #[serde(rename = "totalBytes")]
    pub total_bytes: String,

    pub status: String,

    #[serde(rename = "activatedAt")]
    pub activated_at: String,
}
