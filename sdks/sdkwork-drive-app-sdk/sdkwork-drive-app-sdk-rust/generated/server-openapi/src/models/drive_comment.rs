use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveComment {
    pub id: String,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    pub content: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,

    pub resolved: bool,

    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: String,

    pub version: i64,

    #[serde(rename = "createdBy")]
    pub created_by: String,

    #[serde(rename = "updatedBy")]
    pub updated_by: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
