use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PresignUploadPartRequest {
    #[serde(rename = "uploadId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upload_id: Option<String>,

    #[serde(rename = "requestedTtlSeconds")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_ttl_seconds: Option<i64>,
}
