use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SweepResponse {
    #[serde(rename = "scannedCount")]
    pub scanned_count: i64,

    #[serde(rename = "affectedCount")]
    pub affected_count: i64,

    #[serde(rename = "dryRun")]
    pub dry_run: bool,
}
