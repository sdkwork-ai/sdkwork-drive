use crate::error::{map_service_error, ProblemDetail};
use crate::state::BackendState;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_workspace_service::application::audit_service::{
    DriveAuditService, RecordAuditEventCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::audit_store::SqlAuditStore;

pub(crate) async fn record_label_audit(
    state: &BackendState,
    tenant_id: &str,
    action: &str,
    label_id: &str,
    operator_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let (request_id, trace_id) = current_audit_correlation();
    record_audit_event(
        state,
        tenant_id,
        action,
        "label",
        label_id,
        operator_id,
        request_id,
        trace_id,
    )
    .await
}

pub(crate) async fn record_maintenance_audit(
    state: &BackendState,
    tenant_id: &str,
    action: &str,
    operator_id: &str,
    request_id: Option<String>,
    trace_id: Option<String>,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    record_audit_event(
        state,
        tenant_id,
        action,
        "maintenance",
        "maintenance",
        operator_id,
        request_id,
        trace_id,
    )
    .await
}

pub(crate) async fn record_audit_event(
    state: &BackendState,
    tenant_id: &str,
    action: &str,
    resource_type: &str,
    resource_id: &str,
    operator_id: &str,
    request_id: Option<String>,
    trace_id: Option<String>,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let (request_id, trace_id) = match (request_id, trace_id) {
        (Some(request_id), Some(trace_id)) => (Some(request_id), Some(trace_id)),
        _ => current_audit_correlation(),
    };
    let audit_service = DriveAuditService::new(SqlAuditStore::new(state.pool.clone()));
    audit_service
        .record_event(RecordAuditEventCommand {
            tenant_id: tenant_id.to_string(),
            action: action.to_string(),
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            operator_id: operator_id.to_string(),
            request_id,
            trace_id,
        })
        .await
        .map_err(map_service_error)?;
    Ok(())
}

fn current_audit_correlation() -> (Option<String>, Option<String>) {
    let ids = sdkwork_drive_http::problem_correlation::current_problem_correlation();
    (Some(ids.request_id), Some(ids.trace_id))
}
