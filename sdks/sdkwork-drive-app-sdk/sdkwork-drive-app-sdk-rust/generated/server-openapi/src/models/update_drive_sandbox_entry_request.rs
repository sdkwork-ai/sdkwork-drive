use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateDriveSandboxEntryRequest {
    /// Current canonical sandbox-relative logical path verified against entryId.
    #[serde(rename = "logicalPath")]
    pub logical_path: String,

    /// Canonical sandbox-relative destination directory.
    #[serde(rename = "destinationParentPath")]
    pub destination_parent_path: String,

    /// Portable destination name used for move or rename.
    #[serde(rename = "destinationName")]
    pub destination_name: String,
}
