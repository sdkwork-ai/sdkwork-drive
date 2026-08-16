use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OpenNode {
    pub id: String,

    #[serde(rename = "tenantId")]
    pub tenant_id: String,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "nodeType")]
    pub node_type: String,

    #[serde(rename = "nodeName")]
    pub node_name: String,

    #[serde(rename = "contentType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    #[serde(rename = "contentLength")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_length: Option<i64>,
}
