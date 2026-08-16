use crate::dto::{AuditEventItemResponse, ListAuditEventsQuery};
use crate::error::{map_service_error, service_error_kind, ProblemDetail};
use crate::response::{success_offset_list_page, DriveListHttpResponse};
use crate::state::BackendState;
use crate::tenant_context::authenticated_tenant_id;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use sdkwork_drive_observability::{elapsed_ms, events, has_value, start_timer};
use sdkwork_drive_security::DriveAppContext;
use sdkwork_drive_workspace_service::application::audit_service::{
    DriveAuditService, ListAuditEventsCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::audit_store::SqlAuditStore;
use sdkwork_utils_rust::OffsetListPageParams;

pub(crate) async fn list_audit_events(
    State(state): State<BackendState>,
    Extension(app_context): Extension<DriveAppContext>,
    Query(query): Query<ListAuditEventsQuery>,
) -> Result<DriveListHttpResponse<AuditEventItemResponse>, (StatusCode, Json<ProblemDetail>)> {
    let started = start_timer();
    let tenant_id = Some(authenticated_tenant_id(&app_context));
    let filter_has_tenant_id = true;
    let filter_has_action = has_value(&query.action);
    let filter_has_resource_type = has_value(&query.resource_type);
    let filter_has_resource_id = has_value(&query.resource_id);
    let filter_has_request_id = has_value(&query.request_id);
    let filter_has_trace_id = has_value(&query.trace_id);
    let service = DriveAuditService::new(SqlAuditStore::new(state.pool));
    let page_result = service
        .list_events(ListAuditEventsCommand {
            tenant_id,
            action: query.action,
            resource_type: query.resource_type,
            resource_id: query.resource_id,
            request_id: query.request_id,
            trace_id: query.trace_id,
            page: query.page,
            page_size: query.page_size,
        })
        .await;
    let page = match page_result {
        Ok(page) => page,
        Err(error) => {
            let latency_ms = elapsed_ms(started);
            sdkwork_drive_observability::observe_route!(
                event = events::BACKEND_AUDIT_EVENTS_LIST,
                result = "err",
                latency_ms = latency_ms,
                error_kind = service_error_kind(&error),
                filter_has_tenant_id = filter_has_tenant_id,
                filter_has_action = filter_has_action,
                filter_has_resource_type = filter_has_resource_type,
                filter_has_resource_id = filter_has_resource_id,
                filter_has_request_id = filter_has_request_id,
                filter_has_trace_id = filter_has_trace_id
            );
            return Err(map_service_error(error));
        }
    };
    let latency_ms = elapsed_ms(started);
    sdkwork_drive_observability::observe_route!(
        event = events::BACKEND_AUDIT_EVENTS_LIST,
        result = "ok",
        latency_ms = latency_ms,
        filter_has_tenant_id = filter_has_tenant_id,
        filter_has_action = filter_has_action,
        filter_has_resource_type = filter_has_resource_type,
        filter_has_resource_id = filter_has_resource_id,
        filter_has_request_id = filter_has_request_id,
        filter_has_trace_id = filter_has_trace_id,
        page = page.page,
        page_size = page.page_size,
        total = page.total,
        returned_items = page.items.len() as u64
    );

    Ok(success_offset_list_page(
        page.items
            .into_iter()
            .map(|item| AuditEventItemResponse {
                id: item.id,
                tenant_id: item.tenant_id,
                action: item.action,
                resource_type: item.resource_type,
                resource_id: item.resource_id,
                operator_id: item.operator_id,
                request_id: item.request_id,
                trace_id: item.trace_id,
                created_at: item.created_at,
            })
            .collect(),
        page.total,
        OffsetListPageParams {
            page: i64::from(page.page),
            page_size: i64::from(page.page_size),
            offset: i64::from(page.page - 1) * i64::from(page.page_size),
        },
    ))
}
