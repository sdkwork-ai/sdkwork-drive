use async_trait::async_trait;
use sdkwork_drive_contract::drive::domain_events::admin_audit;
use sqlx::pool::PoolConnection;
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, PgPool, Postgres, Row};

use crate::domain::sandbox_admin::{SandboxAdminGrant, SandboxAdminPage, SandboxAdminVolume};
use crate::infrastructure::sql::runtime_id::next_drive_runtime_id;
use crate::infrastructure::sql::sql_error::{
    is_unique_constraint_violation, normalize_timestamp_text,
};
use crate::infrastructure::sql::transaction::begin_transaction_sql;
use crate::ports::sandbox_admin_store::{
    ListSandboxAdminGrantsQuery, ListSandboxAdminVolumesQuery, NewSandboxAdminGrant,
    NewSandboxAdminVolume, SandboxAdminAuditContext, SandboxAdminStore, UpdateSandboxAdminGrant,
    UpdateSandboxAdminVolume,
};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlSandboxAdminStore {
    pool: PgPool,
}

impl SqlSandboxAdminStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SandboxAdminStore for SqlSandboxAdminStore {
    async fn list_volumes(
        &self,
        query: &ListSandboxAdminVolumesQuery,
    ) -> Result<SandboxAdminPage<SandboxAdminVolume>, DriveServiceError> {
        let rows = sqlx::query(
            "SELECT id, tenant_id, organization_id, display_name, root_entry_id,
                    provider_kind, provider_root_ref, lifecycle_status, default_access,
                    version, created_by, updated_by,
                    CAST(created_at AS TEXT) AS created_at,
                    CAST(updated_at AS TEXT) AS updated_at
             FROM dr_drive_sandbox_volume
             WHERE tenant_id=$1
               AND organization_id=$2
               AND ($3 IS NULL OR lifecycle_status=$3)
               AND ($4 IS NULL OR provider_kind=$4)
             ORDER BY display_name ASC, id ASC
             LIMIT $5 OFFSET $6",
        )
        .bind(&query.tenant_id)
        .bind(&query.organization_id)
        .bind(query.lifecycle_status.as_deref())
        .bind(query.provider_kind.as_deref())
        .bind(query.page_size)
        .bind(query.offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| sql_internal("list sandbox admin volumes", error))?;
        let total_items = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(1)
             FROM dr_drive_sandbox_volume
             WHERE tenant_id=$1
               AND organization_id=$2
               AND ($3 IS NULL OR lifecycle_status=$3)
               AND ($4 IS NULL OR provider_kind=$4)",
        )
        .bind(&query.tenant_id)
        .bind(&query.organization_id)
        .bind(query.lifecycle_status.as_deref())
        .bind(query.provider_kind.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(|error| sql_internal("count sandbox admin volumes", error))?;

        Ok(SandboxAdminPage {
            items: rows.iter().map(map_volume).collect::<Result<_, _>>()?,
            page: query.page,
            page_size: query.page_size,
            total_items,
        })
    }

    async fn get_volume(
        &self,
        tenant_id: &str,
        organization_id: &str,
        sandbox_id: &str,
    ) -> Result<Option<SandboxAdminVolume>, DriveServiceError> {
        let row = sqlx::query(VOLUME_SELECT_BY_ID)
            .bind(tenant_id)
            .bind(organization_id)
            .bind(sandbox_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| sql_internal("get sandbox admin volume", error))?;
        row.as_ref().map(map_volume).transpose()
    }

    async fn create_volume(
        &self,
        volume: &NewSandboxAdminVolume,
        audit: &SandboxAdminAuditContext,
    ) -> Result<SandboxAdminVolume, DriveServiceError> {
        require_matching_audit_scope(&volume.tenant_id, &volume.organization_id, audit)?;
        let mut connection = begin_admin_transaction(&self.pool).await?;
        let result = async {
            sqlx::query(
                "INSERT INTO dr_drive_sandbox_volume (
                    id, tenant_id, organization_id, display_name, root_entry_id,
                    provider_kind, provider_root_ref, lifecycle_status, default_access,
                    version, created_by, updated_by
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 1, $10, $10)",
            )
            .bind(&volume.id)
            .bind(&volume.tenant_id)
            .bind(&volume.organization_id)
            .bind(&volume.display_name)
            .bind(&volume.root_entry_id)
            .bind(&volume.provider_kind)
            .bind(&volume.provider_root_ref)
            .bind(&volume.lifecycle_status)
            .bind(&volume.default_access)
            .bind(&volume.created_by)
            .execute(&mut *connection)
            .await
            .map_err(|error| mutation_sql_error("create sandbox volume", error))?;

            if let Some(grant) = volume.initial_grant.as_ref() {
                insert_grant_on_connection(&mut connection, grant).await?;
            }
            insert_audit_event(
                &mut connection,
                audit,
                admin_audit::sandbox_volume::CREATED,
                "sandbox_volume",
                &volume.id,
            )
            .await?;
            if let Some(grant) = volume.initial_grant.as_ref() {
                insert_audit_event(
                    &mut connection,
                    audit,
                    admin_audit::sandbox_grant::CREATED,
                    "sandbox_grant",
                    &grant.id,
                )
                .await?;
            }
            find_volume_on_connection(
                &mut connection,
                &volume.tenant_id,
                &volume.organization_id,
                &volume.id,
            )
            .await?
            .ok_or_else(|| {
                DriveServiceError::Internal("created sandbox volume could not be read".to_string())
            })
        }
        .await;
        finish_admin_transaction(&mut connection, result).await
    }

    async fn update_volume(
        &self,
        volume: &UpdateSandboxAdminVolume,
        audit: &SandboxAdminAuditContext,
    ) -> Result<SandboxAdminVolume, DriveServiceError> {
        require_matching_audit_scope(&volume.tenant_id, &volume.organization_id, audit)?;
        let mut connection = begin_admin_transaction(&self.pool).await?;
        let result = async {
            let update = sqlx::query(
                "UPDATE dr_drive_sandbox_volume
                 SET display_name=$1,
                     provider_root_ref=$2,
                     lifecycle_status=$3,
                     default_access=$4,
                     version=version + 1,
                     updated_by=$5,
                     updated_at=CURRENT_TIMESTAMP
                 WHERE tenant_id=$6 AND organization_id=$7 AND id=$8 AND version=$9",
            )
            .bind(&volume.display_name)
            .bind(&volume.provider_root_ref)
            .bind(&volume.lifecycle_status)
            .bind(&volume.default_access)
            .bind(&volume.updated_by)
            .bind(&volume.tenant_id)
            .bind(&volume.organization_id)
            .bind(&volume.sandbox_id)
            .bind(volume.expected_version)
            .execute(&mut *connection)
            .await
            .map_err(|error| mutation_sql_error("update sandbox volume", error))?;
            if update.rows_affected() == 0 {
                let exists = find_volume_on_connection(
                    &mut connection,
                    &volume.tenant_id,
                    &volume.organization_id,
                    &volume.sandbox_id,
                )
                .await?
                .is_some();
                return if exists {
                    Err(DriveServiceError::Conflict(
                        "sandbox volume version conflict".to_string(),
                    ))
                } else {
                    Err(DriveServiceError::NotFound(
                        "sandbox volume not found".to_string(),
                    ))
                };
            }
            insert_audit_event(
                &mut connection,
                audit,
                admin_audit::sandbox_volume::UPDATED,
                "sandbox_volume",
                &volume.sandbox_id,
            )
            .await?;
            find_volume_on_connection(
                &mut connection,
                &volume.tenant_id,
                &volume.organization_id,
                &volume.sandbox_id,
            )
            .await?
            .ok_or_else(|| {
                DriveServiceError::Internal("updated sandbox volume could not be read".to_string())
            })
        }
        .await;
        finish_admin_transaction(&mut connection, result).await
    }

    async fn delete_volume(
        &self,
        tenant_id: &str,
        organization_id: &str,
        sandbox_id: &str,
        audit: &SandboxAdminAuditContext,
    ) -> Result<(), DriveServiceError> {
        require_matching_audit_scope(tenant_id, organization_id, audit)?;
        let mut connection = begin_admin_transaction(&self.pool).await?;
        let result = async {
            let delete = sqlx::query(
                "DELETE FROM dr_drive_sandbox_volume
                 WHERE tenant_id=$1 AND organization_id=$2 AND id=$3",
            )
            .bind(tenant_id)
            .bind(organization_id)
            .bind(sandbox_id)
            .execute(&mut *connection)
            .await
            .map_err(|error| mutation_sql_error("delete sandbox volume", error))?;
            if delete.rows_affected() == 0 {
                return Err(DriveServiceError::NotFound(
                    "sandbox volume not found".to_string(),
                ));
            }
            insert_audit_event(
                &mut connection,
                audit,
                admin_audit::sandbox_volume::DELETED,
                "sandbox_volume",
                sandbox_id,
            )
            .await
        }
        .await;
        finish_admin_transaction(&mut connection, result).await
    }

    async fn list_grants(
        &self,
        query: &ListSandboxAdminGrantsQuery,
    ) -> Result<SandboxAdminPage<SandboxAdminGrant>, DriveServiceError> {
        if !volume_exists(
            &self.pool,
            &query.tenant_id,
            &query.organization_id,
            &query.sandbox_id,
        )
        .await?
        {
            return Err(DriveServiceError::NotFound(
                "sandbox volume not found".to_string(),
            ));
        }
        let rows = sqlx::query(
            "SELECT g.id, g.sandbox_id, g.subject_type, g.subject_id, g.access_level,
                    g.granted_by, CAST(g.created_at AS TEXT) AS created_at
             FROM dr_drive_sandbox_grant g
             JOIN dr_drive_sandbox_volume v ON v.id=g.sandbox_id
             WHERE v.tenant_id=$1 AND v.organization_id=$2 AND g.sandbox_id=$3
             ORDER BY g.subject_type ASC, g.subject_id ASC, g.id ASC
             LIMIT $4 OFFSET $5",
        )
        .bind(&query.tenant_id)
        .bind(&query.organization_id)
        .bind(&query.sandbox_id)
        .bind(query.page_size)
        .bind(query.offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| sql_internal("list sandbox admin grants", error))?;
        let total_items = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(1)
             FROM dr_drive_sandbox_grant g
             JOIN dr_drive_sandbox_volume v ON v.id=g.sandbox_id
             WHERE v.tenant_id=$1 AND v.organization_id=$2 AND g.sandbox_id=$3",
        )
        .bind(&query.tenant_id)
        .bind(&query.organization_id)
        .bind(&query.sandbox_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| sql_internal("count sandbox admin grants", error))?;
        Ok(SandboxAdminPage {
            items: rows.iter().map(map_grant).collect::<Result<_, _>>()?,
            page: query.page,
            page_size: query.page_size,
            total_items,
        })
    }

    async fn create_grant(
        &self,
        tenant_id: &str,
        organization_id: &str,
        grant: &NewSandboxAdminGrant,
        audit: &SandboxAdminAuditContext,
    ) -> Result<SandboxAdminGrant, DriveServiceError> {
        require_matching_audit_scope(tenant_id, organization_id, audit)?;
        let mut connection = begin_admin_transaction(&self.pool).await?;
        let result = async {
            if find_volume_on_connection(
                &mut connection,
                tenant_id,
                organization_id,
                &grant.sandbox_id,
            )
            .await?
            .is_none()
            {
                return Err(DriveServiceError::NotFound(
                    "sandbox volume not found".to_string(),
                ));
            }
            insert_grant_on_connection(&mut connection, grant).await?;
            insert_audit_event(
                &mut connection,
                audit,
                admin_audit::sandbox_grant::CREATED,
                "sandbox_grant",
                &grant.id,
            )
            .await?;
            find_grant_on_connection(
                &mut connection,
                tenant_id,
                organization_id,
                &grant.sandbox_id,
                &grant.id,
            )
            .await?
            .ok_or_else(|| {
                DriveServiceError::Internal("created sandbox grant could not be read".to_string())
            })
        }
        .await;
        finish_admin_transaction(&mut connection, result).await
    }

    async fn get_grant(
        &self,
        tenant_id: &str,
        organization_id: &str,
        sandbox_id: &str,
        grant_id: &str,
    ) -> Result<Option<SandboxAdminGrant>, DriveServiceError> {
        let mut connection = self
            .pool
            .acquire()
            .await
            .map_err(|error| sql_internal("acquire sandbox grant lookup", error))?;
        find_grant_on_connection(
            &mut connection,
            tenant_id,
            organization_id,
            sandbox_id,
            grant_id,
        )
        .await
    }

    async fn update_grant(
        &self,
        grant: &UpdateSandboxAdminGrant,
        audit: &SandboxAdminAuditContext,
    ) -> Result<SandboxAdminGrant, DriveServiceError> {
        require_matching_audit_scope(&grant.tenant_id, &grant.organization_id, audit)?;
        let mut connection = begin_admin_transaction(&self.pool).await?;
        let result = async {
            let update = sqlx::query(
                "UPDATE dr_drive_sandbox_grant
                 SET access_level=$1
                 WHERE id=$2 AND sandbox_id=$3
                   AND EXISTS (
                     SELECT 1 FROM dr_drive_sandbox_volume v
                     WHERE v.id=dr_drive_sandbox_grant.sandbox_id
                       AND v.tenant_id=$4 AND v.organization_id=$5
                   )",
            )
            .bind(&grant.access_level)
            .bind(&grant.grant_id)
            .bind(&grant.sandbox_id)
            .bind(&grant.tenant_id)
            .bind(&grant.organization_id)
            .execute(&mut *connection)
            .await
            .map_err(|error| mutation_sql_error("update sandbox grant", error))?;
            if update.rows_affected() == 0 {
                return Err(DriveServiceError::NotFound(
                    "sandbox grant not found".to_string(),
                ));
            }
            insert_audit_event(
                &mut connection,
                audit,
                admin_audit::sandbox_grant::UPDATED,
                "sandbox_grant",
                &grant.grant_id,
            )
            .await?;
            find_grant_on_connection(
                &mut connection,
                &grant.tenant_id,
                &grant.organization_id,
                &grant.sandbox_id,
                &grant.grant_id,
            )
            .await?
            .ok_or_else(|| {
                DriveServiceError::Internal("updated sandbox grant could not be read".to_string())
            })
        }
        .await;
        finish_admin_transaction(&mut connection, result).await
    }

    async fn delete_grant(
        &self,
        tenant_id: &str,
        organization_id: &str,
        sandbox_id: &str,
        grant_id: &str,
        audit: &SandboxAdminAuditContext,
    ) -> Result<(), DriveServiceError> {
        require_matching_audit_scope(tenant_id, organization_id, audit)?;
        let mut connection = begin_admin_transaction(&self.pool).await?;
        let result = async {
            let delete = sqlx::query(
                "DELETE FROM dr_drive_sandbox_grant
                 WHERE id=$1 AND sandbox_id=$2
                   AND EXISTS (
                     SELECT 1 FROM dr_drive_sandbox_volume v
                     WHERE v.id=dr_drive_sandbox_grant.sandbox_id
                       AND v.tenant_id=$3 AND v.organization_id=$4
                   )",
            )
            .bind(grant_id)
            .bind(sandbox_id)
            .bind(tenant_id)
            .bind(organization_id)
            .execute(&mut *connection)
            .await
            .map_err(|error| mutation_sql_error("delete sandbox grant", error))?;
            if delete.rows_affected() == 0 {
                return Err(DriveServiceError::NotFound(
                    "sandbox grant not found".to_string(),
                ));
            }
            insert_audit_event(
                &mut connection,
                audit,
                admin_audit::sandbox_grant::DELETED,
                "sandbox_grant",
                grant_id,
            )
            .await
        }
        .await;
        finish_admin_transaction(&mut connection, result).await
    }
}

const VOLUME_SELECT_BY_ID: &str =
    "SELECT id, tenant_id, organization_id, display_name, root_entry_id,
            provider_kind, provider_root_ref, lifecycle_status, default_access,
            version, created_by, updated_by,
            CAST(created_at AS TEXT) AS created_at,
            CAST(updated_at AS TEXT) AS updated_at
     FROM dr_drive_sandbox_volume
     WHERE tenant_id=$1 AND organization_id=$2 AND id=$3
     LIMIT 1";

async fn begin_admin_transaction(
    pool: &PgPool,
) -> Result<PoolConnection<Postgres>, DriveServiceError> {
    let mut connection = pool
        .acquire()
        .await
        .map_err(|error| sql_internal("acquire sandbox admin transaction", error))?;
    sqlx::query(begin_transaction_sql())
        .execute(&mut *connection)
        .await
        .map_err(|error| sql_internal("begin sandbox admin transaction", error))?;
    Ok(connection)
}

async fn finish_admin_transaction<T>(
    connection: &mut PgConnection,
    result: Result<T, DriveServiceError>,
) -> Result<T, DriveServiceError> {
    match result {
        Ok(value) => {
            sqlx::query("COMMIT")
                .execute(&mut *connection)
                .await
                .map_err(|error| sql_internal("commit sandbox admin transaction", error))?;
            Ok(value)
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            Err(error)
        }
    }
}

async fn find_volume_on_connection(
    connection: &mut PgConnection,
    tenant_id: &str,
    organization_id: &str,
    sandbox_id: &str,
) -> Result<Option<SandboxAdminVolume>, DriveServiceError> {
    let row = sqlx::query(VOLUME_SELECT_BY_ID)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(sandbox_id)
        .fetch_optional(&mut *connection)
        .await
        .map_err(|error| sql_internal("find sandbox admin volume", error))?;
    row.as_ref().map(map_volume).transpose()
}

async fn volume_exists(
    pool: &PgPool,
    tenant_id: &str,
    organization_id: &str,
    sandbox_id: &str,
) -> Result<bool, DriveServiceError> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(1) FROM dr_drive_sandbox_volume
         WHERE tenant_id=$1 AND organization_id=$2 AND id=$3",
    )
    .bind(tenant_id)
    .bind(organization_id)
    .bind(sandbox_id)
    .fetch_one(pool)
    .await
    .map(|count| count > 0)
    .map_err(|error| sql_internal("check sandbox admin volume", error))
}

async fn insert_grant_on_connection(
    connection: &mut PgConnection,
    grant: &NewSandboxAdminGrant,
) -> Result<(), DriveServiceError> {
    sqlx::query(
        "INSERT INTO dr_drive_sandbox_grant (
            id, sandbox_id, subject_type, subject_id, access_level, granted_by
         ) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&grant.id)
    .bind(&grant.sandbox_id)
    .bind(&grant.subject_type)
    .bind(&grant.subject_id)
    .bind(&grant.access_level)
    .bind(&grant.granted_by)
    .execute(&mut *connection)
    .await
    .map_err(|error| mutation_sql_error("create sandbox grant", error))?;
    Ok(())
}

async fn find_grant_on_connection(
    connection: &mut PgConnection,
    tenant_id: &str,
    organization_id: &str,
    sandbox_id: &str,
    grant_id: &str,
) -> Result<Option<SandboxAdminGrant>, DriveServiceError> {
    let row = sqlx::query(
        "SELECT g.id, g.sandbox_id, g.subject_type, g.subject_id, g.access_level,
                g.granted_by, CAST(g.created_at AS TEXT) AS created_at
         FROM dr_drive_sandbox_grant g
         JOIN dr_drive_sandbox_volume v ON v.id=g.sandbox_id
         WHERE v.tenant_id=$1 AND v.organization_id=$2
           AND g.sandbox_id=$3 AND g.id=$4
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(organization_id)
    .bind(sandbox_id)
    .bind(grant_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| sql_internal("find sandbox admin grant", error))?;
    row.as_ref().map(map_grant).transpose()
}

async fn insert_audit_event(
    connection: &mut PgConnection,
    audit: &SandboxAdminAuditContext,
    action: &str,
    resource_type: &str,
    resource_id: &str,
) -> Result<(), DriveServiceError> {
    let audit_id = next_drive_runtime_id("drive sandbox admin audit event")?;
    sqlx::query(
        "INSERT INTO dr_drive_audit_event (
            id, tenant_id, action, resource_type, resource_id,
            operator_id, request_id, trace_id
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(audit_id)
    .bind(&audit.tenant_id)
    .bind(action)
    .bind(resource_type)
    .bind(resource_id)
    .bind(&audit.operator_id)
    .bind(&audit.request_id)
    .bind(&audit.trace_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| sql_internal("append sandbox admin audit event", error))?;
    Ok(())
}

fn map_volume(row: &PgRow) -> Result<SandboxAdminVolume, DriveServiceError> {
    Ok(SandboxAdminVolume {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        display_name: row.get("display_name"),
        root_entry_id: row.get("root_entry_id"),
        provider_kind: row.get("provider_kind"),
        provider_root_ref: row.get("provider_root_ref"),
        lifecycle_status: row.get("lifecycle_status"),
        default_access: row.get("default_access"),
        version: row.get("version"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
        created_at: normalize_timestamp_text(row.get("created_at")),
        updated_at: normalize_timestamp_text(row.get("updated_at")),
    })
}

fn map_grant(row: &PgRow) -> Result<SandboxAdminGrant, DriveServiceError> {
    Ok(SandboxAdminGrant {
        id: row.get("id"),
        sandbox_id: row.get("sandbox_id"),
        subject_type: row.get("subject_type"),
        subject_id: row.get("subject_id"),
        access_level: row.get("access_level"),
        granted_by: row.get("granted_by"),
        created_at: normalize_timestamp_text(row.get("created_at")),
    })
}

fn require_matching_audit_scope(
    tenant_id: &str,
    organization_id: &str,
    audit: &SandboxAdminAuditContext,
) -> Result<(), DriveServiceError> {
    if tenant_id != audit.tenant_id {
        return Err(DriveServiceError::PermissionDenied(
            "sandbox audit tenant does not match mutation tenant".to_string(),
        ));
    }
    if organization_id != audit.organization_id {
        return Err(DriveServiceError::PermissionDenied(
            "sandbox audit organization does not match mutation organization".to_string(),
        ));
    }
    Ok(())
}

fn mutation_sql_error(operation: &str, error: sqlx::Error) -> DriveServiceError {
    let message = error.to_string();
    if is_unique_constraint_violation(&message) {
        return DriveServiceError::Conflict(format!("{operation} conflicts with existing data"));
    }
    sql_internal(operation, error)
}

fn sql_internal(operation: &str, error: sqlx::Error) -> DriveServiceError {
    DriveServiceError::Internal(format!("{operation} failed: {error}"))
}
