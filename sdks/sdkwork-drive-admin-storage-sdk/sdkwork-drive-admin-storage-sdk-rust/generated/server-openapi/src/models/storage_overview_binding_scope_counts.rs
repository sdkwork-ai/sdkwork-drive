use serde::{Deserialize, Serialize};

/// Binding counts per binding scope.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageOverviewBindingScopeCounts {
    /// Bindings scoped to the whole tenant.
    #[serde(rename = "tenantCount")]
    pub tenant_count: String,

    /// Bindings scoped to a single space.
    #[serde(rename = "spaceCount")]
    pub space_count: String,

    /// Bindings scoped to a space type.
    #[serde(rename = "spaceTypeCount")]
    pub space_type_count: String,
}
