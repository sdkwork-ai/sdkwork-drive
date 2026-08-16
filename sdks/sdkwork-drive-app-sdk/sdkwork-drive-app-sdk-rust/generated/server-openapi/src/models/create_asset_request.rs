use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateAssetRequest {
    /// Existing Drive node to expose through /assets.
    #[serde(rename = "driveNodeId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drive_node_id: Option<String>,

    #[serde(rename = "virtualReference")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub virtual_reference: Option<serde_json::Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}
