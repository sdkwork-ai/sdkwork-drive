use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CopyProviderObjectRequest {
    /// Drive object key. UTF-8 1-1024 bytes, trimmed relative key; no leading/trailing slash, double slash, NUL, or period-only path segments.
    #[serde(rename = "sourceObjectKey")]
    pub source_object_key: String,

    /// Drive object key. UTF-8 1-1024 bytes, trimmed relative key; no leading/trailing slash, double slash, NUL, or period-only path segments.
    #[serde(rename = "destinationObjectKey")]
    pub destination_object_key: String,

    /// S3-compatible bucket name. DNS-compatible 3-63 characters; lowercase letters, digits, dots, and hyphens only; must start and end with a letter or digit; no IPv4-looking names, adjacent dots, dot-hyphen adjacency, or reserved S3 affixes.
    #[serde(rename = "destinationBucket")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_bucket: Option<String>,

    #[serde(rename = "metadataDirective")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_directive: Option<String>,
}
