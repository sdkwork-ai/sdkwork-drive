use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct QuotasUpdateResponse {
    pub code: i64,

    /// Operation-specific payload typed per response schema.
    pub data: serde_json::Value,

    /// Server-owned request correlation id.
    #[serde(rename = "traceId")]
    pub trace_id: String,

    #[serde(rename = "tenantId")]
    pub tenant_id: String,

    #[serde(rename = "totalBytes")]
    pub total_bytes: String,

    #[serde(rename = "objectCount")]
    pub object_count: String,

    #[serde(rename = "quotaBytes")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota_bytes: Option<String>,
}
