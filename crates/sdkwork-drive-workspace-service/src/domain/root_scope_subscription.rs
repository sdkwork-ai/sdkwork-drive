#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveRootScopeSubscription {
    pub id: String,
    pub uuid: String,
    pub tenant_id: String,
    pub space_id: String,
    pub consumer_kind: String,
    pub consumer_resource_id: String,
    pub root_node_id: String,
    pub scope_status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}
