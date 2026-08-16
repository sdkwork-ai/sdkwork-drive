use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::sandbox::{AuthorizedSandboxMount, DriveSandboxGrant, DriveSandboxVolume};
use crate::ports::sandbox_principal_resolver::EffectiveSandboxPrincipal;
use crate::ports::sandbox_store::DriveSandboxStore;
use crate::DriveServiceError;

#[derive(Clone, Debug)]
pub struct SqlSandboxStore {
    pool: PgPool,
}

impl SqlSandboxStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveSandboxStore for SqlSandboxStore {
    async fn list_accessible_for_principals(
        &self,
        tenant_id: &str,
        principals: &[EffectiveSandboxPrincipal],
        offset: i64,
        limit: i64,
    ) -> Result<(Vec<DriveSandboxVolume>, i64), DriveServiceError> {
        if principals.is_empty() {
            return Ok((Vec::new(), 0));
        }
        let principal_predicate = principal_predicate_sql(principals.len(), 2);
        let limit_bind_index = 2 + principals.len() * 2;
        let offset_bind_index = limit_bind_index + 1;
        let query_sql = format!(
            "SELECT v.id, v.tenant_id, v.organization_id, v.display_name, v.root_entry_id, v.provider_kind, v.lifecycle_status, v.default_access, CASE WHEN MAX(CASE WHEN g.access_level = 'full' THEN 1 ELSE 0 END) = 1 THEN 'full' ELSE 'read_only' END AS effective_access, v.version FROM dr_drive_sandbox_volume v JOIN dr_drive_sandbox_grant g ON g.sandbox_id = v.id WHERE v.tenant_id = $1 AND v.lifecycle_status <> 'disabled' AND ({principal_predicate}) GROUP BY v.id, v.tenant_id, v.organization_id, v.display_name, v.root_entry_id, v.provider_kind, v.lifecycle_status, v.default_access, v.version ORDER BY v.display_name ASC, v.id ASC LIMIT ${limit_bind_index} OFFSET ${offset_bind_index}"
        );
        let mut query = sqlx::query(sqlx::AssertSqlSafe(query_sql.as_str())).bind(tenant_id);
        for principal in principals {
            query = query
                .bind(&principal.subject_type)
                .bind(&principal.subject_id);
        }
        let rows = query
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "list accessible sandbox volumes failed: {error}"
                ))
            })?;
        let count_query_sql = format!(
            "SELECT COUNT(DISTINCT v.id) AS total FROM dr_drive_sandbox_volume v JOIN dr_drive_sandbox_grant g ON g.sandbox_id = v.id WHERE v.tenant_id = $1 AND v.lifecycle_status <> 'disabled' AND ({principal_predicate})"
        );
        let mut count_query =
            sqlx::query_scalar(sqlx::AssertSqlSafe(count_query_sql.as_str())).bind(tenant_id);
        for principal in principals {
            count_query = count_query
                .bind(&principal.subject_type)
                .bind(&principal.subject_id);
        }
        let total: i64 = count_query.fetch_one(&self.pool).await.map_err(|error| {
            DriveServiceError::Internal(format!("count accessible sandbox volumes failed: {error}"))
        })?;
        Ok((rows.into_iter().map(map_volume).collect(), total))
    }

    async fn list_accessible(
        &self,
        tenant_id: &str,
        subject_type: &str,
        subject_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DriveSandboxVolume>, DriveServiceError> {
        let rows = sqlx::query(
            "SELECT v.id, v.tenant_id, v.organization_id, v.display_name, v.root_entry_id, v.provider_kind, v.lifecycle_status, v.default_access, g.access_level AS effective_access, v.version
             FROM dr_drive_sandbox_volume v
             LEFT JOIN dr_drive_sandbox_grant g ON g.sandbox_id = v.id AND g.subject_type = $2 AND g.subject_id = $3
             WHERE v.tenant_id = $1 AND v.lifecycle_status <> 'disabled' AND g.id IS NOT NULL
             ORDER BY v.display_name ASC, v.id ASC LIMIT $4 OFFSET $5"
        ).bind(tenant_id).bind(subject_type).bind(subject_id).bind(limit).bind(offset).fetch_all(&self.pool).await
            .map_err(|error| DriveServiceError::Internal(format!("list sandbox volumes failed: {error}")))?;
        Ok(rows.into_iter().map(map_volume).collect())
    }

    async fn get_grant(
        &self,
        tenant_id: &str,
        sandbox_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<Option<DriveSandboxGrant>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT g.sandbox_id, g.subject_type, g.subject_id, g.access_level, v.lifecycle_status
             FROM dr_drive_sandbox_grant g JOIN dr_drive_sandbox_volume v ON v.id = g.sandbox_id
             WHERE v.tenant_id = $1 AND g.sandbox_id = $2 AND g.subject_type = $3 AND g.subject_id = $4 AND v.lifecycle_status <> 'disabled'"
        ).bind(tenant_id).bind(sandbox_id).bind(subject_type).bind(subject_id).fetch_optional(&self.pool).await
            .map_err(|error| DriveServiceError::Internal(format!("get sandbox grant failed: {error}")))?;
        Ok(row.map(|row| DriveSandboxGrant {
            sandbox_id: row.get("sandbox_id"),
            subject_type: row.get("subject_type"),
            subject_id: row.get("subject_id"),
            access_level: row.get("access_level"),
            lifecycle_status: row.get("lifecycle_status"),
        }))
    }

    async fn get_authorized_mount_for_principals(
        &self,
        tenant_id: &str,
        sandbox_id: &str,
        principals: &[EffectiveSandboxPrincipal],
    ) -> Result<Option<AuthorizedSandboxMount>, DriveServiceError> {
        if principals.is_empty() {
            return Ok(None);
        }

        let principal_predicate = principal_predicate_sql(principals.len(), 3);
        let query_sql = format!(
            "SELECT v.id, v.root_entry_id, v.provider_kind, v.provider_root_ref, \
                    v.lifecycle_status, v.version, \
                    CASE WHEN MAX(CASE WHEN g.access_level = 'full' THEN 1 ELSE 0 END) = 1 \
                         THEN 'full' ELSE 'read_only' END AS effective_access \
             FROM dr_drive_sandbox_volume v \
             JOIN dr_drive_sandbox_grant g ON g.sandbox_id = v.id \
             WHERE v.tenant_id = $1 AND v.id = $2 \
             AND v.lifecycle_status <> 'disabled' AND ({principal_predicate}) \
             GROUP BY v.id, v.root_entry_id, v.provider_kind, v.provider_root_ref, \
                      v.lifecycle_status, v.version"
        );
        let mut query = sqlx::query(sqlx::AssertSqlSafe(query_sql.as_str()))
            .bind(tenant_id)
            .bind(sandbox_id);
        for principal in principals {
            query = query
                .bind(&principal.subject_type)
                .bind(&principal.subject_id);
        }

        let row = query.fetch_optional(&self.pool).await.map_err(|error| {
            DriveServiceError::Internal(format!("get authorized sandbox mount failed: {error}"))
        })?;

        Ok(row.map(|row| {
            AuthorizedSandboxMount::new(
                row.get("id"),
                row.get("root_entry_id"),
                row.get("provider_kind"),
                row.get("provider_root_ref"),
                row.get("lifecycle_status"),
                row.get("effective_access"),
                row.get("version"),
            )
        }))
    }
}

fn principal_predicate_sql(principal_count: usize, first_bind_index: usize) -> String {
    (0..principal_count)
        .map(|index| {
            let subject_type_bind_index = first_bind_index + index * 2;
            let subject_id_bind_index = subject_type_bind_index + 1;
            format!(
                "(g.subject_type = ${subject_type_bind_index} AND g.subject_id = ${subject_id_bind_index})"
            )
        })
        .collect::<Vec<_>>()
        .join(" OR ")
}

fn map_volume(row: sqlx::postgres::PgRow) -> DriveSandboxVolume {
    DriveSandboxVolume {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        display_name: row.get("display_name"),
        root_entry_id: row.get("root_entry_id"),
        provider_kind: row.get("provider_kind"),
        lifecycle_status: row.get("lifecycle_status"),
        default_access: row.get("default_access"),
        effective_access: row.get("effective_access"),
        version: row.get("version"),
    }
}

#[cfg(test)]
mod tests {
    use super::principal_predicate_sql;

    #[test]
    fn principal_predicate_uses_numbered_cross_engine_bind_parameters() {
        assert_eq!(
            principal_predicate_sql(2, 2),
            "(g.subject_type = $2 AND g.subject_id = $3) OR (g.subject_type = $4 AND g.subject_id = $5)"
        );
        assert_eq!(
            principal_predicate_sql(1, 3),
            "(g.subject_type = $3 AND g.subject_id = $4)"
        );
    }
}
