use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TestStorageProviderResponse {
    #[serde(rename = "providerId")]
    pub provider_id: String,

    pub reachable: bool,
}
