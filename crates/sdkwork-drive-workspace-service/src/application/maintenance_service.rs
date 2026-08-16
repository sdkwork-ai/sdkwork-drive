use crate::domain::maintenance::DriveSweepResult;
use crate::ports::maintenance_store::{
    DriveMaintenanceStore, ListMaintenanceJobsQuery, MaintenanceJobPage, NewDriveMaintenanceJob,
    SweepAbandonedUploadTasksQuery, SweepDeletedObjectsQuery, SweepExpiredUploadContentQuery,
    SweepExpiredUploadSessionsQuery,
};
use sdkwork_utils_rust::MAX_LIST_PAGE_SIZE;

use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SweepObjectStoreCommand {
    pub dry_run: bool,
    pub limit: Option<i64>,
    pub operator_id: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SweepUploadSessionsCommand {
    pub now_epoch_ms: i64,
    pub dry_run: bool,
    pub limit: Option<i64>,
    pub operator_id: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SweepExpiredUploadContentCommand {
    pub now_epoch_ms: i64,
    pub dry_run: bool,
    pub limit: Option<i64>,
    pub operator_id: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SweepAbandonedUploadTasksCommand {
    pub now_epoch_ms: i64,
    pub dry_run: bool,
    pub limit: Option<i64>,
    pub operator_id: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListMaintenanceJobsCommand {
    pub job_type: Option<String>,
    pub status: Option<String>,
    pub operator_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct DriveMaintenanceService<S>
where
    S: DriveMaintenanceStore,
{
    store: S,
}

impl<S> DriveMaintenanceService<S>
where
    S: DriveMaintenanceStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn sweep_object_store(
        &self,
        command: SweepObjectStoreCommand,
    ) -> Result<DriveSweepResult, DriveServiceError> {
        let operator_id = normalize_required_operator_id(command.operator_id)?;
        let request_id = normalize_optional_identifier("request_id", command.request_id, 64)?;
        let trace_id = normalize_optional_identifier("trace_id", command.trace_id, 128)?;
        let limit = normalize_limit(command.limit)?;
        let result = self
            .store
            .sweep_deleted_objects(&SweepDeletedObjectsQuery {
                dry_run: command.dry_run,
                limit,
            })
            .await;
        match result {
            Ok(result) => {
                self.store
                    .insert_maintenance_job(&NewDriveMaintenanceJob {
                        job_type: "object_sweep".to_string(),
                        status: "completed".to_string(),
                        dry_run: result.dry_run,
                        scanned_count: result.scanned_count,
                        affected_count: result.affected_count,
                        operator_id,
                        request_id,
                        trace_id,
                        error_message: None,
                    })
                    .await?;
                Ok(result)
            }
            Err(error) => {
                let error_message = format!("{error:?}");
                self.store
                    .insert_maintenance_job(&NewDriveMaintenanceJob {
                        job_type: "object_sweep".to_string(),
                        status: "failed".to_string(),
                        dry_run: command.dry_run,
                        scanned_count: 0,
                        affected_count: 0,
                        operator_id,
                        request_id,
                        trace_id,
                        error_message: Some(error_message),
                    })
                    .await?;
                Err(error)
            }
        }
    }

    pub async fn sweep_upload_sessions(
        &self,
        command: SweepUploadSessionsCommand,
    ) -> Result<DriveSweepResult, DriveServiceError> {
        let operator_id = normalize_required_operator_id(command.operator_id)?;
        let request_id = normalize_optional_identifier("request_id", command.request_id, 64)?;
        let trace_id = normalize_optional_identifier("trace_id", command.trace_id, 128)?;
        if command.now_epoch_ms <= 0 {
            return Err(DriveServiceError::Validation(
                "now_epoch_ms must be greater than 0".to_string(),
            ));
        }
        let limit = normalize_limit(command.limit)?;
        let result = self
            .store
            .sweep_expired_upload_sessions(&SweepExpiredUploadSessionsQuery {
                now_epoch_ms: command.now_epoch_ms,
                dry_run: command.dry_run,
                limit,
            })
            .await;
        match result {
            Ok(result) => {
                self.store
                    .insert_maintenance_job(&NewDriveMaintenanceJob {
                        job_type: "upload_session_sweep".to_string(),
                        status: "completed".to_string(),
                        dry_run: result.dry_run,
                        scanned_count: result.scanned_count,
                        affected_count: result.affected_count,
                        operator_id,
                        request_id,
                        trace_id,
                        error_message: None,
                    })
                    .await?;
                Ok(result)
            }
            Err(error) => {
                let error_message = format!("{error:?}");
                self.store
                    .insert_maintenance_job(&NewDriveMaintenanceJob {
                        job_type: "upload_session_sweep".to_string(),
                        status: "failed".to_string(),
                        dry_run: command.dry_run,
                        scanned_count: 0,
                        affected_count: 0,
                        operator_id,
                        request_id,
                        trace_id,
                        error_message: Some(error_message),
                    })
                    .await?;
                Err(error)
            }
        }
    }

    pub async fn sweep_expired_upload_content(
        &self,
        command: SweepExpiredUploadContentCommand,
    ) -> Result<DriveSweepResult, DriveServiceError> {
        let operator_id = normalize_required_operator_id(command.operator_id)?;
        let request_id = normalize_optional_identifier("request_id", command.request_id, 64)?;
        let trace_id = normalize_optional_identifier("trace_id", command.trace_id, 128)?;
        if command.now_epoch_ms <= 0 {
            return Err(DriveServiceError::Validation(
                "now_epoch_ms must be greater than 0".to_string(),
            ));
        }
        let limit = normalize_limit(command.limit)?;
        let result = self
            .store
            .sweep_expired_upload_content(&SweepExpiredUploadContentQuery {
                now_epoch_ms: command.now_epoch_ms,
                dry_run: command.dry_run,
                limit,
                operator_id: operator_id.clone(),
                request_id: request_id.clone(),
                trace_id: trace_id.clone(),
            })
            .await;
        match result {
            Ok(result) => {
                self.store
                    .insert_maintenance_job(&NewDriveMaintenanceJob {
                        job_type: "expired_upload_content_sweep".to_string(),
                        status: "completed".to_string(),
                        dry_run: result.dry_run,
                        scanned_count: result.scanned_count,
                        affected_count: result.affected_count,
                        operator_id,
                        request_id,
                        trace_id,
                        error_message: None,
                    })
                    .await?;
                Ok(result)
            }
            Err(error) => {
                let error_message = format!("{error:?}");
                self.store
                    .insert_maintenance_job(&NewDriveMaintenanceJob {
                        job_type: "expired_upload_content_sweep".to_string(),
                        status: "failed".to_string(),
                        dry_run: command.dry_run,
                        scanned_count: 0,
                        affected_count: 0,
                        operator_id,
                        request_id,
                        trace_id,
                        error_message: Some(error_message),
                    })
                    .await?;
                Err(error)
            }
        }
    }

    pub async fn sweep_abandoned_upload_tasks(
        &self,
        command: SweepAbandonedUploadTasksCommand,
    ) -> Result<DriveSweepResult, DriveServiceError> {
        let operator_id = normalize_required_operator_id(command.operator_id)?;
        let request_id = normalize_optional_identifier("request_id", command.request_id, 64)?;
        let trace_id = normalize_optional_identifier("trace_id", command.trace_id, 128)?;
        if command.now_epoch_ms <= 0 {
            return Err(DriveServiceError::Validation(
                "now_epoch_ms must be greater than 0".to_string(),
            ));
        }
        let limit = normalize_limit(command.limit)?;
        let result = self
            .store
            .sweep_abandoned_upload_tasks(&SweepAbandonedUploadTasksQuery {
                now_epoch_ms: command.now_epoch_ms,
                dry_run: command.dry_run,
                limit,
                operator_id: operator_id.clone(),
            })
            .await;
        match result {
            Ok(result) => {
                self.store
                    .insert_maintenance_job(&NewDriveMaintenanceJob {
                        job_type: "abandoned_upload_task_sweep".to_string(),
                        status: "completed".to_string(),
                        dry_run: result.dry_run,
                        scanned_count: result.scanned_count,
                        affected_count: result.affected_count,
                        operator_id,
                        request_id,
                        trace_id,
                        error_message: None,
                    })
                    .await?;
                Ok(result)
            }
            Err(error) => {
                let error_message = format!("{error:?}");
                self.store
                    .insert_maintenance_job(&NewDriveMaintenanceJob {
                        job_type: "abandoned_upload_task_sweep".to_string(),
                        status: "failed".to_string(),
                        dry_run: command.dry_run,
                        scanned_count: 0,
                        affected_count: 0,
                        operator_id,
                        request_id,
                        trace_id,
                        error_message: Some(error_message),
                    })
                    .await?;
                Err(error)
            }
        }
    }

    pub async fn list_maintenance_jobs(
        &self,
        command: ListMaintenanceJobsCommand,
    ) -> Result<MaintenanceJobPage, DriveServiceError> {
        let page = command.page.unwrap_or(1);
        if page == 0 {
            return Err(DriveServiceError::Validation(
                "page must be greater than 0".to_string(),
            ));
        }

        let page_size = command.page_size.unwrap_or(20);
        if !(1..=100).contains(&page_size) {
            return Err(DriveServiceError::Validation(
                "page_size must be in range [1, 100]".to_string(),
            ));
        }

        let job_type = normalize_filter(command.job_type);
        if let Some(job_type) = &job_type {
            if !matches!(
                job_type.as_str(),
                "object_sweep"
                    | "upload_session_sweep"
                    | "expired_upload_content_sweep"
                    | "abandoned_upload_task_sweep"
            ) {
                return Err(DriveServiceError::Validation(
                    "job_type must be one of: object_sweep, upload_session_sweep, expired_upload_content_sweep, abandoned_upload_task_sweep".to_string(),
                ));
            }
        }

        let status = normalize_filter(command.status);
        if let Some(status) = &status {
            if !matches!(status.as_str(), "completed" | "failed") {
                return Err(DriveServiceError::Validation(
                    "status must be one of: completed, failed".to_string(),
                ));
            }
        }

        self.store
            .list_maintenance_jobs(&ListMaintenanceJobsQuery {
                job_type,
                status,
                operator_id: normalize_optional_identifier(
                    "operator_id",
                    command.operator_id,
                    128,
                )?,
                page,
                page_size,
            })
            .await
    }
}

fn normalize_required_operator_id(operator_id: String) -> Result<String, DriveServiceError> {
    normalize_required_identifier("operator_id", operator_id, 128)
}

fn normalize_filter(input: Option<String>) -> Option<String> {
    let value = input?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

fn normalize_required_identifier(
    field_name: &str,
    value: String,
    max_len: usize,
) -> Result<String, DriveServiceError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} is required"
        )));
    }
    validate_identifier(field_name, &value, max_len)?;
    Ok(value)
}

fn normalize_optional_identifier(
    field_name: &str,
    value: Option<String>,
    max_len: usize,
) -> Result<Option<String>, DriveServiceError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim().to_string();
    if value.is_empty() {
        return Ok(None);
    }
    validate_identifier(field_name, &value, max_len)?;
    Ok(Some(value))
}

fn validate_identifier(
    field_name: &str,
    value: &str,
    max_len: usize,
) -> Result<(), DriveServiceError> {
    if value.len() > max_len {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} length must be <= {max_len}"
        )));
    }
    if !value.chars().all(is_allowed_identifier_char) {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} contains invalid characters; allowed: [A-Za-z0-9._:@-]"
        )));
    }
    Ok(())
}

fn is_allowed_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | ':' | '@' | '-')
}

fn normalize_limit(limit: Option<i64>) -> Result<i64, DriveServiceError> {
    // Maintenance sweep batch size (PAGINATION_SPEC §2.5); not interactive list page_size.
    let limit = limit.unwrap_or(i64::from(MAX_LIST_PAGE_SIZE));
    if !(1..=i64::from(MAX_LIST_PAGE_SIZE)).contains(&limit) {
        return Err(DriveServiceError::Validation(format!(
            "limit must be in range [1, {MAX_LIST_PAGE_SIZE}]"
        )));
    }
    Ok(limit)
}
