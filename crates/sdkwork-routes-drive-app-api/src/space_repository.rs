use crate::error::{
    internal_sql_error, not_found_problem, problem, ProblemDetail, SdkWorkResultCode,
};
use axum::http::StatusCode;
use axum::Json;
use sqlx::PgPool;

pub(crate) async fn validate_space_exists(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_space
         WHERE tenant_id=$1 AND id=$2 AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(space_id)
    .fetch_one(pool)
    .await
    .map_err(internal_sql_error("validate dr_drive_space failed"))?;
    if count == 0 {
        return Err(not_found_problem("space not found"));
    }
    Ok(())
}

pub(crate) async fn validate_space_exists_for_change_history(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_space
         WHERE tenant_id=$1 AND id=$2",
    )
    .bind(tenant_id)
    .bind(space_id)
    .fetch_one(pool)
    .await
    .map_err(internal_sql_error(
        "validate dr_drive_space change history failed",
    ))?;
    if count == 0 {
        return Err(not_found_problem("space not found"));
    }
    Ok(())
}

pub(crate) async fn ensure_git_repository_space_root_accepts_node_type(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    parent_node_id: Option<&str>,
    node_type: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if parent_node_id.is_some() || node_type == "folder" {
        return Ok(());
    }

    let space_type = sqlx::query_scalar::<_, String>(
        "SELECT space_type
         FROM dr_drive_space
         WHERE tenant_id=$1 AND id=$2 AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error(
        "validate git repository space root node type failed",
    ))?;

    match space_type.as_deref() {
        Some("git_repository") => Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "git repository space root accepts only repository directories",
            SdkWorkResultCode::ValidationError,
        )),
        Some(_) => Ok(()),
        None => Err(not_found_problem("space not found")),
    }
}
