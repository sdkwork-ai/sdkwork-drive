use crate::audit::record_maintenance_audit;
use crate::dto::{
    ListMaintenanceJobsQuery, MaintenanceJobItemResponse, SweepObjectStoreRequest, SweepResponse,
    SweepUploadSessionsRequest,
};
use crate::error::{invalid_json_problem, map_service_error, service_error_kind, ProblemDetail};
use crate::response::{success_offset_list_page, DriveListHttpResponse};
use crate::state::BackendState;
use crate::tenant_context::authenticated_tenant_id;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use sdkwork_drive_contract::drive::domain_events::admin_audit;
use sdkwork_drive_observability::{elapsed_ms, events, has_value, start_timer};
use sdkwork_drive_security::DriveAppContext;
use sdkwork_drive_workspace_service::application::maintenance_service::{
    DriveMaintenanceService, ListMaintenanceJobsCommand, SweepAbandonedUploadTasksCommand,
    SweepExpiredUploadContentCommand, SweepObjectStoreCommand, SweepUploadSessionsCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::maintenance_store::SqlMaintenanceStore;
use sdkwork_utils_rust::OffsetListPageParams;
use sdkwork_utils_rust::MAX_LIST_PAGE_SIZE;

pub(crate) async fn sweep_object_store(
    State(state): State<BackendState>,
    Extension(context): Extension<DriveAppContext>,
    payload: Result<Json<SweepObjectStoreRequest>, JsonRejection>,
) -> Result<Json<SweepResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let tenant_id = authenticated_tenant_id(&context);
    let operator_id = context.actor_id.clone();
    let request_id = Some(context.request_id.clone());
    let trace_id = Some(context.trace_id.clone());
    let started = start_timer();
    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(state.pool.clone()));
    let result = service
        .sweep_object_store(SweepObjectStoreCommand {
            dry_run: payload.dry_run,
            limit: payload.limit,
            operator_id: operator_id.clone(),
            request_id: request_id.clone(),
            trace_id: trace_id.clone(),
        })
        .await;

    match result {
        Ok(result) => {
            let latency_ms = elapsed_ms(started);
            record_maintenance_audit(
                &state,
                &tenant_id,
                admin_audit::maintenance::OBJECT_SWEEP_EXECUTED,
                &operator_id,
                request_id.clone(),
                trace_id.clone(),
            )
            .await?;
            sdkwork_drive_observability::observe_route!(
                event = events::BACKEND_MAINTENANCE_OBJECT_SWEEP,
                result = "ok",
                latency_ms = latency_ms,
                dry_run = result.dry_run,
                limit = payload.limit.unwrap_or(i64::from(MAX_LIST_PAGE_SIZE)),
                operator_id = operator_id.as_str(),
                has_request_id = request_id.is_some(),
                has_trace_id = trace_id.is_some(),
                scanned_count = result.scanned_count,
                affected_count = result.affected_count
            );

            Ok(Json(SweepResponse {
                scanned_count: result.scanned_count,
                affected_count: result.affected_count,
                dry_run: result.dry_run,
            }))
        }
        Err(error) => {
            let latency_ms = elapsed_ms(started);
            let error_kind = service_error_kind(&error);
            let _ = record_maintenance_audit(
                &state,
                &tenant_id,
                admin_audit::maintenance::OBJECT_SWEEP_FAILED,
                &operator_id,
                request_id.clone(),
                trace_id.clone(),
            )
            .await;
            sdkwork_drive_observability::observe_route!(
                event = events::BACKEND_MAINTENANCE_OBJECT_SWEEP,
                result = "err",
                latency_ms = latency_ms,
                error_kind = error_kind,
                dry_run = payload.dry_run,
                limit = payload.limit.unwrap_or(i64::from(MAX_LIST_PAGE_SIZE)),
                operator_id = operator_id.as_str(),
                has_request_id = request_id.is_some(),
                has_trace_id = trace_id.is_some()
            );
            Err(map_service_error(error))
        }
    }
}

pub(crate) async fn sweep_upload_sessions(
    State(state): State<BackendState>,
    Extension(context): Extension<DriveAppContext>,
    payload: Result<Json<SweepUploadSessionsRequest>, JsonRejection>,
) -> Result<Json<SweepResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let tenant_id = authenticated_tenant_id(&context);
    let operator_id = context.actor_id.clone();
    let request_id = Some(context.request_id.clone());
    let trace_id = Some(context.trace_id.clone());
    let started = start_timer();
    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(state.pool.clone()));
    let result = service
        .sweep_upload_sessions(SweepUploadSessionsCommand {
            now_epoch_ms: payload.now_epoch_ms,
            dry_run: payload.dry_run,
            limit: payload.limit,
            operator_id: operator_id.clone(),
            request_id: request_id.clone(),
            trace_id: trace_id.clone(),
        })
        .await;

    match result {
        Ok(result) => {
            let latency_ms = elapsed_ms(started);
            record_maintenance_audit(
                &state,
                &tenant_id,
                admin_audit::maintenance::UPLOAD_SESSION_SWEEP_EXECUTED,
                &operator_id,
                request_id.clone(),
                trace_id.clone(),
            )
            .await?;
            sdkwork_drive_observability::observe_route!(
                event = events::BACKEND_MAINTENANCE_UPLOAD_SESSION_SWEEP,
                result = "ok",
                latency_ms = latency_ms,
                now_epoch_ms = payload.now_epoch_ms,
                dry_run = result.dry_run,
                limit = payload.limit.unwrap_or(i64::from(MAX_LIST_PAGE_SIZE)),
                operator_id = operator_id.as_str(),
                has_request_id = request_id.is_some(),
                has_trace_id = trace_id.is_some(),
                scanned_count = result.scanned_count,
                affected_count = result.affected_count
            );

            Ok(Json(SweepResponse {
                scanned_count: result.scanned_count,
                affected_count: result.affected_count,
                dry_run: result.dry_run,
            }))
        }
        Err(error) => {
            let latency_ms = elapsed_ms(started);
            let error_kind = service_error_kind(&error);
            let _ = record_maintenance_audit(
                &state,
                &tenant_id,
                admin_audit::maintenance::UPLOAD_SESSION_SWEEP_FAILED,
                &operator_id,
                request_id.clone(),
                trace_id.clone(),
            )
            .await;
            sdkwork_drive_observability::observe_route!(
                event = events::BACKEND_MAINTENANCE_UPLOAD_SESSION_SWEEP,
                result = "err",
                latency_ms = latency_ms,
                error_kind = error_kind,
                now_epoch_ms = payload.now_epoch_ms,
                dry_run = payload.dry_run,
                limit = payload.limit.unwrap_or(i64::from(MAX_LIST_PAGE_SIZE)),
                operator_id = operator_id.as_str(),
                has_request_id = request_id.is_some(),
                has_trace_id = trace_id.is_some()
            );
            Err(map_service_error(error))
        }
    }
}

pub(crate) async fn sweep_expired_upload_content(
    State(state): State<BackendState>,
    Extension(context): Extension<DriveAppContext>,
    payload: Result<Json<SweepUploadSessionsRequest>, JsonRejection>,
) -> Result<Json<SweepResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let tenant_id = authenticated_tenant_id(&context);
    let operator_id = context.actor_id.clone();
    let request_id = Some(context.request_id.clone());
    let trace_id = Some(context.trace_id.clone());
    let started = start_timer();
    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(state.pool.clone()));
    let result = service
        .sweep_expired_upload_content(SweepExpiredUploadContentCommand {
            now_epoch_ms: payload.now_epoch_ms,
            dry_run: payload.dry_run,
            limit: payload.limit,
            operator_id: operator_id.clone(),
            request_id: request_id.clone(),
            trace_id: trace_id.clone(),
        })
        .await;

    match result {
        Ok(result) => {
            let latency_ms = elapsed_ms(started);
            record_maintenance_audit(
                &state,
                &tenant_id,
                admin_audit::maintenance::EXPIRED_UPLOAD_CONTENT_SWEEP_EXECUTED,
                &operator_id,
                request_id.clone(),
                trace_id.clone(),
            )
            .await?;
            sdkwork_drive_observability::observe_route!(
                event = events::BACKEND_MAINTENANCE_EXPIRED_UPLOAD_CONTENT_SWEEP,
                result = "ok",
                latency_ms = latency_ms,
                now_epoch_ms = payload.now_epoch_ms,
                dry_run = result.dry_run,
                limit = payload.limit.unwrap_or(i64::from(MAX_LIST_PAGE_SIZE)),
                operator_id = operator_id.as_str(),
                has_request_id = request_id.is_some(),
                has_trace_id = trace_id.is_some(),
                scanned_count = result.scanned_count,
                affected_count = result.affected_count
            );

            Ok(Json(SweepResponse {
                scanned_count: result.scanned_count,
                affected_count: result.affected_count,
                dry_run: result.dry_run,
            }))
        }
        Err(error) => {
            let latency_ms = elapsed_ms(started);
            let error_kind = service_error_kind(&error);
            let _ = record_maintenance_audit(
                &state,
                &tenant_id,
                admin_audit::maintenance::EXPIRED_UPLOAD_CONTENT_SWEEP_FAILED,
                &operator_id,
                request_id.clone(),
                trace_id.clone(),
            )
            .await;
            sdkwork_drive_observability::observe_route!(
                event = events::BACKEND_MAINTENANCE_EXPIRED_UPLOAD_CONTENT_SWEEP,
                result = "err",
                latency_ms = latency_ms,
                error_kind = error_kind,
                now_epoch_ms = payload.now_epoch_ms,
                dry_run = payload.dry_run,
                limit = payload.limit.unwrap_or(i64::from(MAX_LIST_PAGE_SIZE)),
                operator_id = operator_id.as_str(),
                has_request_id = request_id.is_some(),
                has_trace_id = trace_id.is_some()
            );
            Err(map_service_error(error))
        }
    }
}

pub(crate) async fn sweep_abandoned_upload_tasks(
    State(state): State<BackendState>,
    Extension(context): Extension<DriveAppContext>,
    payload: Result<Json<SweepUploadSessionsRequest>, JsonRejection>,
) -> Result<Json<SweepResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let tenant_id = authenticated_tenant_id(&context);
    let operator_id = context.actor_id.clone();
    let request_id = Some(context.request_id.clone());
    let trace_id = Some(context.trace_id.clone());
    let started = start_timer();
    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(state.pool.clone()));
    let result = service
        .sweep_abandoned_upload_tasks(SweepAbandonedUploadTasksCommand {
            now_epoch_ms: payload.now_epoch_ms,
            dry_run: payload.dry_run,
            limit: payload.limit,
            operator_id: operator_id.clone(),
            request_id: request_id.clone(),
            trace_id: trace_id.clone(),
        })
        .await;

    match result {
        Ok(result) => {
            let latency_ms = elapsed_ms(started);
            record_maintenance_audit(
                &state,
                &tenant_id,
                admin_audit::maintenance::ABANDONED_UPLOAD_TASK_SWEEP_EXECUTED,
                &operator_id,
                request_id.clone(),
                trace_id.clone(),
            )
            .await?;
            sdkwork_drive_observability::observe_route!(
                event = events::BACKEND_MAINTENANCE_ABANDONED_UPLOAD_TASK_SWEEP,
                result = "ok",
                latency_ms = latency_ms,
                now_epoch_ms = payload.now_epoch_ms,
                dry_run = result.dry_run,
                limit = payload.limit.unwrap_or(i64::from(MAX_LIST_PAGE_SIZE)),
                operator_id = operator_id.as_str(),
                has_request_id = request_id.is_some(),
                has_trace_id = trace_id.is_some(),
                scanned_count = result.scanned_count,
                affected_count = result.affected_count
            );

            Ok(Json(SweepResponse {
                scanned_count: result.scanned_count,
                affected_count: result.affected_count,
                dry_run: result.dry_run,
            }))
        }
        Err(error) => {
            let latency_ms = elapsed_ms(started);
            let error_kind = service_error_kind(&error);
            let _ = record_maintenance_audit(
                &state,
                &tenant_id,
                admin_audit::maintenance::ABANDONED_UPLOAD_TASK_SWEEP_FAILED,
                &operator_id,
                request_id.clone(),
                trace_id.clone(),
            )
            .await;
            sdkwork_drive_observability::observe_route!(
                event = events::BACKEND_MAINTENANCE_ABANDONED_UPLOAD_TASK_SWEEP,
                result = "err",
                latency_ms = latency_ms,
                error_kind = error_kind,
                now_epoch_ms = payload.now_epoch_ms,
                dry_run = payload.dry_run,
                limit = payload.limit.unwrap_or(i64::from(MAX_LIST_PAGE_SIZE)),
                operator_id = operator_id.as_str(),
                has_request_id = request_id.is_some(),
                has_trace_id = trace_id.is_some()
            );
            Err(map_service_error(error))
        }
    }
}

pub(crate) async fn list_maintenance_jobs(
    State(state): State<BackendState>,
    Query(query): Query<ListMaintenanceJobsQuery>,
) -> Result<DriveListHttpResponse<MaintenanceJobItemResponse>, (StatusCode, Json<ProblemDetail>)> {
    let started = start_timer();
    let filter_has_job_type = has_value(&query.job_type);
    let filter_has_status = has_value(&query.status);
    let filter_has_operator_id = has_value(&query.operator_id);
    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(state.pool));
    let page_result = service
        .list_maintenance_jobs(ListMaintenanceJobsCommand {
            job_type: query.job_type,
            status: query.status,
            operator_id: query.operator_id,
            page: query.page,
            page_size: query.page_size,
        })
        .await;
    let page = match page_result {
        Ok(page) => page,
        Err(error) => {
            let latency_ms = elapsed_ms(started);
            sdkwork_drive_observability::observe_route!(
                event = events::BACKEND_MAINTENANCE_JOBS_LIST,
                result = "err",
                latency_ms = latency_ms,
                error_kind = service_error_kind(&error),
                filter_has_job_type = filter_has_job_type,
                filter_has_status = filter_has_status,
                filter_has_operator_id = filter_has_operator_id
            );
            return Err(map_service_error(error));
        }
    };
    let latency_ms = elapsed_ms(started);
    sdkwork_drive_observability::observe_route!(
        event = events::BACKEND_MAINTENANCE_JOBS_LIST,
        result = "ok",
        latency_ms = latency_ms,
        filter_has_job_type = filter_has_job_type,
        filter_has_status = filter_has_status,
        filter_has_operator_id = filter_has_operator_id,
        page = page.page,
        page_size = page.page_size,
        total = page.total,
        returned_items = page.items.len() as u64
    );

    Ok(success_offset_list_page(
        page.items
            .into_iter()
            .map(|item| MaintenanceJobItemResponse {
                id: item.id,
                job_type: item.job_type,
                status: item.status,
                dry_run: item.dry_run,
                scanned_count: item.scanned_count,
                affected_count: item.affected_count,
                operator_id: item.operator_id,
                request_id: item.request_id,
                trace_id: item.trace_id,
                error_message: item.error_message,
                started_at: item.started_at,
                finished_at: item.finished_at,
                created_at: item.created_at,
            })
            .collect(),
        page.total,
        OffsetListPageParams {
            page: i64::from(page.page),
            page_size: i64::from(page.page_size),
            offset: i64::from(page.page.saturating_sub(1)) * i64::from(page.page_size),
        },
    ))
}
