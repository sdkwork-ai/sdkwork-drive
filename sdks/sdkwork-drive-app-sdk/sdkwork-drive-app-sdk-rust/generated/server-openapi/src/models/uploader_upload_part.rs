use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UploaderUploadPart {
    pub id: String,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "uploadItemId")]
    pub upload_item_id: String,

    #[serde(rename = "uploadSessionId")]
    pub upload_session_id: String,

    #[serde(rename = "partNo")]
    pub part_no: String,

    #[serde(rename = "offsetBytes")]
    pub offset_bytes: String,

    #[serde(rename = "sizeBytes")]
    pub size_bytes: String,

    pub etag: String,

    #[serde(rename = "checksumSha256Hex")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum_sha256_hex: Option<String>,

    pub status: String,

    #[serde(rename = "retryCount")]
    pub retry_count: String,

    #[serde(rename = "uploadedAtEpochMs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uploaded_at_epoch_ms: Option<String>,
}
