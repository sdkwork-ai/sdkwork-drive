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
        let orphaned_without_space = sqlx::query(
            "DELETE FROM dr_drive_node
             WHERE space_id NOT IN (SELECT id FROM dr_drive_space)",
        )
        .execute(&mut *connection)
        .await?
        .rows_affected() as i64;

        let orphaned_children = sqlx::query(
            "DELETE FROM dr_drive_node
             WHERE parent_node_id IS NOT NULL
               AND parent_node_id NOT IN (SELECT id FROM dr_drive_node)",
        )
        .execute(&mut *connection)
        .await?
        .rows_affected() as i64;

        let cleaned_objects = sqlx::query(
            "DELETE FROM dr_drive_storage_object
             WHERE node_id NOT IN (SELECT id FROM dr_drive_node)",
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
