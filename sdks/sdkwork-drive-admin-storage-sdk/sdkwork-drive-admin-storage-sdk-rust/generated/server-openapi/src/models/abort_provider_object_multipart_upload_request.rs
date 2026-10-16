use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AbortProviderObjectMultipartUploadRequest {
    /// Drive object listing key. UTF-8 1-1024 bytes, trimmed relative key; no leading slash, double slash, NUL, or period-only path segments.
    #[serde(rename = "objectKey")]
    pub object_key: String,

    /// Opaque vendor multipart upload id returned by `...multipartUpload.create`. It is never parsed, only echoed back to the vendor.
    #[serde(rename = "uploadId")]
    pub upload_id: String,
}
