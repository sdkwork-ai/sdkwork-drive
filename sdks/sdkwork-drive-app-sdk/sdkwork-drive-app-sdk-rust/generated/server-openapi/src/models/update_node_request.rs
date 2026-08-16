use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateNodeRequest {
    #[serde(rename = "nodeName")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,

    #[serde(rename = "parentNodeId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_node_id: Option<String>,
}
