use async_trait::async_trait;

use crate::domain::sandbox::AuthorizedSandboxMount;
use crate::domain::sandbox_directory::{
    SandboxDirectoryEntry, SandboxDirectoryPage, SandboxDirectoryPageRequest, SandboxEntryName,
    SandboxFileContent, SandboxLogicalPath,
};
use crate::DriveServiceError;

/// Provider-neutral directory operations over an already-authorized sandbox mount.
/// Implementations must not serialize or log the mount's private root reference.
#[async_trait]
pub trait DriveSandboxDirectoryProvider: Send + Sync {
    fn supports(&self, provider_kind: &str) -> bool;

    async fn list_children(
        &self,
        mount: &AuthorizedSandboxMount,
        parent: &SandboxLogicalPath,
        page: &SandboxDirectoryPageRequest,
    ) -> Result<SandboxDirectoryPage, DriveServiceError>;

    async fn create_directory(
        &self,
        mount: &AuthorizedSandboxMount,
        parent: &SandboxLogicalPath,
        name: &SandboxEntryName,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError>;

    /// Reads the exact target used by idempotent crash recovery. Returns `None` only when the
    /// target does not exist; a file, symlink, or unsupported entry at the target is a conflict.
    async fn get_directory(
        &self,
        mount: &AuthorizedSandboxMount,
        parent: &SandboxLogicalPath,
        name: &SandboxEntryName,
    ) -> Result<Option<SandboxDirectoryEntry>, DriveServiceError>;
}

/// Complete provider-neutral file-system operations over an authorized sandbox mount.
/// Implementations must keep physical provider roots private and enforce containment for every call.
#[async_trait]
pub trait DriveSandboxFileSystemProvider: DriveSandboxDirectoryProvider {
    async fn get_entry(
        &self,
        mount: &AuthorizedSandboxMount,
        logical_path: &SandboxLogicalPath,
    ) -> Result<Option<SandboxDirectoryEntry>, DriveServiceError>;

    async fn read_file(
        &self,
        mount: &AuthorizedSandboxMount,
        logical_path: &SandboxLogicalPath,
        max_bytes: usize,
    ) -> Result<SandboxFileContent, DriveServiceError>;

    async fn create_file(
        &self,
        mount: &AuthorizedSandboxMount,
        parent: &SandboxLogicalPath,
        name: &SandboxEntryName,
        bytes: &[u8],
    ) -> Result<SandboxDirectoryEntry, DriveServiceError>;

    async fn update_file(
        &self,
        mount: &AuthorizedSandboxMount,
        logical_path: &SandboxLogicalPath,
        expected_revision: &str,
        bytes: &[u8],
    ) -> Result<SandboxDirectoryEntry, DriveServiceError>;

    async fn move_entry(
        &self,
        mount: &AuthorizedSandboxMount,
        logical_path: &SandboxLogicalPath,
        destination_parent: &SandboxLogicalPath,
        destination_name: &SandboxEntryName,
        expected_revision: &str,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError>;

    async fn delete_entry(
        &self,
        mount: &AuthorizedSandboxMount,
        logical_path: &SandboxLogicalPath,
        expected_revision: &str,
        recursive: bool,
    ) -> Result<(), DriveServiceError>;
}
