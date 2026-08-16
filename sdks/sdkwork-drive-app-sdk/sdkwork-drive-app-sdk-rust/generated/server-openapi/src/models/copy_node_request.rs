use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CopyNodeRequest {
    pub id: String,

    #[serde(rename = "targetSpaceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_space_id: Option<String>,

    #[serde(rename = "targetParentNodeId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_parent_node_id: Option<String>,

    #[serde(rename = "nodeName")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,
}
