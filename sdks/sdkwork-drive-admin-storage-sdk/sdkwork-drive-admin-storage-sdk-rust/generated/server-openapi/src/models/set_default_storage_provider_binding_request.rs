use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SetDefaultStorageProviderBindingRequest {
    #[serde(rename = "spaceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,

    #[serde(rename = "spaceType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_type: Option<String>,

    #[serde(rename = "providerId")]
    pub provider_id: String,

    /// Storage binding root prefix. UTF-8 1-512 bytes, trimmed relative prefix; no leading/trailing slash, double slash, NUL, or period-only path segments.
    #[serde(rename = "storageRootPrefix")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_root_prefix: Option<String>,
}
