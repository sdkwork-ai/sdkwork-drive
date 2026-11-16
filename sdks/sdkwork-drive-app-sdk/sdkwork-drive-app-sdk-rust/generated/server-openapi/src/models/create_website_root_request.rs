use serde::{Deserialize, Serialize};

use crate::models::{WebsiteRootSelector};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateWebsiteRootRequest {
    #[serde(rename = "rootKey")]
    pub root_key: String,

    #[serde(rename = "displayName")]
    pub display_name: String,

    #[serde(rename = "sourceRoot")]
    pub source_root: WebsiteRootSelector,

    #[serde(rename = "contentMode")]
    pub content_mode: String,
}
