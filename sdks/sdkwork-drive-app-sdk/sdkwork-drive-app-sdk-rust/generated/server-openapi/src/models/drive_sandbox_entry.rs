use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveSandboxEntry {
    /// Stable opaque identity derived by the server from the sandbox and canonical logical path.
    pub id: String,

    #[serde(rename = "sandboxId")]
    pub sandbox_id: String,

    /// Opaque parent identity when available. Clients navigate by logicalPath, not by decoding this value.
    #[serde(rename = "parentId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    pub name: String,

    pub kind: String,

    /// Canonical sandbox-relative logical path using '/' separators. This is never a physical provider path.
    #[serde(rename = "logicalPath")]
    pub logical_path: String,

    /// Opaque filesystem metadata revision for view reconciliation.
    pub revision: String,
}
