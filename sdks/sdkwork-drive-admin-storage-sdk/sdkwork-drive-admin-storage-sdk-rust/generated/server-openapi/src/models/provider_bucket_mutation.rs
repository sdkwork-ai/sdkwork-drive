use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProviderBucketMutation {
    #[serde(rename = "providerId")]
    pub provider_id: String,

    pub bucket: String,

    pub changed: bool,
}
