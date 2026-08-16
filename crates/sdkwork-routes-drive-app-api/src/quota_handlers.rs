use crate::app_context::DriveRequestContext;
use crate::dto::QuotaSummaryResponse;
use crate::error::{map_service_error, ProblemDetail};
use crate::response::success_resource;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_workspace_service::application::quota_service::{
    DriveQuotaService, GetTenantQuotaSummaryCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::quota_store::SqlQuotaStore;
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};

pub(crate) async fn get_quota_summary(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<QuotaSummaryResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
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

    Ok(success_resource(QuotaSummaryResponse {
        tenant_id: summary.tenant_id,
        used_bytes: summary.total_bytes,
        object_count: summary.object_count,
        quota_bytes,
    }))
}
