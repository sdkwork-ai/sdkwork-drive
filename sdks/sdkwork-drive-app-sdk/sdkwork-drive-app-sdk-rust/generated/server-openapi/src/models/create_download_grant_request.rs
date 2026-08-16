use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateDownloadGrantRequest {
    #[serde(rename = "requestedTtlSeconds")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_ttl_seconds: Option<i64>,
}
