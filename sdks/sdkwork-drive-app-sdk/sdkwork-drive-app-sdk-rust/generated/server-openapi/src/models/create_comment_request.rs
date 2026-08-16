use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateCommentRequest {
    pub id: String,

    pub content: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
}
