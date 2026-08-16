use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateCommentReplyRequest {
    pub id: String,

    pub content: String,
}
