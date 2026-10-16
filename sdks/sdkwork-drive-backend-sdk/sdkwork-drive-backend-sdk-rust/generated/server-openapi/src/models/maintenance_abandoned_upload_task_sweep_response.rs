use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MaintenanceAbandonedUploadTaskSweepResponse {
    pub code: i64,

    /// Operation-specific payload typed per response schema.
    pub data: serde_json::Value,

    /// Server-owned request correlation id.
    #[serde(rename = "traceId")]
    pub trace_id: String,

    #[serde(rename = "scannedCount")]
    pub scanned_count: String,

    #[serde(rename = "affectedCount")]
    pub affected_count: String,

    #[serde(rename = "dryRun")]
    pub dry_run: bool,
}
