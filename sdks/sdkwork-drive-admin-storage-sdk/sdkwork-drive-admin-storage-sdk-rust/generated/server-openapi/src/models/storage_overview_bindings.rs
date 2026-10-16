use serde::{Deserialize, Serialize};

use crate::models::{StorageOverviewBindingScopeCounts};

/// Binding inventory and health signals.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageOverviewBindings {
    /// Binding rows regardless of lifecycle status.
    #[serde(rename = "totalCount")]
    pub total_count: String,

    /// Bindings in lifecycle status active.
    #[serde(rename = "activeCount")]
    pub active_count: String,

    /// Bindings that are disabled or deleted.
    #[serde(rename = "inactiveCount")]
    pub inactive_count: String,

    #[serde(rename = "byScope")]
    pub by_scope: StorageOverviewBindingScopeCounts,

    /// False means new writes in this tenant have no default provider.
    #[serde(rename = "hasTenantDefault")]
    pub has_tenant_default: bool,

    /// Id of the tenant default binding; null when unset.
    #[serde(rename = "tenantDefaultBindingId")]
    pub tenant_default_binding_id: String,

    /// Provider resolved by the tenant default binding; null when unset.
    #[serde(rename = "tenantDefaultProviderId")]
    pub tenant_default_provider_id: String,
}
