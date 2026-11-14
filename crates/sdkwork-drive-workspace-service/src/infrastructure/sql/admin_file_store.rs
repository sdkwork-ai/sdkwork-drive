use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::ports::admin_file_store::{
    DriveAdminFileRecord, DriveAdminFileStore, ListAdminFilesQuery,
};
use crate::DriveServiceError;

/// SQL projection of the admin file list.
///
/// The projection joins each node with its owning space so an operator can see
/// the workspace a file belongs to without a second round trip; the storage
/// object plane is intentionally absent, because object keys and provider
/// coordinates are not admin metadata.
pub struct SqlAdminFileStore {
    pool: PgPool,
}

impl SqlAdminFileStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn resolve_paths(
        &self,
        tenant_id: &str,
        node_ids: &[String],
    ) -> Result<HashMap<String, String>, DriveServiceError> {
        if node_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let rows = sqlx::query(
            "WITH RECURSIVE chain AS (
                SELECT
                    n.id AS leaf_id,
                    n.parent_node_id,
                    n.node_name::text AS node_name,
                    n.node_name::text AS file_path,
                    0 AS depth
                FROM dr_drive_node n
                WHERE n.tenant_id = $1 AND n.id = ANY($2)
                UNION ALL
                SELECT
                    c.leaf_id,
                    p.parent_node_id,
                    p.node_name::text,
                    p.node_name::text || '/' || c.file_path,
                    c.depth + 1
                FROM chain c
                JOIN dr_drive_node p
                  ON p.id = c.parent_node_id
                 AND p.tenant_id = $1
                WHERE c.depth < 64
            )
            SELECT leaf_id, file_path
            FROM chain
            WHERE parent_node_id IS NULL",
        )
        .bind(tenant_id)
        .bind(node_ids.to_vec())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("resolve admin file paths failed: {error}"))
        })?;

        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    row.get::<String, _>("leaf_id"),
                    row.get::<String, _>("file_path"),
                )
            })
            .collect())
    }
}

#[async_trait]
impl DriveAdminFileStore for SqlAdminFileStore {
    async fn list_admin_files(
        &self,
        query: &ListAdminFilesQuery,
    ) -> Result<Vec<DriveAdminFileRecord>, DriveServiceError> {
        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT
                n.id,
                n.tenant_id,
                n.space_id,
                s.display_name AS space_name,
                n.space_type,
                s.owner_subject_type,
                s.owner_subject_id,
                n.parent_node_id,
                n.node_type,
                n.node_name,
                n.file_extension,
                n.head_content_type,
                n.head_content_type_group,
                COALESCE(n.head_content_length, 0) AS head_content_length,
                n.lifecycle_status,
                n.version,
                CAST(n.created_by AS TEXT) AS created_by,
                CAST(n.updated_by AS TEXT) AS updated_by,
                CAST(n.created_at AS TEXT) AS created_at,
                CAST(n.updated_at AS TEXT) AS updated_at
             FROM dr_drive_node n
             JOIN dr_drive_space s
               ON s.id = n.space_id
              AND s.tenant_id = n.tenant_id
             WHERE n.tenant_id = ",
        );
        builder.push_bind(&query.tenant_id);

        if let Some(space_id) = &query.space_id {
            builder.push(" AND n.space_id = ").push_bind(space_id);
        }
        if let Some(space_type) = &query.space_type {
            builder.push(" AND n.space_type = ").push_bind(space_type);
        }
        if let Some(node_type) = &query.node_type {
            builder.push(" AND n.node_type = ").push_bind(node_type);
        }
        if let Some(lifecycle_status) = &query.lifecycle_status {
            builder
                .push(" AND n.lifecycle_status = ")
                .push_bind(lifecycle_status);
        }
        if let Some(content_type_group) = &query.content_type_group {
            builder
                .push(" AND n.head_content_type_group = ")
                .push_bind(content_type_group);
        }
        if let Some(owner_subject_id) = &query.owner_subject_id {
            builder
                .push(" AND s.owner_subject_id = ")
                .push_bind(owner_subject_id);
        }
        if let Some(keyword) = &query.keyword {
            builder
                .push(" AND n.node_name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(keyword)))
                .push(" ESCAPE '\\'");
        }

        builder
            .push(" ORDER BY n.updated_at DESC, n.id DESC LIMIT ")
            .push_bind(query.limit)
            .push(" OFFSET ")
            .push_bind(query.offset);

        let rows = builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "list dr_drive_node admin files failed: {error}"
                ))
            })?;

        let mut records = rows
            .iter()
            .map(map_admin_file_row)
            .collect::<Result<Vec<_>, _>>()?;
        if records.is_empty() {
            return Ok(records);
        }

        let node_ids = records
            .iter()
            .map(|record| record.id.clone())
            .collect::<Vec<_>>();
        let paths = self.resolve_paths(&query.tenant_id, &node_ids).await?;
        for record in &mut records {
            if let Some(path) = paths.get(&record.id) {
                record.file_path = path.clone();
            }
        }
        Ok(records)
    }
}

fn map_admin_file_row(row: &PgRow) -> Result<DriveAdminFileRecord, DriveServiceError> {
    let decode = |field: &str, error: sqlx::Error| {
        DriveServiceError::Internal(format!("decode admin file field {field} failed: {error}"))
    };
    let created_at: String = row
        .try_get("created_at")
        .map_err(|e| decode("created_at", e))?;
    let updated_at: String = row
        .try_get("updated_at")
        .map_err(|e| decode("updated_at", e))?;

    Ok(DriveAdminFileRecord {
        id: row.try_get("id").map_err(|e| decode("id", e))?,
        tenant_id: row
            .try_get("tenant_id")
            .map_err(|e| decode("tenant_id", e))?,
        space_id: row.try_get("space_id").map_err(|e| decode("space_id", e))?,
        space_name: row
            .try_get("space_name")
            .map_err(|e| decode("space_name", e))?,
        space_type: row
            .try_get("space_type")
            .map_err(|e| decode("space_type", e))?,
        owner_subject_type: row
            .try_get("owner_subject_type")
            .map_err(|e| decode("owner_subject_type", e))?,
        owner_subject_id: row
            .try_get("owner_subject_id")
            .map_err(|e| decode("owner_subject_id", e))?,
        parent_node_id: row
            .try_get("parent_node_id")
            .map_err(|e| decode("parent_node_id", e))?,
        node_type: row
            .try_get("node_type")
            .map_err(|e| decode("node_type", e))?,
        node_name: row
            .try_get("node_name")
            .map_err(|e| decode("node_name", e))?,
        file_path: row
            .try_get("node_name")
            .map_err(|e| decode("node_name", e))?,
        file_extension: row
            .try_get("file_extension")
            .map_err(|e| decode("file_extension", e))?,
        content_type: row
            .try_get("head_content_type")
            .map_err(|e| decode("head_content_type", e))?,
        content_type_group: row
            .try_get("head_content_type_group")
            .map_err(|e| decode("head_content_type_group", e))?,
        content_length: row
            .try_get("head_content_length")
            .map_err(|e| decode("head_content_length", e))?,
        lifecycle_status: row
            .try_get("lifecycle_status")
            .map_err(|e| decode("lifecycle_status", e))?,
        version: row.try_get("version").map_err(|e| decode("version", e))?,
        created_by: row
            .try_get("created_by")
            .map_err(|e| decode("created_by", e))?,
        updated_by: row
            .try_get("updated_by")
            .map_err(|e| decode("updated_by", e))?,
        created_at: normalize_timestamp_text(created_at),
        updated_at: normalize_timestamp_text(updated_at),
    })
}

/// Escape the three ILIKE metacharacters so an operator searching for `100%`
/// matches the literal name instead of every file.
fn escape_like_pattern(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '%' => escaped.push_str("\\%"),
            '_' => escaped.push_str("\\_"),
            other => escaped.push(other),
        }
    }
    escaped
}

fn normalize_timestamp_text(value: String) -> String {
    super::sql_error::normalize_timestamp_text(value)
}

#[cfg(test)]
mod tests {
    use super::escape_like_pattern;

    #[test]
    fn escapes_ilike_wildcards() {
        assert_eq!("100\\%", escape_like_pattern("100%"));
        assert_eq!("a\\_b", escape_like_pattern("a_b"));
        assert_eq!("c\\\\d", escape_like_pattern("c\\d"));
        assert_eq!("plain", escape_like_pattern("plain"));
    }
}
