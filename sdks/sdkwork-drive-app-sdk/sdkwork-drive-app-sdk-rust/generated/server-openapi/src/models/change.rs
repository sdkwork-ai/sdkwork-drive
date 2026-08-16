use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Change {
    #[serde(rename = "sequenceNo")]
    pub sequence_no: i64,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "nodeId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,

    #[serde(rename = "eventType")]
    pub event_type: String,

    #[serde(rename = "actorId")]
    pub actor_id: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,
}
