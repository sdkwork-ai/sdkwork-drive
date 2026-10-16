use serde::{Deserialize, Serialize};

/// Platform-wide provider catalog (dr_drive_storage_provider_kind) coverage. The catalog has no tenant dimension, so these counts are shared by every tenant.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageOverviewCatalog {
    /// Catalog entries.
    #[serde(rename = "totalCount")]
    pub total_count: String,

    /// Catalog entries operators may configure.
    #[serde(rename = "enabledCount")]
    pub enabled_count: String,

    /// Catalog entries disabled for new configurations.
    #[serde(rename = "disabledCount")]
    pub disabled_count: String,
}
