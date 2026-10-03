use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CopyProviderObjectRequest {
    /// Drive object key. UTF-8 1-1024 bytes, trimmed relative key; no leading/trailing slash, double slash, NUL, or period-only path segments.
    #[serde(rename = "sourceObjectKey")]
    pub source_object_key: String,

    /// Drive object key. UTF-8 1-1024 bytes, trimmed relative key; no leading/trailing slash, double slash, NUL, or period-only path segments.
    #[serde(rename = "destinationObjectKey")]
    pub destination_object_key: String,

    /// Source bucket override for the copy. Absent means the bucket configured on the storage provider. The provider endpoint, region, and credentials still apply.
    #[serde(rename = "sourceBucket")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_bucket: Option<String>,

    /// S3-compatible bucket name. DNS-compatible 3-63 characters; lowercase letters, digits, dots, and hyphens only; must start and end with a letter or digit; no IPv4-looking names, adjacent dots, dot-hyphen adjacency, or reserved S3 affixes.
    #[serde(rename = "destinationBucket")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_bucket: Option<String>,

    /// Region the copy's buckets live in, exactly as the bucket inventory reports it on that row. The server-side copy is issued at the destination endpoint, so this is what addresses the destination bucket's own regional endpoint per the vendor's own endpoint convention; absent, or a convention that cannot express it, keeps the endpoint the provider configuration stores.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    #[serde(rename = "metadataDirective")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_directive: Option<String>,
}
