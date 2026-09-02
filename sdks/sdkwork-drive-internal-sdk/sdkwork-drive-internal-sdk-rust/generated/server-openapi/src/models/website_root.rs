use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WebsiteRoot {
    pub uuid: String,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "sourceRootMode")]
    pub source_root_mode: String,

    #[serde(rename = "contentMode")]
    pub content_mode: String,

    #[serde(rename = "activeGeneration")]
    pub active_generation: String,

    #[serde(rename = "rootStatus")]
    pub root_status: String,

    pub capabilities: Vec<String>,

    pub version: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
