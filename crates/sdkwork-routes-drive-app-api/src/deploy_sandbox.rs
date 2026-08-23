//! Deployments Local Projects sandbox: `/opt/deploy` only.
//!
//! Materializes a stable `local_filesystem` volume from
//! `SDKWORK_DEPLOY_SANDBOX_KEY` / `SDKWORK_DEPLOY_SANDBOX_ROOT` so Deployments
//! Admin can browse the space clone tree through Drive App sandbox APIs.
//! Physical paths never leave this crate into App responses.

use std::path::{Path, PathBuf};

use sqlx::PgPool;

use crate::app_context::DriveRequestContext;

const DEFAULT_SANDBOX_KEY: &str = "deploy.local.opt_deploy";
const DEFAULT_SANDBOX_ROOT: &str = "/opt/deploy";
const DEFAULT_TENANT_ID: &str = "100001";
const BOOTSTRAP_ACTOR: &str = "sdkwork-webserver-deploy-sandbox";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeploySandboxConfig {
    pub key: String,
    pub root: PathBuf,
    pub display_name: String,
    pub tenant_id: String,
}

pub fn deploy_sandbox_config_from_env() -> Option<DeploySandboxConfig> {
    let key = std::env::var("SDKWORK_DEPLOY_SANDBOX_KEY")
        .unwrap_or_else(|_| DEFAULT_SANDBOX_KEY.to_owned());
    let root = std::env::var("SDKWORK_DEPLOY_SANDBOX_ROOT")
        .unwrap_or_else(|_| DEFAULT_SANDBOX_ROOT.to_owned());
    if key.trim().is_empty() || root.trim().is_empty() {
        return None;
    }
    let disabled = std::env::var("SDKWORK_DEPLOY_SANDBOX_ENABLED")
        .map(|value| matches!(value.as_str(), "0" | "false" | "FALSE" | "no" | "NO"))
        .unwrap_or(false);
    if disabled {
        return None;
    }
    let tenant_id = std::env::var("SDKWORK_DEPLOY_SANDBOX_TENANT_ID")
        .or_else(|_| std::env::var("SDKWORK_DATABASE_TENANT_ID"))
        .unwrap_or_else(|_| DEFAULT_TENANT_ID.to_owned());
    Some(DeploySandboxConfig {
        key: key.trim().to_owned(),
        root: PathBuf::from(root.trim()),
        display_name: std::env::var("SDKWORK_DEPLOY_SANDBOX_DISPLAY_NAME")
            .unwrap_or_else(|_| "Local deploy (/opt/deploy)".to_owned()),
        tenant_id: tenant_id.trim().to_owned(),
    })
}

/// Idempotent volume create for the deploy sandbox root (startup hook).
pub async fn ensure_deploy_opt_deploy_sandbox(
    pool: &PgPool,
    root: &Path,
    key: &str,
) -> Result<(), sqlx::Error> {
    let config = DeploySandboxConfig {
        key: key.to_owned(),
        root: root.to_path_buf(),
        display_name: "Local deploy (/opt/deploy)".to_owned(),
        tenant_id: DEFAULT_TENANT_ID.to_owned(),
    };
    ensure_deploy_sandbox_volume(pool, &config).await
}

pub async fn ensure_deploy_sandbox_volume(
    pool: &PgPool,
    config: &DeploySandboxConfig,
) -> Result<(), sqlx::Error> {
    if let Some(parent) = config.root.parent() {
        std::fs::create_dir_all(parent).map_err(sqlx::Error::Io)?;
    }
    std::fs::create_dir_all(&config.root).map_err(sqlx::Error::Io)?;
    let canonical_root = std::fs::canonicalize(&config.root).map_err(sqlx::Error::Io)?;
    let canonical_root = canonical_root.to_string_lossy().into_owned();
    let root_entry_id = format!("{}:root", config.key);

    sqlx::query(
        "INSERT INTO dr_drive_sandbox_volume (
            id, tenant_id, organization_id, display_name, root_entry_id,
            provider_kind, provider_root_ref, lifecycle_status, default_access,
            version, created_by, updated_by
         ) VALUES ($1, $2, '0', $3, $4, 'local_filesystem', $5, 'active', 'full', 1, $6, $6)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(&config.key)
    .bind(&config.tenant_id)
    .bind(&config.display_name)
    .bind(&root_entry_id)
    .bind(&canonical_root)
    .bind(BOOTSTRAP_ACTOR)
    .execute(pool)
    .await?;

    Ok(())
}

const DEPLOY_LOCAL_PROJECTS_READ: &str = "deploy.local_projects.read";
const DEPLOY_LOCAL_PROJECTS_WRITE: &str = "deploy.local_projects.write";

/// Resolves grant access for the deploy sandbox from IAM permission_scope.
///
/// - write / Drive sandbox admin → `full`
/// - read → `read_only`
/// - empty scope (bootstrap / unrestricted tokens) → `full`
/// - other non-empty scopes without deploy/drive privilege → no grant
pub fn resolve_deploy_sandbox_grant_access(permission_scope: &[String]) -> Option<&'static str> {
    let has = |needle: &str| permission_scope.iter().any(|scope| scope == needle);
    let has_write = has(DEPLOY_LOCAL_PROJECTS_WRITE)
        || has(sdkwork_drive_security::DRIVE_SANDBOXES_ADMIN_PERMISSION)
        || has(sdkwork_drive_security::DRIVE_STORAGE_ADMIN_PERMISSION)
        || has(sdkwork_drive_security::DRIVE_BACKEND_ADMIN_WILDCARD);
    let has_read = has_write || has(DEPLOY_LOCAL_PROJECTS_READ);
    if has_write {
        return Some("full");
    }
    if has_read {
        return Some("read_only");
    }
    if permission_scope.is_empty() {
        return Some("full");
    }
    None
}

/// Grants the authenticated user (and org, when present) access to the deploy sandbox
/// when IAM scope allows local project management.
pub async fn ensure_deploy_sandbox_grants_for_context(
    pool: &PgPool,
    context: &DriveRequestContext,
    config: &DeploySandboxConfig,
) -> Result<(), sqlx::Error> {
    let Some(access_level) = resolve_deploy_sandbox_grant_access(&context.permission_scope) else {
        return Ok(());
    };
    ensure_deploy_sandbox_volume(pool, config).await?;

    let user_grant_id = format!("{}:grant:user:{}", config.key, context.user_id);
    sqlx::query(
        "INSERT INTO dr_drive_sandbox_grant (
            id, sandbox_id, subject_type, subject_id, access_level, granted_by
         ) VALUES ($1, $2, 'user', $3, $4, $5)
         ON CONFLICT DO NOTHING",
    )
    .bind(&user_grant_id)
    .bind(&config.key)
    .bind(&context.user_id)
    .bind(access_level)
    .bind(&context.actor_id)
    .execute(pool)
    .await?;

    if let Some(organization_id) = context.organization_id.as_deref() {
        if !organization_id.is_empty() {
            let org_grant_id = format!("{}:grant:organization:{}", config.key, organization_id);
            sqlx::query(
                "INSERT INTO dr_drive_sandbox_grant (
                    id, sandbox_id, subject_type, subject_id, access_level, granted_by
                 ) VALUES ($1, $2, 'organization', $3, $4, $5)
                 ON CONFLICT DO NOTHING",
            )
            .bind(&org_grant_id)
            .bind(&config.key)
            .bind(organization_id)
            .bind(access_level)
            .bind(&context.actor_id)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grant_access_empty_scope_is_full() {
        assert_eq!(resolve_deploy_sandbox_grant_access(&[]), Some("full"));
    }

    #[test]
    fn grant_access_read_is_read_only() {
        assert_eq!(
            resolve_deploy_sandbox_grant_access(&["deploy.local_projects.read".to_owned()]),
            Some("read_only")
        );
    }

    #[test]
    fn grant_access_write_is_full() {
        assert_eq!(
            resolve_deploy_sandbox_grant_access(&["deploy.local_projects.write".to_owned()]),
            Some("full")
        );
    }

    #[test]
    fn grant_access_unrelated_scope_is_denied() {
        assert_eq!(
            resolve_deploy_sandbox_grant_access(&["sites.read".to_owned()]),
            None
        );
    }

    #[tokio::test]
    async fn deploy_sandbox_volume_is_idempotent() {
        let Some((pool, _guard)) = sdkwork_drive_test_support::postgres_test_database().await else {
            return;
        };
        let root = tempfile::tempdir().expect("temp deploy root");
        let config = DeploySandboxConfig {
            key: "deploy.local.test".to_owned(),
            root: root.path().to_path_buf(),
            display_name: "Test deploy".to_owned(),
            tenant_id: "tenant-deploy".to_owned(),
        };
        ensure_deploy_sandbox_volume(&pool, &config)
            .await
            .expect("create");
        ensure_deploy_sandbox_volume(&pool, &config)
            .await
            .expect("repeat");
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM dr_drive_sandbox_volume WHERE id = $1",
        )
        .bind(&config.key)
        .fetch_one(&pool)
        .await
        .expect("count");
        assert_eq!(count, 1);
    }
}
