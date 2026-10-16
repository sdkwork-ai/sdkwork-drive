use serde::{Deserialize, Serialize};

use crate::models::{WebsiteRoot, WebsiteSync};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WebsiteSyncActivation {
    pub sync: WebsiteSync,

    #[serde(rename = "websiteRoot")]
    pub website_root: WebsiteRoot,
}
