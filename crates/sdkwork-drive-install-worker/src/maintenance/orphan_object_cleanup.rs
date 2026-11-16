use sqlx::PgPool;

/// Cleanup result for orphan nodes.
#[derive(Debug, Clone)]
pub struct OrphanCleanupResult {
    pub orphaned_nodes: i64,
    pub cleaned_objects: i64,
}

/// Clean up orphan nodes that reference missing spaces or parents.
pub async fn cleanup_orphan_objects(pool: &PgPool) -> Result<OrphanCleanupResult, sqlx::Error> {
    let mut connection = pool.acquire().await?;
    sqlx::query("BEGIN").execute(&mut *connection).await?;

    let cleanup_result = async {
        // Never cascade-delete a node while it still owns storage metadata:
        // the cascade would remove the only locator for the provider bytes and
        // leak them permanently. Mark those objects for the physical purge
        // sweep first; the node is deleted on a later run, after the purge
        // removes the metadata rows.
        let cleaned_objects = sqlx::query(
            "UPDATE dr_drive_storage_object object
             SET lifecycle_status = 'deleted',
                 updated_by = 'install-worker',
                 updated_at = CURRENT_TIMESTAMP
             WHERE object.lifecycle_status = 'active'
               AND object.node_id IN (
                   SELECT node.id
                   FROM dr_drive_node node
                   WHERE node.space_id NOT IN (SELECT id FROM dr_drive_space)
                      OR (
                          node.parent_node_id IS NOT NULL
                          AND node.parent_node_id NOT IN (SELECT id FROM dr_drive_node)
                      )
               )",
        )
        .execute(&mut *connection)
        .await?
        .rows_affected() as i64;

        let orphaned_without_space = sqlx::query(
            "DELETE FROM dr_drive_node
             WHERE space_id NOT IN (SELECT id FROM dr_drive_space)
               AND id NOT IN (SELECT DISTINCT node_id FROM dr_drive_storage_object)",
        )
        .execute(&mut *connection)
        .await?
        .rows_affected() as i64;

        let orphaned_children = sqlx::query(
            "DELETE FROM dr_drive_node
             WHERE parent_node_id IS NOT NULL
               AND parent_node_id NOT IN (SELECT id FROM dr_drive_node)
               AND id NOT IN (SELECT DISTINCT node_id FROM dr_drive_storage_object)",
        )
        .execute(&mut *connection)
        .await?
        .rows_affected() as i64;

        Ok::<OrphanCleanupResult, sqlx::Error>(OrphanCleanupResult {
            orphaned_nodes: orphaned_without_space + orphaned_children,
            cleaned_objects,
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
