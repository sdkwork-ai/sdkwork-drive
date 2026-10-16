use serde::{Deserialize, Serialize};

use crate::models::{StorageOverviewProviderUsage};

/// Tenant-scoped provider inventory: only providers this tenant actually depends on are counted, i.e. providers referenced by one of the tenant's bindings or holding one of its objects.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageOverviewProviders {
    /// Provider configurations regardless of status.
    #[serde(rename = "totalCount")]
    pub total_count: String,

    /// Providers in status active.
    #[serde(rename = "activeCount")]
    pub active_count: String,

    /// Providers in status disabled.
    #[serde(rename = "disabledCount")]
    pub disabled_count: String,

    /// Providers in status deleted.
    #[serde(rename = "deletedCount")]
    pub deleted_count: String,

    /// Per-provider usage ordered by descending usedBytes.
    pub usage: Vec<StorageOverviewProviderUsage>,
}
