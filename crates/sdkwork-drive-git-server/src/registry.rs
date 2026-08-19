use gitserver_core::discovery::RepoInfo;
use gitserver_core::dynamic_registry::{MutableRepoRegistry, RepoResolver};
use gitserver_core::error::{Error, Result};
use sdkwork_drive_object_runtime::DriveObjectStoreRuntime;
use sqlx::PgPool;
use tokio::runtime::Handle;

use crate::auth::GitAuthContext;
use crate::bare_repo::ensure_bare_repository;
use crate::repo_path::{
    parse_git_repo_relative_path, resolve_bare_repository_path, GitRepoPathError,
};

#[derive(Clone)]
pub struct DriveGitRepoRegistry {
    pool: PgPool,
    object_runtime: DriveObjectStoreRuntime,
}

impl DriveGitRepoRegistry {
    pub fn new(pool: PgPool) -> Self {
        Self {
            object_runtime: DriveObjectStoreRuntime::new(pool.clone()),
            pool,
        }
    }

    fn resolve_for_auth(
        &self,
        auth: &GitAuthContext,
        relative: &str,
    ) -> std::result::Result<RepoInfo, GitRepoPathError> {
        let parts = parse_git_repo_relative_path(relative)?;
        let handle = Handle::current();
        let absolute_path = tokio::task::block_in_place(|| {
            handle.block_on(resolve_bare_repository_path(
                &self.pool,
                &self.object_runtime,
                &auth.tenant_id,
                &parts.space_id,
                &parts.repo_name,
            ))
        })?;
        ensure_bare_repository(&absolute_path)?;
        let relative_path = relative.trim().trim_matches('/').to_string();
        Ok(RepoInfo {
            name: format!("{}.git", parts.repo_name),
            relative_path,
            absolute_path,
            description: None,
        })
    }
}

impl RepoResolver for DriveGitRepoRegistry {
    fn resolve(&self, relative: &str) -> Result<RepoInfo> {
        let auth = GitAuthContext::current().ok_or_else(|| {
            Error::Protocol("git request is missing authenticated drive context".into())
        })?;
        self.resolve_for_auth(&auth, relative)
            .map_err(map_path_error)
    }

    fn list(&self) -> Result<Vec<RepoInfo>> {
        Err(Error::Protocol(
            "listing git repositories is not supported on the git smart HTTP surface".into(),
        ))
    }
}

impl MutableRepoRegistry for DriveGitRepoRegistry {
    fn register(&self, _repo: RepoInfo) -> Result<()> {
        Err(Error::Protocol(
            "dynamic git repository registration is managed by drive folder provisioning".into(),
        ))
    }

    fn unregister(&self, _relative: &str) -> Result<()> {
        Err(Error::Protocol(
            "dynamic git repository unregistration is managed by drive folder lifecycle".into(),
        ))
    }
}

fn map_path_error(error: GitRepoPathError) -> Error {
    match error {
        GitRepoPathError::InvalidPath(message) => Error::PathTraversal(message.into()),
        GitRepoPathError::UnsupportedProvider => Error::Protocol(
            "git repositories require a local filesystem storage provider".into(),
        ),
        GitRepoPathError::SpaceNotFound | GitRepoPathError::RepositoryNotFound => {
            Error::RepoNotFound(error.to_string())
        }
        GitRepoPathError::Database(message) => Error::Protocol(message),
        GitRepoPathError::Io(error) => Error::Io(error),
    }
}
