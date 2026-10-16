use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateDriveSandboxFileContentRequest {
    /// Canonical sandbox-relative logical file path verified against entryId.
    #[serde(rename = "logicalPath")]
    pub logical_path: String,

    /// UTF-8 text or canonical base64 content. Decoded content is limited to 4194304 bytes.
    pub content: String,

    pub encoding: String,
}
