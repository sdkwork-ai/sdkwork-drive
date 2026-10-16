use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageMigration {
    pub id: String,

    pub name: String,

    #[serde(rename = "sourceProviderId")]
    pub source_provider_id: String,

    #[serde(rename = "targetProviderId")]
    pub target_provider_id: String,

    #[serde(rename = "targetBucket")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_bucket: Option<String>,

    pub status: String,

    /// When true, every migrated object is re-pointed at the target provider once the run completes without failures. When false the bytes land on the target but the registry keeps reading from the source.
    #[serde(rename = "applyBindingSwitch")]
    pub apply_binding_switch: bool,

    /// Objects queued when the run was seeded.
    #[serde(rename = "objectsTotal")]
    pub objects_total: String,

    /// Objects copied and whose checksum was verified.
    #[serde(rename = "objectsCopied")]
    pub objects_copied: String,

    /// Objects that failed verification.
    #[serde(rename = "objectsFailed")]
    pub objects_failed: String,

    /// Objects still pending, derived from the queue.
    #[serde(rename = "objectsOutstanding")]
    pub objects_outstanding: String,

    #[serde(rename = "bytesCopied")]
    pub bytes_copied: String,

    #[serde(rename = "progressRatio")]
    pub progress_ratio: f64,

    #[serde(rename = "failureMessage")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_message: Option<String>,

    #[serde(rename = "createdBy")]
    pub created_by: String,

    #[serde(rename = "updatedBy")]
    pub updated_by: String,
}
