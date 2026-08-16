use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::sandbox_directory::{SandboxDirectoryEntry, SandboxEntryKind};
use crate::infrastructure::sql::runtime_id::next_drive_runtime_id;
use crate::infrastructure::sql::sql_error::is_unique_constraint_violation;
use crate::ports::sandbox_mutation_operation_store::{
    BeginSandboxMutationOperation, CompleteSandboxMutationOperation,
    DriveSandboxMutationOperationStore, SandboxMutationOperationBeginResult, SandboxMutationResult,
};
use crate::DriveServiceError;

#[derive(Clone, Debug)]
pub struct SqlSandboxMutationOperationStore {
    pool: PgPool,
}

impl SqlSandboxMutationOperationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveSandboxMutationOperationStore for SqlSandboxMutationOperationStore {
    async fn begin_or_load(
        &self,
        operation: &BeginSandboxMutationOperation,
    ) -> Result<SandboxMutationOperationBeginResult, DriveServiceError> {
        let operation_id = next_drive_runtime_id("sandbox mutation operation")?;
        let insert = sqlx::query(
            "INSERT INTO dr_drive_sandbox_mutation_operation (
                id, tenant_id, sandbox_id, actor_id, idempotency_key_hash, request_fingerprint,
                mutation_kind, parent_logical_path, entry_name, operation_status, lease_token,
                lease_expires_at_ms
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending', $10, $11)",
        )
        .bind(operation_id)
        .bind(&operation.tenant_id)
        .bind(&operation.sandbox_id)
        .bind(&operation.actor_id)
        .bind(&operation.idempotency_key_hash)
        .bind(&operation.request_fingerprint)
        .bind(&operation.mutation_kind)
        .bind(&operation.parent_logical_path)
        .bind(&operation.entry_name)
        .bind(&operation.lease_token)
        .bind(operation.lease_expires_at_ms)
        .execute(&self.pool)
        .await;

        match insert {
            Ok(_) => Ok(SandboxMutationOperationBeginResult::Started {
                operation_id,
                lease_token: operation.lease_token.clone(),
            }),
            Err(error) if is_unique_constraint_violation(&error.to_string()) => {
                let row = load_by_idempotency_scope(
                    &self.pool,
                    &operation.tenant_id,
                    &operation.sandbox_id,
                    &operation.actor_id,
                    &operation.idempotency_key_hash,
                )
                .await?
                .ok_or_else(|| {
                    DriveServiceError::Internal(
                        "sandbox mutation operation disappeared after uniqueness conflict"
                            .to_string(),
                    )
                })?;
                let stored_fingerprint: String = row.get("request_fingerprint");
                if stored_fingerprint != operation.request_fingerprint {
                    return Err(DriveServiceError::Conflict(
                        "idempotency key belongs to a different sandbox mutation request"
                            .to_string(),
                    ));
                }
                map_begin_result(&row)
            }
            Err(error) => Err(DriveServiceError::Internal(format!(
                "begin sandbox mutation operation failed: {error}"
            ))),
        }
    }

    async fn try_claim_pending(
        &self,
        operation_id: i64,
        tenant_id: &str,
        now_ms: i64,
        lease_token: &str,
        lease_expires_at_ms: i64,
    ) -> Result<bool, DriveServiceError> {
        let result = sqlx::query(
            "UPDATE dr_drive_sandbox_mutation_operation
             SET lease_token=$1, lease_expires_at_ms=$2, updated_at=CURRENT_TIMESTAMP
             WHERE id=$3 AND tenant_id=$4 AND operation_status='pending'
               AND lease_expires_at_ms <= $5",
        )
        .bind(lease_token)
        .bind(lease_expires_at_ms)
        .bind(operation_id)
        .bind(tenant_id)
        .bind(now_ms)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "claim pending sandbox mutation operation failed: {error}"
            ))
        })?;
        Ok(result.rows_affected() == 1)
    }

    async fn complete_with_audit(
        &self,
        operation: &CompleteSandboxMutationOperation,
    ) -> Result<SandboxMutationResult, DriveServiceError> {
        let mut transaction = self.pool.begin().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "begin sandbox mutation completion transaction failed: {error}"
            ))
        })?;
        let (entry_id, parent_id, kind, logical_path, revision, deleted) =
            mutation_result_columns(&operation.result);
        let mut update_sql = String::from(
            "UPDATE dr_drive_sandbox_mutation_operation
             SET operation_status='completed', result_entry_id=$1, result_parent_id=$2,
                 result_entry_kind=$3, result_logical_path=$4, result_revision=$5,
                 result_deleted=$6, updated_at=CURRENT_TIMESTAMP
             WHERE id=$7 AND tenant_id=$8 AND operation_status='pending'",
        );
        if operation.lease_token.is_some() {
            update_sql.push_str(" AND lease_token=$9");
        }
        let mut update = sqlx::query(sqlx::AssertSqlSafe(update_sql.as_str()))
            .bind(entry_id)
            .bind(parent_id)
            .bind(kind)
            .bind(logical_path)
            .bind(revision)
            .bind(deleted)
            .bind(operation.operation_id)
            .bind(&operation.tenant_id);
        if let Some(lease_token) = operation.lease_token.as_deref() {
            update = update.bind(lease_token);
        }
        let updated = update.execute(&mut *transaction).await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "persist sandbox mutation operation result failed: {error}"
            ))
        })?;

        if updated.rows_affected() == 1 {
            let audit_id = next_drive_runtime_id("sandbox mutation audit event")?;
            sqlx::query(
                "INSERT INTO dr_drive_audit_event (
                    id, tenant_id, action, resource_type, resource_id, operator_id,
                    request_id, trace_id
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(audit_id)
            .bind(&operation.tenant_id)
            .bind(&operation.audit_action)
            .bind(&operation.audit_resource_type)
            .bind(&operation.audit_resource_id)
            .bind(&operation.operator_id)
            .bind(&operation.request_id)
            .bind(&operation.trace_id)
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "persist sandbox mutation audit event failed: {error}"
                ))
            })?;
            transaction.commit().await.map_err(|error| {
                DriveServiceError::Internal(format!(
                    "commit sandbox mutation completion failed: {error}"
                ))
            })?;
            return Ok(operation.result.clone());
        }

        let row = load_by_id_in_transaction(
            &mut transaction,
            operation.operation_id,
            &operation.tenant_id,
        )
        .await?
        .ok_or_else(|| {
            DriveServiceError::Internal("sandbox mutation operation was not found".to_string())
        })?;
        let result = map_completed_result(&row)?;
        transaction.commit().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "commit replayed sandbox mutation completion failed: {error}"
            ))
        })?;
        Ok(result)
    }

    async fn mark_conflict(
        &self,
        operation_id: i64,
        tenant_id: &str,
        lease_token: &str,
    ) -> Result<Option<SandboxMutationResult>, DriveServiceError> {
        let mut transaction = self.pool.begin().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "begin sandbox mutation conflict transaction failed: {error}"
            ))
        })?;
        let updated = sqlx::query(
            "UPDATE dr_drive_sandbox_mutation_operation
             SET operation_status='failed_conflict', updated_at=CURRENT_TIMESTAMP
             WHERE id=$1 AND tenant_id=$2 AND operation_status='pending' AND lease_token=$3",
        )
        .bind(operation_id)
        .bind(tenant_id)
        .bind(lease_token)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "mark sandbox mutation operation conflict failed: {error}"
            ))
        })?;
        if updated.rows_affected() == 1 {
            transaction.commit().await.map_err(|error| {
                DriveServiceError::Internal(format!(
                    "commit sandbox mutation conflict failed: {error}"
                ))
            })?;
            return Ok(None);
        }

        let row = load_by_id_in_transaction(&mut transaction, operation_id, tenant_id)
            .await?
            .ok_or_else(|| {
                DriveServiceError::Internal("sandbox mutation operation was not found".to_string())
            })?;
        let status: String = row.get("operation_status");
        let completed = if status == "completed" {
            Some(map_completed_result(&row)?)
        } else {
            None
        };
        transaction.commit().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "commit sandbox mutation conflict replay failed: {error}"
            ))
        })?;
        Ok(completed)
    }
}

async fn load_by_idempotency_scope(
    pool: &PgPool,
    tenant_id: &str,
    sandbox_id: &str,
    actor_id: &str,
    idempotency_key_hash: &str,
) -> Result<Option<sqlx::postgres::PgRow>, DriveServiceError> {
    sqlx::query(
        "SELECT id, tenant_id, sandbox_id, actor_id, request_fingerprint, mutation_kind,
                parent_logical_path, entry_name, operation_status, lease_token,
                lease_expires_at_ms, result_entry_id, result_parent_id, result_entry_kind,
                result_logical_path, result_revision, result_deleted
         FROM dr_drive_sandbox_mutation_operation
         WHERE tenant_id=$1 AND sandbox_id=$2 AND actor_id=$3 AND idempotency_key_hash=$4",
    )
    .bind(tenant_id)
    .bind(sandbox_id)
    .bind(actor_id)
    .bind(idempotency_key_hash)
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "load sandbox mutation operation by idempotency key failed: {error}"
        ))
    })
}

async fn load_by_id_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    operation_id: i64,
    tenant_id: &str,
) -> Result<Option<sqlx::postgres::PgRow>, DriveServiceError> {
    sqlx::query(
        "SELECT id, tenant_id, sandbox_id, actor_id, request_fingerprint, mutation_kind,
                parent_logical_path, entry_name, operation_status, lease_token,
                lease_expires_at_ms, result_entry_id, result_parent_id, result_entry_kind,
                result_logical_path, result_revision, result_deleted
         FROM dr_drive_sandbox_mutation_operation WHERE id=$1 AND tenant_id=$2",
    )
    .bind(operation_id)
    .bind(tenant_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "load sandbox mutation operation by id failed: {error}"
        ))
    })
}

fn map_begin_result(
    row: &sqlx::postgres::PgRow,
) -> Result<SandboxMutationOperationBeginResult, DriveServiceError> {
    let status: String = row.get("operation_status");
    match status.as_str() {
        "pending" => Ok(SandboxMutationOperationBeginResult::Pending {
            operation_id: row.get("id"),
            lease_expires_at_ms: row.get("lease_expires_at_ms"),
        }),
        "completed" => Ok(SandboxMutationOperationBeginResult::Completed(
            map_completed_result(row)?,
        )),
        "failed_conflict" => Ok(SandboxMutationOperationBeginResult::FailedConflict),
        _ => Err(DriveServiceError::Internal(
            "sandbox mutation operation has invalid status".to_string(),
        )),
    }
}

fn map_completed_result(
    row: &sqlx::postgres::PgRow,
) -> Result<SandboxMutationResult, DriveServiceError> {
    let status: String = row.get("operation_status");
    if status != "completed" {
        return Err(DriveServiceError::Conflict(
            "sandbox mutation operation is not complete".to_string(),
        ));
    }
    let deleted = decode_result_deleted(row)?;
    if deleted {
        return Ok(SandboxMutationResult::Deleted);
    }
    let kind: Option<String> = row.get("result_entry_kind");
    let kind = match kind.as_deref() {
        Some("directory") => SandboxEntryKind::Directory,
        Some("file") => SandboxEntryKind::File,
        _ => {
            return Err(DriveServiceError::Internal(
                "completed sandbox mutation operation has invalid result kind".to_string(),
            ))
        }
    };
    let entry_id = required_result(row, "result_entry_id")?;
    let parent_id = required_result(row, "result_parent_id")?;
    let logical_path = required_result(row, "result_logical_path")?;
    let revision = required_result(row, "result_revision")?;
    Ok(SandboxMutationResult::Entry(SandboxDirectoryEntry {
        id: entry_id,
        sandbox_id: row.get("sandbox_id"),
        parent_id,
        parent_logical_path: row.get("parent_logical_path"),
        name: row.get("entry_name"),
        kind,
        logical_path,
        revision,
    }))
}

fn decode_result_deleted(row: &sqlx::postgres::PgRow) -> Result<bool, DriveServiceError> {
    if let Ok(value) = row.try_get::<bool, _>("result_deleted") {
        return Ok(value);
    }
    row.try_get::<i64, _>("result_deleted")
        .map(|value| value != 0)
        .map_err(|_| {
            DriveServiceError::Internal(
                "completed sandbox mutation operation has invalid deletion result".to_string(),
            )
        })
}

fn required_result(row: &sqlx::postgres::PgRow, column: &str) -> Result<String, DriveServiceError> {
    row.get::<Option<String>, _>(column)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            DriveServiceError::Internal(
                "completed sandbox mutation operation result is incomplete".to_string(),
            )
        })
}

type MutationResultColumns<'a> = (
    Option<&'a str>,
    Option<&'a str>,
    Option<&'static str>,
    Option<&'a str>,
    Option<&'a str>,
    bool,
);

fn mutation_result_columns(result: &SandboxMutationResult) -> MutationResultColumns<'_> {
    match result {
        SandboxMutationResult::Entry(entry) => (
            Some(entry.id.as_str()),
            Some(entry.parent_id.as_str()),
            Some(entry_kind_wire(entry.kind)),
            Some(entry.logical_path.as_str()),
            Some(entry.revision.as_str()),
            false,
        ),
        SandboxMutationResult::Deleted => (None, None, None, None, None, true),
    }
}

fn entry_kind_wire(kind: SandboxEntryKind) -> &'static str {
    match kind {
        SandboxEntryKind::Directory => "directory",
        SandboxEntryKind::File => "file",
    }
}
