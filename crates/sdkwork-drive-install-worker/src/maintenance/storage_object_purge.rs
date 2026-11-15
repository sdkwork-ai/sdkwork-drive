use sdkwork_drive_object_runtime::DriveObjectStoreRuntime;
use sdkwork_drive_storage_contract::{
    DeleteObjectRequest, DriveObjectLocator, DriveObjectStoreErrorKind,
};
use sqlx::{PgPool, Row};

/// Result of one physical object-purge batch.
#[derive(Debug, Clone, Default)]
pub struct StorageObjectPurgeResult {
    pub scanned_objects: i64,
    pub purged_objects: i64,
    pub failed_objects: i64,
}

/// Physically delete provider bytes for storage rows already retired in the
/// database, then remove the metadata row.
///
/// Empty-trash, retention hard-delete, failed uploads and uploader cleanup all
/// retire a `dr_drive_storage_object` row to `deleted`. Until this sweep runs,
/// those bytes continue to incur vendor storage cost. The row is kept until the
/// provider confirms deletion (or reports not-found) so a transient provider
/// failure is retried instead of silently leaking the object.
pub async fn purge_deleted_storage_objects(
    pool: &PgPool,
    limit: i64,
) -> Result<StorageObjectPurgeResult, sqlx::Error> {
    let bounded_limit = limit.clamp(1, 1_000);
    let rows = sqlx::query(
        "SELECT id, storage_provider_id, bucket, object_key
         FROM dr_drive_storage_object
         WHERE lifecycle_status = 'deleted'
         ORDER BY updated_at ASC, id ASC
         LIMIT $1",
    )
    .bind(bounded_limit)
    .fetch_all(pool)
    .await?;

    let runtime = DriveObjectStoreRuntime::new(pool.clone());
    let mut result = StorageObjectPurgeResult {
        scanned_objects: rows.len() as i64,
        ..StorageObjectPurgeResult::default()
    };

    for row in rows {
        let object_id: String = row.get("id");
        let provider_id: String = row.get("storage_provider_id");
        let bucket: String = row.get("bucket");
        let object_key: String = row.get("object_key");

        let store = match runtime.resolve_locator(&provider_id, &bucket).await {
            Ok(store) => store,
            Err(error) => {
                result.failed_objects += 1;
                let _ = sqlx::query(
                    "UPDATE dr_drive_storage_object
                     SET updated_at = CURRENT_TIMESTAMP
                     WHERE id = $1 AND lifecycle_status = 'deleted'",
                )
                .bind(&object_id)
                .execute(pool)
                .await;
                tracing::warn!(
                    event = "drive.install_worker.storage_object_purge_resolve_failed",
                    object_id = %object_id,
                    provider_id = %provider_id,
                    error = ?error,
                    "could not resolve provider for deleted storage object"
                );
                continue;
            }
        };

        match store
            .delete_object(DeleteObjectRequest {
                locator: DriveObjectLocator {
                    bucket: bucket.clone(),
                    object_key: object_key.clone(),
                },
            })
            .await
        {
            Ok(_) => {}
            Err(error) if error.kind == DriveObjectStoreErrorKind::NotFound => {}
            Err(error) => {
                result.failed_objects += 1;
                let _ = sqlx::query(
                    "UPDATE dr_drive_storage_object
                     SET updated_at = CURRENT_TIMESTAMP
                     WHERE id = $1 AND lifecycle_status = 'deleted'",
                )
                .bind(&object_id)
                .execute(pool)
                .await;
                tracing::warn!(
                    event = "drive.install_worker.storage_object_purge_failed",
                    object_id = %object_id,
                    provider_id = %provider_id,
                    bucket = %bucket,
                    object_key = %object_key,
                    error = ?error,
                    "provider object deletion failed; keeping metadata for retry"
                );
                continue;
            }
        }

        sqlx::query(
            "DELETE FROM dr_drive_storage_object
             WHERE id = $1 AND lifecycle_status = 'deleted'",
        )
        .bind(&object_id)
        .execute(pool)
        .await?;
        result.purged_objects += 1;
    }

    Ok(result)
}
