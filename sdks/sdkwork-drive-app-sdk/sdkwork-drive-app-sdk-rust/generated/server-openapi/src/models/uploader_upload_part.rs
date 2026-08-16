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
    pub part_no: i64,

    #[serde(rename = "offsetBytes")]
    pub offset_bytes: i64,

    #[serde(rename = "sizeBytes")]
    pub size_bytes: i64,

    pub etag: String,

    #[serde(rename = "checksumSha256Hex")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum_sha256_hex: Option<String>,

    pub status: String,

    #[serde(rename = "retryCount")]
    pub retry_count: i64,

    #[serde(rename = "uploadedAtEpochMs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uploaded_at_epoch_ms: Option<i64>,
}
