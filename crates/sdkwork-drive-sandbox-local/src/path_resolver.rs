use std::fs;
use std::path::{Component, Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SandboxPathError {
    #[error("sandbox root is unavailable")]
    RootUnavailable,
    #[error("sandbox paths must be relative")]
    AbsolutePath,
    #[error("sandbox paths must not traverse parent directories")]
    ParentTraversal,
    #[error("sandbox path is invalid")]
    InvalidPath,
    #[error("sandbox path resolves outside its root")]
    OutsideRoot,
    #[error("sandbox path could not be resolved")]
    Unresolvable,
}

#[derive(Debug, Clone)]
pub struct SandboxPathResolver {
    root: PathBuf,
}

impl SandboxPathResolver {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, SandboxPathError> {
        let root = fs::canonicalize(root).map_err(|_| SandboxPathError::RootUnavailable)?;
        if !root.is_dir() {
            return Err(SandboxPathError::RootUnavailable);
        }
        Ok(Self { root })
    }

    pub fn resolve_existing(&self, relative_path: &str) -> Result<PathBuf, SandboxPathError> {
        let candidate = self.unresolved_candidate(relative_path)?;
        let canonical = fs::canonicalize(candidate).map_err(|_| SandboxPathError::Unresolvable)?;
        self.ensure_contained(canonical)
    }

    /// Resolves a target that may not exist. Existing ancestors are canonicalized so a
    /// symlink in the parent chain cannot redirect a create/write operation outside root.
    pub fn resolve_create_target(&self, relative_path: &str) -> Result<PathBuf, SandboxPathError> {
        let candidate = self.unresolved_candidate(relative_path)?;
        let parent = candidate.parent().ok_or(SandboxPathError::InvalidPath)?;
        let canonical_parent =
            fs::canonicalize(parent).map_err(|_| SandboxPathError::Unresolvable)?;
        let contained_parent = self.ensure_contained(canonical_parent)?;
        let name = candidate.file_name().ok_or(SandboxPathError::InvalidPath)?;
        Ok(contained_parent.join(name))
    }

    fn unresolved_candidate(&self, relative_path: &str) -> Result<PathBuf, SandboxPathError> {
        let path = Path::new(relative_path.trim());
        if relative_path.trim().is_empty() {
            return Err(SandboxPathError::InvalidPath);
        }
        if path.is_absolute() {
            return Err(SandboxPathError::AbsolutePath);
        }
        if path
            .components()
            .any(|part| matches!(part, Component::RootDir | Component::Prefix(_)))
        {
            return Err(SandboxPathError::AbsolutePath);
        }
        if path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
        {
            return Err(SandboxPathError::ParentTraversal);
        }
        Ok(self.root.join(path))
    }

    fn ensure_contained(&self, candidate: PathBuf) -> Result<PathBuf, SandboxPathError> {
        if candidate.starts_with(&self.root) {
            Ok(candidate)
        } else {
            Err(SandboxPathError::OutsideRoot)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_absolute_and_parent_paths() {
        let temp = tempfile::tempdir().expect("tempdir");
        let resolver = SandboxPathResolver::new(temp.path()).expect("resolver");
        assert_eq!(
            resolver.resolve_create_target("../escape"),
            Err(SandboxPathError::ParentTraversal)
        );
        assert_eq!(
            resolver.resolve_create_target("/escape"),
            Err(SandboxPathError::AbsolutePath)
        );
    }

    #[test]
    fn resolves_new_path_inside_root() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(temp.path().join("projects")).expect("projects");
        let resolver = SandboxPathResolver::new(temp.path()).expect("resolver");
        let target = resolver
            .resolve_create_target("projects/new-project")
            .expect("target");
        assert!(target.starts_with(std::fs::canonicalize(temp.path()).expect("canonical root")));
    }
}
