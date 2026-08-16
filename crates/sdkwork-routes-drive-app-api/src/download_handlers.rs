use crate::acl;
use crate::app_context::DriveRequestContext;
use crate::collaboration_repository::enforce_share_link_download_limit_for_subject;
use crate::dto::*;
use crate::error::{
    map_download_token_error, map_service_error, service_error_kind, status_error_kind,
    ProblemDetail,
};
use crate::node_repository::find_active_node;
use crate::object_store::build_download_service;
use crate::response::{success_created_command_data, success_envelope};
use crate::route_change::record_change;
use crate::state::AppState;
use crate::time::current_epoch_ms;
use crate::validators::*;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use sdkwork_drive_contract::drive::domain_events as drive_events;
use sdkwork_drive_observability::{elapsed_ms, error_kinds, events, start_timer};
use sdkwork_drive_workspace_service::application::download_service::{
    parse_download_token_for_tenant, CreateDownloadUrlCommand, ResolveDownloadTokenCommand,
};
use sdkwork_drive_workspace_service::DriveServiceError;
use sdkwork_utils_rust::SdkWorkApiResponse;

pub(crate) async fn create_node_download_grant(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(node_id): Path<String>,
    Json(payload): Json<CreateDownloadGrantRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<CreateDownloadUrlResponse>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "reader").await?;
    enforce_share_link_download_limit_for_subject(
        &state.pool,
        &tenant_id,
        &node.space_id,
        &node_id,
        &ctx.subject_type,
        &ctx.subject_id,
    )
    .await?;
    let requested_ttl_seconds = validate_requested_ttl_seconds(
        payload.requested_ttl_seconds,
        120,
        30,
        300,
        "requestedTtlSeconds",
    )?;
    let operator_id = ctx.resolve_operator_id()?;
    let service = build_download_service(&state);
    let result = service
        .create_download_url(CreateDownloadUrlCommand {
            tenant_id: tenant_id.clone(),
            node_id: node_id.clone(),
            requested_ttl_seconds,
            request_base_url: state.download_public_base_url.clone(),
        })
        .await
        .map_err(map_service_error)?;
    record_change(
        &state.pool,
        &tenant_id,
        &node.space_id,
        Some(&node_id),
        drive_events::download_grant::CREATED,
        &operator_id,
    )
    .await?;
    Ok(success_created_command_data(CreateDownloadUrlResponse {
        download_url: result.download_url,
        signed_source_url: result.signed_source_url,
        expires_at_epoch_ms: result.expires_at_epoch_ms,
        method: result.method,
    }))
}
pub(crate) async fn create_node_download_url(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(node_id): Path<String>,
    Query(query): Query<NodeDownloadUrlQuery>,
) -> Result<Json<SdkWorkApiResponse<CreateDownloadUrlResponse>>, (StatusCode, Json<ProblemDetail>)>
{
    let tenant_id = ctx.resolve_tenant_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "reader").await?;
    enforce_share_link_download_limit_for_subject(
        &state.pool,
        &tenant_id,
        &node.space_id,
        &node_id,
        &ctx.subject_type,
        &ctx.subject_id,
    )
    .await?;
    let requested_ttl_seconds = validate_requested_ttl_seconds(
        query.requested_ttl_seconds,
        120,
        30,
        300,
        "requestedTtlSeconds",
    )?;
    let service = build_download_service(&state);
    let result = service
        .create_download_url(CreateDownloadUrlCommand {
            tenant_id,
            node_id,
            requested_ttl_seconds,
            request_base_url: state.download_public_base_url.clone(),
        })
        .await
        .map_err(map_service_error)?;
    Ok(success_envelope(CreateDownloadUrlResponse {
        download_url: result.download_url,
        signed_source_url: result.signed_source_url,
        expires_at_epoch_ms: result.expires_at_epoch_ms,
        method: result.method,
    }))
}
pub(crate) async fn create_download_url(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Json(payload): Json<CreateDownloadUrlRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<CreateDownloadUrlResponse>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let started = start_timer();
    let service = build_download_service(&state);
    let requested_ttl_seconds = validate_requested_ttl_seconds(
        payload.requested_ttl_seconds,
        120,
        30,
        300,
        "requestedTtlSeconds",
    )?;
    let tenant_id = ctx.resolve_tenant_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &payload.node_id).await?;
    acl::ensure_ctx_node_role(
        &state.pool,
        &ctx,
        &node.space_id,
        &payload.node_id,
        "reader",
    )
    .await?;
    enforce_share_link_download_limit_for_subject(
        &state.pool,
        &tenant_id,
        &node.space_id,
        &payload.node_id,
        &ctx.subject_type,
        &ctx.subject_id,
    )
    .await?;
    let result_value = service
        .create_download_url(CreateDownloadUrlCommand {
            tenant_id,
            node_id: payload.node_id,
            requested_ttl_seconds,
            request_base_url: state.download_public_base_url.clone(),
        })
        .await;
    let result = match result_value {
        Ok(result) => result,
        Err(error) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_DOWNLOAD_URLS_CREATE,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = service_error_kind(&error),
                requested_ttl_seconds = requested_ttl_seconds
            );
            return Err(map_service_error(error));
        }
    };
    let latency_ms = elapsed_ms(started);
    sdkwork_drive_observability::observe_route!(
        event = events::APP_DOWNLOAD_URLS_CREATE,
        result = "ok",
        latency_ms = latency_ms,
        requested_ttl_seconds = requested_ttl_seconds,
        expires_at_epoch_ms = result.expires_at_epoch_ms,
        method = result.method.as_str()
    );

    Ok(success_created_command_data(CreateDownloadUrlResponse {
        download_url: result.download_url,
        signed_source_url: result.signed_source_url,
        expires_at_epoch_ms: result.expires_at_epoch_ms,
        method: result.method,
    }))
}
pub(crate) async fn resolve_download_token(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(token): Path<String>,
) -> Result<Json<SdkWorkApiResponse<CreateDownloadUrlResponse>>, (StatusCode, Json<ProblemDetail>)>
{
    let started = start_timer();
    let tenant_id = ctx.resolve_tenant_id()?;
    let parsed = match parse_download_token_for_tenant(token.trim(), &tenant_id) {
        Ok(parsed) => parsed,
        Err(error) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_DOWNLOAD_TOKENS_RESOLVE,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = service_error_kind(&error),
                method = "GET"
            );
            return Err(map_download_token_error(error));
        }
    };
    if parsed.expires_at_epoch_ms <= current_epoch_ms() {
        sdkwork_drive_observability::observe_route!(
            event = events::APP_DOWNLOAD_TOKENS_RESOLVE,
            result = "err",
            latency_ms = elapsed_ms(started),
            error_kind = error_kinds::NOT_FOUND,
            method = "GET"
        );
        return Err(map_download_token_error(DriveServiceError::NotFound(
            "download token has expired".to_string(),
        )));
    }
    let node = match find_active_node(&state.pool, &tenant_id, &parsed.node_id).await {
        Ok(node) => node,
        Err((status, problem)) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_DOWNLOAD_TOKENS_RESOLVE,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = status_error_kind(status),
                method = "GET"
            );
            return Err((status, problem));
        }
    };
    if let Err(error) =
        acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &parsed.node_id, "reader")
            .await
    {
        sdkwork_drive_observability::observe_route!(
            event = events::APP_DOWNLOAD_TOKENS_RESOLVE,
            result = "err",
            latency_ms = elapsed_ms(started),
            error_kind = status_error_kind(error.0),
            method = "GET"
        );
        return Err(error);
    }
    let service = build_download_service(&state);
    let result_value = service
        .resolve_download_token(ResolveDownloadTokenCommand {
            tenant_id,
            token: token.clone(),
        })
        .await;
    let result = match result_value {
        Ok(result) => result,
        Err(error) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_DOWNLOAD_TOKENS_RESOLVE,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = service_error_kind(&error),
                method = "GET"
            );
            return Err(map_download_token_error(error));
        }
    };
    let latency_ms = elapsed_ms(started);
    sdkwork_drive_observability::observe_route!(
        event = events::APP_DOWNLOAD_TOKENS_RESOLVE,
        result = "ok",
        latency_ms = latency_ms,
        method = "GET"
    );

    let download_url = format!(
        "{}/download_tokens/{}",
        state.download_public_base_url.trim_end_matches('/'),
        token.trim()
    );
    Ok(success_envelope(CreateDownloadUrlResponse {
        download_url,
        signed_source_url: result.signed_source_url,
        expires_at_epoch_ms: result.expires_at_epoch_ms,
        method: result.method,
    }))
}
