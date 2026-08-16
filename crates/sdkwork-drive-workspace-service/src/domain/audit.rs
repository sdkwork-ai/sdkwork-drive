#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveAuditEvent {
    pub id: i64,
    pub tenant_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub operator_id: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub created_at: String,
}
