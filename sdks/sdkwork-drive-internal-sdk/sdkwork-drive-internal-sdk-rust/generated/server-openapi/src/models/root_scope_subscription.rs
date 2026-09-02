use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RootScopeSubscription {
    pub uuid: String,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "consumerKind")]
    pub consumer_kind: String,

    #[serde(rename = "consumerResourceId")]
    pub consumer_resource_id: String,

    #[serde(rename = "rootNodeId")]
    pub root_node_id: String,

    #[serde(rename = "scopeStatus")]
    pub scope_status: String,

    pub version: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
