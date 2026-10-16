use serde::{Deserialize, Serialize};

use crate::models::{WebsiteGeneration, WebsiteRoot};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WebsiteGenerationActivation {
    #[serde(rename = "sourceGeneration")]
    pub source_generation: WebsiteGeneration,

    #[serde(rename = "websiteRoot")]
    pub website_root: WebsiteRoot,
}
