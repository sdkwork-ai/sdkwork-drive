use crate::dto::{DriveWatchChannelResponse, InsertWatchChannel};
use crate::error::{
    internal_problem, internal_sql_error, is_unique_constraint_error, not_found_problem, problem,
    ProblemDetail, SdkWorkResultCode,
};
use crate::mappers::map_watch_channel_row;
use axum::http::StatusCode;
use axum::Json;
use sqlx::PgPool;

pub(crate) async fn find_watch_channel(
    pool: &PgPool,
    tenant_id: &str,
    channel_id: &str,
) -> Result<DriveWatchChannelResponse, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT id, tenant_id, space_id, node_id, resource_type, resource_id,
                channel_type, address, expiration_epoch_ms, lifecycle_status, version
         FROM dr_drive_watch_channel
         WHERE tenant_id=$1 AND id=$2",
    )
    .bind(tenant_id)
    .bind(channel_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("find dr_drive_watch_channel failed"))?;
    let Some(row) = row else {
        return Err(not_found_problem("watch channel not found"));
    };
    Ok(map_watch_channel_row(&row))
}

pub(crate) async fn insert_watch_channel(
    pool: &PgPool,
    command: InsertWatchChannel<'_>,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let result = sqlx::query(
        "INSERT INTO dr_drive_watch_channel (
            id, tenant_id, space_id, node_id, resource_type, resource_id,
            channel_type, address, token_hash, expiration_epoch_ms,
            lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'active', 1, $11, $11)",
    )
    .bind(command.id)
    .bind(command.tenant_id)
    .bind(command.space_id)
    .bind(command.node_id)
    .bind(command.resource_type)
    .bind(command.resource_id)
    .bind(command.channel_type)
    .bind(command.address)
    .bind(command.token_hash)
    .bind(command.expiration_epoch_ms)
    .bind(command.operator_id)
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(error) if is_unique_constraint_error(&error) => Err(problem(
            StatusCode::CONFLICT,
            "conflict",
            "watch channel id already exists",
            SdkWorkResultCode::Conflict,
        )),
        Err(error) => Err(internal_problem(format!(
            "insert dr_drive_watch_channel failed: {error}"
        ))),
    }
}
