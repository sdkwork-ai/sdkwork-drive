use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PurgeDriveSandboxEntryRequest {
    /// Canonical sandbox-relative logical path verified against entryId.
    #[serde(rename = "logicalPath")]
    pub logical_path: String,

    /// Must be true to remove a non-empty directory. Files and empty directories may use false.
    pub recursive: bool,
}
