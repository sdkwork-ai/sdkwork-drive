use crate::dto::{CommentRecord, CommentReplyRecord, ShareLinkRecord};
use crate::error::{
    internal_sql_error, not_found_problem, problem, share_link_download_limit_problem,
    share_link_expired_problem, ProblemDetail, SdkWorkResultCode,
};
use crate::mappers::{map_comment_reply_row, map_comment_row, map_share_link_record};
use crate::time::current_epoch_ms;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_workspace_service::drive_share_token_hash;
use sqlx::PgPool;
use sqlx::Row;

pub(crate) async fn find_comment(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
    comment_id: &str,
) -> Result<CommentRecord, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT id, tenant_id, node_id, content, anchor, resolved, lifecycle_status,
                version, created_by, updated_by, CAST(created_at AS TEXT) AS created_at, CAST(updated_at AS TEXT) AS updated_at
         FROM dr_drive_node_comment
         WHERE tenant_id=$1 AND node_id=$2 AND id=$3 AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(node_id)
    .bind(comment_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("find dr_drive_node_comment failed"))?;
    let Some(row) = row else {
        return Err(not_found_problem("comment not found"));
    };
    Ok(map_comment_row(&row))
}

pub(crate) async fn find_comment_reply(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
    comment_id: &str,
    reply_id: &str,
) -> Result<CommentReplyRecord, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT id, tenant_id, node_id, comment_id, content, lifecycle_status,
                version, created_by, updated_by, CAST(created_at AS TEXT) AS created_at, CAST(updated_at AS TEXT) AS updated_at
         FROM dr_drive_node_comment_reply
         WHERE tenant_id=$1
           AND node_id=$2
           AND comment_id=$3
           AND id=$4
           AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(node_id)
    .bind(comment_id)
    .bind(reply_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error(
        "find dr_drive_node_comment_reply failed",
    ))?;
    let Some(row) = row else {
        return Err(not_found_problem("comment reply not found"));
    };
    Ok(map_comment_reply_row(&row))
}

pub(crate) async fn find_share_link(
    pool: &PgPool,
    tenant_id: &str,
    share_link_id: &str,
) -> Result<ShareLinkRecord, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT id, tenant_id, node_id, role, expires_at_epoch_ms, download_limit,
                download_count, access_code_hash, lifecycle_status, version, created_by
         FROM dr_drive_node_share_link
         WHERE tenant_id=$1 AND id=$2 AND lifecycle_status != 'deleted'",
    )
    .bind(tenant_id)
    .bind(share_link_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("find dr_drive_node_share_link failed"))?;
    let Some(row) = row else {
        return Err(not_found_problem("share link not found"));
    };
    Ok(map_share_link_record(&row))
}

pub(crate) async fn find_active_share_link_by_token_for_tenant(
    pool: &PgPool,
    tenant_id: &str,
    token: &str,
) -> Result<ShareLinkRecord, (StatusCode, Json<ProblemDetail>)> {
    let token_hash = drive_share_token_hash(token.trim());
    let row = sqlx::query(
        "SELECT id, tenant_id, node_id, role, expires_at_epoch_ms, download_limit,
                download_count, access_code_hash, lifecycle_status, version, created_by
         FROM dr_drive_node_share_link
         WHERE tenant_id=$1 AND token_hash=$2 AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(token_hash)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error(
        "find dr_drive_node_share_link by tenant and token failed",
    ))?;
    let Some(row) = row else {
        return Err(not_found_problem("share link not found"));
    };
    let share_link = map_share_link_record(&row);
    if let Some(expires_at_epoch_ms) = share_link.expires_at_epoch_ms {
        if expires_at_epoch_ms <= current_epoch_ms() {
            return Err(share_link_expired_problem());
        }
    }
    Ok(share_link)
}

pub(crate) async fn claim_share_link_download_slot(
    pool: &PgPool,
    share_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let now_epoch_ms = current_epoch_ms();
    let result = sqlx::query(
        "UPDATE dr_drive_node_share_link
         SET download_count=download_count + 1, updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE id=$1
           AND lifecycle_status='active'
           AND (expires_at_epoch_ms IS NULL OR expires_at_epoch_ms > $2)
           AND (download_limit IS NULL OR download_count < download_limit)",
    )
    .bind(share_id)
    .bind(now_epoch_ms)
    .execute(pool)
    .await
    .map_err(internal_sql_error(
        "claim dr_drive_node_share_link download slot failed",
    ))?;

    if result.rows_affected() == 1 {
        return Ok(());
    }

    map_share_link_download_slot_failure(pool, share_id, now_epoch_ms).await
}

async fn map_share_link_download_slot_failure(
    pool: &PgPool,
    share_id: &str,
    now_epoch_ms: i64,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT lifecycle_status, expires_at_epoch_ms, download_limit, download_count
         FROM dr_drive_node_share_link
         WHERE id=$1
         LIMIT 1",
    )
    .bind(share_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error(
        "query dr_drive_node_share_link download slot failure failed",
    ))?;

    let Some(row) = row else {
        return Err(not_found_problem("share link not found"));
    };

    let lifecycle_status: String = row.get("lifecycle_status");
    if lifecycle_status != "active" {
        return Err(not_found_problem("share link not found"));
    }
    let expires_at_epoch_ms: Option<i64> = row.get("expires_at_epoch_ms");
    if expires_at_epoch_ms.is_some_and(|expires_at| expires_at <= now_epoch_ms) {
        return Err(share_link_expired_problem());
    }
    let download_limit: Option<i64> = row.get("download_limit");
    let download_count: i64 = row.get("download_count");
    if download_limit.is_some_and(|limit| download_count >= limit) {
        return Err(share_link_download_limit_problem());
    }

    Err(problem(
        StatusCode::CONFLICT,
        "conflict",
        "share link download slot changed concurrently",
        SdkWorkResultCode::Conflict,
    ))
}

pub(crate) async fn enforce_share_link_download_limit_for_subject(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    node_id: &str,
    subject_type: &str,
    subject_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    use sdkwork_drive_workspace_service::application::permission_service::SqlDrivePermissionService;

    let permission_service = SqlDrivePermissionService::new(pool.clone());
    if permission_service
        .is_space_owner(tenant_id, space_id, subject_type, subject_id)
        .await
        .map_err(crate::error::map_service_error)?
    {
        return Ok(());
    }

    let share_link_id: Option<String> = sqlx::query_scalar(
        "SELECT sl.id
         FROM dr_drive_node_share_link sl
         INNER JOIN dr_drive_node_permission p
           ON p.tenant_id = sl.tenant_id
          AND p.node_id = sl.node_id
          AND p.subject_type = $3
          AND p.subject_id = $4
          AND p.lifecycle_status = 'active'
         WHERE sl.tenant_id = $1
           AND sl.node_id = $2
           AND sl.lifecycle_status = 'active'
           AND sl.download_limit IS NOT NULL
         ORDER BY sl.created_at DESC, sl.id ASC
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(node_id)
    .bind(subject_type)
    .bind(subject_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error(
        "resolve share link download limit for subject failed",
    ))?;

    let Some(share_link_id) = share_link_id else {
        return Ok(());
    };

    claim_share_link_download_slot(pool, &share_link_id).await
}
