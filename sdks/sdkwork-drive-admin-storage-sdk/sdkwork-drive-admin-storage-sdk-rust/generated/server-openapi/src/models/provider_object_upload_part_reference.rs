use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProviderObjectUploadPartReference {
    /// Vendor part number, 1-based and contiguous. Vendors require at least 5 MiB per part except the last one.
    #[serde(rename = "partNumber")]
    pub part_number: i64,

    /// ETag the vendor returned for that part. Browsers can only read it when the bucket CORS policy exposes the ETag header.
    pub etag: String,
}
