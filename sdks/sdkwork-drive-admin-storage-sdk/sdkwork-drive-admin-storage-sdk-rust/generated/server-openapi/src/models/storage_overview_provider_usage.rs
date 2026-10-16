use serde::{Deserialize, Serialize};

/// One provider row of the usage breakdown.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageOverviewProviderUsage {
    /// Provider configuration id.
    #[serde(rename = "providerId")]
    pub provider_id: String,

    /// Operator-facing provider name.
    pub name: String,

    /// Provider kind key, e.g. aliyun_oss.
    #[serde(rename = "providerKind")]
    pub provider_kind: String,

    /// Provider status: active, disabled or deleted.
    pub status: String,

    /// Bucket the provider writes to.
    pub bucket: String,

    /// Objects stored through this provider.
    #[serde(rename = "objectCount")]
    pub object_count: String,

    /// Sum of content_length stored through this provider.
    #[serde(rename = "usedBytes")]
    pub used_bytes: String,

    /// Active bindings that resolve to this provider.
    #[serde(rename = "bindingCount")]
    pub binding_count: String,

    /// True when this provider carries the tenant default binding.
    #[serde(rename = "isTenantDefault")]
    pub is_tenant_default: bool,

    /// Share of total used bytes held by this provider, 0..1.
    #[serde(rename = "capacityShare")]
    pub capacity_share: f64,
}
