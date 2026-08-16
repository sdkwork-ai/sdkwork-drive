use crate::dto::{OpenNodeResponse, OpenShareLinkResponse};
use crate::error::{
    internal_sql_error, problem, share_link_access_code_problem, ProblemDetail, SdkWorkResultCode,
};
use crate::time::now_epoch_ms;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_workspace_service::{drive_share_token_hash, share_link_access_code_matches};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

pub(crate) async fn find_active_share_link(
    pool: &PgPool,
    token: &str,
    access_code: Option<&str>,
) -> Result<PgRow, (StatusCode, Json<ProblemDetail>)> {
    let token_hash = drive_share_token_hash(token);
    let row = sqlx::query(
        "SELECT
            sl.id AS share_id,
            sl.tenant_id AS share_tenant_id,
            sl.role,
            sl.expires_at_epoch_ms,
            sl.download_limit,
            sl.download_count,
            sl.access_code_hash,
            n.id AS node_id,
            n.tenant_id AS node_tenant_id,
            n.space_id,
            n.node_type,
            n.node_name,
            so.storage_provider_id,
            so.bucket,
            so.object_key,
            so.content_type,
            so.content_length
         FROM dr_drive_node_share_link sl
         JOIN dr_drive_node n ON n.id = sl.node_id AND n.tenant_id = sl.tenant_id
         LEFT JOIN dr_drive_storage_object so
           ON so.tenant_id = n.tenant_id
          AND so.node_id = n.id
          AND so.lifecycle_status='active'
          AND so.version_no = (
              SELECT MAX(version_no)
              FROM dr_drive_storage_object
              WHERE tenant_id=n.tenant_id AND node_id=n.id AND lifecycle_status='active'
          )
         WHERE sl.token_hash=$1
           AND sl.lifecycle_status='active'
           AND n.lifecycle_status='active'
         LIMIT 1",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("query dr_drive_node_share_link failed"))?;
    let Some(row) = row else {
        return Err(problem(
            StatusCode::NOT_FOUND,
            "not found",
            "share link not found",
            SdkWorkResultCode::NotFound,
        ));
    };

    let expires_at_epoch_ms: Option<i64> = row.get("expires_at_epoch_ms");
    if let Some(expires_at_epoch_ms) = expires_at_epoch_ms {
        if expires_at_epoch_ms <= now_epoch_ms() {
            return Err(problem(
                StatusCode::GONE,
                "share link expired",
                "share link expired",
                SdkWorkResultCode::Gone,
            ));
        }
    }

    let access_code_hash: Option<String> = row.try_get("access_code_hash").ok().flatten();
    if !share_link_access_code_matches(access_code_hash.as_deref(), access_code) {
        return Err(share_link_access_code_problem());
    }

    Ok(row)
}

pub(crate) async fn claim_share_link_download_slot(
    pool: &PgPool,
    share_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let now_epoch_ms = now_epoch_ms();
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

    map_share_link_claim_failure(pool, share_id, now_epoch_ms).await
}

async fn map_share_link_claim_failure(
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
        return Err(problem(
            StatusCode::NOT_FOUND,
            "not found",
            "share link not found",
            SdkWorkResultCode::NotFound,
        ));
    };

    let lifecycle_status: String = row.get("lifecycle_status");
    if lifecycle_status != "active" {
        return Err(problem(
            StatusCode::NOT_FOUND,
            "not found",
            "share link not found",
            SdkWorkResultCode::NotFound,
        ));
    }
    let expires_at_epoch_ms: Option<i64> = row.get("expires_at_epoch_ms");
    if let Some(expires_at_epoch_ms) = expires_at_epoch_ms {
        if expires_at_epoch_ms <= now_epoch_ms {
            return Err(problem(
                StatusCode::GONE,
                "share link expired",
                "share link expired",
                SdkWorkResultCode::Gone,
            ));
        }
    }
    let download_limit: Option<i64> = row.get("download_limit");
    let download_count: i64 = row.get("download_count");
    if download_limit.is_some_and(|limit| download_count >= limit) {
        return Err(problem(
            StatusCode::TOO_MANY_REQUESTS,
            "download limit exceeded",
            "share link download limit exceeded",
            SdkWorkResultCode::RateLimitExceeded,
        ));
    }

    Err(problem(
        StatusCode::CONFLICT,
        "conflict",
        "share link download slot changed concurrently",
        SdkWorkResultCode::Conflict,
    ))
}

pub(crate) async fn release_share_link_download_slot(
    pool: &PgPool,
    share_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE dr_drive_node_share_link
         SET download_count=download_count - 1, updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE id=$1 AND download_count > 0",
    )
    .bind(share_id)
    .execute(pool)
    .await
    .map(|_| ())
}

pub(crate) fn map_share_link_row(row: &PgRow) -> OpenShareLinkResponse {
    OpenShareLinkResponse {
        id: row.get("share_id"),
        tenant_id: row.get("share_tenant_id"),
        role: row.get("role"),
        expires_at_epoch_ms: row.get("expires_at_epoch_ms"),
        download_limit: row.get("download_limit"),
        download_count: row.get("download_count"),
        access_code_required: row
            .try_get::<Option<String>, _>("access_code_hash")
            .ok()
            .flatten()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false),
        node: OpenNodeResponse {
            id: row.get("node_id"),
            tenant_id: row.get("node_tenant_id"),
            space_id: row.get("space_id"),
            node_type: row.get("node_type"),
            node_name: row.get("node_name"),
            content_type: row.get("content_type"),
            content_length: row.get("content_length"),
        },
    }
}
