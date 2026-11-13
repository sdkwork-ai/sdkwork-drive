use async_trait::async_trait;
use sqlx::Row;
use sqlx::{PgConnection, PgPool};

use crate::domain::node_version::DriveNodeVersion;
use crate::domain::upload::{DriveUploadSession, DriveUploadSessionState};
use crate::infrastructure::sql::sql_error::is_unique_constraint_violation;
use crate::infrastructure::sql::stored_version_store::SqlStoredVersionStore;
use crate::ports::stored_version_store::{
    CompleteStoredVersionUploadCommand, DriveStoredVersionStore,
};
use crate::ports::upload_session_store::{DriveUploadSessionStore, NewDriveUploadSession};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlUploadSessionStore {
    pool: PgPool,
}

/// Unconditional state write. `tenant_id`/`id` are the last two bind parameters
/// so the same statement serves the pool path and the transaction path.
const UPDATE_SESSION_STATE_SQL: &str = "UPDATE dr_drive_upload_session
     SET state=$1, updated_by=$2, updated_at=CURRENT_TIMESTAMP, version=version + 1
     WHERE tenant_id=$3 AND id=$4";

/// Write a session state on an existing transaction connection.
///
/// Completion must write its terminal state inside the same transaction that
/// publishes the storage object and node version; using the pool here would
/// commit the state even when that transaction rolls back.
///
/// # Errors
///
/// `NotFound` when no row matched the tenant/id pair; `Internal` when the write
/// fails.
pub(crate) async fn update_session_state_on_connection(
    connection: &mut PgConnection,
    tenant_id: &str,
    upload_session_id: &str,
    next_state: &str,
    operator_id: &str,
) -> Result<(), DriveServiceError> {
    let affected = sqlx::query(UPDATE_SESSION_STATE_SQL)
        .bind(next_state)
        .bind(operator_id)
        .bind(tenant_id)
        .bind(upload_session_id)
        .execute(&mut *connection)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "update dr_drive_upload_session state failed: {error}"
            ))
        })?
        .rows_affected();
    if affected == 0 {
        return Err(DriveServiceError::NotFound(
            "upload session not found".to_string(),
        ));
    }
    Ok(())
}

impl SqlUploadSessionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Reject a session id that is already taken.
    ///
    /// Session ids are client-supplied, so a collision must be reported before
    /// any multipart upload is opened for the id.
    ///
    /// # Errors
    ///
    /// `Conflict` when the id exists; `Internal` when the probe fails.
    pub async fn ensure_id_available(
        &self,
        upload_session_id: &str,
    ) -> Result<(), DriveServiceError> {
        let taken: i64 = sqlx::query_scalar(
            "SELECT COUNT(1)
             FROM dr_drive_upload_session
             WHERE id=$1",
        )
        .bind(upload_session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "check dr_drive_upload_session id conflict failed: {error}"
            ))
        })?;
        if taken > 0 {
            return Err(DriveServiceError::Conflict(
                "upload session id already exists".to_string(),
            ));
        }
        Ok(())
    }

    /// Load one session inside the tenant.
    ///
    /// # Errors
    ///
    /// `Internal` when the read fails; a missing row is `Ok(None)` so callers
    /// choose between `NotFound` and an idempotent replay.
    pub async fn find_by_id(
        &self,
        tenant_id: &str,
        upload_session_id: &str,
    ) -> Result<Option<DriveUploadSession>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, space_id, node_id, bucket, object_key,
                    idempotency_key, storage_provider_id, storage_upload_id, state,
                    expires_at_epoch_ms, version
             FROM dr_drive_upload_session
             WHERE tenant_id=$1 AND id=$2",
        )
        .bind(tenant_id)
        .bind(upload_session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("find dr_drive_upload_session failed: {error}"))
        })?;
        row.as_ref().map(map_session_row).transpose()
    }

    /// Write a session state unconditionally.
    ///
    /// # Errors
    ///
    /// `NotFound` when no row matched the tenant/id pair; `Internal` when the
    /// write fails.
    pub async fn update_state(
        &self,
        tenant_id: &str,
        upload_session_id: &str,
        next_state: &str,
        operator_id: &str,
    ) -> Result<(), DriveServiceError> {
        let affected = sqlx::query(UPDATE_SESSION_STATE_SQL)
            .bind(next_state)
            .bind(operator_id)
            .bind(tenant_id)
            .bind(upload_session_id)
            .execute(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "update dr_drive_upload_session state failed: {error}"
                ))
            })?
            .rows_affected();
        if affected == 0 {
            return Err(DriveServiceError::NotFound(
                "upload session not found".to_string(),
            ));
        }
        Ok(())
    }

    /// Move a session back to `uploading` while presigning a part.
    ///
    /// Guarded on the mutable states and the deadline, so a presign racing a
    /// completion cannot resurrect a dead session.
    ///
    /// # Errors
    ///
    /// `Conflict` when the row is no longer mutable or has expired; `Internal`
    /// when the write fails.
    pub async fn mark_uploading(
        &self,
        tenant_id: &str,
        upload_session_id: &str,
        operator_id: &str,
        now_epoch_ms: i64,
    ) -> Result<(), DriveServiceError> {
        let affected = sqlx::query(
            "UPDATE dr_drive_upload_session
             SET state='uploading', updated_by=$1, updated_at=CURRENT_TIMESTAMP, version=version + 1
             WHERE tenant_id=$2
               AND id=$3
               AND state IN ('created', 'uploading')
               AND expires_at_epoch_ms > $4",
        )
        .bind(operator_id)
        .bind(tenant_id)
        .bind(upload_session_id)
        .bind(now_epoch_ms)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "mark dr_drive_upload_session uploading failed: {error}"
            ))
        })?
        .rows_affected();
        if affected == 0 {
            return Err(DriveServiceError::Conflict(
                "upload session is no longer mutable".to_string(),
            ));
        }
        Ok(())
    }

    /// Claim completion with an optimistic version check.
    ///
    /// # Errors
    ///
    /// `Conflict` when another completer won the race (or the state moved on);
    /// `Internal` when the write fails.
    pub async fn claim_completion(
        &self,
        upload_session: &DriveUploadSession,
        operator_id: &str,
    ) -> Result<(), DriveServiceError> {
        let affected = sqlx::query(
            "UPDATE dr_drive_upload_session
             SET state='completing', updated_by=$1, updated_at=CURRENT_TIMESTAMP, version=version + 1
             WHERE tenant_id=$2
               AND id=$3
               AND version=$4
               AND state IN ('created', 'uploading')",
        )
        .bind(operator_id)
        .bind(&upload_session.tenant_id)
        .bind(&upload_session.id)
        .bind(upload_session.version)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "claim dr_drive_upload_session completion failed: {error}"
            ))
        })?
        .rows_affected();
        if affected == 0 {
            return Err(DriveServiceError::Conflict(
                "upload session completion is already in progress or no longer mutable".to_string(),
            ));
        }
        Ok(())
    }

    /// Release a completion claim so the client can retry.
    ///
    /// Guarded on `state = 'completing'` so a concurrent abort is not undone.
    ///
    /// # Returns
    ///
    /// `true` when the claim was released, `false` when the session had already
    /// left `completing` (nothing was written).
    ///
    /// # Errors
    ///
    /// `Internal` when the write fails.
    pub async fn release_completion_claim(
        &self,
        tenant_id: &str,
        upload_session_id: &str,
        operator_id: &str,
    ) -> Result<bool, DriveServiceError> {
        let affected = sqlx::query(
            "UPDATE dr_drive_upload_session
             SET state='uploading', updated_by=$1, updated_at=CURRENT_TIMESTAMP, version=version + 1
             WHERE tenant_id=$2 AND id=$3 AND state='completing'",
        )
        .bind(operator_id)
        .bind(tenant_id)
        .bind(upload_session_id)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "release dr_drive_upload_session completion claim failed: {error}"
            ))
        })?
        .rows_affected();
        Ok(affected > 0)
    }
}

fn map_session_row(row: &sqlx::postgres::PgRow) -> Result<DriveUploadSession, DriveServiceError> {
    let state_raw: String = row.get("state");
    let state = DriveUploadSessionState::try_from_str(&state_raw).ok_or_else(|| {
        DriveServiceError::Internal(format!("unknown upload session state: {state_raw}"))
    })?;
    Ok(DriveUploadSession {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        space_id: row.get("space_id"),
        node_id: row.get("node_id"),
        bucket: row.get("bucket"),
        object_key: row.get("object_key"),
        idempotency_key: row.get("idempotency_key"),
        storage_provider_id: row.get("storage_provider_id"),
        storage_upload_id: row.get("storage_upload_id"),
        state,
        expires_at_epoch_ms: row.get("expires_at_epoch_ms"),
        version: row.get("version"),
    })
}

#[async_trait]
impl DriveUploadSessionStore for SqlUploadSessionStore {
    async fn find_by_idempotency(
        &self,
        tenant_id: &str,
        space_id: &str,
        node_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<DriveUploadSession>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, space_id, node_id, bucket, object_key,
                    idempotency_key, storage_provider_id, storage_upload_id, state,
                    expires_at_epoch_ms, version
             FROM dr_drive_upload_session
             WHERE tenant_id=$1
               AND space_id=$2
               AND node_id=$3
               AND idempotency_key=$4
             LIMIT 1",
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(node_id)
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "query dr_drive_upload_session by idempotency failed: {error}"
            ))
        })?;

        let Some(row) = row else {
            return Ok(None);
        };

        map_session_row(&row).map(Some)
    }

    async fn insert_upload_session(
        &self,
        new_session: &NewDriveUploadSession,
    ) -> Result<DriveUploadSession, DriveServiceError> {
        let result = sqlx::query(
            "INSERT INTO dr_drive_upload_session (
                id, tenant_id, space_id, node_id, bucket, object_key,
                idempotency_key, storage_provider_id, storage_upload_id, state,
                expires_at_epoch_ms, version, created_by, updated_by
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 1, $12, $13)",
        )
        .bind(&new_session.id)
        .bind(&new_session.tenant_id)
        .bind(&new_session.space_id)
        .bind(&new_session.node_id)
        .bind(&new_session.bucket)
        .bind(&new_session.object_key)
        .bind(&new_session.idempotency_key)
        .bind(&new_session.storage_provider_id)
        .bind(&new_session.storage_upload_id)
        .bind(&new_session.state)
        .bind(new_session.expires_at_epoch_ms)
        .bind(&new_session.created_by)
        .bind(&new_session.updated_by)
        .execute(&self.pool)
        .await;

        if let Err(error) = result {
            let message = error.to_string();
            if is_unique_constraint_violation(&message) {
                return Err(DriveServiceError::Conflict(
                    "upload session idempotency key already exists".to_string(),
                ));
            }
            return Err(DriveServiceError::Internal(format!(
                "insert dr_drive_upload_session failed: {message}"
            )));
        }

        let row = sqlx::query(
            "SELECT id, tenant_id, space_id, node_id, bucket, object_key,
                    idempotency_key, storage_provider_id, storage_upload_id, state,
                    expires_at_epoch_ms, version
             FROM dr_drive_upload_session
             WHERE id=$1",
        )
        .bind(&new_session.id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "read inserted dr_drive_upload_session failed: {error}"
            ))
        })?;

        map_session_row(&row)
    }
}

#[async_trait]
impl DriveStoredVersionStore for SqlUploadSessionStore {
    /// Delegate stored-version completion to the repository that owns it.
    ///
    /// Both capabilities live on this store so a caller that already builds
    /// `DriveUploadService::new(SqlUploadSessionStore::new(pool))` can call
    /// `complete_stored_version` without a second constructor.
    async fn complete_stored_version(
        &self,
        command: &CompleteStoredVersionUploadCommand,
    ) -> Result<DriveNodeVersion, DriveServiceError> {
        SqlStoredVersionStore::new(self.pool.clone())
            .complete_stored_version(command)
            .await
    }
}
