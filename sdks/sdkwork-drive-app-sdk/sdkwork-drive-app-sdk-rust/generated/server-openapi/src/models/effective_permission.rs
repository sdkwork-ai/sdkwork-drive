use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EffectivePermission {
    pub id: String,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "targetNodeId")]
    pub target_node_id: String,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    #[serde(rename = "subjectType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_type: Option<String>,

    #[serde(rename = "subjectId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<String>,

    pub role: String,

    pub inherited: bool,

    #[serde(rename = "inheritedFromNodeId")]
    pub inherited_from_node_id: String,

    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: String,

    pub version: i64,
}
