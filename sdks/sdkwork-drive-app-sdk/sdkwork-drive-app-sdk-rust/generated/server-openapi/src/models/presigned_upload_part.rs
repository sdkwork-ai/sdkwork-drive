use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PresignedUploadPart {
    #[serde(rename = "uploadUrl")]
    pub upload_url: String,

    #[serde(rename = "expiresAtEpochMs")]
    pub expires_at_epoch_ms: i64,

    pub method: String,

    pub headers: std::collections::HashMap<String, String>,

    #[serde(rename = "partNo")]
    pub part_no: i64,

    #[serde(rename = "uploadId")]
    pub upload_id: String,
}
