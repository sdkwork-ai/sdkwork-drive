use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateCommentRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved: Option<bool>,
}
