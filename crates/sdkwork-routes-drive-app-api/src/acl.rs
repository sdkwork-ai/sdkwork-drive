use crate::app_context::DriveRequestContext;
use crate::dto::{ChangeResponse, DriveWatchChannelResponse, PageRequest};
use crate::error::{
    internal_sql_error, map_service_error, problem, ProblemDetail, SdkWorkResultCode,
};
use crate::node_repository::find_node;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_contract::api::pagination_cursor::{
    encode_change_sequence_cursor, encode_offset_cursor,
};
use sdkwork_drive_workspace_service::application::permission_service::SqlDrivePermissionService;
use sdkwork_drive_workspace_service::ports::permission_store::{
    DriveEffectiveNodeAccess, ResolveEffectiveNodeAccessCommand,
};
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::postgres::PgRow;
use sqlx::PgPool;
use sqlx::Row;
use std::collections::{BTreeSet, HashMap};
use std::future::Future;

struct ReaderPaginationAclCache {
    space_owner: HashMap<String, bool>,
}

impl ReaderPaginationAclCache {
    fn new() -> Self {
        Self {
            space_owner: HashMap::new(),
        }
    }

    async fn is_space_owner(
        &mut self,
        pool: &PgPool,
        tenant_id: &str,
        space_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<bool, (StatusCode, Json<ProblemDetail>)> {
        if let Some(is_owner) = self.space_owner.get(space_id) {
            return Ok(*is_owner);
        }
        let service = SqlDrivePermissionService::new(pool.clone());
        let is_owner = service
            .is_space_owner(tenant_id, space_id, subject_type, subject_id)
            .await
            .map_err(map_service_error)?;
        self.space_owner.insert(space_id.to_string(), is_owner);
        Ok(is_owner)
    }
}

pub async fn is_subject_space_owner(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    subject_type: &str,
    subject_id: &str,
) -> Result<bool, (StatusCode, Json<ProblemDetail>)> {
    let mut cache = ReaderPaginationAclCache::new();
    cache
        .is_space_owner(pool, tenant_id, space_id, subject_type, subject_id)
        .await
}

pub async fn paginate_offset_limited_items<T, F, Fut>(
    page: crate::dto::PageRequest,
    mut fetch_batch: F,
    map_row: fn(&PgRow) -> T,
) -> Result<(Vec<T>, Option<String>), (StatusCode, Json<ProblemDetail>)>
where
    F: FnMut(i64, usize) -> Fut,
    Fut: Future<Output = Result<Vec<PgRow>, (StatusCode, Json<ProblemDetail>)>>,
{
    let batch_limit = (page.limit + 1) as usize;
    let rows = fetch_batch(page.offset, batch_limit).await?;
    let has_more = rows.len() > page.limit as usize;
    let items = rows
        .iter()
        .take(page.limit as usize)
        .map(map_row)
        .collect::<Vec<_>>();
    let next_page_token = if has_more {
        encode_offset_cursor(page.offset + page.limit)
    } else {
        None
    };
    Ok((items, next_page_token))
}

pub fn permission_denied_problem() -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::FORBIDDEN,
        "permission denied",
        "subject does not have required access to the drive node",
        SdkWorkResultCode::PermissionRequired,
    )
}

pub async fn ensure_subject_role(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    node_id: &str,
    subject_type: &str,
    subject_id: &str,
    required_role: &str,
) -> Result<DriveEffectiveNodeAccess, (StatusCode, Json<ProblemDetail>)> {
    let service = SqlDrivePermissionService::new(pool.clone());
    let access = service
        .resolve_effective_node_access(ResolveEffectiveNodeAccessCommand {
            tenant_id: tenant_id.to_string(),
            space_id: space_id.to_string(),
            node_id: node_id.to_string(),
            subject_type: subject_type.to_string(),
            subject_id: subject_id.to_string(),
        })
        .await
        .map_err(map_service_error)?;
    if !access.allows_role(required_role) {
        return Err(permission_denied_problem());
    }
    Ok(access)
}

pub async fn ensure_ctx_node_role(
    pool: &PgPool,
    ctx: &DriveRequestContext,
    space_id: &str,
    node_id: &str,
    required_role: &str,
) -> Result<DriveEffectiveNodeAccess, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    ensure_subject_role(
        pool,
        &tenant_id,
        space_id,
        node_id,
        &subject_type,
        &subject_id,
        required_role,
    )
    .await
}

pub async fn ensure_list_parent_reader(
    pool: &PgPool,
    ctx: &DriveRequestContext,
    space_id: &str,
    parent_node_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    ensure_subject_space_scoped_reader(
        pool,
        &tenant_id,
        space_id,
        parent_node_id,
        &subject_type,
        &subject_id,
    )
    .await
}

pub async fn ensure_subject_space_scoped_reader(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    parent_node_id: Option<&str>,
    subject_type: &str,
    subject_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let service = SqlDrivePermissionService::new(pool.clone());
    if service
        .is_space_owner(tenant_id, space_id, subject_type, subject_id)
        .await
        .map_err(map_service_error)?
    {
        return Ok(());
    }

    let anchor_node_id = match parent_node_id {
        Some(parent_id) => parent_id.to_string(),
        None => match service
            .resolve_space_permission_anchor_node(tenant_id, space_id)
            .await
        {
            Ok(anchor_node_id) => anchor_node_id,
            Err(DriveServiceError::NotFound(_)) => return Err(permission_denied_problem()),
            Err(error) => return Err(map_service_error(error)),
        },
    };

    if ensure_subject_role(
        pool,
        tenant_id,
        space_id,
        &anchor_node_id,
        subject_type,
        subject_id,
        "reader",
    )
    .await
    .is_ok()
    {
        return Ok(());
    }

    if parent_node_id.is_none()
        && subject_has_any_space_permission_grant(
            pool,
            tenant_id,
            space_id,
            subject_type,
            subject_id,
        )
        .await?
    {
        return Ok(());
    }

    Err(permission_denied_problem())
}

async fn subject_has_any_space_permission_grant(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    subject_type: &str,
    subject_id: &str,
) -> Result<bool, (StatusCode, Json<ProblemDetail>)> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(1)
         FROM dr_drive_node_permission p
         INNER JOIN dr_drive_node n
            ON n.tenant_id = p.tenant_id
           AND n.id = p.node_id
         WHERE p.tenant_id = $1
           AND n.space_id = $2
           AND p.lifecycle_status = 'active'
           AND p.subject_type = $3
           AND p.subject_id = $4",
    )
    .bind(tenant_id)
    .bind(space_id)
    .bind(subject_type)
    .bind(subject_id)
    .fetch_one(pool)
    .await
    .map_err(internal_sql_error(
        "count dr_drive_node_permission grants for space scope failed",
    ))?;
    Ok(count > 0)
}

pub async fn ensure_space_owner(
    pool: &PgPool,
    ctx: &DriveRequestContext,
    space_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    let service = SqlDrivePermissionService::new(pool.clone());
    if service
        .is_space_owner(&tenant_id, space_id, &subject_type, &subject_id)
        .await
        .map_err(map_service_error)?
    {
        return Ok(());
    }

    let anchor_node_id = match service
        .resolve_space_permission_anchor_node(&tenant_id, space_id)
        .await
    {
        Ok(anchor_node_id) => anchor_node_id,
        Err(DriveServiceError::NotFound(_)) => return Err(permission_denied_problem()),
        Err(error) => return Err(map_service_error(error)),
    };
    ensure_subject_role(
        pool,
        &tenant_id,
        space_id,
        &anchor_node_id,
        &subject_type,
        &subject_id,
        "owner",
    )
    .await?;
    Ok(())
}

pub async fn ensure_parent_writer(
    pool: &PgPool,
    ctx: &DriveRequestContext,
    space_id: &str,
    parent_node_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    let service = SqlDrivePermissionService::new(pool.clone());
    if service
        .is_space_owner(&tenant_id, space_id, &subject_type, &subject_id)
        .await
        .map_err(map_service_error)?
    {
        return Ok(());
    }
    let anchor_node_id = match parent_node_id {
        Some(parent_id) => parent_id.to_string(),
        None => SqlDrivePermissionService::new(pool.clone())
            .resolve_space_permission_anchor_node(&tenant_id, space_id)
            .await
            .map_err(map_service_error)?,
    };
    ensure_subject_role(
        pool,
        &tenant_id,
        space_id,
        &anchor_node_id,
        &subject_type,
        &subject_id,
        "writer",
    )
    .await?;
    Ok(())
}

async fn is_space_owner_any_lifecycle(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    subject_type: &str,
    subject_id: &str,
) -> Result<bool, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT owner_subject_type, owner_subject_id
         FROM dr_drive_space
         WHERE tenant_id = $1 AND id = $2",
    )
    .bind(tenant_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("read dr_drive_space owner failed"))?;
    Ok(row.is_some_and(|row| {
        row.get::<String, _>("owner_subject_type") == subject_type
            && row.get::<String, _>("owner_subject_id") == subject_id
    }))
}

pub async fn ensure_space_change_feed_reader(
    pool: &PgPool,
    ctx: &DriveRequestContext,
    space_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    if is_space_owner_any_lifecycle(pool, &tenant_id, space_id, &subject_type, &subject_id).await? {
        return Ok(());
    }
    ensure_list_parent_reader(pool, ctx, space_id, None).await
}

pub async fn ensure_watch_channel_role(
    pool: &PgPool,
    ctx: &DriveRequestContext,
    channel: &DriveWatchChannelResponse,
    required_role: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    if let Some(node_id) = channel.node_id.as_deref() {
        let node = find_node(pool, &tenant_id, node_id).await?;
        ensure_ctx_node_role(pool, ctx, &node.space_id, node_id, required_role).await?;
        return Ok(());
    }
    let space_id = channel.space_id.as_deref().ok_or_else(|| {
        problem(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error",
            "watch channel is missing space scope",
            SdkWorkResultCode::InternalError,
        )
    })?;
    if required_role == "reader" || required_role == "commenter" {
        ensure_list_parent_reader(pool, ctx, space_id, None).await
    } else {
        ensure_parent_writer(pool, ctx, space_id, None).await
    }
}

pub async fn ensure_node_ids_role(
    pool: &PgPool,
    ctx: &DriveRequestContext,
    node_ids: &[String],
    required_role: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let mut seen = BTreeSet::new();
    for node_id in node_ids {
        if !seen.insert(node_id.as_str()) {
            continue;
        }
        let node = find_node(pool, &tenant_id, node_id).await?;
        ensure_ctx_node_role(pool, ctx, &node.space_id, node_id, required_role).await?;
    }
    Ok(())
}

pub async fn paginate_cursor_limited_changes<F, Fut>(
    page: PageRequest,
    mut fetch_batch: F,
) -> Result<(Vec<ChangeResponse>, Option<String>), (StatusCode, Json<ProblemDetail>)>
where
    F: FnMut(i64, usize) -> Fut,
    Fut: Future<Output = Result<Vec<ChangeResponse>, (StatusCode, Json<ProblemDetail>)>>,
{
    let batch_limit = (page.limit + 1) as usize;
    let items = fetch_batch(page.offset, batch_limit).await?;
    let has_more = items.len() > page.limit as usize;
    let mut items = items;
    let next_page_token = if has_more {
        items.truncate(page.limit as usize);
        items
            .last()
            .and_then(|item| encode_change_sequence_cursor(item.sequence_no))
    } else {
        None
    };
    Ok((items, next_page_token))
}
