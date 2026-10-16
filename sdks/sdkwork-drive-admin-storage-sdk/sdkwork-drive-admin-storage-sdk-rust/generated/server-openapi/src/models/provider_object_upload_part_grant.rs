use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProviderObjectUploadPartGrant {
    /// Vendor part number, 1-based and contiguous. Vendors require at least 5 MiB per part except the last one.
    #[serde(rename = "partNumber")]
    pub part_number: i64,

    /// HTTP method to use against `url`. Vendors return PUT.
    pub method: String,

    /// Absolute presigned vendor URL. The bytes go here, never through this API.
    pub url: String,

    /// Headers that are part of the signature. Send them verbatim with the upload or the vendor rejects it with 403.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,

    #[serde(rename = "expiresAtEpochMs")]
    pub expires_at_epoch_ms: String,
}
