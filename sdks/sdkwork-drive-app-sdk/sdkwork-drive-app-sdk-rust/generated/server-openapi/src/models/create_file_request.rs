use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateFileRequest {
    pub id: String,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "parentNodeId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_node_id: Option<String>,

    #[serde(rename = "nodeName")]
    pub node_name: String,

    #[serde(rename = "uploadSessionId")]
    pub upload_session_id: String,

    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,

    #[serde(rename = "expiresAtEpochMs")]
    pub expires_at_epoch_ms: i64,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// Deprecated compatibility field. The service ignores this value and generates an internal sdkwork-drive/v1 object key.
    #[serde(rename = "objectKey")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_key: Option<String>,
}
