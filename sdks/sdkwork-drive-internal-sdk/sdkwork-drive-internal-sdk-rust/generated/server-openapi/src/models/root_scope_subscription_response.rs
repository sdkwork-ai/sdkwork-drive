use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RootScopeSubscriptionResponse {
    pub code: i64,

    pub data: serde_json::Value,

    #[serde(rename = "traceId")]
    pub trace_id: String,
}
