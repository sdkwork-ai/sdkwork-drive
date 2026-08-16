use crate::app_context::DriveRequestContext;
use crate::audit::record_storage_provider_kind_audit;
use crate::dto::{
    OffsetPage, SetStorageProviderKindEnabledRequest, StorageProviderKindResponse,
};
use crate::error::{invalid_json_problem, map_service_error, ProblemDetail};
use crate::response::{success_list_page_simple, StorageListHttpResponse};
use crate::state::AdminStorageState;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_contract::drive::domain_events::admin_audit;
use sdkwork_drive_workspace_service::application::storage_provider_kind_service::{
    DriveStorageProviderKindService, SetStorageProviderKindEnabledCommand,
    StorageProviderKindSummary,
};
use sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_kind_store::SqlStorageProviderKindStore;

fn kind_service(state: &AdminStorageState) -> DriveStorageProviderKindService<SqlStorageProviderKindStore> {
    DriveStorageProviderKindService::new(SqlStorageProviderKindStore::new(state.pool.clone()))
}

fn map_storage_provider_kind(summary: StorageProviderKindSummary) -> StorageProviderKindResponse {
    StorageProviderKindResponse {
        provider_kind: summary.kind.provider_kind,
        display_name: summary.kind.display_name,
        enabled: summary.kind.enabled,
        sort_order: i64::from(summary.kind.sort_order),
        version: summary.kind.version,
        config_count: summary.config_count,
    }
}

const ALL_KINDS_PAGE: OffsetPage = OffsetPage {
    limit: 100,
    offset: 0,
};

pub(crate) async fn list_storage_provider_kinds(
    State(state): State<AdminStorageState>,
) -> Result<StorageListHttpResponse<StorageProviderKindResponse>, (StatusCode, Json<ProblemDetail>)> {
    let summaries = kind_service(&state)
        .list_storage_provider_kinds()
        .await
        .map_err(map_service_error)?;
    let items = summaries
        .into_iter()
        .map(map_storage_provider_kind)
        .collect::<Vec<_>>();
    Ok(success_list_page_simple(items, ALL_KINDS_PAGE, None))
}

pub(crate) async fn initialize_storage_provider_kinds(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
) -> Result<StorageListHttpResponse<StorageProviderKindResponse>, (StatusCode, Json<ProblemDetail>)> {
    let operator_id = ctx.resolve_operator_id()?;
    kind_service(&state)
        .initialize_storage_provider_kinds()
        .await
        .map_err(map_service_error)?;
    record_storage_provider_kind_audit(
        &state,
        admin_audit::storage_provider_kind::INITIALIZED,
        "builtin",
        &operator_id,
    )
    .await?;
    let summaries = kind_service(&state)
        .list_storage_provider_kinds()
        .await
        .map_err(map_service_error)?;
    let items = summaries
        .into_iter()
        .map(map_storage_provider_kind)
        .collect::<Vec<_>>();
    Ok(success_list_page_simple(items, ALL_KINDS_PAGE, None))
}

pub(crate) async fn set_storage_provider_kind_enabled(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_kind): Path<String>,
    payload: Result<Json<SetStorageProviderKindEnabledRequest>, JsonRejection>,
) -> Result<Json<StorageProviderKindResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let operator_id = ctx.resolve_operator_id()?;
    let updated = kind_service(&state)
        .set_storage_provider_kind_enabled(SetStorageProviderKindEnabledCommand {
            provider_kind: provider_kind.clone(),
            enabled: payload.enabled,
        })
        .await
        .map_err(map_service_error)?;
    let action = if payload.enabled {
        admin_audit::storage_provider_kind::ENABLED
    } else {
        admin_audit::storage_provider_kind::DISABLED
    };
    record_storage_provider_kind_audit(&state, action, &provider_kind, &operator_id).await?;
    let summary = kind_service(&state)
        .list_storage_provider_kinds()
        .await
        .map_err(map_service_error)?
        .into_iter()
        .find(|summary| summary.kind.provider_kind == updated.provider_kind)
        .ok_or_else(|| {
            map_service_error(sdkwork_drive_workspace_service::DriveServiceError::NotFound(
                "storage provider kind not found".to_string(),
            ))
        })?;
    Ok(Json(map_storage_provider_kind(summary)))
}
