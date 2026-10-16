use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateDriveSandboxDirectoryRequest {
    /// Canonical sandbox-relative logical parent path. The empty value selects the sandbox root.
    #[serde(rename = "parentPath")]
    pub parent_path: String,

    /// Portable single directory name. Separators, dot segments, control characters, and operating-system reserved names are rejected.
    pub name: String,
}
