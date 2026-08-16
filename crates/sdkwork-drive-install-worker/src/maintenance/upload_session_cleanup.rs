use sqlx::PgPool;

/// Cleanup result for expired upload sessions.
#[derive(Debug, Clone)]
pub struct CleanupResult {
    pub expired_sessions: i64,
    pub cleaned_parts: i64,
    pub retired_uploading_nodes: i64,
    pub retired_storage_objects: i64,
}

/// Clean up expired upload sessions and retire orphaned uploading metadata.
///
/// Marks expired `dr_drive_upload_session` rows, removes associated upload parts,
/// soft-deletes uploading nodes, and retires active storage objects tied to those nodes.
pub async fn cleanup_expired_sessions(pool: &PgPool) -> Result<CleanupResult, sqlx::Error> {
    let now = chrono::Utc::now().timestamp_millis();
    let mut connection = pool.acquire().await?;

    sqlx::query("BEGIN").execute(&mut *connection).await?;

    let cleanup_result = async {
        let expired_sessions = sqlx::query(
            "UPDATE dr_drive_upload_session
             SET state = 'expired',
                 updated_by = 'install-worker',
                 updated_at = CURRENT_TIMESTAMP
             WHERE state IN ('created', 'uploading', 'completing')
               AND expires_at_epoch_ms < $1",
        )
        .bind(now)
        .execute(&mut *connection)
        .await?
        .rows_affected() as i64;

        let cleaned_parts = sqlx::query(
            "DELETE FROM dr_drive_upload_part
             WHERE upload_session_id IN (
                 SELECT id FROM dr_drive_upload_session WHERE state = 'expired'
             )",
        )
        .execute(&mut *connection)
        .await?
        .rows_affected() as i64;

        let retired_uploading_nodes = sqlx::query(
            "UPDATE dr_drive_node
             SET lifecycle_status = 'deleted',
                 content_state = 'failed',
                 updated_by = 'install-worker',
                 updated_at = CURRENT_TIMESTAMP,
                 version = version + 1
             WHERE content_state IN ('uploading', 'empty')
               AND lifecycle_status = 'active'
               AND id IN (
                   SELECT node_id
                   FROM dr_drive_upload_session
                   WHERE state = 'expired'
                     AND expires_at_epoch_ms < $1
               )",
        )
        .bind(now)
        .execute(&mut *connection)
        .await?
        .rows_affected() as i64;

        let retired_storage_objects = sqlx::query(
            "UPDATE dr_drive_storage_object
             SET lifecycle_status = 'deleted',
                 updated_by = 'install-worker',
                 updated_at = CURRENT_TIMESTAMP
             WHERE lifecycle_status = 'active'
               AND node_id IN (
                   SELECT node_id
                   FROM dr_drive_upload_session
                   WHERE state = 'expired'
                     AND expires_at_epoch_ms < $1
               )",
        )
        .bind(now)
        .execute(&mut *connection)
        .await?
        .rows_affected() as i64;

        Ok::<CleanupResult, sqlx::Error>(CleanupResult {
            expired_sessions,
            cleaned_parts,
            retired_uploading_nodes,
            retired_storage_objects,
        })
    }
    .await;

    match cleanup_result {
        Ok(result) => {
            sqlx::query("COMMIT").execute(&mut *connection).await?;
            Ok(result)
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            Err(error)
        }
    }
}
