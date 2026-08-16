use crate::acl;
use crate::app_context::DriveRequestContext;
use crate::collaboration_repository::{
    find_active_share_link_by_token_for_tenant, find_share_link,
};
use crate::dto::{
    apply_optional_i64_patch, ClaimShareLinkResponse, CreateShareLinkRequest,
    CreateShareLinkResponse, NodeMutationQuery, PageQuery, ShareLinkResponse,
    UpdateShareLinkRequest,
};
use crate::error::{
    internal_problem, internal_sql_error, is_unique_constraint_error, map_service_error,
    not_found_problem, problem, ProblemDetail, SdkWorkResultCode,
};
use crate::mappers::map_share_link_row;
use crate::node_repository::find_active_node;
use crate::response::{
    current_trace_id, no_content, success_created_command_data, success_list_page_simple,
    success_resource, DriveListHttpResponse,
};
use crate::route_change::record_change;
use crate::state::AppState;
use crate::validators::{
    next_page_token, parse_page_request, validate_optional_future_epoch_ms,
    validate_optional_non_negative_i64, validate_share_link_role, validate_share_link_token,
    validate_subject_type,
};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_contract::drive::domain_events as drive_events;
use sdkwork_drive_workspace_service::application::permission_service::SqlDrivePermissionService;
use sdkwork_drive_workspace_service::drive_share_token_hash;
use sdkwork_drive_workspace_service::ports::permission_store::{
    GrantDriveNodePermissionCommand, ResolveEffectiveNodeAccessCommand,
};
use sdkwork_drive_workspace_service::{
    drive_share_access_code_hash, generate_share_link_token, validate_share_link_access_code,
};
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};

pub(crate) async fn list_share_links(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(node_id): Path<String>,
    Query(query): Query<PageQuery>,
) -> Result<DriveListHttpResponse<ShareLinkResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let page = parse_page_request(query.page_size, query.page_token)?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "reader").await?;
    let rows = sqlx::query(
        "SELECT id, tenant_id, node_id, role, expires_at_epoch_ms, download_limit,
                download_count, access_code_hash, lifecycle_status, version
         FROM dr_drive_node_share_link
         WHERE tenant_id=$1 AND node_id=$2 AND lifecycle_status='active'
         ORDER BY created_at DESC
         LIMIT $3 OFFSET $4",
    )
    .bind(&tenant_id)
    .bind(&node_id)
    .bind(page.limit + 1)
    .bind(page.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_sql_error("list dr_drive_node_share_link failed"))?;
    let mut items = rows.iter().map(map_share_link_row).collect::<Vec<_>>();
    let next_page_token = next_page_token(&mut items, page);

    Ok(success_list_page_simple(items, page, next_page_token))
}

pub(crate) async fn get_share_link(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(share_link_id): Path<String>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<ShareLinkResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let current = find_share_link(&state.pool, &tenant_id, &share_link_id).await?;
    let node = find_active_node(&state.pool, &tenant_id, &current.node_id).await?;
    acl::ensure_ctx_node_role(
        &state.pool,
        &ctx,
        &node.space_id,
        &current.node_id,
        "reader",
    )
    .await?;
    Ok(success_resource(ShareLinkResponse::from(current)))
}

pub(crate) async fn create_share_link(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(node_id): Path<String>,
    Json(payload): Json<CreateShareLinkRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<CreateShareLinkResponse>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;

    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "owner").await?;
    let token = {
        let trimmed = payload.token.trim();
        if trimmed.is_empty() {
            generate_share_link_token()
        } else {
            validate_share_link_token(trimmed)?;
            trimmed.to_string()
        }
    };
    let token_hash = drive_share_token_hash(&token);
    let role = payload
        .role
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "reader".to_string());
    validate_share_link_role(&role)?;
    validate_optional_future_epoch_ms(payload.expires_at_epoch_ms, "expiresAtEpochMs")?;
    validate_optional_non_negative_i64(payload.download_limit, "downloadLimit")?;
    let access_code_plain = payload
        .access_code
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    if let Some(access_code) = access_code_plain.as_deref() {
        validate_share_link_access_code(access_code).map_err(map_service_error)?;
    }
    let access_code_hash = access_code_plain
        .as_deref()
        .map(drive_share_access_code_hash);
    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, access_code_hash, role, expires_at_epoch_ms, download_limit,
            download_count, lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, 'active', 1, $9, $9)",
    )
    .bind(&payload.id)
    .bind(&tenant_id)
    .bind(&node_id)
    .bind(token_hash)
    .bind(access_code_hash)
    .bind(role)
    .bind(payload.expires_at_epoch_ms)
    .bind(payload.download_limit)
    .bind(&operator_id)
    .execute(&state.pool)
    .await
    .map_err(|error| {
        if is_unique_constraint_error(&error) {
            return problem(
                StatusCode::CONFLICT,
                "conflict",
                "share token already exists",
                SdkWorkResultCode::Conflict,
            );
        }
        internal_problem(format!("insert dr_drive_node_share_link failed: {error}"))
    })?;

    record_change(
        &state.pool,
        &tenant_id,
        &node.space_id,
        Some(&node_id),
        drive_events::share_link::CREATED,
        &operator_id,
    )
    .await?;

    let row = sqlx::query(
        "SELECT id, tenant_id, node_id, role, expires_at_epoch_ms, download_limit,
                download_count, access_code_hash, lifecycle_status, version
         FROM dr_drive_node_share_link
         WHERE tenant_id=$1 AND id=$2",
    )
    .bind(&tenant_id)
    .bind(&payload.id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_sql_error("read dr_drive_node_share_link failed"))?;
    let response = map_share_link_row(&row);
    Ok(success_created_command_data(CreateShareLinkResponse {
        link: response,
        token,
        access_code: access_code_plain,
    }))
}

pub(crate) async fn update_share_link(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(share_link_id): Path<String>,
    Json(payload): Json<UpdateShareLinkRequest>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<ShareLinkResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let current = find_share_link(&state.pool, &tenant_id, &share_link_id).await?;
    let node = find_active_node(&state.pool, &tenant_id, &current.node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &current.node_id, "owner").await?;
    let role = payload
        .role
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or(current.role.clone());
    validate_share_link_role(&role)?;
    let expires_at_epoch_ms =
        apply_optional_i64_patch(payload.expires_at_epoch_ms, current.expires_at_epoch_ms);
    let download_limit = apply_optional_i64_patch(payload.download_limit, current.download_limit);
    validate_optional_future_epoch_ms(expires_at_epoch_ms, "expiresAtEpochMs")?;
    validate_optional_non_negative_i64(download_limit, "downloadLimit")?;

    let affected = sqlx::query(
        "UPDATE dr_drive_node_share_link
         SET role=$1, expires_at_epoch_ms=$2, download_limit=$3,
             updated_by=$4, updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$5 AND id=$6 AND lifecycle_status='active'",
    )
    .bind(&role)
    .bind(expires_at_epoch_ms)
    .bind(download_limit)
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&share_link_id)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("update dr_drive_node_share_link failed"))?
    .rows_affected();
    if affected == 0 {
        return Err(not_found_problem("share link not found"));
    }
    record_change(
        &state.pool,
        &tenant_id,
        &node.space_id,
        Some(&current.node_id),
        drive_events::share_link::UPDATED,
        &operator_id,
    )
    .await?;

    let updated = find_share_link(&state.pool, &tenant_id, &share_link_id).await?;
    Ok(success_resource(ShareLinkResponse::from(updated)))
}

pub(crate) async fn revoke_share_link(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(share_link_id): Path<String>,
    Query(_query): Query<NodeMutationQuery>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let current = find_share_link(&state.pool, &tenant_id, &share_link_id).await?;
    let node = find_active_node(&state.pool, &tenant_id, &current.node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &current.node_id, "owner").await?;
    let affected = sqlx::query(
        "UPDATE dr_drive_node_share_link
         SET lifecycle_status='deleted', updated_by=$1, updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$2 AND id=$3 AND lifecycle_status='active'",
    )
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&share_link_id)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("revoke dr_drive_node_share_link failed"))?
    .rows_affected();
    if affected > 0 {
        record_change(
            &state.pool,
            &tenant_id,
            &node.space_id,
            Some(&current.node_id),
            drive_events::share_link::REVOKED,
            &operator_id,
        )
        .await?;
    }
    let _revoked = affected > 0;
    Ok(no_content())
}

pub(crate) async fn claim_share_link(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(token): Path<String>,
) -> Result<
    (StatusCode, Json<SdkWorkApiResponse<ClaimShareLinkResponse>>),
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    validate_subject_type(&subject_type)?;
    let share_link =
        find_active_share_link_by_token_for_tenant(&state.pool, &tenant_id, &token).await?;
    validate_share_link_role(&share_link.role)?;
    let node = find_active_node(&state.pool, &tenant_id, &share_link.node_id).await?;
    let claim_role = share_link_claim_grant_role(&share_link.role);
    let permission_service = SqlDrivePermissionService::new(state.pool.clone());
    let before_access = permission_service
        .resolve_effective_node_access(ResolveEffectiveNodeAccessCommand {
            tenant_id: tenant_id.clone(),
            space_id: node.space_id.clone(),
            node_id: share_link.node_id.clone(),
            subject_type: subject_type.clone(),
            subject_id: subject_id.clone(),
        })
        .await
        .map_err(map_service_error)?;
    let already_claimed = before_access.allows_role(claim_role);
    let grant = permission_service
        .grant_node_permission(GrantDriveNodePermissionCommand {
            tenant_id: tenant_id.clone(),
            node_id: share_link.node_id.clone(),
            subject_type: subject_type.clone(),
            subject_id: subject_id.clone(),
            role: claim_role.to_string(),
            operator_id: share_link.created_by.clone(),
        })
        .await
        .map_err(map_service_error)?;
    if !already_claimed {
        record_change(
            &state.pool,
            &tenant_id,
            &node.space_id,
            Some(&share_link.node_id),
            drive_events::share_link::CLAIMED,
            &subject_id,
        )
        .await?;
    }
    Ok((
        if already_claimed {
            StatusCode::OK
        } else {
            StatusCode::CREATED
        },
        Json(SdkWorkApiResponse::success(
            ClaimShareLinkResponse {
                share_link_id: share_link.id,
                node_id: share_link.node_id,
                space_id: node.space_id,
                role: claim_role.to_string(),
                permission_id: grant.id,
                already_claimed,
            },
            current_trace_id(),
        )),
    ))
}

fn share_link_claim_grant_role(link_role: &str) -> &'static str {
    match link_role {
        "commenter" => "commenter",
        "writer" => "writer",
        _ => "reader",
    }
}
