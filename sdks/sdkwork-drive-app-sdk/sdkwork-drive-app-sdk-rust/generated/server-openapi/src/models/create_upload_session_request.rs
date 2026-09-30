use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateUploadSessionRequest {
    #[serde(rename = "sessionId")]
    pub session_id: String,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    /// Optional storage provider the caller wants this upload written to. It must be an active provider of the caller's tenant. When omitted, the provider is resolved from the bucket, the space binding, the space-type binding, or the tenant binding, in that order.
    #[serde(rename = "storageProviderId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_provider_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// Deprecated compatibility field. The service ignores this value and generates an internal sdkwork-drive/v1 object key.
    #[serde(rename = "objectKey")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_key: Option<String>,

    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,

    #[serde(rename = "expiresAtEpochMs")]
    pub expires_at_epoch_ms: String,
}
