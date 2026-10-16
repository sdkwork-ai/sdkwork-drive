use serde::{Deserialize, Serialize};

use crate::models::{StorageMigration};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageMigrationRunReport {
    pub migration: StorageMigration,

    #[serde(rename = "copiedThisBatch")]
    pub copied_this_batch: String,

    #[serde(rename = "failedThisBatch")]
    pub failed_this_batch: String,

    /// True once the run reached `succeeded` on this call.
    pub completed: bool,
}
