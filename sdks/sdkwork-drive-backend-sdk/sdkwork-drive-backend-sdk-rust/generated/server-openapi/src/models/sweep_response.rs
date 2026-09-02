use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SweepResponse {
    #[serde(rename = "scannedCount")]
    pub scanned_count: String,

    #[serde(rename = "affectedCount")]
    pub affected_count: String,

    #[serde(rename = "dryRun")]
    pub dry_run: bool,
}
