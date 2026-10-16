use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PresignProviderObjectUploadPartsRequest {
    /// Drive object listing key. UTF-8 1-1024 bytes, trimmed relative key; no leading slash, double slash, NUL, or period-only path segments.
    #[serde(rename = "objectKey")]
    pub object_key: String,

    /// Opaque vendor multipart upload id returned by `...multipartUpload.create`. It is never parsed, only echoed back to the vendor.
    #[serde(rename = "uploadId")]
    pub upload_id: String,

    /// Part numbers to sign in this call. Batched so a single request can cover the parts a client is about to send concurrently.
    #[serde(rename = "partNumbers")]
    pub part_numbers: Vec<i64>,

    /// Grant lifetime. Defaults to 3600 seconds when absent.
    #[serde(rename = "expiresInSeconds")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_in_seconds: Option<i64>,
}
