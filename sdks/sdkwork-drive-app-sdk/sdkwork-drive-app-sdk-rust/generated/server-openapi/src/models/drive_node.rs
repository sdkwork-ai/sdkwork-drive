use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveNode {
    pub id: String,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "parentNodeId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_node_id: Option<String>,

    #[serde(rename = "nodeType")]
    pub node_type: String,

    #[serde(rename = "nodeName")]
    pub node_name: String,

    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: String,

    pub version: i64,

    /// Target node id when nodeType is shortcut.
    #[serde(rename = "shortcutTargetNodeId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcut_target_node_id: Option<String>,

    /// Drive uploader usage context identifier. Optional semantic context for idempotency, ownership, and cleanup scoping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<String>,

    /// Drive uploader usage context identifier. Optional semantic context for idempotency, ownership, and cleanup scoping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    #[serde(rename = "spaceType")]
    pub space_type: String,

    /// File content lifecycle on the node. Folders and shortcuts remain empty.
    #[serde(rename = "contentState")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_state: Option<String>,

    /// Normalized file extension without a leading dot.
    #[serde(rename = "fileExtension")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_extension: Option<String>,

    /// MIME type of the latest active file version.
    #[serde(rename = "contentType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// Coarse file category derived from contentType for list filtering and icons.
    #[serde(rename = "contentTypeGroup")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type_group: Option<String>,

    /// Byte size of the latest active file version.
    #[serde(rename = "contentLength")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_length: Option<i64>,

    /// Optional UI folder color from node property ui.folderColor.
    #[serde(rename = "folderColor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_color: Option<String>,

    /// Server-owned node creation timestamp.
    #[serde(rename = "createdAt")]
    pub created_at: String,

    /// Server-owned node last modification timestamp.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
