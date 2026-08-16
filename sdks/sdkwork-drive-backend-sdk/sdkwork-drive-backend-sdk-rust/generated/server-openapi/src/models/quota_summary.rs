use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct QuotaSummary {
    #[serde(rename = "tenantId")]
    pub tenant_id: String,

    #[serde(rename = "totalBytes")]
    pub total_bytes: i64,

    #[serde(rename = "objectCount")]
    pub object_count: i64,

    #[serde(rename = "quotaBytes")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota_bytes: Option<i64>,
}
