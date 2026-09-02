use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateRootScopeSubscriptionRequest {
    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "knowledgeBaseId")]
    pub knowledge_base_id: String,
}
