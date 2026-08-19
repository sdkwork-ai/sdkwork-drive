use std::path::{Path, PathBuf};

use sdkwork_drive_storage_contract::DriveStorageProviderKind;
use sqlx::PgPool;
use url::Url;

const GIT_REPOS_SEGMENT: &str = "git-repos";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRepoPathParts {
    pub space_id: String,
    pub repo_name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GitRepoPathError {
    #[error("invalid git repository path: {0}")]
    InvalidPath(String),
    #[error("storage provider is not local filesystem")]
    UnsupportedProvider,
    #[error("git repository space was not found")]
    SpaceNotFound,
    #[error("git repository directory was not found")]
    RepositoryNotFound,
    #[error("database error: {0}")]
    Database(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn git_repo_relative_path(space_id: &str, repo_name: &str) -> String {
    format!("{space_id}/{}.git", normalize_repo_name(repo_name))
}

pub fn parse_git_repo_relative_path(relative: &str) -> Result<GitRepoPathParts, GitRepoPathError> {
    let trimmed = relative.trim().trim_matches('/');
    if trimmed.is_empty() {
        return Err(GitRepoPathError::InvalidPath(
            "repository path is empty".into(),
        ));
    }

    let (space_id, repo_with_suffix) = trimmed.split_once('/').ok_or_else(|| {
        GitRepoPathError::InvalidPath(format!("expected space/repo.git: {relative}"))
    })?;
    if space_id.trim().is_empty() {
        return Err(GitRepoPathError::InvalidPath(
            "space id is required".into(),
        ));
    }

    let repo_name = repo_with_suffix
        .strip_suffix(".git")
        .ok_or_else(|| {
            GitRepoPathError::InvalidPath(format!(
                "repository path must end with .git: {relative}"
            ))
        })?;
    let repo_name = normalize_repo_name(repo_name);
    if repo_name.is_empty() {
        return Err(GitRepoPathError::InvalidPath(
            "repository name is required".into(),
        ));
    }

    Ok(GitRepoPathParts {
        space_id: space_id.to_string(),
        repo_name,
    })
}

pub async fn resolve_bare_repository_path(
    pool: &PgPool,
    _object_runtime: &sdkwork_drive_object_runtime::DriveObjectStoreRuntime,
    tenant_id: &str,
    space_id: &str,
    repo_name: &str,
) -> Result<PathBuf, GitRepoPathError> {
    let repo_name = normalize_repo_name(repo_name);
    validate_git_repository_node(pool, tenant_id, space_id, &repo_name).await?;

    let provider = resolve_git_storage_provider(pool, tenant_id, space_id).await?;
    if provider.provider_kind != DriveStorageProviderKind::LocalFilesystem {
        return Err(GitRepoPathError::UnsupportedProvider);
    }

    let local_root = local_root_from_endpoint(&provider.endpoint_url)?;
    let storage_root_prefix = provider.storage_root_prefix;
    let node_id = find_git_repository_node_id(pool, tenant_id, space_id, &repo_name).await?;
    let relative = format!("{storage_root_prefix}/{GIT_REPOS_SEGMENT}/{node_id}.git");
    Ok(local_root.join(relative))
}

pub(crate) fn normalize_repo_name(repo_name: &str) -> String {
    repo_name.trim().trim_end_matches(".git").trim().to_string()
}

async fn validate_git_repository_node(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    repo_name: &str,
) -> Result<(), GitRepoPathError> {
    let space_type = sqlx::query_scalar::<_, String>(
        "SELECT space_type
         FROM dr_drive_space
         WHERE tenant_id=$1 AND id=$2 AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| GitRepoPathError::Database(error.to_string()))?;

    match space_type.as_deref() {
        Some("git_repository") => {}
        Some(_) | None => return Err(GitRepoPathError::SpaceNotFound),
    }

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id=$1
           AND space_id=$2
           AND parent_node_id IS NULL
           AND node_type='folder'
           AND node_name=$3
           AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(space_id)
    .bind(repo_name)
    .fetch_one(pool)
    .await
    .map_err(|error| GitRepoPathError::Database(error.to_string()))?;
    if count == 0 {
        return Err(GitRepoPathError::RepositoryNotFound);
    }
    Ok(())
}

async fn find_git_repository_node_id(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    repo_name: &str,
) -> Result<String, GitRepoPathError> {
    sqlx::query_scalar(
        "SELECT id
         FROM dr_drive_node
         WHERE tenant_id=$1
           AND space_id=$2
           AND parent_node_id IS NULL
           AND node_type='folder'
           AND node_name=$3
           AND lifecycle_status='active'
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(space_id)
    .bind(repo_name)
    .fetch_optional(pool)
    .await
    .map_err(|error| GitRepoPathError::Database(error.to_string()))?
    .ok_or(GitRepoPathError::RepositoryNotFound)
}

struct ResolvedGitStorageProvider {
    provider_kind: DriveStorageProviderKind,
    endpoint_url: String,
    storage_root_prefix: String,
}

async fn resolve_git_storage_provider(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
) -> Result<ResolvedGitStorageProvider, GitRepoPathError> {
    let row = sqlx::query_as::<_, (String, String, String)>(
        "SELECT provider.provider_kind, provider.endpoint_url, binding.storage_root_prefix
         FROM dr_drive_storage_provider_binding binding
         INNER JOIN dr_drive_storage_provider provider ON provider.id = binding.provider_id
         WHERE binding.tenant_id=$1
           AND binding.space_id=$2
           AND binding.purpose='primary'
           AND binding.lifecycle_status='active'
           AND provider.status='active'
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| GitRepoPathError::Database(error.to_string()))?;

    if let Some(row) = row {
        return Ok(storage_binding_row_into_resolved(row));
    }

    let row = sqlx::query_as::<_, (String, String, String)>(
        "SELECT provider.provider_kind, provider.endpoint_url, binding.storage_root_prefix
         FROM dr_drive_storage_provider_binding binding
         INNER JOIN dr_drive_storage_provider provider ON provider.id = binding.provider_id
         WHERE binding.tenant_id=$1
           AND binding.binding_scope='space_type'
           AND binding.purpose='git_repository'
           AND binding.lifecycle_status='active'
           AND provider.status='active'
         LIMIT 1",
    )
    .bind(tenant_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| GitRepoPathError::Database(error.to_string()))?
    .ok_or(GitRepoPathError::SpaceNotFound)?;

    Ok(storage_binding_row_into_resolved(row))
}

fn storage_binding_row_into_resolved(
    (provider_kind, endpoint_url, storage_root_prefix): (String, String, String),
) -> ResolvedGitStorageProvider {
    ResolvedGitStorageProvider {
        provider_kind: DriveStorageProviderKind::try_from_str(&provider_kind)
            .unwrap_or(DriveStorageProviderKind::Custom(provider_kind)),
        endpoint_url,
        storage_root_prefix,
    }
}

fn local_root_from_endpoint(endpoint_url: &str) -> Result<PathBuf, GitRepoPathError> {
    let url = Url::parse(endpoint_url).map_err(|_| GitRepoPathError::UnsupportedProvider)?;
    if url.scheme() != "file" || (url.has_host() && url.host_str() != Some("localhost")) {
        return Err(GitRepoPathError::UnsupportedProvider);
    }
    url.to_file_path()
        .map_err(|_| GitRepoPathError::UnsupportedProvider)
}

pub fn bare_repository_exists(path: &Path) -> bool {
    gix::open(path)
        .map(|repo| repo.is_bare())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{
        git_repo_relative_path, normalize_repo_name, parse_git_repo_relative_path, GitRepoPathParts,
    };

    #[test]
    fn git_repo_relative_path_formats_space_and_repo() {
        assert_eq!(
            git_repo_relative_path("space-1", "my-repo"),
            "space-1/my-repo.git"
        );
    }

    #[test]
    fn parse_git_repo_relative_path_accepts_standard_shape() {
        let parsed = parse_git_repo_relative_path("space-1/my-repo.git").expect("parse");
        assert_eq!(
            parsed,
            GitRepoPathParts {
                space_id: "space-1".to_string(),
                repo_name: "my-repo".to_string(),
            }
        );
    }

    #[test]
    fn parse_git_repo_relative_path_rejects_missing_git_suffix() {
        let error = parse_git_repo_relative_path("space-1/my-repo").unwrap_err();
        assert!(error.to_string().contains(".git"));
    }

    #[test]
    fn normalize_repo_name_strips_suffix_and_whitespace() {
        assert_eq!(normalize_repo_name(" demo.git "), "demo");
    }
}
