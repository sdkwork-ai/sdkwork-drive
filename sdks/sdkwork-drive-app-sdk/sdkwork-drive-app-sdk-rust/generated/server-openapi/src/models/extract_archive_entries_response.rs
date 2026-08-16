use serde::{Deserialize, Serialize};

use crate::models::{DriveNode};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ExtractArchiveEntriesResponse {
    pub items: Vec<DriveNode>,

    #[serde(rename = "extractedCount")]
    pub extracted_count: i64,
}
