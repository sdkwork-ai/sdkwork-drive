use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateProviderObjectContent {
    /// Object content. Empty content with a trailing-slash object key creates a directory placeholder.
    pub content: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,

    /// Optional object content type.
    #[serde(rename = "contentType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}
