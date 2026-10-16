use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ActivateWebsiteGenerationRequest {
    #[serde(rename = "expectedRootVersion")]
    pub expected_root_version: String,

    #[serde(rename = "expectedGeneration")]
    pub expected_generation: String,
}
