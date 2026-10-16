use serde::{Deserialize, Serialize};

/// Tenant-level capacity figures derived from dr_drive_storage_object.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageOverviewCapacity {
    /// Object rows regardless of lifecycle status.
    #[serde(rename = "totalObjectCount")]
    pub total_object_count: String,

    /// Objects currently in lifecycle status active.
    #[serde(rename = "activeObjectCount")]
    pub active_object_count: String,

    /// Objects awaiting physical reclaim.
    #[serde(rename = "deletedObjectCount")]
    pub deleted_object_count: String,

    /// Sum of content_length over active objects.
    #[serde(rename = "usedBytes")]
    pub used_bytes: String,

    /// Mean content_length over active objects; 0 when there are none.
    #[serde(rename = "averageObjectBytes")]
    pub average_object_bytes: String,

    /// Largest active object content_length; null when there are none.
    #[serde(rename = "largestObjectBytes")]
    pub largest_object_bytes: String,

    /// Distinct buckets that hold object rows in this scope.
    #[serde(rename = "bucketCount")]
    pub bucket_count: String,

    /// Tenant quota max_bytes; null when the tenant has no quota row.
    #[serde(rename = "quotaBytes")]
    pub quota_bytes: String,

    /// True when a quota row exists with a non-null max_bytes.
    #[serde(rename = "quotaConfigured")]
    pub quota_configured: bool,

    /// usedBytes / quotaBytes. May exceed 1 when the quota is overrun; null without a quota.
    #[serde(rename = "quotaUsageRatio")]
    pub quota_usage_ratio: f64,
}
