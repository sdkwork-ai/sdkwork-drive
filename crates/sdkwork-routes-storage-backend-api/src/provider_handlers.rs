use crate::app_context::DriveRequestContext;
use crate::audit::record_storage_provider_audit;
use crate::dto::*;
use crate::error::{
    invalid_json_problem, map_object_store_route_error, map_provider_account_error,
    map_service_error, problem, ProblemDetail, SdkWorkResultCode,
};
use crate::object_store::{
    build_full_s3_object_store_for_provider, provider_supports_s3_object_store,
};
use crate::provider_lookup::get_provider;
use crate::provider_mappers::{
    map_storage_provider, map_storage_provider_capabilities, parse_storage_provider_kind,
};
use crate::response::{no_content, success_list_page_simple, StorageListHttpResponse};
use crate::state::AdminStorageState;
use crate::validators::{next_page_token, parse_offset_page};
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_contract::drive::domain_events::admin_audit;
use sdkwork_drive_storage_contract::{DriveObjectStore, HeadBucketRequest};
use sdkwork_drive_workspace_service::application::storage_provider_kind_service::DriveStorageProviderKindService;
use sdkwork_drive_workspace_service::application::storage_provider_service::{
    CreateStorageProviderCommand, DeleteStorageProviderCommand, DriveStorageProviderService,
    ListStorageProvidersCommand, RotateStorageProviderCredentialCommand,
    SetStorageProviderStatusCommand, StorageProviderCapabilitiesCommand,
    TestStorageProviderCommand, UpdateStorageProviderCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_kind_store::SqlStorageProviderKindStore;
use sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_store::SqlStorageProviderStore;
use sdkwork_drive_workspace_service::DriveServiceError;
use sdkwork_iam_provider_account_service::{ACCOUNT_SCOPE_USER, ACCOUNT_STATUS_ACTIVE};

async fn ensure_provider_kind_available(
    state: &AdminStorageState,
    provider_kind: &sdkwork_drive_workspace_service::domain::storage_provider::DriveStorageProviderKind,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    DriveStorageProviderKindService::new(SqlStorageProviderKindStore::new(state.pool.clone()))
        .ensure_storage_provider_kind_available(provider_kind)
        .await
        .map_err(map_service_error)
}

pub(crate) async fn list_storage_providers(
    State(state): State<AdminStorageState>,
    Query(query): Query<ListStorageProvidersQuery>,
) -> Result<StorageListHttpResponse<StorageProviderResponse>, (StatusCode, Json<ProblemDetail>)> {
    let page = parse_offset_page(query.page_size, query.page_token)?;
    let service =
        DriveStorageProviderService::new(SqlStorageProviderStore::new(state.pool.clone()));
    let mut items = service
        .list_storage_providers(ListStorageProvidersCommand {
            status: query.status,
            offset: page.offset,
            limit: page.limit + 1,
        })
        .await
        .map_err(map_service_error)?
        .into_iter()
        .map(map_storage_provider)
        .collect::<Vec<_>>();
    let next_page_token = next_page_token(&mut items, page);
    Ok(success_list_page_simple(items, page, next_page_token))
}

/// Check that a storage provider may be bound to this account-center account.
///
/// Two things are verified, neither of which the column CHECK constraints can
/// see:
///
/// 1. **The account exists and the caller may use it.** Previously only the
///    *shape* of `provider_account_id` was validated, so a typo or another
///    tenant's account id was accepted at write time and only surfaced later as
///    an opaque failure on the first object operation.
/// 2. **The account is not `user`-scoped.** `dr_drive_storage_provider` is a
///    tenant-level resource (it has a `tenant_id` and no owner column), while a
///    `user`-scoped account belongs to one person. Binding the two together
///    would make a tenant-wide provider depend on a personal credential, and
///    the implicit "whose account is this" answer would change whenever the
///    owner was removed.
pub(crate) async fn ensure_bindable_provider_account(
    state: &AdminStorageState,
    tenant_id: &str,
    operator_id: &str,
    provider_account_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let Some(account_id) = provider_account_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };

    let account = sdkwork_iam_provider_account_service::load_bound_account(
        &state.pool,
        tenant_id,
        Some(operator_id),
        account_id,
    )
    .await
    .map_err(map_provider_account_error)?;

    if account.scope_type == ACCOUNT_SCOPE_USER {
        return Err(problem(
            StatusCode::CONFLICT,
            "provider account scope not bindable",
            "a storage provider is a tenant-level resource and cannot be bound to a user-scoped account; publish a tenant or platform account instead",
            SdkWorkResultCode::Conflict,
        ));
    }
    if account.status != ACCOUNT_STATUS_ACTIVE {
        return Err(problem(
            StatusCode::CONFLICT,
            "provider account not active",
            format!(
                "provider account {} is {}; an active account is required to bind a storage provider",
                account.id, account.status
            ),
            SdkWorkResultCode::Conflict,
        ));
    }
    Ok(())
}

pub(crate) async fn create_storage_provider(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    payload: Result<Json<CreateStorageProviderRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<StorageProviderResponse>), (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let operator_id = ctx.resolve_operator_id()?;
    let tenant_id = ctx.resolve_tenant_id()?;
    let service =
        DriveStorageProviderService::new(SqlStorageProviderStore::new(state.pool.clone()));
    let provider_kind =
        parse_storage_provider_kind(&payload.provider_kind).map_err(map_service_error)?;
    ensure_provider_kind_available(&state, &provider_kind).await?;
    ensure_bindable_provider_account(
        &state,
        &tenant_id,
        &operator_id,
        payload.provider_account_id.as_deref(),
    )
    .await?;
    let created = service
        .create_storage_provider(CreateStorageProviderCommand {
            id: payload.id,
            tenant_id,
            provider_kind,
            name: payload.name,
            endpoint_url: payload.endpoint_url,
            region: payload.region,
            bucket: payload.bucket,
            path_style: payload.path_style,
            strict_tls: payload.strict_tls,
            credential_ref: payload.credential_ref,
            provider_account_id: payload.provider_account_id,
            server_side_encryption_mode: payload.server_side_encryption_mode,
            default_storage_class: payload.default_storage_class,
            status: payload.status,
            operator_id: operator_id.clone(),
        })
        .await
        .map_err(map_service_error)?;
    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::CREATED,
        &created.id,
        &operator_id,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(map_storage_provider(created))))
}

pub(crate) async fn get_storage_provider(
    State(state): State<AdminStorageState>,
    Path(provider_id): Path<String>,
) -> Result<Json<StorageProviderResponse>, (StatusCode, Json<ProblemDetail>)> {
    let provider = get_provider(&state, &provider_id).await?;
    Ok(Json(map_storage_provider(provider)))
}

pub(crate) async fn update_storage_provider(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_id): Path<String>,
    payload: Result<Json<UpdateStorageProviderRequest>, JsonRejection>,
) -> Result<Json<StorageProviderResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let operator_id = ctx.resolve_operator_id()?;
    let tenant_id = ctx.resolve_tenant_id()?;
    if payload
        .status
        .as_deref()
        .is_some_and(|status| status.trim().eq_ignore_ascii_case("active"))
    {
        let current = get_provider(&state, &provider_id).await?;
        ensure_provider_kind_available(&state, &current.provider_kind).await?;
    }
    ensure_bindable_provider_account(
        &state,
        &tenant_id,
        &operator_id,
        payload.provider_account_id.as_deref(),
    )
    .await?;
    let service =
        DriveStorageProviderService::new(SqlStorageProviderStore::new(state.pool.clone()));
    let updated = service
        .update_storage_provider(UpdateStorageProviderCommand {
            provider_id,
            name: payload.name,
            endpoint_url: payload.endpoint_url,
            region: payload.region,
            bucket: payload.bucket,
            path_style: payload.path_style,
            strict_tls: payload.strict_tls,
            credential_ref: payload.credential_ref,
            provider_account_id: payload.provider_account_id,
            server_side_encryption_mode: payload.server_side_encryption_mode,
            default_storage_class: payload.default_storage_class,
            status: payload.status,
            operator_id: operator_id.clone(),
        })
        .await
        .map_err(map_service_error)?;
    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::UPDATED,
        &updated.id,
        &operator_id,
    )
    .await?;
    Ok(Json(map_storage_provider(updated)))
}

pub(crate) async fn delete_storage_provider(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let operator_id = ctx.resolve_operator_id()?;
    let service =
        DriveStorageProviderService::new(SqlStorageProviderStore::new(state.pool.clone()));
    let deleted = service
        .delete_storage_provider(DeleteStorageProviderCommand {
            provider_id: provider_id.clone(),
            operator_id: operator_id.clone(),
        })
        .await
        .map_err(map_service_error)?;
    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::DELETED,
        &provider_id,
        &operator_id,
    )
    .await?;
    let _deleted = deleted.deleted;
    Ok(no_content())
}

pub(crate) async fn get_storage_provider_capabilities(
    State(state): State<AdminStorageState>,
    Path(provider_id): Path<String>,
) -> Result<Json<StorageProviderCapabilitiesResponse>, (StatusCode, Json<ProblemDetail>)> {
    let service =
        DriveStorageProviderService::new(SqlStorageProviderStore::new(state.pool.clone()));
    let capabilities = service
        .get_storage_provider_capabilities(StorageProviderCapabilitiesCommand { provider_id })
        .await
        .map_err(map_service_error)?;
    Ok(Json(map_storage_provider_capabilities(capabilities)))
}

pub(crate) async fn test_storage_provider(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_id): Path<String>,
) -> Result<Json<TestStorageProviderResponse>, (StatusCode, Json<ProblemDetail>)> {
    let operator_id = ctx.resolve_operator_id()?;
    let provider = get_provider(&state, &provider_id).await?;
    if provider.status == "deleted" {
        return Err(map_service_error(DriveServiceError::Conflict(
            "deleted storage provider cannot be tested".to_string(),
        )));
    }
    let reachable = if provider_supports_s3_object_store(&provider.provider_kind) {
        let object_store = build_full_s3_object_store_for_provider(&state, &provider).await?;
        object_store
            .head_bucket(HeadBucketRequest {
                bucket: provider.bucket.clone(),
            })
            .await
            .map_err(map_object_store_route_error)?;
        true
    } else {
        let service =
            DriveStorageProviderService::new(SqlStorageProviderStore::new(state.pool.clone()));
        service
            .test_storage_provider(TestStorageProviderCommand {
                provider_id: provider_id.clone(),
            })
            .await
            .map_err(map_service_error)?
            .reachable
    };
    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::TESTED,
        &provider_id,
        &operator_id,
    )
    .await?;
    Ok(Json(TestStorageProviderResponse {
        provider_id: provider.id,
        reachable,
    }))
}

pub(crate) async fn activate_storage_provider(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_id): Path<String>,
) -> Result<Json<StorageProviderResponse>, (StatusCode, Json<ProblemDetail>)> {
    let operator_id = ctx.resolve_operator_id()?;
    set_storage_provider_status(state, provider_id, operator_id, "active").await
}

pub(crate) async fn deactivate_storage_provider(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_id): Path<String>,
) -> Result<Json<StorageProviderResponse>, (StatusCode, Json<ProblemDetail>)> {
    let operator_id = ctx.resolve_operator_id()?;
    set_storage_provider_status(state, provider_id, operator_id, "disabled").await
}

pub(crate) async fn rotate_storage_provider_credentials(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_id): Path<String>,
    payload: Result<Json<RotateStorageProviderCredentialRequest>, JsonRejection>,
) -> Result<Json<StorageProviderResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let operator_id = ctx.resolve_operator_id()?;
    let service =
        DriveStorageProviderService::new(SqlStorageProviderStore::new(state.pool.clone()));
    let updated = service
        .rotate_storage_provider_credential(RotateStorageProviderCredentialCommand {
            provider_id: provider_id.clone(),
            credential_ref: payload.credential_ref,
            operator_id: operator_id.clone(),
        })
        .await
        .map_err(map_service_error)?;
    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::CREDENTIALS_ROTATED,
        &provider_id,
        &operator_id,
    )
    .await?;
    Ok(Json(map_storage_provider(updated)))
}

pub(crate) async fn set_storage_provider_status(
    state: AdminStorageState,
    provider_id: String,
    operator_id: String,
    status: &str,
) -> Result<Json<StorageProviderResponse>, (StatusCode, Json<ProblemDetail>)> {
    if status == "active" {
        let current = get_provider(&state, &provider_id).await?;
        ensure_provider_kind_available(&state, &current.provider_kind).await?;
    }
    let service =
        DriveStorageProviderService::new(SqlStorageProviderStore::new(state.pool.clone()));
    let updated = service
        .set_storage_provider_status(SetStorageProviderStatusCommand {
            provider_id: provider_id.clone(),
            status: status.to_string(),
            operator_id: operator_id.clone(),
        })
        .await
        .map_err(map_service_error)?;
    let action = match status {
        "active" => admin_audit::storage_provider::ACTIVATED,
        "disabled" => admin_audit::storage_provider::DEACTIVATED,
        _ => admin_audit::storage_provider::STATUS_CHANGED,
    };
    record_storage_provider_audit(&state, action, &provider_id, &operator_id).await?;
    Ok(Json(map_storage_provider(updated)))
}
