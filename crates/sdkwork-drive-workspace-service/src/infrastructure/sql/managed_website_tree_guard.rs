use sqlx::{PgConnection, Row};

use crate::DriveServiceError;

const MAXIMUM_ANCESTRY_DEPTH: i64 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedWebsiteTreeSystemOverrideReason {
    ContentPolicyQuarantine,
    ExpiredWebsitePublishingCleanup,
    TenantAuthorizedSpaceRetirement,
}

impl ManagedWebsiteTreeSystemOverrideReason {
    fn audit_action(self) -> &'static str {
        match self {
            Self::ContentPolicyQuarantine => {
                "drive.website_tree.system_override.content_policy_quarantine"
            }
            Self::ExpiredWebsitePublishingCleanup => {
                "drive.website_tree.system_override.expired_publishing_cleanup"
            }
            Self::TenantAuthorizedSpaceRetirement => {
                "drive.website_tree.system_override.tenant_authorized_space_retirement"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedWebsiteTreeSystemOverride {
    reason: ManagedWebsiteTreeSystemOverrideReason,
    operator_id: String,
}

impl ManagedWebsiteTreeSystemOverride {
    pub fn authorize(
        reason: ManagedWebsiteTreeSystemOverrideReason,
        operator_id: &str,
    ) -> Result<Self, DriveServiceError> {
        let operator_id = operator_id.trim();
        if operator_id.is_empty() {
            return Err(DriveServiceError::Validation(
                "managed website tree system override requires operator_id".to_string(),
            ));
        }
        Ok(Self {
            reason,
            operator_id: operator_id.to_string(),
        })
    }

    pub async fn record_on_connection(
        &self,
        connection: &mut PgConnection,
        tenant_id: &str,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<(), DriveServiceError> {
        let audit_id = super::runtime_id::next_drive_runtime_id(
            "managed website tree system override audit event",
        )?;
        sqlx::query(
            "INSERT INTO dr_drive_audit_event (
                id, tenant_id, action, resource_type, resource_id, operator_id,
                request_id, trace_id
             ) VALUES ($1, $2, $3, $4, $5, $6, NULL, NULL)",
        )
        .bind(audit_id)
        .bind(tenant_id)
        .bind(self.reason.audit_action())
        .bind(resource_type)
        .bind(resource_id)
        .bind(&self.operator_id)
        .execute(&mut *connection)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "record managed website tree system override audit failed: {error}"
            ))
        })?;
        Ok(())
    }
}

/// Rejects mutations that would change an immutable atomic website tree.
///
/// Callers must invoke this function inside the same write transaction as the
/// protected mutation. PostgreSQL locks the owning sync or generation rows selected below.
pub async fn ensure_managed_website_node_mutation_allowed(
    connection: &mut PgConnection,
    tenant_id: &str,
    node_id: &str,
) -> Result<(), DriveServiceError> {
    if let Some(sync_status) = find_owning_sync_status(connection, tenant_id, node_id).await? {
        if matches!(sync_status.as_str(), "created" | "uploading" | "ready") {
            return Ok(());
        }
        return Err(DriveServiceError::Conflict(format!(
            "WebsiteSync staging tree is immutable while sync status is {sync_status}"
        )));
    }

    if let Some(generation_status) =
        find_atomic_generation_status(connection, tenant_id, node_id).await?
    {
        return Err(DriveServiceError::Conflict(format!(
            "atomic WebsiteRoot {generation_status} generation is immutable"
        )));
    }

    Ok(())
}

/// Rejects creation or copy into an immutable atomic website parent.
pub async fn ensure_managed_website_parent_mutation_allowed(
    connection: &mut PgConnection,
    tenant_id: &str,
    space_id: &str,
    parent_node_id: Option<&str>,
) -> Result<(), DriveServiceError> {
    if let Some(parent_node_id) = parent_node_id {
        return ensure_managed_website_node_mutation_allowed(connection, tenant_id, parent_node_id)
            .await;
    }

    let lock_clause = postgres_lock_clause("root");
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT root.id
         FROM dr_drive_website_root root
         WHERE root.tenant_id=$1
           AND root.space_id=$2
           AND root.source_root_mode='space_root'
           AND root.content_mode='atomic_generation'
           AND root.root_status != 'archived'
         ORDER BY root.id ASC
         LIMIT 1{lock_clause}"
    )))
    .bind(tenant_id)
    .bind(space_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "check atomic WebsiteRoot space-root mutation guard failed: {error}"
        ))
    })?;
    if row.is_some() {
        return Err(DriveServiceError::Conflict(
            "atomic SPACE_ROOT WebsiteRoot is immutable".to_string(),
        ));
    }
    Ok(())
}

/// Rejects bulk Space mutation when it would cross any managed atomic tree.
pub async fn ensure_managed_website_space_mutation_allowed(
    connection: &mut PgConnection,
    tenant_id: &str,
    space_id: &str,
) -> Result<(), DriveServiceError> {
    let lock_clause = postgres_lock_clause("root");
    let root = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT root.id
         FROM dr_drive_website_root root
         WHERE root.tenant_id=$1
           AND root.space_id=$2
           AND root.content_mode='atomic_generation'
           AND root.root_status != 'archived'
         ORDER BY root.id ASC
         LIMIT 1{lock_clause}"
    )))
    .bind(tenant_id)
    .bind(space_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "check atomic WebsiteRoot space mutation guard failed: {error}"
        ))
    })?;
    if root.is_some() {
        return Err(DriveServiceError::Conflict(
            "atomic WebsiteRoot Space contents are immutable".to_string(),
        ));
    }

    let lock_clause = postgres_lock_clause("sync");
    let sync = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT sync.id
         FROM dr_drive_website_sync sync
         WHERE sync.tenant_id=$1
           AND sync.space_id=$2
           AND sync.sync_status NOT IN ('aborted', 'expired')
         ORDER BY sync.id ASC
         LIMIT 1{lock_clause}"
    )))
    .bind(tenant_id)
    .bind(space_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "check WebsiteSync space mutation guard failed: {error}"
        ))
    })?;
    if sync.is_some() {
        return Err(DriveServiceError::Conflict(
            "WebsiteSync Space contents require a system override".to_string(),
        ));
    }
    Ok(())
}

async fn find_owning_sync_status(
    connection: &mut PgConnection,
    tenant_id: &str,
    node_id: &str,
) -> Result<Option<String>, DriveServiceError> {
    let lock_clause = postgres_lock_clause("sync");
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "WITH RECURSIVE ancestry(id, parent_node_id, depth) AS (
            SELECT id, parent_node_id, 0
            FROM dr_drive_node
            WHERE tenant_id=$1 AND id=$2
            UNION ALL
            SELECT parent.id, parent.parent_node_id, ancestry.depth + 1
            FROM dr_drive_node parent
            INNER JOIN ancestry ON ancestry.parent_node_id=parent.id
            WHERE parent.tenant_id=$1 AND ancestry.depth < $3
         )
         SELECT sync.sync_status
         FROM dr_drive_website_sync sync
         INNER JOIN ancestry ON ancestry.id=sync.staging_node_id
         WHERE sync.tenant_id=$1
         ORDER BY ancestry.depth ASC
         LIMIT 1{lock_clause}"
    )))
    .bind(tenant_id)
    .bind(node_id)
    .bind(MAXIMUM_ANCESTRY_DEPTH)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "check WebsiteSync staging mutation guard failed: {error}"
        ))
    })?;
    Ok(row.map(|row| row.get("sync_status")))
}

async fn find_atomic_generation_status(
    connection: &mut PgConnection,
    tenant_id: &str,
    node_id: &str,
) -> Result<Option<String>, DriveServiceError> {
    let lock_clause = postgres_lock_clause("generation, root");
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "WITH RECURSIVE ancestry(id, parent_node_id, depth) AS (
            SELECT id, parent_node_id, 0
            FROM dr_drive_node
            WHERE tenant_id=$1 AND id=$2
            UNION ALL
            SELECT parent.id, parent.parent_node_id, ancestry.depth + 1
            FROM dr_drive_node parent
            INNER JOIN ancestry ON ancestry.parent_node_id=parent.id
            WHERE parent.tenant_id=$1 AND ancestry.depth < $3
         )
         SELECT generation.generation_status
         FROM dr_drive_website_root_generation generation
         INNER JOIN dr_drive_website_root root
           ON root.id=generation.website_root_id
          AND root.tenant_id=generation.tenant_id
         INNER JOIN ancestry ON ancestry.id=generation.root_node_id
         WHERE generation.tenant_id=$1
           AND root.content_mode='atomic_generation'
           AND generation.generation_status IN ('current', 'retained')
         ORDER BY ancestry.depth ASC
         LIMIT 1{lock_clause}"
    )))
    .bind(tenant_id)
    .bind(node_id)
    .bind(MAXIMUM_ANCESTRY_DEPTH)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "check atomic WebsiteRoot generation mutation guard failed: {error}"
        ))
    })?;
    Ok(row.map(|row| row.get("generation_status")))
}

fn postgres_lock_clause(aliases: &str) -> String {
    format!(" FOR UPDATE OF {aliases}")
}
