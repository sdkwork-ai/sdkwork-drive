use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EmptyTrashResponse {
    #[serde(rename = "deletedCount")]
    pub deleted_count: i64,

    #[serde(rename = "skippedCount")]
    pub skipped_count: i64,

    /// True when more trashed items remain after this batch (500-item cap per request).
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}
