use crate::error::{map_service_error, ProblemDetail};
use crate::state::AdminStorageState;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_workspace_service::application::audit_service::{
    DriveAuditService, RecordAuditEventCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::audit_store::SqlAuditStore;

pub(crate) async fn record_storage_provider_audit(
    state: &AdminStorageState,
    action: &str,
    provider_id: &str,
    operator_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    record_audit_event(state, action, "storage_provider", provider_id, operator_id).await
}

pub(crate) async fn record_storage_provider_kind_audit(
    state: &AdminStorageState,
    action: &str,
    provider_kind: &str,
    operator_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    record_audit_event(state, action, "storage_provider_kind", provider_kind, operator_id).await
}

pub(crate) async fn record_audit_event(
    state: &AdminStorageState,
    action: &str,
    resource_type: &str,
    resource_id: &str,
    operator_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let ids = sdkwork_drive_http::problem_correlation::current_problem_correlation();
    let audit_service = DriveAuditService::new(SqlAuditStore::new(state.pool.clone()));
    audit_service
        .record_event(RecordAuditEventCommand {
            tenant_id: "0".to_string(),
            action: action.to_string(),
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            operator_id: operator_id.to_string(),
            request_id: Some(ids.request_id),
            trace_id: Some(ids.trace_id),
        })
        .await
        .map_err(map_service_error)?;
    Ok(())
}
