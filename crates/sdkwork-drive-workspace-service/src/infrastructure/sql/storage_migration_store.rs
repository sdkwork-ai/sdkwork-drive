use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::storage_migration::{
    DriveStorageMigration, DriveStorageMigrationItem, DriveStorageMigrationItemOutcome,
    DriveStorageMigrationItemStatus, DriveStorageMigrationSourceObject,
    DriveStorageMigrationStatus, NewDriveStorageMigration, NewDriveStorageMigrationItem,
};
use crate::ports::storage_migration_store::DriveStorageMigrationStore;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlStorageMigrationStore {
    pool: PgPool,
}

impl SqlStorageMigrationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveStorageMigrationStore for SqlStorageMigrationStore {
    async fn create_storage_migration(
        &self,
        migration: &NewDriveStorageMigration,
    ) -> Result<DriveStorageMigration, DriveServiceError> {
        // The header opens with `objects_total = 0`; the real count is stamped
        // when the queue is appended. A run that was planned but never seeded
        // therefore reports zero work rather than a phantom total.
        let header = sqlx::query(
            "INSERT INTO dr_drive_storage_migration
                (id, tenant_id, source_provider_id, target_provider_id, name, status,
                 objects_total, target_bucket, apply_binding_switch, created_by, updated_by)
             VALUES ($1, $2, $3, $4, $5, 'pending', 0, $6, $7, $8, $8)
             RETURNING id, tenant_id, source_provider_id, target_provider_id, name, status,
                       objects_total, objects_copied, objects_failed, bytes_copied, target_bucket,
                       apply_binding_switch, failure_message, created_by, updated_by",
        )
        .bind(&migration.id)
        .bind(&migration.tenant_id)
        .bind(&migration.source_provider_id)
        .bind(&migration.target_provider_id)
        .bind(&migration.name)
        .bind(migration.target_bucket.as_deref())
        .bind(migration.apply_binding_switch)
        .bind(&migration.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_storage_migration_error("insert storage migration"))?;

        map_migration_row(&header)
    }

    async fn append_storage_migration_items(
        &self,
        migration_id: &str,
        items: &[NewDriveStorageMigrationItem],
    ) -> Result<(), DriveServiceError> {
        if items.is_empty() {
            return Ok(());
        }
        let mut transaction = self.pool.begin().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "begin storage migration queue transaction failed: {error}"
            ))
        })?;

        for item in items {
            // `ON CONFLICT DO NOTHING` makes a re-seed after a crash a no-op
            // instead of a duplicate-key failure, keyed on the run/object pair.
            // Tenant and both provider ids are copied from the run header
            // rather than bound from the caller, so a queue entry can never
            // claim a tenant or a provider the run does not own.
            sqlx::query(
                "INSERT INTO dr_drive_storage_migration_item
                    (id, migration_id, tenant_id, storage_object_id, source_provider_id,
                     source_bucket, source_object_key, target_provider_id, target_bucket,
                     target_object_key, content_type, content_length, checksum_sha256_hex, status)
                 SELECT $1, m.id, m.tenant_id, $3, m.source_provider_id,
                        $4, $5, m.target_provider_id, $6, $7, $8, $9, $10, 'pending'
                 FROM dr_drive_storage_migration m
                 WHERE m.id = $2
                 ON CONFLICT (migration_id, storage_object_id) DO NOTHING",
            )
            .bind(&item.id)
            .bind(migration_id)
            .bind(&item.storage_object_id)
            .bind(&item.source_bucket)
            .bind(&item.source_object_key)
            .bind(&item.target_bucket)
            .bind(&item.target_object_key)
            .bind(&item.content_type)
            .bind(item.content_length)
            .bind(&item.checksum_sha256_hex)
            .execute(&mut *transaction)
            .await
            .map_err(map_storage_migration_error("insert storage migration item"))?;
        }

        // The total is re-derived from the queue rather than incremented, so a
        // re-seed that inserted nothing cannot inflate it.
        sqlx::query(
            "UPDATE dr_drive_storage_migration
             SET objects_total = (
                    SELECT count(*)::bigint
                    FROM dr_drive_storage_migration_item
                    WHERE migration_id = $1
                 ),
                 updated_at = NOW()
             WHERE id = $1",
        )
        .bind(migration_id)
        .execute(&mut *transaction)
        .await
        .map_err(map_storage_migration_error("stamp storage migration total"))?;

        transaction.commit().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "commit storage migration queue transaction failed: {error}"
            ))
        })
    }

    async fn list_storage_provider_objects_for_migration(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<Vec<DriveStorageMigrationSourceObject>, DriveServiceError> {
        let rows = sqlx::query(
            "SELECT id, bucket, object_key, content_type, content_length, checksum_sha256_hex
             FROM dr_drive_storage_object
             WHERE tenant_id = $1
               AND storage_provider_id = $2
               AND lifecycle_status = 'active'
             ORDER BY id ASC",
        )
        .bind(tenant_id)
        .bind(provider_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_storage_migration_error("list migration source objects"))?;

        Ok(rows
            .iter()
            .map(|row| DriveStorageMigrationSourceObject {
                id: row.get("id"),
                bucket: row.get("bucket"),
                object_key: row.get("object_key"),
                content_type: row.get("content_type"),
                content_length: row.get("content_length"),
                checksum_sha256_hex: row.get("checksum_sha256_hex"),
            })
            .collect())
    }

    async fn find_storage_migration(
        &self,
        migration_id: &str,
    ) -> Result<Option<DriveStorageMigration>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, source_provider_id, target_provider_id, name, status,
                    objects_total, objects_copied, objects_failed, bytes_copied, target_bucket,
                    apply_binding_switch, failure_message, created_by, updated_by
             FROM dr_drive_storage_migration
             WHERE id = $1",
        )
        .bind(migration_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_storage_migration_error("find storage migration"))?;
        row.as_ref().map(map_migration_row).transpose()
    }

    async fn list_storage_migrations(
        &self,
        tenant_id: &str,
        status: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DriveStorageMigration>, DriveServiceError> {
        let rows = sqlx::query(
            "SELECT id, tenant_id, source_provider_id, target_provider_id, name, status,
                    objects_total, objects_copied, objects_failed, bytes_copied, target_bucket,
                    apply_binding_switch, failure_message, created_by, updated_by
             FROM dr_drive_storage_migration
             WHERE tenant_id = $1
               AND ($2::text IS NULL OR status = $2)
             ORDER BY created_at DESC, id DESC
             OFFSET $3 LIMIT $4",
        )
        .bind(tenant_id)
        .bind(status)
        .bind(offset)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(map_storage_migration_error("list storage migrations"))?;

        rows.iter().map(map_migration_row).collect()
    }

    async fn list_storage_migration_items(
        &self,
        migration_id: &str,
        status: Option<DriveStorageMigrationItemStatus>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DriveStorageMigrationItem>, DriveServiceError> {
        let status = status.map(DriveStorageMigrationItemStatus::as_str);
        let rows = sqlx::query(
            "SELECT id, migration_id, tenant_id, storage_object_id, source_provider_id,
                    source_bucket, source_object_key, target_provider_id, target_bucket,
                    target_object_key, content_type, content_length, checksum_sha256_hex,
                    status, verified_checksum_sha256_hex, failure_message
             FROM dr_drive_storage_migration_item
             WHERE migration_id = $1
               AND ($2::text IS NULL OR status = $2)
             ORDER BY id ASC
             OFFSET $3 LIMIT $4",
        )
        .bind(migration_id)
        .bind(status)
        .bind(offset)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(map_storage_migration_error("list storage migration items"))?;

        rows.iter().map(map_migration_item_row).collect()
    }

    async fn count_storage_migration_items(
        &self,
        migration_id: &str,
    ) -> Result<[i64; 3], DriveServiceError> {
        let row = sqlx::query(
            "SELECT
               count(*) FILTER (WHERE status = 'pending')::bigint AS pending_count,
               count(*) FILTER (WHERE status = 'copied')::bigint AS copied_count,
               count(*) FILTER (WHERE status = 'failed')::bigint AS failed_count
             FROM dr_drive_storage_migration_item
             WHERE migration_id = $1",
        )
        .bind(migration_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_storage_migration_error("count storage migration items"))?;

        Ok([
            row.get::<i64, _>("pending_count"),
            row.get::<i64, _>("copied_count"),
            row.get::<i64, _>("failed_count"),
        ])
    }

    async fn record_storage_migration_item_outcome(
        &self,
        migration_id: &str,
        item_id: &str,
        outcome: &DriveStorageMigrationItemOutcome,
        updated_by: &str,
    ) -> Result<(), DriveServiceError> {
        let mut transaction = self.pool.begin().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "begin storage migration item transaction failed: {error}"
            ))
        })?;

        let next_status = if outcome.copied { "copied" } else { "failed" };

        // Read the prior status under the transaction, then transition only if
        // the item is still in a state an attempt may settle.
        //
        // Which prior states are admissible:
        // - `pending` is a first attempt;
        // - `failed`  is a retry, which is how a resumed run clears a transient
        //             error;
        // - `copied`  is deliberately excluded. Re-recording a copied item
        //             would double-count it into the run totals, and the copy
        //             is already known-good, so a second attempt teaches
        //             nothing.
        //
        // `FOR UPDATE` holds the row for the rest of the transaction, so two
        // workers racing the same item cannot both read `pending`.
        let previous_status: Option<String> = sqlx::query_scalar(
            "SELECT status FROM dr_drive_storage_migration_item
             WHERE id = $1 AND migration_id = $2
             FOR UPDATE",
        )
        .bind(item_id)
        .bind(migration_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(map_storage_migration_error("read storage migration item"))?;

        // Missing row, or an item another worker already settled.
        let settled_by_another_worker = match previous_status.as_deref() {
            Some("pending") | Some("failed") => false,
            _ => true,
        };
        if settled_by_another_worker {
            transaction.rollback().await.map_err(|error| {
                DriveServiceError::Internal(format!(
                    "rollback storage migration item transaction failed: {error}"
                ))
            })?;
            return Ok(());
        }
        let was_retry = previous_status.as_deref() == Some("failed");

        sqlx::query(
            "UPDATE dr_drive_storage_migration_item
             SET status = $3,
                 verified_checksum_sha256_hex = $4,
                 failure_message = $5,
                 updated_at = NOW()
             WHERE id = $1 AND migration_id = $2",
        )
        .bind(item_id)
        .bind(migration_id)
        .bind(next_status)
        .bind(outcome.verified_checksum_sha256_hex.as_deref())
        .bind(outcome.failure_message.as_deref())
        .execute(&mut *transaction)
        .await
        .map_err(map_storage_migration_error("update storage migration item"))?;

        // Counters move in the same transaction as the item transition. A split
        // write would let a crash leave the run reporting fewer copies than the
        // items actually show, which is the one number an operator uses to
        // decide whether the data is safe to switch over.
        //
        // `was_retry` keeps the totals describing the *current* state of the
        // queue rather than the number of attempts ever made: a retry that
        // finally succeeds moves one object from `failed` to `copied`, so
        // `failed` must come back down or the run would never be able to reach
        // the zero-failure condition that gates the switch.
        let counter_sql = if outcome.copied && was_retry {
            "UPDATE dr_drive_storage_migration
             SET objects_copied = objects_copied + 1,
                 objects_failed = GREATEST(objects_failed - 1, 0),
                 bytes_copied = bytes_copied + $2,
                 updated_by = $3,
                 updated_at = NOW()
             WHERE id = $1"
        } else if outcome.copied {
            "UPDATE dr_drive_storage_migration
             SET objects_copied = objects_copied + 1,
                 bytes_copied = bytes_copied + $2,
                 updated_by = $3,
                 updated_at = NOW()
             WHERE id = $1"
        } else {
            "UPDATE dr_drive_storage_migration
             SET objects_failed = objects_failed + 1,
                 updated_by = $2,
                 updated_at = NOW()
             WHERE id = $1"
        };
        let counter = if outcome.copied {
            sqlx::query(counter_sql)
                .bind(migration_id)
                .bind(outcome.bytes)
                .bind(updated_by)
                .execute(&mut *transaction)
                .await
        } else {
            sqlx::query(counter_sql)
                .bind(migration_id)
                .bind(updated_by)
                .execute(&mut *transaction)
                .await
        }
        .map_err(map_storage_migration_error(
            "advance storage migration counters",
        ))?;

        if counter.rows_affected() == 0 {
            return Err(DriveServiceError::NotFound(format!(
                "storage migration {migration_id} not found"
            )));
        }

        transaction.commit().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "commit storage migration item transaction failed: {error}"
            ))
        })
    }

    async fn transition_storage_migration(
        &self,
        migration_id: &str,
        expected_status: DriveStorageMigrationStatus,
        next_status: DriveStorageMigrationStatus,
        failure_message: Option<&str>,
        updated_by: &str,
    ) -> Result<bool, DriveServiceError> {
        // `finished_at` is stamped for the two terminal states only. A resumable
        // `failed` run deliberately keeps it null so the console cannot present
        // it as finished.
        let finished = next_status.is_terminal();
        let result = sqlx::query(
            "UPDATE dr_drive_storage_migration
             SET status = $3,
                 failure_message = $4,
                 updated_by = $5,
                 updated_at = NOW(),
                 started_at = CASE
                     WHEN $3 = 'running' AND started_at IS NULL THEN NOW()
                     ELSE started_at
                 END,
                 finished_at = CASE WHEN $6 THEN NOW() ELSE finished_at END
             WHERE id = $1 AND status = $2",
        )
        .bind(migration_id)
        .bind(expected_status.as_str())
        .bind(next_status.as_str())
        .bind(failure_message)
        .bind(updated_by)
        .bind(finished)
        .execute(&self.pool)
        .await
        .map_err(map_storage_migration_error("transition storage migration"))?;

        Ok(result.rows_affected() > 0)
    }

    async fn apply_storage_migration_repoint(
        &self,
        migration_id: &str,
        target_provider_id: &str,
        target_bucket: &str,
        updated_by: &str,
    ) -> Result<i64, DriveServiceError> {
        // One statement, so there is never a moment where part of the tenant
        // reads from the old provider. `lifecycle_status = 'active'` is not
        // filtered: a soft-deleted object still has bytes in the old bucket and
        // a future restore would read them, so its location record must move
        // too.
        let result = sqlx::query(
            "UPDATE dr_drive_storage_object o
             SET storage_provider_id = $2,
                 bucket = $3,
                 updated_by = $4,
                 updated_at = NOW()
             FROM dr_drive_storage_migration_item i
             WHERE i.migration_id = $1
               AND i.storage_object_id = o.id
               AND i.status = 'copied'",
        )
        .bind(migration_id)
        .bind(target_provider_id)
        .bind(target_bucket)
        .bind(updated_by)
        .execute(&self.pool)
        .await
        .map_err(map_storage_migration_error(
            "apply storage migration repoint",
        ))?;

        Ok(result.rows_affected() as i64)
    }
}

fn map_storage_migration_error(context: &'static str) -> impl Fn(sqlx::Error) -> DriveServiceError {
    move |error| DriveServiceError::Internal(format!("{context} failed: {error}"))
}

fn map_migration_row(
    row: &sqlx::postgres::PgRow,
) -> Result<DriveStorageMigration, DriveServiceError> {
    Ok(DriveStorageMigration {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        source_provider_id: row.get("source_provider_id"),
        target_provider_id: row.get("target_provider_id"),
        name: row.get("name"),
        status: DriveStorageMigrationStatus::parse(row.get::<String, _>("status").as_str())?,
        objects_total: row.get("objects_total"),
        objects_copied: row.get("objects_copied"),
        objects_failed: row.get("objects_failed"),
        bytes_copied: row.get("bytes_copied"),
        target_bucket: row.get("target_bucket"),
        apply_binding_switch: row.get("apply_binding_switch"),
        failure_message: row.get("failure_message"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    })
}

fn map_migration_item_row(
    row: &sqlx::postgres::PgRow,
) -> Result<DriveStorageMigrationItem, DriveServiceError> {
    Ok(DriveStorageMigrationItem {
        id: row.get("id"),
        migration_id: row.get("migration_id"),
        tenant_id: row.get("tenant_id"),
        storage_object_id: row.get("storage_object_id"),
        source_provider_id: row.get("source_provider_id"),
        source_bucket: row.get("source_bucket"),
        source_object_key: row.get("source_object_key"),
        target_provider_id: row.get("target_provider_id"),
        target_bucket: row.get("target_bucket"),
        target_object_key: row.get("target_object_key"),
        content_type: row.get("content_type"),
        content_length: row.get("content_length"),
        checksum_sha256_hex: row.get("checksum_sha256_hex"),
        status: DriveStorageMigrationItemStatus::parse(row.get::<String, _>("status").as_str())?,
        verified_checksum_sha256_hex: row.get("verified_checksum_sha256_hex"),
        failure_message: row.get("failure_message"),
    })
}
