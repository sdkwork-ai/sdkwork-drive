use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateProviderObjectMultipartUploadRequest {
    /// Drive object listing key. UTF-8 1-1024 bytes, trimmed relative key; no leading slash, double slash, NUL, or period-only path segments.
    #[serde(rename = "objectKey")]
    pub object_key: String,

    /// Content type recorded on the finished object. Signed into each part grant.
    #[serde(rename = "contentType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// Optional lowercase hex SHA-256 of the whole object, when the client already has it.
    #[serde(rename = "checksumSha256Hex")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum_sha256_hex: Option<String>,
}
