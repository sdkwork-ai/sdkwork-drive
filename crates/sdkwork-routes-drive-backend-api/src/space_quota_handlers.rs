use crate::audit::record_audit_event;
use crate::dto::{ListSpacesQuery, QuotaQuery, QuotaResponse, UpdateQuotaPolicyRequest};
use crate::error::{invalid_json_problem, map_service_error, validation_problem, ProblemDetail};
use crate::mappers::map_space;
use crate::response::{success_list_page_simple, DriveListHttpResponse};
use crate::state::BackendState;
use crate::tenant_context::authenticated_tenant_id;
use crate::validators::{next_page_token, parse_offset_page};
use axum::extract::rejection::JsonRejection;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use sdkwork_drive_contract::drive::domain_events::admin_audit;
use sdkwork_drive_security::DriveAppContext;
use sdkwork_drive_workspace_service::application::quota_service::{
    ClearTenantQuotaPolicyCommand, DriveQuotaService, GetTenantQuotaSummaryCommand,
    UpsertTenantQuotaPolicyCommand,
};
use sdkwork_drive_workspace_service::application::space_service::{
    DriveSpaceService, ListSpacesCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::quota_store::SqlQuotaStore;
use sdkwork_drive_workspace_service::infrastructure::sql::space_store::SqlSpaceStore;

pub(crate) async fn list_spaces(
    State(state): State<BackendState>,
    Extension(app_context): Extension<DriveAppContext>,
    Query(query): Query<ListSpacesQuery>,
) -> Result<DriveListHttpResponse<crate::dto::SpaceResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = authenticated_tenant_id(&app_context);
    let page = parse_offset_page(query.page_size, query.page_token)?;
    let service = DriveSpaceService::new(SqlSpaceStore::new(state.pool));
    let mut items = service
        .list_spaces(ListSpacesCommand {
            tenant_id,
            owner_subject_type: query.owner_subject_type,
            owner_subject_id: query.owner_subject_id,
            space_type: query.space_type,
            offset: page.offset,
            limit: page.limit + 1,
        })
        .await
        .map_err(map_service_error)?
        .into_iter()
        .map(map_space)
        .collect::<Vec<_>>();
    let next_page_token = next_page_token(&mut items, page);

    Ok(success_list_page_simple(items, page, next_page_token))
}

pub(crate) async fn list_quotas(
    State(state): State<BackendState>,
    Extension(app_context): Extension<DriveAppContext>,
    Query(_query): Query<QuotaQuery>,
) -> Result<Json<QuotaResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = authenticated_tenant_id(&app_context);
    let service = DriveQuotaService::new(SqlQuotaStore::new(state.pool.clone()));
    let summary = service
        .get_tenant_quota_summary(GetTenantQuotaSummaryCommand {
            tenant_id: tenant_id.clone(),
        })
        .await
        .map_err(map_service_error)?;
    let quota_bytes = service
        .resolve_effective_max_bytes(&tenant_id)
        .await
        .map_err(map_service_error)?;

    Ok(Json(QuotaResponse {
        tenant_id: summary.tenant_id,
        total_bytes: summary.total_bytes,
        object_count: summary.object_count,
        quota_bytes,
    }))
}

pub(crate) async fn update_quota_policy(
    State(state): State<BackendState>,
    Extension(app_context): Extension<DriveAppContext>,
    payload: Result<Json<UpdateQuotaPolicyRequest>, JsonRejection>,
) -> Result<Json<QuotaResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let tenant_id = authenticated_tenant_id(&app_context);
    let operator_id = app_context.actor_id.clone();
    let service = DriveQuotaService::new(SqlQuotaStore::new(state.pool.clone()));

    if payload.clear_tenant_policy.unwrap_or(false) {
        service
            .clear_tenant_quota_policy(ClearTenantQuotaPolicyCommand {
                tenant_id: tenant_id.clone(),
            })
            .await
            .map_err(map_service_error)?;
    } else if let Some(quota_bytes) = payload.quota_bytes {
        if quota_bytes <= 0 {
            return Err(validation_problem(
                "quotaBytes must be greater than 0 when updating tenant quota policy",
            ));
        }
        service
            .upsert_tenant_quota_policy(UpsertTenantQuotaPolicyCommand {
                tenant_id: tenant_id.clone(),
                max_bytes: Some(quota_bytes),
                operator_id: operator_id.clone(),
            })
            .await
            .map_err(map_service_error)?;
    } else {
        return Err(validation_problem(
            "either quotaBytes or clearTenantPolicy must be provided",
        ));
    }

    record_audit_event(
        &state,
        &tenant_id,
        admin_audit::quota::UPDATED,
        "tenant_quota",
        &tenant_id,
        &operator_id,
        None,
        None,
    )
    .await?;

    let summary = service
        .get_tenant_quota_summary(GetTenantQuotaSummaryCommand {
            tenant_id: tenant_id.clone(),
        })
        .await
        .map_err(map_service_error)?;
    let quota_bytes = service
        .resolve_effective_max_bytes(&tenant_id)
        .await
        .map_err(map_service_error)?;

    Ok(Json(QuotaResponse {
        tenant_id: summary.tenant_id,
        total_bytes: summary.total_bytes,
        object_count: summary.object_count,
        quota_bytes,
    }))
}
