use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AuditEvent {
    pub id: i64,

    #[serde(rename = "tenantId")]
    pub tenant_id: String,

    pub action: String,

    #[serde(rename = "resourceType")]
    pub resource_type: String,

    #[serde(rename = "resourceId")]
    pub resource_id: String,

    #[serde(rename = "operatorId")]
    pub operator_id: String,

    #[serde(rename = "correlationId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,

    #[serde(rename = "traceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    #[serde(rename = "createdAt")]
    pub created_at: String,
}
