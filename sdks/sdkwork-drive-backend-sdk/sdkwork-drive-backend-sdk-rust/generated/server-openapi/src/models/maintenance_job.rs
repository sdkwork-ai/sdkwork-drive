use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MaintenanceJob {
    pub id: i64,

    #[serde(rename = "jobType")]
    pub job_type: String,

    pub status: String,

    #[serde(rename = "dryRun")]
    pub dry_run: bool,

    #[serde(rename = "scannedCount")]
    pub scanned_count: i64,

    #[serde(rename = "affectedCount")]
    pub affected_count: i64,

    #[serde(rename = "operatorId")]
    pub operator_id: String,

    #[serde(rename = "correlationId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,

    #[serde(rename = "traceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    #[serde(rename = "errorMessage")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    #[serde(rename = "startedAt")]
    pub started_at: String,

    #[serde(rename = "finishedAt")]
    pub finished_at: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,
}
