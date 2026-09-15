use sqlx::PgPool;

/// Quota reconciliation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaRecalculationResult {
    pub tenants_scanned: i64,
    pub storage_objects_retired: i64,
    pub tenants_over_quota: i64,
}

/// Reconcile tenant storage usage by retiring active objects whose nodes can no longer
/// reference them, and reporting tenants that exceed configured quota caps.
///
/// Only terminally removed nodes (`deleted`) or missing nodes lose their objects here.
/// A `trashed` node is *reversible* - the recycle bin restore path flips it back to
/// `active` - so retiring its object would leave the restored node pointing at a
/// collection that no longer exists, i.e. a file that downloads as 404. Trash storage is
/// released by an explicit retention purge, not by quota reconciliation.
pub async fn recalculate_quotas(pool: &PgPool) -> Result<QuotaRecalculationResult, sqlx::Error> {
    let storage_objects_retired = sqlx::query(
        "UPDATE dr_drive_storage_object
         SET lifecycle_status = 'deleted',
             updated_at = CURRENT_TIMESTAMP
         WHERE lifecycle_status = 'active'
           AND (
             node_id IS NULL
             OR NOT EXISTS (
               SELECT 1
               FROM dr_drive_node n
               WHERE n.tenant_id = dr_drive_storage_object.tenant_id
                 AND n.id = dr_drive_storage_object.node_id
                 AND n.lifecycle_status <> 'deleted'
             )
           )",
    )
    .execute(pool)
    .await?
    .rows_affected() as i64;

    let tenants_scanned = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT tenant_id)
         FROM dr_drive_storage_object
         WHERE lifecycle_status = 'active'",
    )
    .fetch_one(pool)
    .await?;

    let tenants_over_quota = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(1)
         FROM dr_drive_tenant_quota q
         INNER JOIN (
             SELECT tenant_id, CAST(COALESCE(SUM(content_length), 0) AS BIGINT) AS total_bytes
             FROM dr_drive_storage_object
             WHERE lifecycle_status = 'active'
             GROUP BY tenant_id
         ) usage ON usage.tenant_id = q.tenant_id
         WHERE q.max_bytes IS NOT NULL
           AND usage.total_bytes > q.max_bytes",
    )
    .fetch_one(pool)
    .await?;

    Ok(QuotaRecalculationResult {
        tenants_scanned,
        storage_objects_retired,
        tenants_over_quota,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn recalculate_quotas_retires_objects_for_deleted_nodes_only() {
        let Some((pool, _database_guard)) =
            sdkwork_drive_test_support::postgres_test_database().await
        else {
            return;
        };

        sqlx::query(
            "INSERT INTO dr_drive_storage_provider (
                id, provider_kind, name, endpoint_url, bucket, status, version,
                created_by, updated_by
             ) VALUES (
                'provider-1', 'local_filesystem', 'Local', 'file:///tmp/drive-test', 'bucket-1',
                'active', 1, 'u1', 'u1'
             )",
        )
        .execute(&pool)
        .await
        .expect("insert provider");

        sqlx::query(
            "INSERT INTO dr_drive_space (
                id, tenant_id, owner_subject_type, owner_subject_id, space_type, display_name,
                lifecycle_status, version, created_by, updated_by
             ) VALUES (
                'space-1', 'tenant-1', 'user', 'u1', 'team', 'Team', 'active', 1, 'u1', 'u1'
             )",
        )
        .execute(&pool)
        .await
        .expect("insert space");

        // A terminally deleted node: its object can never be referenced again.
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, node_type, node_name, lifecycle_status, version,
                created_by, updated_by
             ) VALUES ('node-deleted', 'tenant-1', 'space-1', 'file', 'gone.txt', 'deleted', 1, 'u1', 'u1')",
        )
        .execute(&pool)
        .await
        .expect("insert deleted node");

        // A trashed node: still restorable through the recycle bin.
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, node_type, node_name, lifecycle_status, version,
                created_by, updated_by
             ) VALUES ('node-trashed', 'tenant-1', 'space-1', 'file', 'doc.txt', 'trashed', 1, 'u1', 'u1')",
        )
        .execute(&pool)
        .await
        .expect("insert trashed node");

        for (object_id, node_id, object_key) in [
            ("obj-deleted", "node-deleted", "objects/obj-deleted.txt"),
            ("obj-trashed", "node-trashed", "objects/obj-trashed.txt"),
        ] {
            sqlx::query(
                "INSERT INTO dr_drive_storage_object (
                    id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
                    content_type, content_length, checksum_sha256_hex, lifecycle_status,
                    created_by, updated_by
                 ) VALUES (
                    $1, 'tenant-1', $2, 1, 'provider-1', 'bucket-1', $3,
                    'text/plain', 128,
                    'sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
                    'active', 'u1', 'u1'
                 )",
            )
            .bind(object_id)
            .bind(node_id)
            .bind(object_key)
            .execute(&pool)
            .await
            .expect("insert storage object");
        }

        let result = recalculate_quotas(&pool).await.expect("recalculate quotas");
        assert_eq!(result.storage_objects_retired, 1);
        assert_eq!(result.tenants_scanned, 1);

        for (object_id, expected_status) in [
            ("obj-deleted", "deleted"),
            // Restoring this node must still find its content.
            ("obj-trashed", "active"),
        ] {
            let status: String = sqlx::query_scalar(
                "SELECT lifecycle_status FROM dr_drive_storage_object WHERE id = $1",
            )
            .bind(object_id)
            .fetch_one(&pool)
            .await
            .expect("load object status");
            assert_eq!(status, expected_status, "unexpected status for {object_id}");
        }
    }
}
