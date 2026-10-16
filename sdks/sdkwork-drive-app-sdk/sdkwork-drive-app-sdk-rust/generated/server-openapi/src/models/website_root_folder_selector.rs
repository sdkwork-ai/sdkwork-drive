use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WebsiteRootFolderSelector {
    pub mode: String,

    #[serde(rename = "folderNodeId")]
    pub folder_node_id: String,
}
