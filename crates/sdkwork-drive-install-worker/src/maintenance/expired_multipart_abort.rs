use sdkwork_drive_object_runtime::DriveObjectStoreRuntime;
use sdkwork_drive_storage_contract::{
    AbortMultipartUploadRequest, DriveObjectLocator, DriveObjectStoreErrorKind,
};
use sqlx::{PgPool, Row};

/// Result of one expired multipart abort batch.
#[derive(Debug, Clone, Default)]
pub struct ExpiredMultipartAbortResult {
    pub scanned_sessions: i64,
    pub aborted_uploads: i64,
    pub failed_uploads: i64,
}

/// Abort vendor-side multipart uploads that belong to expired sessions.
///
/// The SQL upload-session cleanup can delete `dr_drive_upload_part` rows, but
/// those rows are Drive metadata only. The vendor keeps every uploaded part (and
/// bills for it) until `AbortMultipartUpload` is called. This sweep closes that
/// leak and marks the session `aborted` once no provider work remains.
pub async fn abort_expired_multipart_uploads(
    pool: &PgPool,
    limit: i64,
) -> Result<ExpiredMultipartAbortResult, sqlx::Error> {
    let bounded_limit = limit.clamp(1, 1_000);
    let rows = sqlx::query(
        "SELECT id, storage_provider_id, bucket, object_key, storage_upload_id
         FROM dr_drive_upload_session
         WHERE state = 'expired'
           AND storage_upload_id IS NOT NULL
           AND storage_upload_id <> ''
         ORDER BY updated_at ASC, id ASC
         LIMIT $1",
    )
    .bind(bounded_limit)
    .fetch_all(pool)
    .await?;

    let runtime = DriveObjectStoreRuntime::new(pool.clone());
    let mut result = ExpiredMultipartAbortResult {
        scanned_sessions: rows.len() as i64,
        ..ExpiredMultipartAbortResult::default()
    };

    for row in rows {
        let session_id: String = row.get("id");
        let provider_id: String = row.get("storage_provider_id");
        let bucket: String = row.get("bucket");
        let object_key: String = row.get("object_key");
        let upload_id: String = row.get("storage_upload_id");

        let store = match runtime.resolve_locator(&provider_id, &bucket).await {
            Ok(store) => store,
            Err(error) => {
                result.failed_uploads += 1;
                let _ = sqlx::query(
                    "UPDATE dr_drive_upload_session
                     SET updated_at = CURRENT_TIMESTAMP
                     WHERE id = $1 AND state = 'expired'",
                )
                .bind(&session_id)
                .execute(pool)
                .await;
                tracing::warn!(
                    event = "drive.install_worker.multipart_abort_resolve_failed",
                    session_id = %session_id,
                    provider_id = %provider_id,
                    error = ?error,
                    "could not resolve provider for expired multipart upload"
                );
                continue;
            }
        };

        match store
            .abort_multipart_upload(AbortMultipartUploadRequest {
                locator: DriveObjectLocator {
                    bucket: bucket.clone(),
                    object_key,
                },
                upload_id,
            })
            .await
        {
            Ok(()) => {}
            Err(error)
                if matches!(
                    error.kind,
                    DriveObjectStoreErrorKind::NotFound | DriveObjectStoreErrorKind::NotSupported
                ) => {}
            Err(error) => {
                result.failed_uploads += 1;
                let _ = sqlx::query(
                    "UPDATE dr_drive_upload_session
                     SET updated_at = CURRENT_TIMESTAMP
                     WHERE id = $1 AND state = 'expired'",
                )
                .bind(&session_id)
                .execute(pool)
                .await;
                tracing::warn!(
                    event = "drive.install_worker.multipart_abort_failed",
                    session_id = %session_id,
                    provider_id = %provider_id,
                    error = ?error,
                    "multipart abort failed; keeping expired session for retry"
                );
                continue;
            }
        }

        sqlx::query(
            "UPDATE dr_drive_upload_session
             SET state = 'aborted',
                 updated_by = 'install-worker',
                 updated_at = CURRENT_TIMESTAMP,
                 version = version + 1
             WHERE id = $1 AND state = 'expired'",
        )
        .bind(&session_id)
        .execute(pool)
        .await?;
        result.aborted_uploads += 1;
    }

    Ok(result)
}
