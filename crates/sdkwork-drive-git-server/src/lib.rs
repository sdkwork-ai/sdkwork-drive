mod auth;
mod bare_repo;
mod registry;
mod repo_path;

pub use auth::{validate_git_request_auth, GitAuthContext};
pub use bare_repo::{ensure_bare_repository, provision_git_repository_directory};
pub use registry::DriveGitRepoRegistry;
pub use repo_path::{
    git_repo_relative_path, resolve_bare_repository_path, GitRepoPathError, GitRepoPathParts,
};
