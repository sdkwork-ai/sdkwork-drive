#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveMaintenanceJob {
    pub id: i64,
    pub job_type: String,
    pub status: String,
    pub dry_run: bool,
    pub scanned_count: i64,
    pub affected_count: i64,
    pub operator_id: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub finished_at: String,
    pub created_at: String,
}
