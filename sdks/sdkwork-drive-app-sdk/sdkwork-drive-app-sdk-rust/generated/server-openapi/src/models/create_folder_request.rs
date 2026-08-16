use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateFolderRequest {
    /// Optional client-supplied node id for offline or idempotent retries. When omitted, the service assigns a server-generated id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "parentNodeId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_node_id: Option<String>,

    #[serde(rename = "nodeName")]
    pub node_name: String,
}
