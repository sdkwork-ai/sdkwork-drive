use async_trait::async_trait;

use crate::domain::maintenance::DriveSweepResult;
use crate::domain::maintenance_job::DriveMaintenanceJob;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SweepDeletedObjectsQuery {
    pub dry_run: bool,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct SweepExpiredUploadSessionsQuery {
    pub now_epoch_ms: i64,
    pub dry_run: bool,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct SweepExpiredUploadContentQuery {
    pub now_epoch_ms: i64,
    pub dry_run: bool,
    pub limit: i64,
    pub operator_id: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SweepAbandonedUploadTasksQuery {
    pub now_epoch_ms: i64,
    pub dry_run: bool,
    pub limit: i64,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct NewDriveMaintenanceJob {
    pub job_type: String,
    pub status: String,
    pub dry_run: bool,
    pub scanned_count: i64,
    pub affected_count: i64,
    pub operator_id: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListMaintenanceJobsQuery {
    pub job_type: Option<String>,
    pub status: Option<String>,
    pub operator_id: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct MaintenanceJobPage {
    pub items: Vec<DriveMaintenanceJob>,
    pub page: u32,
    pub page_size: u32,
    pub total: i64,
}

#[async_trait]
pub trait DriveMaintenanceStore: Send + Sync {
    async fn sweep_deleted_objects(
        &self,
        query: &SweepDeletedObjectsQuery,
    ) -> Result<DriveSweepResult, DriveServiceError>;

    async fn sweep_expired_upload_sessions(
        &self,
        query: &SweepExpiredUploadSessionsQuery,
    ) -> Result<DriveSweepResult, DriveServiceError>;

    async fn sweep_expired_upload_content(
        &self,
        query: &SweepExpiredUploadContentQuery,
    ) -> Result<DriveSweepResult, DriveServiceError>;

    async fn sweep_abandoned_upload_tasks(
        &self,
        query: &SweepAbandonedUploadTasksQuery,
    ) -> Result<DriveSweepResult, DriveServiceError>;

    async fn insert_maintenance_job(
        &self,
        new_job: &NewDriveMaintenanceJob,
    ) -> Result<DriveMaintenanceJob, DriveServiceError>;

    async fn list_maintenance_jobs(
        &self,
        query: &ListMaintenanceJobsQuery,
    ) -> Result<MaintenanceJobPage, DriveServiceError>;
}
