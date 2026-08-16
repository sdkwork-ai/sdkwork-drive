use async_trait::async_trait;
use sqlx::{PgConnection, PgPool, Row};

use crate::infrastructure::sql::managed_website_tree_guard::{
    ManagedWebsiteTreeSystemOverride, ManagedWebsiteTreeSystemOverrideReason,
};
use crate::infrastructure::sql::{begin_transaction_sql, next_drive_runtime_id};
use crate::ports::website_publishing_maintenance::{
    DriveWebsitePublishingMaintenanceStore, WebsiteTreeCleanupCandidate, WebsiteTreeCleanupKind,
    WebsiteTreeStorageObject,
};
use crate::DriveServiceError;

const MAXIMUM_TREE_DEPTH: i64 = 128;

#[derive(Debug, Clone)]
pub struct SqlWebsitePublishingMaintenanceStore {
    pool: PgPool,
}

impl SqlWebsitePublishingMaintenanceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn begin(&self) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, DriveServiceError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            internal("acquire website publishing maintenance connection", error)
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| internal("begin website publishing maintenance transaction", error))?;
        Ok(connection)
    }
}

#[async_trait]
impl DriveWebsitePublishingMaintenanceStore for SqlWebsitePublishingMaintenanceStore {
    async fn expire_stale_syncs(
        &self,
        limit: i64,
        operator_id: &str,
    ) -> Result<u64, DriveServiceError> {
        let mut connection = self.begin().await?;
        let result = async {
            let rows = sqlx::query(
                "SELECT id, tenant_id
                 FROM dr_drive_website_sync
                 WHERE expires_at <= CURRENT_TIMESTAMP
                   AND (
                     sync_status IN ('created', 'uploading', 'ready')
                     OR (
                       sync_status='validating'
                       AND lease_expires_at <= CURRENT_TIMESTAMP
                     )
                   )
                 ORDER BY expires_at ASC, id ASC
                 LIMIT $1",
            )
            .bind(limit)
            .fetch_all(&mut *connection)
            .await
            .map_err(|error| internal("list stale WebsiteSync rows", error))?;
            let mut expired = 0_u64;
            for row in rows {
                let sync_id: String = row.get("id");
                let tenant_id: String = row.get("tenant_id");
                let updated = sqlx::query(
                    "UPDATE dr_drive_website_sync
                     SET sync_status='expired', lease_owner=NULL, lease_token=NULL,
                         lease_expires_at=NULL, error_code='WEBSITE_SYNC_EXPIRED',
                         error_summary='Website sync expired before activation',
                         updated_by=$1, updated_at=CURRENT_TIMESTAMP, version=version + 1
                     WHERE tenant_id=$2 AND id=$3
                       AND expires_at <= CURRENT_TIMESTAMP
                       AND (
                         sync_status IN ('created', 'uploading', 'ready')
                         OR (
                           sync_status='validating'
                           AND lease_expires_at <= CURRENT_TIMESTAMP
                         )
                       )",
                )
                .bind(operator_id)
                .bind(&tenant_id)
                .bind(&sync_id)
                .execute(&mut *connection)
                .await
                .map_err(|error| internal("expire stale WebsiteSync", error))?;
                if updated.rows_affected() == 1 {
                    insert_audit(
                        &mut connection,
                        &tenant_id,
                        "drive.website_sync.expired",
                        "website_sync",
                        &sync_id,
                        operator_id,
                    )
                    .await?;
                    expired += 1;
                }
            }
            Ok(expired)
        }
        .await;
        finish_transaction(&mut connection, result).await
    }

    async fn claim_next_cleanup_candidate(
        &self,
        operator_id: &str,
    ) -> Result<Option<WebsiteTreeCleanupCandidate>, DriveServiceError> {
        let mut connection = self.begin().await?;
        let result = async {
            let mut generation = select_generation_candidate(&mut connection, "deleting").await?;
            if generation.is_none() {
                generation = select_generation_candidate(&mut connection, "expired").await?;
                if let Some((generation_id, tenant_id, _)) = generation.as_ref() {
                    let claimed = sqlx::query(
                        "UPDATE dr_drive_website_root_generation
                         SET generation_status='deleting'
                         WHERE id=$1 AND tenant_id=$2 AND generation_status='expired'
                           AND (retention_until IS NULL OR retention_until <= CURRENT_TIMESTAMP)",
                    )
                    .bind(generation_id)
                    .bind(tenant_id)
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| internal("claim expired WebsiteRoot generation", error))?;
                    if claimed.rows_affected() != 1 {
                        generation = None;
                    } else {
                        ManagedWebsiteTreeSystemOverride::authorize(
                            ManagedWebsiteTreeSystemOverrideReason::ExpiredWebsitePublishingCleanup,
                            operator_id,
                        )?
                        .record_on_connection(
                            &mut connection,
                            tenant_id,
                            "website_root_generation",
                            generation_id,
                        )
                        .await?;
                    }
                }
            }
            if let Some((resource_id, tenant_id, root_node_id)) = generation {
                let delete_tree = !tree_has_live_reference(
                    &mut connection,
                    &tenant_id,
                    &root_node_id,
                    Some(&resource_id),
                )
                .await?;
                return Ok(Some(WebsiteTreeCleanupCandidate {
                    kind: WebsiteTreeCleanupKind::ExpiredGeneration,
                    resource_id,
                    tenant_id,
                    root_node_id,
                    delete_tree,
                }));
            }

            let row = sqlx::query(
                "SELECT sync.id, sync.tenant_id, sync.staging_node_id
                 FROM dr_drive_website_sync sync
                 INNER JOIN dr_drive_node node
                   ON node.id=sync.staging_node_id AND node.tenant_id=sync.tenant_id
                 WHERE sync.sync_status IN ('failed', 'aborted', 'expired')
                   AND node.lifecycle_status != 'deleted'
                   AND NOT EXISTS (
                     SELECT 1 FROM dr_drive_website_root_generation generation
                     WHERE generation.tenant_id=sync.tenant_id
                       AND generation.root_node_id=sync.staging_node_id
                   )
                 ORDER BY sync.updated_at ASC, sync.id ASC
                 LIMIT 1",
            )
            .fetch_optional(&mut *connection)
            .await
            .map_err(|error| internal("select terminal WebsiteSync cleanup candidate", error))?;
            Ok(row.map(|row| WebsiteTreeCleanupCandidate {
                kind: WebsiteTreeCleanupKind::TerminalSync,
                resource_id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                root_node_id: row.get("staging_node_id"),
                delete_tree: true,
            }))
        }
        .await;
        finish_transaction(&mut connection, result).await
    }

    async fn list_candidate_storage_objects(
        &self,
        candidate: &WebsiteTreeCleanupCandidate,
        limit: i64,
    ) -> Result<Vec<WebsiteTreeStorageObject>, DriveServiceError> {
        if !candidate.delete_tree {
            return Ok(Vec::new());
        }
        ensure_candidate_current(&self.pool, candidate).await?;
        let rows = sqlx::query(
            "WITH RECURSIVE tree(id, depth) AS (
               SELECT id, 0 FROM dr_drive_node WHERE tenant_id=$1 AND id=$2
               UNION ALL
               SELECT child.id, tree.depth + 1
               FROM dr_drive_node child
               INNER JOIN tree ON child.parent_node_id=tree.id
               WHERE child.tenant_id=$1 AND tree.depth < $3
             )
             SELECT object.id, object.storage_provider_id,
                    provider.version AS storage_provider_version,
                    object.bucket, object.object_key
             FROM dr_drive_storage_object object
             INNER JOIN tree ON tree.id=object.node_id
             INNER JOIN dr_drive_storage_provider provider
               ON provider.id=object.storage_provider_id
             WHERE object.tenant_id=$1 AND object.lifecycle_status='active'
             ORDER BY object.id ASC
             LIMIT $4",
        )
        .bind(&candidate.tenant_id)
        .bind(&candidate.root_node_id)
        .bind(MAXIMUM_TREE_DEPTH)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| internal("list website tree storage objects", error))?;
        Ok(rows
            .into_iter()
            .map(|row| WebsiteTreeStorageObject {
                id: row.get("id"),
                storage_provider_id: row.get("storage_provider_id"),
                storage_provider_version: row.get("storage_provider_version"),
                bucket: row.get("bucket"),
                object_key: row.get("object_key"),
            })
            .collect())
    }

    async fn mark_storage_object_deleted(
        &self,
        candidate: &WebsiteTreeCleanupCandidate,
        storage_object_id: &str,
        operator_id: &str,
    ) -> Result<bool, DriveServiceError> {
        ensure_candidate_current(&self.pool, candidate).await?;
        let updated = sqlx::query(
            "WITH RECURSIVE tree(id, depth) AS (
               SELECT id, 0 FROM dr_drive_node WHERE tenant_id=$1 AND id=$2
               UNION ALL
               SELECT child.id, tree.depth + 1
               FROM dr_drive_node child
               INNER JOIN tree ON child.parent_node_id=tree.id
               WHERE child.tenant_id=$1 AND tree.depth < $3
             )
             UPDATE dr_drive_storage_object
             SET lifecycle_status='deleted', updated_by=$4, updated_at=CURRENT_TIMESTAMP
             WHERE tenant_id=$1 AND id=$5 AND lifecycle_status='active'
               AND node_id IN (SELECT id FROM tree)",
        )
        .bind(&candidate.tenant_id)
        .bind(&candidate.root_node_id)
        .bind(MAXIMUM_TREE_DEPTH)
        .bind(operator_id)
        .bind(storage_object_id)
        .execute(&self.pool)
        .await
        .map_err(|error| internal("mark website tree storage object deleted", error))?;
        Ok(updated.rows_affected() == 1)
    }

    async fn complete_cleanup_candidate(
        &self,
        candidate: &WebsiteTreeCleanupCandidate,
        operator_id: &str,
    ) -> Result<u64, DriveServiceError> {
        let mut connection = self.begin().await?;
        let result = async {
            ensure_candidate_current(&mut *connection, candidate).await?;
            let delete_tree = match candidate.kind {
                WebsiteTreeCleanupKind::ExpiredGeneration => {
                    !tree_has_live_reference(
                        &mut connection,
                        &candidate.tenant_id,
                        &candidate.root_node_id,
                        Some(&candidate.resource_id),
                    )
                    .await?
                }
                WebsiteTreeCleanupKind::TerminalSync => true,
            };
            let mut deleted_nodes = 0_u64;
            if delete_tree {
                let active_objects: i64 = count_active_tree_objects(
                    &mut connection,
                    &candidate.tenant_id,
                    &candidate.root_node_id,
                )
                .await?;
                if active_objects != 0 {
                    return Err(DriveServiceError::Conflict(
                        "website tree cleanup still has active storage objects".to_string(),
                    ));
                }
                deleted_nodes = sqlx::query(
                    "WITH RECURSIVE tree(id, depth) AS (
                       SELECT id, 0 FROM dr_drive_node WHERE tenant_id=$1 AND id=$2
                       UNION ALL
                       SELECT child.id, tree.depth + 1
                       FROM dr_drive_node child
                       INNER JOIN tree ON child.parent_node_id=tree.id
                       WHERE child.tenant_id=$1 AND tree.depth < $3
                     )
                     UPDATE dr_drive_node
                     SET lifecycle_status='deleted', version=version + 1,
                         updated_by=$4, updated_at=CURRENT_TIMESTAMP
                     WHERE tenant_id=$1 AND lifecycle_status != 'deleted'
                       AND id IN (SELECT id FROM tree)",
                )
                .bind(&candidate.tenant_id)
                .bind(&candidate.root_node_id)
                .bind(MAXIMUM_TREE_DEPTH)
                .bind(operator_id)
                .execute(&mut *connection)
                .await
                .map_err(|error| internal("delete website tree nodes", error))?
                .rows_affected();
            }
            if candidate.kind == WebsiteTreeCleanupKind::ExpiredGeneration {
                let updated = sqlx::query(
                    "UPDATE dr_drive_website_root_generation
                     SET generation_status='deleted'
                     WHERE tenant_id=$1 AND id=$2 AND generation_status='deleting'",
                )
                .bind(&candidate.tenant_id)
                .bind(&candidate.resource_id)
                .execute(&mut *connection)
                .await
                .map_err(|error| internal("complete WebsiteRoot generation cleanup", error))?;
                if updated.rows_affected() != 1 {
                    return Err(DriveServiceError::Conflict(
                        "WebsiteRoot generation cleanup fence changed".to_string(),
                    ));
                }
            }
            ManagedWebsiteTreeSystemOverride::authorize(
                ManagedWebsiteTreeSystemOverrideReason::ExpiredWebsitePublishingCleanup,
                operator_id,
            )?
            .record_on_connection(
                &mut connection,
                &candidate.tenant_id,
                candidate.kind.as_str(),
                &candidate.resource_id,
            )
            .await?;
            Ok(deleted_nodes)
        }
        .await;
        finish_transaction(&mut connection, result).await
    }
}

async fn select_generation_candidate(
    connection: &mut PgConnection,
    status: &str,
) -> Result<Option<(String, String, String)>, DriveServiceError> {
    let row = sqlx::query(
        "SELECT id, tenant_id, root_node_id
         FROM dr_drive_website_root_generation
         WHERE generation_status=$1
           AND ($1='deleting' OR retention_until IS NULL OR retention_until <= CURRENT_TIMESTAMP)
         ORDER BY retention_until ASC, generation_no ASC, id ASC
         LIMIT 1",
    )
    .bind(status)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| internal("select WebsiteRoot generation cleanup candidate", error))?;
    Ok(row.map(|row| (row.get("id"), row.get("tenant_id"), row.get("root_node_id"))))
}

async fn ensure_candidate_current<'e, E>(
    executor: E,
    candidate: &WebsiteTreeCleanupCandidate,
) -> Result<(), DriveServiceError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let current: i64 = match candidate.kind {
        WebsiteTreeCleanupKind::ExpiredGeneration => {
            sqlx::query_scalar(
                "SELECT COUNT(1) FROM dr_drive_website_root_generation
             WHERE tenant_id=$1 AND id=$2 AND root_node_id=$3
               AND generation_status='deleting'",
            )
            .bind(&candidate.tenant_id)
            .bind(&candidate.resource_id)
            .bind(&candidate.root_node_id)
            .fetch_one(executor)
            .await
        }
        WebsiteTreeCleanupKind::TerminalSync => {
            sqlx::query_scalar(
                "SELECT COUNT(1) FROM dr_drive_website_sync
             WHERE tenant_id=$1 AND id=$2 AND staging_node_id=$3
               AND sync_status IN ('failed', 'aborted', 'expired')
               AND NOT EXISTS (
                 SELECT 1 FROM dr_drive_website_root_generation generation
                 WHERE generation.tenant_id=$1 AND generation.root_node_id=$3
               )",
            )
            .bind(&candidate.tenant_id)
            .bind(&candidate.resource_id)
            .bind(&candidate.root_node_id)
            .fetch_one(executor)
            .await
        }
    }
    .map_err(|error| internal("validate website tree cleanup candidate", error))?;
    if current != 1 {
        return Err(DriveServiceError::Conflict(
            "website tree cleanup candidate changed".to_string(),
        ));
    }
    Ok(())
}

async fn tree_has_live_reference(
    connection: &mut PgConnection,
    tenant_id: &str,
    root_node_id: &str,
    excluded_generation_id: Option<&str>,
) -> Result<bool, DriveServiceError> {
    let references: i64 = sqlx::query_scalar(
        "SELECT (
           SELECT COUNT(1) FROM dr_drive_website_root root
           WHERE root.tenant_id=$1 AND root.active_node_id=$2
             AND root.root_status != 'archived'
         ) + (
           SELECT COUNT(1) FROM dr_drive_website_root_generation generation
           WHERE generation.tenant_id=$1 AND generation.root_node_id=$2
             AND generation.generation_status IN ('current', 'retained')
             AND ($3 IS NULL OR generation.id != $3)
         )",
    )
    .bind(tenant_id)
    .bind(root_node_id)
    .bind(excluded_generation_id)
    .fetch_one(&mut *connection)
    .await
    .map_err(|error| internal("check live website tree references", error))?;
    Ok(references > 0)
}

async fn count_active_tree_objects(
    connection: &mut PgConnection,
    tenant_id: &str,
    root_node_id: &str,
) -> Result<i64, DriveServiceError> {
    sqlx::query_scalar(
        "WITH RECURSIVE tree(id, depth) AS (
           SELECT id, 0 FROM dr_drive_node WHERE tenant_id=$1 AND id=$2
           UNION ALL
           SELECT child.id, tree.depth + 1
           FROM dr_drive_node child
           INNER JOIN tree ON child.parent_node_id=tree.id
           WHERE child.tenant_id=$1 AND tree.depth < $3
         )
         SELECT COUNT(1)
         FROM dr_drive_storage_object object
         INNER JOIN tree ON tree.id=object.node_id
         WHERE object.tenant_id=$1 AND object.lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(root_node_id)
    .bind(MAXIMUM_TREE_DEPTH)
    .fetch_one(&mut *connection)
    .await
    .map_err(|error| internal("count active website tree storage objects", error))
}

async fn insert_audit(
    connection: &mut PgConnection,
    tenant_id: &str,
    action: &str,
    resource_type: &str,
    resource_id: &str,
    operator_id: &str,
) -> Result<(), DriveServiceError> {
    let audit_id = next_drive_runtime_id("website publishing maintenance audit event")?;
    sqlx::query(
        "INSERT INTO dr_drive_audit_event (
           id, tenant_id, action, resource_type, resource_id, operator_id,
           request_id, trace_id
         ) VALUES ($1, $2, $3, $4, $5, $6, NULL, NULL)",
    )
    .bind(audit_id)
    .bind(tenant_id)
    .bind(action)
    .bind(resource_type)
    .bind(resource_id)
    .bind(operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("insert website publishing maintenance audit", error))?;
    Ok(())
}

async fn finish_transaction<T>(
    connection: &mut PgConnection,
    result: Result<T, DriveServiceError>,
) -> Result<T, DriveServiceError> {
    match result {
        Ok(value) => {
            sqlx::query("COMMIT")
                .execute(&mut *connection)
                .await
                .map_err(|error| internal("commit website publishing maintenance", error))?;
            Ok(value)
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            Err(error)
        }
    }
}

fn internal(operation: &str, error: sqlx::Error) -> DriveServiceError {
    DriveServiceError::Internal(format!("{operation} failed: {error}"))
}
