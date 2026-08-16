#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveChangeRecord {
    pub sequence_no: i64,
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: Option<String>,
    pub event_type: String,
    pub actor_id: String,
    pub created_at: String,
}
