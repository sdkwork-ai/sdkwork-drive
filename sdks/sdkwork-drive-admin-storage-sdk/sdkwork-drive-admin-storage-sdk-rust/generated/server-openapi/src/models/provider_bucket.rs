use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProviderBucket {
    #[serde(rename = "providerId")]
    pub provider_id: String,

    pub bucket: String,

    pub exists: bool,
}
