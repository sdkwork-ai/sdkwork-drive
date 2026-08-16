use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProviderObjectMutation {
    #[serde(rename = "providerId")]
    pub provider_id: String,

    pub bucket: String,

    /// Drive object key. UTF-8 1-1024 bytes, trimmed relative key; no leading/trailing slash, double slash, NUL, or period-only path segments.
    #[serde(rename = "objectKey")]
    pub object_key: String,

    pub changed: bool,
}
