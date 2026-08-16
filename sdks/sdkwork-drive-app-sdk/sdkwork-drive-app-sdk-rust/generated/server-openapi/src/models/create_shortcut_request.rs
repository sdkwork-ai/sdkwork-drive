use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateShortcutRequest {
    pub id: String,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "parentNodeId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_node_id: Option<String>,

    #[serde(rename = "nodeName")]
    pub node_name: String,

    #[serde(rename = "targetNodeId")]
    pub target_node_id: String,
}
