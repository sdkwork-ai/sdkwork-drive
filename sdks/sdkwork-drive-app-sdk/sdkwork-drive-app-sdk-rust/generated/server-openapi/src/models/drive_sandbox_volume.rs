use serde::{Deserialize, Serialize};

use crate::models::{DriveSandboxCapabilities};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveSandboxVolume {
    /// Opaque sandbox identifier.
    pub id: String,

    #[serde(rename = "displayName")]
    pub display_name: String,

    /// Opaque logical root entry identifier.
    #[serde(rename = "rootEntryId")]
    pub root_entry_id: String,

    #[serde(rename = "effectiveAccess")]
    pub effective_access: String,

    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: String,

    pub capabilities: DriveSandboxCapabilities,

    /// Opaque optimistic concurrency revision.
    pub revision: String,
}
