use sqlx::PgConnection;
use sqlx::PgPool;
use sqlx::Row;

use crate::domain::uploader::content_type_group_for;
use crate::infrastructure::sql::begin_transaction_sql;
use crate::DriveServiceError;

/// Denormalized latest-file metadata stored on `dr_drive_node` for fast listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileNodeHeadSnapshot {
    pub file_extension: Option<String>,
    pub content_type: String,
    pub content_type_group: String,
    pub content_length: i64,
    pub version_no: i64,
    pub checksum_sha256_hex: String,
}

/// Columns selected for Drive node list/detail API responses with a table alias.
pub const NODE_API_SELECT_JOIN_COLUMNS: &str = "\
    n.id, n.tenant_id, n.space_id, n.space_type, n.parent_node_id, n.shortcut_target_node_id, \
    n.node_type, n.node_name, n.scene, n.source, n.content_state, n.file_extension, \
    n.head_content_type, n.head_content_type_group, n.head_content_length, \
    n.lifecycle_status, n.version, CAST(n.created_at AS TEXT) AS created_at, CAST(n.updated_at AS TEXT) AS updated_at";

pub fn file_extension_from_name(file_name: &str) -> Option<String> {
    let normalized = file_name.trim();
    let (_, extension) = normalized.rsplit_once('.')?;
    if extension.is_empty() || extension.contains('/') {
        return None;
    }
    Some(extension.to_ascii_lowercase())
}

/// Columns selected for Drive node list/detail API responses.
pub const NODE_API_SELECT_COLUMNS: &str = "\
    id, tenant_id, space_id, space_type, parent_node_id, shortcut_target_node_id, \
    node_type, node_name, scene, source, content_state, file_extension, \
    head_content_type, head_content_type_group, head_content_length, \
    lifecycle_status, version, CAST(created_at AS TEXT) AS created_at, CAST(updated_at AS TEXT) AS updated_at";

pub async fn apply_file_node_head_snapshot(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
    operator_id: &str,
    snapshot: &FileNodeHeadSnapshot,
) -> Result<(), DriveServiceError> {
    let mut connection = pool.acquire().await.map_err(|error| {
        DriveServiceError::Internal(format!(
            "acquire file node head transaction connection failed: {error}"
        ))
    })?;
    sqlx::query(begin_transaction_sql())
        .execute(&mut *connection)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("begin file node head transaction failed: {error}"))
        })?;

    let result = apply_file_node_head_snapshot_in_transaction(
        &mut connection,
        tenant_id,
        node_id,
        operator_id,
        snapshot,
    )
    .await;
    match result {
        Ok(()) => {
            sqlx::query("COMMIT")
                .execute(&mut *connection)
                .await
                .map_err(|error| {
                    DriveServiceError::Internal(format!(
                        "commit file node head transaction failed: {error}"
                    ))
                })?;
            Ok(())
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            Err(error)
        }
    }
}

pub async fn apply_file_node_head_snapshot_in_transaction(
    connection: &mut PgConnection,
    tenant_id: &str,
    node_id: &str,
    operator_id: &str,
    snapshot: &FileNodeHeadSnapshot,
) -> Result<(), DriveServiceError> {
    super::managed_website_tree_guard::ensure_managed_website_node_mutation_allowed(
        connection, tenant_id, node_id,
    )
    .await?;

    sqlx::query(
        "UPDATE dr_drive_node
         SET content_state='ready',
             file_extension=$3,
             head_content_type=$4,
             head_content_type_group=$5,
             head_content_length=$6,
             head_version_no=$7,
             head_checksum_sha256_hex=$8,
             updated_by=$9,
             updated_at=CURRENT_TIMESTAMP,
             version=version + 1
         WHERE tenant_id=$1
           AND id=$2
           AND node_type='file'
           AND lifecycle_status != 'deleted'
           AND (
               content_state != 'ready'
               OR head_content_type IS NULL
               OR head_version_no IS NULL
           )",
    )
    .bind(tenant_id)
    .bind(node_id)
    .bind(snapshot.file_extension.as_deref())
    .bind(&snapshot.content_type)
    .bind(&snapshot.content_type_group)
    .bind(snapshot.content_length)
    .bind(snapshot.version_no)
    .bind(&snapshot.checksum_sha256_hex)
    .bind(operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("apply file node head snapshot failed: {error}"))
    })?;
    Ok(())
}

pub async fn sync_file_node_head_from_active_storage(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
    operator_id: &str,
) -> Result<(), DriveServiceError> {
    let row = sqlx::query(
        "SELECT o.content_type, o.content_length, o.version_no, o.checksum_sha256_hex,
                ui.file_extension, ui.content_type_group
         FROM dr_drive_storage_object o
         LEFT JOIN dr_drive_upload_item ui
           ON ui.tenant_id=o.tenant_id
          AND ui.node_id=o.node_id
          AND ui.status='completed'
         INNER JOIN dr_drive_node n
           ON n.tenant_id=o.tenant_id
          AND n.id=o.node_id
         WHERE o.tenant_id=$1
           AND o.node_id=$2
           AND o.lifecycle_status='active'
         ORDER BY o.version_no DESC, ui.updated_at DESC
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(node_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "read active storage object for node head sync failed: {error}"
        ))
    })?;

    let Some(row) = row else {
        return Err(DriveServiceError::NotFound(
            "active storage object not found for node head sync".to_string(),
        ));
    };

    let content_type: String = row.get("content_type");
    let content_type_group: Option<String> = row.try_get("content_type_group").ok().flatten();
    let snapshot = FileNodeHeadSnapshot {
        file_extension: row.try_get("file_extension").ok().flatten(),
        content_type: content_type.clone(),
        content_type_group: content_type_group
            .unwrap_or_else(|| content_type_group_for(&content_type).to_string()),
        content_length: row.get("content_length"),
        version_no: row.get("version_no"),
        checksum_sha256_hex: row.get("checksum_sha256_hex"),
    };

    apply_file_node_head_snapshot(pool, tenant_id, node_id, operator_id, &snapshot).await
}
