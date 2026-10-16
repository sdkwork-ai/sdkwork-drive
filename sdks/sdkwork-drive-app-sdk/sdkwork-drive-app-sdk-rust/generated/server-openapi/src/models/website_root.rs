use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WebsiteRoot {
    pub uuid: String,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "rootKey")]
    pub root_key: String,

    #[serde(rename = "displayName")]
    pub display_name: String,

    #[serde(rename = "sourceRootMode")]
    pub source_root_mode: String,

    #[serde(rename = "selectedFolderNodeId")]
    pub selected_folder_node_id: String,

    #[serde(rename = "contentMode")]
    pub content_mode: String,

    #[serde(rename = "activeNodeId")]
    pub active_node_id: String,

    #[serde(rename = "activeGeneration")]
    pub active_generation: String,

    #[serde(rename = "rootStatus")]
    pub root_status: String,

    pub version: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
