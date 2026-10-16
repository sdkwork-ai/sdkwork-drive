use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateDriveSandboxFileRequest {
    /// Canonical sandbox-relative logical parent path. The empty value selects the sandbox root.
    #[serde(rename = "parentPath")]
    pub parent_path: String,

    /// Portable single file name. Separators, dot segments, control characters, and operating-system reserved names are rejected.
    pub name: String,

    /// UTF-8 text or canonical base64 content. Decoded content is limited to 4194304 bytes.
    pub content: String,

    pub encoding: String,
}
