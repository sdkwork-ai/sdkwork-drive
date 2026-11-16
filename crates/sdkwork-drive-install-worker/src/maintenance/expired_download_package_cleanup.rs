use sdkwork_drive_object_runtime::DriveObjectStoreRuntime;
use sdkwork_drive_storage_contract::{
    DeleteObjectRequest, DriveObjectLocator, DriveObjectStoreErrorKind,
};
use sqlx::{PgPool, Row};

/// Result of one expired download-package cleanup batch.
#[derive(Debug, Clone, Default)]
pub struct DownloadPackageCleanupResult {
    pub scanned_packages: i64,
    pub cleaned_packages: i64,
    pub failed_packages: i64,
}

/// Delete the archive object of every expired download package.
///
/// The package row records who asked for which files, so it is kept for audit;
/// only the generated ZIP in the bucket is removed. A package whose archive was
/// already deleted (or whose provider no longer has it) is still marked expired,
/// which stops it from being served and prevents repeated provider probes.
pub async fn cleanup_expired_download_packages(
    pool: &PgPool,
    limit: i64,
) -> Result<DownloadPackageCleanupResult, sqlx::Error> {
    let bounded_limit = limit.clamp(1, 1_000);
    let now_epoch_ms = chrono::Utc::now().timestamp_millis();
    let rows = sqlx::query(
        "SELECT id, storage_provider_id, bucket, archive_object_key
         FROM dr_drive_download_package
         WHERE state = 'ready'
           AND expires_at_epoch_ms <= $1
         ORDER BY expires_at_epoch_ms ASC, id ASC
         LIMIT $2",
    )
    .bind(now_epoch_ms)
    .bind(bounded_limit)
    .fetch_all(pool)
    .await?;

    let runtime = DriveObjectStoreRuntime::new(pool.clone());
    let mut result = DownloadPackageCleanupResult {
        scanned_packages: rows.len() as i64,
        ..DownloadPackageCleanupResult::default()
    };

    for row in rows {
        let package_id: String = row.get("id");
        let provider_id: String = row.get("storage_provider_id");
        let bucket: String = row.get("bucket");
        let object_key: String = row.get("archive_object_key");

        let store = match runtime.resolve_locator(&provider_id, &bucket).await {
            Ok(store) => store,
            Err(error) => {
                result.failed_packages += 1;
                let _ = sqlx::query(
                    "UPDATE dr_drive_download_package
                     SET updated_at = CURRENT_TIMESTAMP
                     WHERE id = $1 AND state = 'ready'",
                )
                .bind(&package_id)
                .execute(pool)
                .await;
                tracing::warn!(
                    event = "drive.install_worker.download_package_cleanup_resolve_failed",
                    package_id = %package_id,
                    provider_id = %provider_id,
                    error = ?error,
                    "could not resolve provider for expired download package"
                );
                continue;
            }
        };

        match store
            .delete_object(DeleteObjectRequest {
                locator: DriveObjectLocator {
                    bucket: bucket.clone(),
                    object_key,
                },
            })
            .await
        {
            Ok(_) => {}
            Err(error) if error.kind == DriveObjectStoreErrorKind::NotFound => {}
            Err(error) => {
                result.failed_packages += 1;
                let _ = sqlx::query(
                    "UPDATE dr_drive_download_package
                     SET updated_at = CURRENT_TIMESTAMP
                     WHERE id = $1 AND state = 'ready'",
                )
                .bind(&package_id)
                .execute(pool)
                .await;
                tracing::warn!(
                    event = "drive.install_worker.download_package_cleanup_failed",
                    package_id = %package_id,
                    provider_id = %provider_id,
                    error = ?error,
                    "download package archive deletion failed; keeping row for retry"
                );
                continue;
            }
        }

        sqlx::query(
            "UPDATE dr_drive_download_package
             SET state = 'expired',
                 archive_size_bytes = 0,
                 updated_by = 'install-worker',
                 updated_at = CURRENT_TIMESTAMP,
                 version = version + 1
             WHERE id = $1 AND state = 'ready'",
        )
        .bind(&package_id)
        .execute(pool)
        .await?;
        result.cleaned_packages += 1;
    }

    Ok(result)
}
