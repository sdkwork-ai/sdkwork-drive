use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, PgPool, Row};

use crate::domain::website_root::{
    DriveWebsiteContentMode, DriveWebsiteRoot, DriveWebsiteSourceRootMode,
};
use crate::infrastructure::sql::{begin_transaction_sql, next_drive_runtime_id};
use crate::ports::website_root_store::{
    CreateDriveWebsiteRoot, CreateDriveWebsiteRootResult, DriveWebsiteRootStore,
};
use crate::DriveServiceError;

const WEBSITE_ROOT_SELECT_COLUMNS: &str = "id, uuid, tenant_id, space_id, root_key, display_name, source_root_mode, selected_folder_node_id, content_mode, active_node_id, active_generation, root_status, version, CAST(created_at AS TEXT) AS created_at, CAST(updated_at AS TEXT) AS updated_at";

#[derive(Debug, Clone)]
pub struct SqlWebsiteRootStore {
    pool: PgPool,
}

impl SqlWebsiteRootStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveWebsiteRootStore for SqlWebsiteRootStore {
    async fn create_or_get(
        &self,
        root: &CreateDriveWebsiteRoot,
    ) -> Result<CreateDriveWebsiteRootResult, DriveServiceError> {
        let selector_key = website_root_selector_key(root)?;
        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire WebsiteRoot transaction connection failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "begin WebsiteRoot transaction failed: {error}"
                ))
            })?;

        let result = create_or_get_on_connection(&mut connection, root, &selector_key).await;
        match result {
            Ok(created) => {
                sqlx::query("COMMIT")
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "commit WebsiteRoot transaction failed: {error}"
                        ))
                    })?;
                Ok(created)
            }
            Err(error) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                if matches!(error, DriveServiceError::Conflict(_)) {
                    if let Some(existing) = find_by_selector_on_connection(
                        &mut connection,
                        &root.tenant_id,
                        &root.space_id,
                        &selector_key,
                    )
                    .await?
                    {
                        return Ok(CreateDriveWebsiteRootResult {
                            root: existing,
                            created: false,
                        });
                    }
                }
                Err(error)
            }
        }
    }

    async fn get_by_uuid(
        &self,
        tenant_id: &str,
        root_uuid: &str,
    ) -> Result<DriveWebsiteRoot, DriveServiceError> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {WEBSITE_ROOT_SELECT_COLUMNS}
             FROM dr_drive_website_root
             WHERE tenant_id=$1 AND uuid=$2",
        )))
        .bind(tenant_id)
        .bind(root_uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| DriveServiceError::Internal(format!("get WebsiteRoot failed: {error}")))?;
        row.as_ref()
            .map(map_website_root)
            .transpose()?
            .ok_or_else(|| DriveServiceError::NotFound("WebsiteRoot not found".to_string()))
    }

    async fn list_by_space(
        &self,
        tenant_id: &str,
        space_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DriveWebsiteRoot>, DriveServiceError> {
        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {WEBSITE_ROOT_SELECT_COLUMNS}
             FROM dr_drive_website_root
             WHERE tenant_id=$1
               AND space_id=$2
               AND root_status != 'archived'
             ORDER BY created_at ASC, id ASC
             LIMIT $3 OFFSET $4",
        )))
        .bind(tenant_id)
        .bind(space_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("list WebsiteRoots failed: {error}"))
        })?;
        rows.iter().map(map_website_root).collect()
    }
}

async fn create_or_get_on_connection(
    connection: &mut PgConnection,
    root: &CreateDriveWebsiteRoot,
    selector_key: &str,
) -> Result<CreateDriveWebsiteRootResult, DriveServiceError> {
    let space_type: Option<String> = sqlx::query_scalar(
        "SELECT space_type
         FROM dr_drive_space
         WHERE tenant_id=$1 AND id=$2 AND lifecycle_status='active'",
    )
    .bind(&root.tenant_id)
    .bind(&root.space_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("validate WebsiteRoot space failed: {error}"))
    })?;
    if space_type.as_deref() != Some("website") {
        return Err(DriveServiceError::Validation(
            "WebsiteRoot requires an active website Space".to_string(),
        ));
    }

    if let Some(existing) =
        find_by_selector_on_connection(connection, &root.tenant_id, &root.space_id, selector_key)
            .await?
    {
        return Ok(CreateDriveWebsiteRootResult {
            root: existing,
            created: false,
        });
    }

    let active_node_id = match root.source_root_mode {
        DriveWebsiteSourceRootMode::SpaceRoot => {
            return Err(DriveServiceError::Conflict(
                "the Website Space default SPACE_ROOT is missing".to_string(),
            ));
        }
        DriveWebsiteSourceRootMode::Folder => {
            let folder_node_id = root.selected_folder_node_id.as_deref().ok_or_else(|| {
                DriveServiceError::Validation("FOLDER requires selected_folder_node_id".to_string())
            })?;
            validate_selected_folder(connection, &root.tenant_id, &root.space_id, folder_node_id)
                .await?;
            folder_node_id.to_string()
        }
    };

    let root_id = next_drive_runtime_id("WebsiteRoot")?.to_string();
    let root_uuid = uuid::Uuid::new_v4().to_string();
    let generation_id = next_drive_runtime_id("WebsiteRoot generation")?.to_string();
    sqlx::query(
        "INSERT INTO dr_drive_website_root (
            id, uuid, tenant_id, space_id, root_key, display_name,
            source_root_mode, selected_folder_node_id, selector_key,
            content_mode, active_node_id, active_generation, root_status,
            last_switch_by, version, created_by, updated_by
         ) VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, $9,
            $10, $11, 1, 'active',
            $12, 1, $12, $12
         )",
    )
    .bind(&root_id)
    .bind(&root_uuid)
    .bind(&root.tenant_id)
    .bind(&root.space_id)
    .bind(&root.root_key)
    .bind(&root.display_name)
    .bind(root.source_root_mode.as_str())
    .bind(root.selected_folder_node_id.as_deref())
    .bind(selector_key)
    .bind(root.content_mode.as_str())
    .bind(&active_node_id)
    .bind(&root.operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        let message = error.to_string();
        if message.to_ascii_lowercase().contains("unique") {
            DriveServiceError::Conflict(
                "WebsiteRoot root_key or selector already exists".to_string(),
            )
        } else {
            DriveServiceError::Internal(format!("insert WebsiteRoot failed: {message}"))
        }
    })?;

    sqlx::query(
        "INSERT INTO dr_drive_website_root_generation (
            id, tenant_id, website_root_id, generation_no, root_node_id,
            source_sync_id, manifest_sha256, file_count, total_bytes,
            generation_status, activated_by
         ) VALUES ($1, $2, $3, 1, $4, NULL, NULL, 0, 0, 'current', $5)",
    )
    .bind(&generation_id)
    .bind(&root.tenant_id)
    .bind(&root_id)
    .bind(&active_node_id)
    .bind(&root.operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("insert WebsiteRoot generation failed: {error}"))
    })?;

    let created =
        find_by_selector_on_connection(connection, &root.tenant_id, &root.space_id, selector_key)
            .await?
            .ok_or_else(|| {
                DriveServiceError::Internal("read inserted WebsiteRoot failed".to_string())
            })?;
    Ok(CreateDriveWebsiteRootResult {
        root: created,
        created: true,
    })
}

fn website_root_selector_key(root: &CreateDriveWebsiteRoot) -> Result<String, DriveServiceError> {
    match root.source_root_mode {
        DriveWebsiteSourceRootMode::SpaceRoot => Ok("space_root".to_string()),
        DriveWebsiteSourceRootMode::Folder => Ok(format!(
            "folder:{}",
            root.selected_folder_node_id.as_deref().ok_or_else(|| {
                DriveServiceError::Validation("FOLDER requires selected_folder_node_id".to_string())
            })?
        )),
    }
}

async fn validate_selected_folder(
    connection: &mut PgConnection,
    tenant_id: &str,
    space_id: &str,
    folder_node_id: &str,
) -> Result<(), DriveServiceError> {
    let selected: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT node_type, parent_node_id
         FROM dr_drive_node
         WHERE tenant_id=$1
           AND space_id=$2
           AND id=$3
           AND space_type='website'
           AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(space_id)
    .bind(folder_node_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("validate WebsiteRoot folder failed: {error}"))
    })?;
    match selected {
        Some((node_type, Some(_))) if node_type == "folder" => {}
        _ => {
            return Err(DriveServiceError::Validation(
                "selected WebsiteRoot node must be an active descendant folder in the same website Space"
                    .to_string(),
            ))
        }
    }

    let reserved_count: i64 = sqlx::query_scalar(
        "WITH RECURSIVE lineage(id, parent_node_id, node_name) AS (
            SELECT id, parent_node_id, node_name
            FROM dr_drive_node
            WHERE tenant_id=$1 AND space_id=$2 AND id=$3 AND lifecycle_status='active'
            UNION ALL
            SELECT parent.id, parent.parent_node_id, parent.node_name
            FROM dr_drive_node parent
            INNER JOIN lineage ON lineage.parent_node_id=parent.id
            WHERE parent.tenant_id=$1
              AND parent.space_id=$2
              AND parent.lifecycle_status='active'
         )
         SELECT COUNT(1)
         FROM lineage
         WHERE lower(node_name) IN ('.sdkwork', '.trash', 'trash', '.staging', 'staging', '.versions', 'versions')",
    )
    .bind(tenant_id)
    .bind(space_id)
    .bind(folder_node_id)
    .fetch_one(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "validate WebsiteRoot reserved ancestry failed: {error}"
        ))
    })?;
    if reserved_count > 0 {
        return Err(DriveServiceError::Validation(
            "selected WebsiteRoot folder is inside a reserved Drive namespace".to_string(),
        ));
    }
    Ok(())
}

async fn find_by_selector_on_connection(
    connection: &mut PgConnection,
    tenant_id: &str,
    space_id: &str,
    selector_key: &str,
) -> Result<Option<DriveWebsiteRoot>, DriveServiceError> {
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {WEBSITE_ROOT_SELECT_COLUMNS}
         FROM dr_drive_website_root
         WHERE tenant_id=$1
           AND space_id=$2
           AND selector_key=$3
           AND root_status IN ('active', 'suspended', 'invalid')",
    )))
    .bind(tenant_id)
    .bind(space_id)
    .bind(selector_key)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("find WebsiteRoot selector failed: {error}"))
    })?;
    row.as_ref().map(map_website_root).transpose()
}

fn map_website_root(row: &PgRow) -> Result<DriveWebsiteRoot, DriveServiceError> {
    let source_root_mode_raw: String = row.get("source_root_mode");
    let source_root_mode = DriveWebsiteSourceRootMode::try_from_str(&source_root_mode_raw)
        .ok_or_else(|| {
            DriveServiceError::Internal(format!(
                "unknown WebsiteRoot source_root_mode: {source_root_mode_raw}"
            ))
        })?;
    let content_mode_raw: String = row.get("content_mode");
    let content_mode =
        DriveWebsiteContentMode::try_from_str(&content_mode_raw).ok_or_else(|| {
            DriveServiceError::Internal(format!(
                "unknown WebsiteRoot content_mode: {content_mode_raw}"
            ))
        })?;
    Ok(DriveWebsiteRoot {
        id: row.get("id"),
        uuid: row.get("uuid"),
        tenant_id: row.get("tenant_id"),
        space_id: row.get("space_id"),
        root_key: row.get("root_key"),
        display_name: row.get("display_name"),
        source_root_mode,
        selected_folder_node_id: row.get("selected_folder_node_id"),
        content_mode,
        active_node_id: row.get("active_node_id"),
        active_generation: row.get("active_generation"),
        root_status: row.get("root_status"),
        version: row.get("version"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
