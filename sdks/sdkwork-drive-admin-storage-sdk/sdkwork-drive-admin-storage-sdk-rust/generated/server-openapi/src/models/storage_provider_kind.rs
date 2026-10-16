use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageProviderKind {
    /// Provider kind key, e.g. aliyun_oss.
    #[serde(rename = "providerKind")]
    pub provider_kind: String,

    /// Provider brand display name.
    #[serde(rename = "displayName")]
    pub display_name: String,

    /// Whether the provider kind is enabled for new or reactivated configurations.
    pub enabled: bool,

    /// Display ordering for admin surfaces.
    #[serde(rename = "sortOrder")]
    pub sort_order: String,

    /// Optimistic locking version.
    pub version: String,

    /// Number of non-deleted provider configurations of this kind.
    #[serde(rename = "configCount")]
    pub config_count: String,
}
