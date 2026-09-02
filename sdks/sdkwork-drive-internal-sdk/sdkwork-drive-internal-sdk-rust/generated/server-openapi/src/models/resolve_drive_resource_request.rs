use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ResolveDriveResourceRequest {
    #[serde(rename = "scopeType")]
    pub scope_type: String,

    #[serde(rename = "scopeUuid")]
    pub scope_uuid: String,

    #[serde(rename = "relativePath")]
    pub relative_path: String,

    #[serde(rename = "pinnedGeneration")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned_generation: Option<String>,

    #[serde(rename = "pinnedNodeVersionId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned_node_version_id: Option<String>,
}
