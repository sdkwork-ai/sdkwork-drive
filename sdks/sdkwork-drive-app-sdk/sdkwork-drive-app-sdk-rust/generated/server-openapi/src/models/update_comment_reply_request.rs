use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateCommentReplyRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}
