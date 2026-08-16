use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ExtractArchiveEntriesRequest {
    #[serde(rename = "entryPaths")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_paths: Option<Vec<String>>,

    #[serde(rename = "targetParentNodeId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_parent_node_id: Option<String>,
}
