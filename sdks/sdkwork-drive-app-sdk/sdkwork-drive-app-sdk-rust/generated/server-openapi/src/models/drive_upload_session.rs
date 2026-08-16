use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveUploadSession {
    pub id: String,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    pub bucket: String,

    #[serde(rename = "objectKey")]
    pub object_key: String,

    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,

    pub state: String,

    #[serde(rename = "expiresAtEpochMs")]
    pub expires_at_epoch_ms: i64,

    pub version: i64,

    /// Drive storage provider id bound to this upload session.
    #[serde(rename = "storageProviderId")]
    pub storage_provider_id: String,

    /// Provider-side multipart upload id used by the configured object store.
    #[serde(rename = "storageUploadId")]
    pub storage_upload_id: String,
}
