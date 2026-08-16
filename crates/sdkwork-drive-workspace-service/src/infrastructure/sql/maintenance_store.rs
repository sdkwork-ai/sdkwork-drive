use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::PgPool;
use sqlx::Row;

use crate::domain::maintenance::DriveSweepResult;
use crate::domain::maintenance_job::DriveMaintenanceJob;
use crate::infrastructure::change_recorder::{
    record_drive_change_on_connection, RecordDriveChangeCommand,
};
use crate::infrastructure::sql::begin_transaction_sql;
use crate::infrastructure::sql::managed_website_tree_guard::ensure_managed_website_node_mutation_allowed;
use crate::infrastructure::sql::runtime_id::next_drive_runtime_id;
use crate::ports::maintenance_store::{
    DriveMaintenanceStore, ListMaintenanceJobsQuery, MaintenanceJobPage, NewDriveMaintenanceJob,
    SweepAbandonedUploadTasksQuery, SweepDeletedObjectsQuery, SweepExpiredUploadContentQuery,
    SweepExpiredUploadSessionsQuery,
};
use crate::DriveServiceError;
use sdkwork_drive_contract::drive::domain_events as drive_events;

#[derive(Debug, Clone)]
pub struct SqlMaintenanceStore {
    pool: PgPool,
}

impl SqlMaintenanceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveMaintenanceStore for SqlMaintenanceStore {
    async fn sweep_deleted_objects(
        &self,
        query: &SweepDeletedObjectsQuery,
    ) -> Result<DriveSweepResult, DriveServiceError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(1)
             FROM dr_drive_storage_object
             WHERE lifecycle_status='deleted'",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "count deleted dr_drive_storage_object failed: {error}"
            ))
        })?;
        let scanned_count = total.min(query.limit);

        let affected_count = if query.dry_run || scanned_count == 0 {
            0
        } else {
            sqlx::query(
                "DELETE FROM dr_drive_storage_object
                 WHERE id IN (
                     SELECT id
                     FROM dr_drive_storage_object
                     WHERE lifecycle_status='deleted'
                     ORDER BY id ASC
                     LIMIT $1
                 )",
            )
            .bind(query.limit)
            .execute(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "delete deleted dr_drive_storage_object failed: {error}"
                ))
            })?
            .rows_affected() as i64
        };

        Ok(DriveSweepResult {
            scanned_count,
            affected_count,
            dry_run: query.dry_run,
        })
    }

    async fn sweep_expired_upload_sessions(
        &self,
        query: &SweepExpiredUploadSessionsQuery,
    ) -> Result<DriveSweepResult, DriveServiceError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(1)
             FROM dr_drive_upload_session
             WHERE state IN ('created', 'uploading', 'completing')
               AND expires_at_epoch_ms < $1",
        )
        .bind(query.now_epoch_ms)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "count expired dr_drive_upload_session failed: {error}"
            ))
        })?;
        let scanned_count = total.min(query.limit);

        let affected_count = if query.dry_run || scanned_count == 0 {
            0
        } else {
            sqlx::query(
                "UPDATE dr_drive_upload_session
                 SET state='expired',
                     version=version + 1,
                     updated_at=CURRENT_TIMESTAMP
                 WHERE id IN (
                     SELECT id
                     FROM dr_drive_upload_session
                     WHERE state IN ('created', 'uploading', 'completing')
                       AND expires_at_epoch_ms < $1
                     ORDER BY expires_at_epoch_ms ASC
                     LIMIT $2
                 )",
            )
            .bind(query.now_epoch_ms)
            .bind(query.limit)
            .execute(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "update expired dr_drive_upload_session failed: {error}"
                ))
            })?
            .rows_affected() as i64
        };

        Ok(DriveSweepResult {
            scanned_count,
            affected_count,
            dry_run: query.dry_run,
        })
    }

    async fn sweep_expired_upload_content(
        &self,
        query: &SweepExpiredUploadContentQuery,
    ) -> Result<DriveSweepResult, DriveServiceError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(1)
             FROM dr_drive_upload_item
             WHERE status='completed'
               AND cleanup_status='active'
               AND retention_mode='temporary'
               AND cleanup_action IN ('soft_delete', 'hard_delete')
               AND retention_expires_at_epoch_ms <= $1",
        )
        .bind(query.now_epoch_ms)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "count expired dr_drive_upload_item content failed: {error}"
            ))
        })?;
        let scanned_count = total.min(query.limit);

        if query.dry_run || scanned_count == 0 {
            return Ok(DriveSweepResult {
                scanned_count,
                affected_count: 0,
                dry_run: query.dry_run,
            });
        }

        let rows = sqlx::query(
            "SELECT
                ui.id AS upload_item_id,
                ui.tenant_id,
                ui.organization_id,
                ui.user_id,
                ui.space_id,
                ui.node_id,
                ui.content_type,
                ui.content_type_group,
                ui.content_length,
                ui.checksum_sha256_hex,
                ui.cleanup_action,
                n.lifecycle_status AS node_lifecycle_status,
                so.id AS storage_object_id,
                so.bucket AS object_bucket,
                so.object_key
             FROM dr_drive_upload_item ui
             JOIN dr_drive_node n ON n.id = ui.node_id
             LEFT JOIN dr_drive_storage_object so
               ON so.node_id = ui.node_id
              AND so.tenant_id = ui.tenant_id
              AND so.lifecycle_status = 'active'
             WHERE ui.status='completed'
               AND ui.cleanup_status='active'
               AND ui.retention_mode='temporary'
               AND ui.cleanup_action IN ('soft_delete', 'hard_delete')
               AND ui.retention_expires_at_epoch_ms <= $1
             ORDER BY ui.retention_expires_at_epoch_ms ASC, ui.id ASC, so.version_no DESC
             LIMIT $2",
        )
        .bind(query.now_epoch_ms)
        .bind(query.limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "list expired dr_drive_upload_item content failed: {error}"
            ))
        })?;

        let mut affected_count = 0_i64;
        for row in rows {
            let upload_item_id: String = row.get("upload_item_id");
            let tenant_id: String = row.get("tenant_id");
            let node_id: String = row.get("node_id");
            let before_lifecycle_status: String = row.get("node_lifecycle_status");
            let cleanup_action: String = row.get("cleanup_action");
            let (operation_type, node_lifecycle_status, cleanup_status) =
                if cleanup_action == "hard_delete" {
                    ("hard_delete", "deleted", "hard_deleted")
                } else {
                    ("soft_delete", "trashed", "soft_deleted")
                };

            let mut connection = self.pool.acquire().await.map_err(|error| {
                DriveServiceError::Internal(format!(
                    "acquire expired upload content cleanup transaction failed: {error}"
                ))
            })?;
            sqlx::query(begin_transaction_sql())
                .execute(&mut *connection)
                .await
                .map_err(|error| {
                    DriveServiceError::Internal(format!(
                        "begin expired upload content cleanup transaction failed: {error}"
                    ))
                })?;

            let tx_result: Result<bool, DriveServiceError> = async {
                ensure_managed_website_node_mutation_allowed(
                    &mut connection,
                    &tenant_id,
                    &node_id,
                )
                .await?;

                let object_delete_status = if cleanup_action == "hard_delete" {
                    let object_rows = sqlx::query(
                        "UPDATE dr_drive_storage_object
                         SET lifecycle_status='deleted',
                             updated_by=$1,
                             updated_at=CURRENT_TIMESTAMP
                         WHERE tenant_id=$2
                           AND node_id=$3
                           AND lifecycle_status='active'",
                    )
                    .bind(&query.operator_id)
                    .bind(&tenant_id)
                    .bind(&node_id)
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "hard delete expired upload storage object failed: {error}"
                        ))
                    })?
                    .rows_affected();
                    if object_rows == 0 {
                        "missing"
                    } else {
                        "deleted"
                    }
                } else {
                    "not_required"
                };

                let node_rows = sqlx::query(
                    "UPDATE dr_drive_node
                     SET lifecycle_status=$1,
                         version=version + 1,
                         updated_by=$2,
                         updated_at=CURRENT_TIMESTAMP
                     WHERE tenant_id=$3
                       AND id=$4
                       AND lifecycle_status='active'",
                )
                .bind(node_lifecycle_status)
                .bind(&query.operator_id)
                .bind(&tenant_id)
                .bind(&node_id)
                .execute(&mut *connection)
                .await
                .map_err(|error| {
                    DriveServiceError::Internal(format!(
                        "soft delete expired upload node failed: {error}"
                    ))
                })?
                .rows_affected();

                let item_rows = sqlx::query(
                    "UPDATE dr_drive_upload_item
                     SET cleanup_status=$1,
                         updated_by=$2,
                         updated_at=CURRENT_TIMESTAMP
                     WHERE tenant_id=$3
                       AND id=$4
                       AND cleanup_status='active'",
                )
                .bind(cleanup_status)
                .bind(&query.operator_id)
                .bind(&tenant_id)
                .bind(&upload_item_id)
                .execute(&mut *connection)
                .await
                .map_err(|error| {
                    DriveServiceError::Internal(format!(
                        "mark expired dr_drive_upload_item soft deleted failed: {error}"
                    ))
                })?
                .rows_affected();

                if item_rows == 0 {
                    return Ok(false);
                }

                sqlx::query(
                    "INSERT INTO dr_drive_file_sensitive_operation (
                        id, tenant_id, organization_id, user_id, space_id, node_id,
                        storage_object_id, upload_item_id, operation_type, operation_reason,
                        content_type, content_type_group, content_length, checksum_sha256_hex,
                        object_bucket, object_key, before_lifecycle_status, after_lifecycle_status,
                        operator_id, maintenance_job_id, request_id, trace_id, object_delete_status
                    ) VALUES (
                        $1, $2, $3, $4, $5, $6,
                        $7, $8, $9, 'retention_expired',
                        $10, $11, $12, $13,
                        $14, $15, $16, $17,
                        $18, NULL, $19, $20, $21
                    )",
                )
                .bind(format!(
                    "fso-expired-upload-{operation_type}-{upload_item_id}"
                ))
                .bind(&tenant_id)
                .bind(row.get::<Option<String>, _>("organization_id"))
                .bind(row.get::<Option<String>, _>("user_id"))
                .bind(row.get::<String, _>("space_id"))
                .bind(&node_id)
                .bind(row.get::<Option<String>, _>("storage_object_id"))
                .bind(&upload_item_id)
                .bind(operation_type)
                .bind(row.get::<String, _>("content_type"))
                .bind(row.get::<String, _>("content_type_group"))
                .bind(row.get::<i64, _>("content_length"))
                .bind(row.get::<Option<String>, _>("checksum_sha256_hex"))
                .bind(row.get::<Option<String>, _>("object_bucket"))
                .bind(row.get::<Option<String>, _>("object_key"))
                .bind(before_lifecycle_status)
                .bind(node_lifecycle_status)
                .bind(&query.operator_id)
                .bind(query.request_id.as_deref())
                .bind(query.trace_id.as_deref())
                .bind(object_delete_status)
                .execute(&mut *connection)
                .await
                .map_err(|error| {
                    DriveServiceError::Internal(format!(
                        "insert dr_drive_file_sensitive_operation for expired upload failed: {error}"
                    ))
                })?;

                if node_rows > 0 {
                    let space_id: String = row.get("space_id");
                    record_drive_change_on_connection(
                        &mut connection,
                        RecordDriveChangeCommand {
                            tenant_id: &tenant_id,
                            space_id: &space_id,
                            node_id: Some(&node_id),
                            event_type: drive_events::uploader::CONTENT_EXPIRED,
                            actor_id: &query.operator_id,
                        },
                    )
                    .await?;
                }

                Ok(node_rows > 0)
            }
            .await;

            match tx_result {
                Ok(committed) => {
                    sqlx::query("COMMIT")
                        .execute(&mut *connection)
                        .await
                        .map_err(|error| {
                            DriveServiceError::Internal(format!(
                                "commit expired upload content cleanup transaction failed: {error}"
                            ))
                        })?;
                    if committed {
                        affected_count += 1;
                    }
                }
                Err(error) => {
                    let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                    return Err(error);
                }
            }
        }

        Ok(DriveSweepResult {
            scanned_count,
            affected_count,
            dry_run: query.dry_run,
        })
    }

    async fn sweep_abandoned_upload_tasks(
        &self,
        query: &SweepAbandonedUploadTasksQuery,
    ) -> Result<DriveSweepResult, DriveServiceError> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(1)
             FROM dr_drive_upload_item ui
             INNER JOIN dr_drive_upload_session us ON us.id = ui.upload_session_id
             WHERE ui.status IN ('prepared', 'uploading', 'paused', 'completing')
               AND (us.state = 'expired' OR us.expires_at_epoch_ms < $1)",
        )
        .bind(query.now_epoch_ms)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "count abandoned dr_drive_upload_item failed: {error}"
            ))
        })?;
        let scanned_count = total.min(query.limit);

        let affected_count = if query.dry_run || scanned_count == 0 {
            0
        } else {
            sqlx::query(
                "UPDATE dr_drive_upload_item
                 SET status='failed',
                     updated_by=$3,
                     updated_at=CURRENT_TIMESTAMP
                 WHERE id IN (
                     SELECT ui.id
                     FROM dr_drive_upload_item ui
                     INNER JOIN dr_drive_upload_session us ON us.id = ui.upload_session_id
                     WHERE ui.status IN ('prepared', 'uploading', 'paused', 'completing')
                       AND (us.state = 'expired' OR us.expires_at_epoch_ms < $1)
                     ORDER BY us.expires_at_epoch_ms ASC, ui.id ASC
                     LIMIT $2
                 )",
            )
            .bind(query.now_epoch_ms)
            .bind(query.limit)
            .bind(&query.operator_id)
            .execute(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "update abandoned dr_drive_upload_item failed: {error}"
                ))
            })?
            .rows_affected() as i64
        };

        Ok(DriveSweepResult {
            scanned_count,
            affected_count,
            dry_run: query.dry_run,
        })
    }

    async fn insert_maintenance_job(
        &self,
        new_job: &NewDriveMaintenanceJob,
    ) -> Result<DriveMaintenanceJob, DriveServiceError> {
        let inserted_id = next_drive_runtime_id("drive maintenance job")?;
        sqlx::query(
            "INSERT INTO dr_drive_maintenance_job (
                id, job_type, status, dry_run, scanned_count, affected_count,
                operator_id, request_id, trace_id, error_message,
                started_at, finished_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(inserted_id)
        .bind(&new_job.job_type)
        .bind(&new_job.status)
        .bind(new_job.dry_run)
        .bind(new_job.scanned_count)
        .bind(new_job.affected_count)
        .bind(&new_job.operator_id)
        .bind(&new_job.request_id)
        .bind(&new_job.trace_id)
        .bind(&new_job.error_message)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("insert dr_drive_maintenance_job failed: {error}"))
        })?;

        let row = sqlx::query(
            "SELECT id, job_type, status, dry_run, scanned_count, affected_count,
                    operator_id, request_id, trace_id, error_message,
                    CAST(started_at AS TEXT) AS started_at,
                    CAST(finished_at AS TEXT) AS finished_at,
                    CAST(created_at AS TEXT) AS created_at
             FROM dr_drive_maintenance_job
             WHERE id=$1",
        )
        .bind(inserted_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "read inserted dr_drive_maintenance_job failed: {error}"
            ))
        })?;
        map_row_to_maintenance_job(&row)
    }

    async fn list_maintenance_jobs(
        &self,
        query: &ListMaintenanceJobsQuery,
    ) -> Result<MaintenanceJobPage, DriveServiceError> {
        let offset = i64::from((query.page - 1) * query.page_size);
        let limit = i64::from(query.page_size);

        let rows = sqlx::query(
            "SELECT id, job_type, status, dry_run, scanned_count, affected_count,
                    operator_id, request_id, trace_id, error_message,
                    CAST(started_at AS TEXT) AS started_at,
                    CAST(finished_at AS TEXT) AS finished_at,
                    CAST(created_at AS TEXT) AS created_at
             FROM dr_drive_maintenance_job
             WHERE ($1 IS NULL OR job_type = $1)
               AND ($2 IS NULL OR status = $2)
               AND ($3 IS NULL OR operator_id = $3)
             ORDER BY id DESC
             LIMIT $4 OFFSET $5",
        )
        .bind(query.job_type.as_deref())
        .bind(query.status.as_deref())
        .bind(query.operator_id.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("list dr_drive_maintenance_job failed: {error}"))
        })?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(1)
             FROM dr_drive_maintenance_job
             WHERE ($1 IS NULL OR job_type = $1)
               AND ($2 IS NULL OR status = $2)
               AND ($3 IS NULL OR operator_id = $3)",
        )
        .bind(query.job_type.as_deref())
        .bind(query.status.as_deref())
        .bind(query.operator_id.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("count dr_drive_maintenance_job failed: {error}"))
        })?;

        let items = rows
            .iter()
            .map(map_row_to_maintenance_job)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(MaintenanceJobPage {
            items,
            page: query.page,
            page_size: query.page_size,
            total,
        })
    }
}

fn map_row_to_maintenance_job(row: &PgRow) -> Result<DriveMaintenanceJob, DriveServiceError> {
    let job_type: String = row.get("job_type");
    if job_type.trim().is_empty() {
        return Err(DriveServiceError::Internal(
            "dr_drive_maintenance_job.job_type is empty".to_string(),
        ));
    }
    let status: String = row.get("status");
    if status.trim().is_empty() {
        return Err(DriveServiceError::Internal(
            "dr_drive_maintenance_job.status is empty".to_string(),
        ));
    }

    Ok(DriveMaintenanceJob {
        id: row.get("id"),
        job_type,
        status,
        dry_run: get_bool(row, "dry_run")?,
        scanned_count: row.get("scanned_count"),
        affected_count: row.get("affected_count"),
        operator_id: row.get("operator_id"),
        request_id: row.get("request_id"),
        trace_id: row.get("trace_id"),
        error_message: row.get("error_message"),
        started_at: normalize_timestamp_text(row.get("started_at")),
        finished_at: normalize_timestamp_text(row.get("finished_at")),
        created_at: normalize_timestamp_text(row.get("created_at")),
    })
}

fn normalize_timestamp_text(value: String) -> String {
    let trimmed = value.trim();
    if trimmed.contains('T') {
        return trimmed.to_string();
    }
    let normalized = trimmed.replace(' ', "T");
    if normalized.ends_with('Z') {
        normalized
    } else {
        format!("{normalized}Z")
    }
}

fn get_bool(row: &PgRow, column: &str) -> Result<bool, DriveServiceError> {
    row.try_get::<bool, _>(column)
        .or_else(|_| row.try_get::<i64, _>(column).map(|value| value != 0))
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "decode dr_drive_maintenance_job.{column} as bool failed: {error}"
            ))
        })
}
