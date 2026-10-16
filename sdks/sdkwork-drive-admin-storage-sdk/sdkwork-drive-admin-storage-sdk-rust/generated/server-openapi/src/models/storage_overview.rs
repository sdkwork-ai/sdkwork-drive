use serde::{Deserialize, Serialize};

use crate::models::{StorageOverviewBindings, StorageOverviewCapacity, StorageOverviewCatalog, StorageOverviewProviders, StorageOverviewTrendPoint};

/// Storage center dashboard aggregate. Capacity, usage and binding figures are computed over the same tenant scope so they are mutually consistent; the catalog block is platform-wide and intentionally not tenant-scoped.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageOverview {
    /// Server timestamp the aggregate was computed at.
    #[serde(rename = "generatedAt")]
    pub generated_at: String,

    /// Tenant every capacity, usage and binding figure is scoped to. The catalog block is platform-wide and ignores this.
    #[serde(rename = "scopeTenantId")]
    pub scope_tenant_id: String,

    pub capacity: StorageOverviewCapacity,

    pub providers: StorageOverviewProviders,

    pub bindings: StorageOverviewBindings,

    pub catalog: StorageOverviewCatalog,

    /// Monthly ingestion buckets, oldest first.
    pub trend: Vec<StorageOverviewTrendPoint>,
}
