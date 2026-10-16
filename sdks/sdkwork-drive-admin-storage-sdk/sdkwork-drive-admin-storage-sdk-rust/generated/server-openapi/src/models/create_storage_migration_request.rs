use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateStorageMigrationRequest {
    /// Caller-owned id. Supplying it makes a retried plan idempotent instead of opening a second run for the same intended migration.
    pub id: String,

    pub name: String,

    /// Provider to drain. May be disabled; a deleted provider is rejected.
    #[serde(rename = "sourceProviderId")]
    pub source_provider_id: String,

    /// Provider to copy onto. Must be active.
    #[serde(rename = "targetProviderId")]
    pub target_provider_id: String,

    /// Target bucket. Defaults to the target provider's own bucket.
    #[serde(rename = "targetBucket")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_bucket: Option<String>,

    /// Re-point objects and bindings at the target once every object is copied and verified.
    #[serde(rename = "applyBindingSwitch")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apply_binding_switch: Option<bool>,
}
