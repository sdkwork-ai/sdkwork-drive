use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProviderObject {
    #[serde(rename = "providerId")]
    pub provider_id: String,

    pub bucket: String,

    /// Object list entry kind. `prefix` represents an object-store common prefix returned by delimiter-based browsing.
    #[serde(rename = "objectKind")]
    pub object_kind: String,

    /// Drive object listing key. UTF-8 1-1024 bytes, trimmed relative key; prefixes may end with a slash; no leading slash, double slash, NUL, or period-only path segments.
    #[serde(rename = "objectKey")]
    pub object_key: String,

    #[serde(rename = "contentLength")]
    pub content_length: i64,

    #[serde(rename = "contentType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,

    #[serde(rename = "versionId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,

    #[serde(rename = "storageClass")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,

    #[serde(rename = "lastModifiedEpochMs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified_epoch_ms: Option<i64>,
}
