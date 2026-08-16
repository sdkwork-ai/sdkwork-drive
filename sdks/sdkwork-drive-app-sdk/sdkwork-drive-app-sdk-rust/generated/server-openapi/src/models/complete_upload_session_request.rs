use serde::{Deserialize, Serialize};

use crate::models::{CompletedUploadPart};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CompleteUploadSessionRequest {
    #[serde(rename = "uploadId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upload_id: Option<String>,

    #[serde(rename = "contentType")]
    pub content_type: String,

    #[serde(rename = "contentLength")]
    pub content_length: i64,

    #[serde(rename = "checksumSha256Hex")]
    pub checksum_sha256_hex: String,

    /// Completed multipart upload parts ordered by ascending unique partNo.
    pub parts: Vec<CompletedUploadPart>,
}
