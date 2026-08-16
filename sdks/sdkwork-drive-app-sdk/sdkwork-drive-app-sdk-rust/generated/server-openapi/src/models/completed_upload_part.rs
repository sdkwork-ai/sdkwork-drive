use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CompletedUploadPart {
    #[serde(rename = "partNo")]
    pub part_no: i64,

    pub etag: String,
}
