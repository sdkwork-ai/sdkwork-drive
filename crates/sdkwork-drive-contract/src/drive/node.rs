use serde::{Deserialize, Serialize};

/// Drive node types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriveNodeType {
    /// A resource with file content and metadata.
    File,
    /// A container for child nodes.
    Folder,
    /// A pointer to another Drive node without duplicating content.
    Shortcut,
    /// A logical resource projected from another system.
    VirtualReference,
}

impl DriveNodeType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::File => "file",
            Self::Folder => "folder",
            Self::Shortcut => "shortcut",
            Self::VirtualReference => "virtual_reference",
        }
    }
}

/// Drive node entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveNode {
    pub id: String,
    pub space_id: String,
    pub parent_id: Option<String>,
    pub node_type: DriveNodeType,
    pub name: String,
    pub version: i64,
    pub content_state: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// Node content metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeContentMeta {
    pub content_type: Option<String>,
    pub size_bytes: u64,
    pub checksum_sha256_hex: Option<String>,
    pub etag: Option<String>,
}
