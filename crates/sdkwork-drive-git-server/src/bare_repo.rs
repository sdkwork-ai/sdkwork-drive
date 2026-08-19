use std::fs;
use std::path::Path;

use crate::repo_path::{bare_repository_exists, GitRepoPathError};

pub fn ensure_bare_repository(path: &Path) -> Result<(), GitRepoPathError> {
    if bare_repository_exists(path) {
        return Ok(());
    }

    if path.exists() {
        return Err(GitRepoPathError::InvalidPath(format!(
            "path exists but is not a bare git repository: {}",
            path.display()
        )));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    gix::init_bare(path).map_err(|error| {
        GitRepoPathError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("failed to initialize bare git repository: {error}"),
        ))
    })?;
    Ok(())
}

pub async fn provision_git_repository_directory(
    pool: &sqlx::PgPool,
    object_runtime: &sdkwork_drive_object_runtime::DriveObjectStoreRuntime,
    tenant_id: &str,
    space_id: &str,
    repo_name: &str,
) -> Result<std::path::PathBuf, GitRepoPathError> {
    let path =
        crate::repo_path::resolve_bare_repository_path(pool, object_runtime, tenant_id, space_id, repo_name)
            .await?;
    ensure_bare_repository(&path)?;
    Ok(path)
}
