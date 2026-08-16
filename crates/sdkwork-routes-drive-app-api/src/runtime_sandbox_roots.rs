use std::path::{Path, PathBuf};

use sdkwork_utils_rust::sha256_hash;
use sqlx::PgPool;

use crate::app_context::DriveRequestContext;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeSandboxRoot {
    pub(crate) display_name: String,
    pub(crate) path: PathBuf,
}

pub(crate) fn discover_runtime_sandbox_roots() -> Vec<RuntimeSandboxRoot> {
    discover_runtime_sandbox_roots_with(|path| {
        path.is_dir()
            && std::fs::read_dir(path)
                .map(|mut entries| entries.next().transpose().is_ok())
                .unwrap_or(false)
    })
}

fn discover_runtime_sandbox_roots_with(
    mut is_accessible_directory: impl FnMut(&Path) -> bool,
) -> Vec<RuntimeSandboxRoot> {
    #[cfg(windows)]
    {
        ('A'..='Z')
            .filter_map(|letter| {
                let path = PathBuf::from(format!("{letter}:\\"));
                is_accessible_directory(&path).then(|| RuntimeSandboxRoot {
                    display_name: format!("Local Disk ({letter}:)"),
                    path,
                })
            })
            .collect()
    }

    #[cfg(not(windows))]
    {
        let path = PathBuf::from("/");
        if is_accessible_directory(&path) {
            vec![RuntimeSandboxRoot {
                display_name: "Filesystem (/)".to_owned(),
                path,
            }]
        } else {
            Vec::new()
        }
    }
}

pub(crate) async fn ensure_runtime_sandbox_roots(
    pool: &PgPool,
    context: &DriveRequestContext,
    roots: &[RuntimeSandboxRoot],
) -> Result<(), sqlx::Error> {
    if roots.is_empty() {
        return Ok(());
    }

    let mut transaction = pool.begin().await?;
    for root in roots {
        let canonical_root = std::fs::canonicalize(&root.path).map_err(sqlx::Error::Io)?;
        let canonical_root = canonical_root.to_string_lossy().into_owned();
        let sandbox_hash = stable_hash(&[
            b"sdkwork-drive-runtime-sandbox-v1",
            context.tenant_id.as_bytes(),
            canonical_root.as_bytes(),
        ]);
        let sandbox_id = format!("runtime-volume-{}", &sandbox_hash[..48]);
        let root_entry_id = format!("runtime-root-{}", &sandbox_hash[..48]);
        let grant_hash = stable_hash(&[
            b"sdkwork-drive-runtime-grant-v1",
            sandbox_id.as_bytes(),
            context.user_id.as_bytes(),
        ]);
        let grant_id = format!("runtime-grant-{}", &grant_hash[..48]);

        sqlx::query(
            "INSERT INTO dr_drive_sandbox_volume (
                id, tenant_id, organization_id, display_name, root_entry_id,
                provider_kind, provider_root_ref, lifecycle_status, default_access,
                version, created_by, updated_by
             ) VALUES ($1, $2, $3, $4, $5, 'local_filesystem', $6, 'active', 'full', 1, $7, $8)
             ON CONFLICT DO NOTHING",
        )
        .bind(&sandbox_id)
        .bind(&context.tenant_id)
        .bind(&context.organization_id)
        .bind(&root.display_name)
        .bind(&root_entry_id)
        .bind(&canonical_root)
        .bind(&context.actor_id)
        .bind(&context.actor_id)
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "INSERT INTO dr_drive_sandbox_grant (
                id, sandbox_id, subject_type, subject_id, access_level, granted_by
             ) VALUES ($1, $2, 'user', $3, 'full', $4)
             ON CONFLICT DO NOTHING",
        )
        .bind(grant_id)
        .bind(&sandbox_id)
        .bind(&context.user_id)
        .bind(&context.actor_id)
        .execute(&mut *transaction)
        .await?;
    }
    transaction.commit().await
}

fn stable_hash(parts: &[&[u8]]) -> String {
    let mut input = Vec::new();
    for part in parts {
        if !input.is_empty() {
            input.push(0);
        }
        input.extend_from_slice(part);
    }
    sha256_hash(&input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use sdkwork_drive_sandbox_local::LocalSandboxDirectoryProvider;
    use sdkwork_drive_workspace_service::{
        application::sandbox_directory_service::{
            DriveSandboxDirectoryService, ListSandboxDirectoryCommand,
        },
        application::sandbox_service::DriveSandboxService,
        infrastructure::sql::{
            sandbox_mutation_operation_store::SqlSandboxMutationOperationStore,
            sandbox_store::SqlSandboxStore,
        },
        ports::sandbox_principal_resolver::EffectiveSandboxPrincipal,
    };

    use sqlx::Row;

    #[cfg(windows)]
    #[test]
    fn windows_discovery_returns_each_accessible_drive() {
        let roots = discover_runtime_sandbox_roots_with(|path| {
            path == Path::new("C:\\") || path == Path::new("E:\\")
        });

        assert_eq!(
            roots,
            vec![
                RuntimeSandboxRoot {
                    display_name: "Local Disk (C:)".to_owned(),
                    path: PathBuf::from("C:\\"),
                },
                RuntimeSandboxRoot {
                    display_name: "Local Disk (E:)".to_owned(),
                    path: PathBuf::from("E:\\"),
                },
            ]
        );
    }

    #[tokio::test]
    async fn runtime_roots_and_user_grants_are_created_idempotently() {
        let Some((pool, _database_guard)) =
            sdkwork_drive_test_support::postgres_test_database().await
        else {
            return;
        };
        let root = tempfile::tempdir().expect("create runtime root");
        let context = DriveRequestContext {
            tenant_id: "tenant-runtime".to_owned(),
            user_id: "user-runtime".to_owned(),
            organization_id: Some("organization-runtime".to_owned()),
            app_id: Some("birdcoder".to_owned()),
            actor_id: "user-runtime".to_owned(),
            subject_type: "user".to_owned(),
            subject_id: "user-runtime".to_owned(),
            auth_level: sdkwork_web_core::WebAuthLevel::Password,
            request_id: "request-runtime".to_owned(),
            trace_id: "trace-runtime".to_owned(),
            from_token: true,
        };
        let roots = [RuntimeSandboxRoot {
            display_name: "Runtime root".to_owned(),
            path: root.path().to_path_buf(),
        }];
        std::fs::create_dir(root.path().join("workspace"))
            .expect("create visible runtime directory");

        ensure_runtime_sandbox_roots(&pool, &context, &roots)
            .await
            .expect("create runtime roots");
        ensure_runtime_sandbox_roots(&pool, &context, &roots)
            .await
            .expect("repeat runtime roots");

        let row = sqlx::query(
            "SELECT COUNT(*) AS volume_count,
                    (SELECT COUNT(*) FROM dr_drive_sandbox_grant) AS grant_count
             FROM dr_drive_sandbox_volume",
        )
        .fetch_one(&pool)
        .await
        .expect("count runtime roots");
        assert_eq!(row.get::<i64, _>("volume_count"), 1);
        assert_eq!(row.get::<i64, _>("grant_count"), 1);

        let principals = [EffectiveSandboxPrincipal {
            subject_type: "user".to_owned(),
            subject_id: context.user_id.clone(),
        }];
        let (volumes, total) = DriveSandboxService::new(SqlSandboxStore::new(pool.clone()))
            .list_accessible_for_principals(&context.tenant_id, &principals, 0, 20)
            .await
            .expect("list materialized runtime root");
        assert_eq!(total, 1);
        assert_eq!(volumes.len(), 1);
        assert_eq!(volumes[0].display_name, "Runtime root");

        let page = DriveSandboxDirectoryService::new(
            SqlSandboxStore::new(pool.clone()),
            LocalSandboxDirectoryProvider,
            SqlSandboxMutationOperationStore::new(pool),
        )
        .list_children(ListSandboxDirectoryCommand {
            tenant_id: context.tenant_id.clone(),
            sandbox_id: volumes[0].id.clone(),
            principals: principals.to_vec(),
            parent_logical_path: String::new(),
            page_size: 20,
            cursor: None,
        })
        .await
        .expect("browse materialized runtime root");
        assert!(page.items.iter().any(|entry| entry.name == "workspace"));
    }

    #[cfg(not(windows))]
    #[test]
    fn unix_discovery_exposes_the_filesystem_root() {
        let roots = discover_runtime_sandbox_roots_with(|path| path == Path::new("/"));

        assert_eq!(
            roots,
            vec![RuntimeSandboxRoot {
                display_name: "Filesystem (/)".to_owned(),
                path: PathBuf::from("/"),
            }]
        );
    }
}
