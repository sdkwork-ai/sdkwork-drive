use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NodeCapabilitiesResponse {
    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    #[serde(rename = "subjectType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_type: Option<String>,

    #[serde(rename = "subjectId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<String>,

    pub role: String,

    pub source: String,

    #[serde(rename = "permissionId")]
    pub permission_id: String,

    pub inherited: bool,

    #[serde(rename = "inheritedFromNodeId")]
    pub inherited_from_node_id: String,

    #[serde(rename = "canRead")]
    pub can_read: bool,

    #[serde(rename = "canComment")]
    pub can_comment: bool,

    #[serde(rename = "canWrite")]
    pub can_write: bool,

    #[serde(rename = "canDownload")]
    pub can_download: bool,

    #[serde(rename = "canCopy")]
    pub can_copy: bool,

    #[serde(rename = "canMove")]
    pub can_move: bool,

    #[serde(rename = "canTrash")]
    pub can_trash: bool,

    #[serde(rename = "canRestore")]
    pub can_restore: bool,

    #[serde(rename = "canDelete")]
    pub can_delete: bool,

    #[serde(rename = "canShare")]
    pub can_share: bool,

    #[serde(rename = "canManagePermissions")]
    pub can_manage_permissions: bool,

    #[serde(rename = "canManageVersions")]
    pub can_manage_versions: bool,
}
