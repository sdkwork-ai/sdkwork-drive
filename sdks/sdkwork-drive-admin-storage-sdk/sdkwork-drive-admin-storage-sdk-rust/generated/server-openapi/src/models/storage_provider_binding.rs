use serde::{Deserialize, Serialize};

use crate::models::{StorageProvider};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageProviderBinding {
    pub id: String,

    #[serde(rename = "tenantId")]
    pub tenant_id: String,

    #[serde(rename = "spaceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,

    #[serde(rename = "providerId")]
    pub provider_id: String,

    #[serde(rename = "bindingScope")]
    pub binding_scope: String,

    pub purpose: String,

    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: String,

    pub version: i64,

    #[serde(rename = "storageProvider")]
    pub storage_provider: StorageProvider,

    /// Storage binding root prefix. UTF-8 1-512 bytes, trimmed relative prefix; no leading/trailing slash, double slash, NUL, or period-only path segments.
    #[serde(rename = "storageRootPrefix")]
    pub storage_root_prefix: String,
}
