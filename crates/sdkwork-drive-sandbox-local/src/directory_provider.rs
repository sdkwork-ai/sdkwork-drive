use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Write};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::UNIX_EPOCH;

use async_trait::async_trait;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, Metadata, OpenOptions};
use sdkwork_drive_workspace_service::domain::sandbox::AuthorizedSandboxMount;
use sdkwork_drive_workspace_service::domain::sandbox_directory::{
    SandboxDirectoryEntry, SandboxDirectoryPage, SandboxDirectoryPageRequest, SandboxEntryKind,
    SandboxEntryName, SandboxFileContent, SandboxLogicalPath,
};
use sdkwork_drive_workspace_service::ports::sandbox_directory_provider::{
    DriveSandboxDirectoryProvider, DriveSandboxFileSystemProvider,
};
use sdkwork_drive_workspace_service::DriveServiceError;
use sdkwork_utils_rust::{base64url_decode, base64url_encode, sha256_hash};

const LOCAL_FILESYSTEM_PROVIDER_KIND: &str = "local_filesystem";
const CURSOR_VERSION: &[u8] = b"v1";
const MUTATION_LOCK_STRIPES: usize = 64;

#[derive(Debug, Clone, Copy, Default)]
pub struct LocalSandboxDirectoryProvider;

#[async_trait]
impl DriveSandboxDirectoryProvider for LocalSandboxDirectoryProvider {
    fn supports(&self, provider_kind: &str) -> bool {
        provider_kind == LOCAL_FILESYSTEM_PROVIDER_KIND
    }

    async fn list_children(
        &self,
        mount: &AuthorizedSandboxMount,
        parent: &SandboxLogicalPath,
        page: &SandboxDirectoryPageRequest,
    ) -> Result<SandboxDirectoryPage, DriveServiceError> {
        let input = LocalListInput {
            sandbox_id: mount.sandbox_id().to_string(),
            root_entry_id: mount.root_entry_id().to_string(),
            private_root_ref: mount.private_root_ref().to_string(),
            parent: parent.clone(),
            page: page.clone(),
        };
        tokio::task::spawn_blocking(move || list_children_sync(input))
            .await
            .map_err(|_| {
                DriveServiceError::Internal(
                    "sandbox directory provider task was interrupted".to_string(),
                )
            })?
    }

    async fn create_directory(
        &self,
        mount: &AuthorizedSandboxMount,
        parent: &SandboxLogicalPath,
        name: &SandboxEntryName,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        let input = LocalCreateInput {
            sandbox_id: mount.sandbox_id().to_string(),
            root_entry_id: mount.root_entry_id().to_string(),
            private_root_ref: mount.private_root_ref().to_string(),
            parent: parent.clone(),
            name: name.clone(),
        };
        tokio::task::spawn_blocking(move || create_directory_sync(input))
            .await
            .map_err(|_| {
                DriveServiceError::Internal(
                    "sandbox directory provider task was interrupted".to_string(),
                )
            })?
    }

    async fn get_directory(
        &self,
        mount: &AuthorizedSandboxMount,
        parent: &SandboxLogicalPath,
        name: &SandboxEntryName,
    ) -> Result<Option<SandboxDirectoryEntry>, DriveServiceError> {
        let input = LocalCreateInput {
            sandbox_id: mount.sandbox_id().to_string(),
            root_entry_id: mount.root_entry_id().to_string(),
            private_root_ref: mount.private_root_ref().to_string(),
            parent: parent.clone(),
            name: name.clone(),
        };
        tokio::task::spawn_blocking(move || get_directory_sync(input))
            .await
            .map_err(|_| {
                DriveServiceError::Internal(
                    "sandbox directory provider task was interrupted".to_string(),
                )
            })?
    }
}

#[async_trait]
impl DriveSandboxFileSystemProvider for LocalSandboxDirectoryProvider {
    async fn get_entry(
        &self,
        mount: &AuthorizedSandboxMount,
        logical_path: &SandboxLogicalPath,
    ) -> Result<Option<SandboxDirectoryEntry>, DriveServiceError> {
        let input = LocalEntryInput::new(mount, logical_path);
        tokio::task::spawn_blocking(move || get_entry_sync(input))
            .await
            .map_err(|_| interrupted_provider_task())?
    }

    async fn read_file(
        &self,
        mount: &AuthorizedSandboxMount,
        logical_path: &SandboxLogicalPath,
        max_bytes: usize,
    ) -> Result<SandboxFileContent, DriveServiceError> {
        let input = LocalEntryInput::new(mount, logical_path);
        tokio::task::spawn_blocking(move || read_file_sync(input, max_bytes))
            .await
            .map_err(|_| interrupted_provider_task())?
    }

    async fn create_file(
        &self,
        mount: &AuthorizedSandboxMount,
        parent: &SandboxLogicalPath,
        name: &SandboxEntryName,
        bytes: &[u8],
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        let input = LocalCreateFileInput {
            entry: LocalEntryInput::new(mount, &parent.join(name)),
            bytes: bytes.to_vec(),
        };
        tokio::task::spawn_blocking(move || create_file_sync(input))
            .await
            .map_err(|_| interrupted_provider_task())?
    }

    async fn update_file(
        &self,
        mount: &AuthorizedSandboxMount,
        logical_path: &SandboxLogicalPath,
        expected_revision: &str,
        bytes: &[u8],
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        let input = LocalUpdateFileInput {
            entry: LocalEntryInput::new(mount, logical_path),
            expected_revision: expected_revision.to_string(),
            bytes: bytes.to_vec(),
        };
        tokio::task::spawn_blocking(move || update_file_sync(input))
            .await
            .map_err(|_| interrupted_provider_task())?
    }

    async fn move_entry(
        &self,
        mount: &AuthorizedSandboxMount,
        logical_path: &SandboxLogicalPath,
        destination_parent: &SandboxLogicalPath,
        destination_name: &SandboxEntryName,
        expected_revision: &str,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        let input = LocalMoveInput {
            entry: LocalEntryInput::new(mount, logical_path),
            destination_parent: destination_parent.clone(),
            destination_name: destination_name.clone(),
            expected_revision: expected_revision.to_string(),
        };
        tokio::task::spawn_blocking(move || move_entry_sync(input))
            .await
            .map_err(|_| interrupted_provider_task())?
    }

    async fn delete_entry(
        &self,
        mount: &AuthorizedSandboxMount,
        logical_path: &SandboxLogicalPath,
        expected_revision: &str,
        recursive: bool,
    ) -> Result<(), DriveServiceError> {
        let input = LocalDeleteInput {
            entry: LocalEntryInput::new(mount, logical_path),
            expected_revision: expected_revision.to_string(),
            recursive,
        };
        tokio::task::spawn_blocking(move || delete_entry_sync(input))
            .await
            .map_err(|_| interrupted_provider_task())?
    }
}

struct LocalListInput {
    sandbox_id: String,
    root_entry_id: String,
    private_root_ref: String,
    parent: SandboxLogicalPath,
    page: SandboxDirectoryPageRequest,
}

struct LocalCreateInput {
    sandbox_id: String,
    root_entry_id: String,
    private_root_ref: String,
    parent: SandboxLogicalPath,
    name: SandboxEntryName,
}

#[derive(Clone)]
struct LocalEntryInput {
    sandbox_id: String,
    root_entry_id: String,
    private_root_ref: String,
    logical_path: SandboxLogicalPath,
}

impl LocalEntryInput {
    fn new(mount: &AuthorizedSandboxMount, logical_path: &SandboxLogicalPath) -> Self {
        Self {
            sandbox_id: mount.sandbox_id().to_string(),
            root_entry_id: mount.root_entry_id().to_string(),
            private_root_ref: mount.private_root_ref().to_string(),
            logical_path: logical_path.clone(),
        }
    }
}

struct LocalCreateFileInput {
    entry: LocalEntryInput,
    bytes: Vec<u8>,
}

struct LocalUpdateFileInput {
    entry: LocalEntryInput,
    expected_revision: String,
    bytes: Vec<u8>,
}

struct LocalMoveInput {
    entry: LocalEntryInput,
    destination_parent: SandboxLogicalPath,
    destination_name: SandboxEntryName,
    expected_revision: String,
}

struct LocalDeleteInput {
    entry: LocalEntryInput,
    expected_revision: String,
    recursive: bool,
}

fn list_children_sync(input: LocalListInput) -> Result<SandboxDirectoryPage, DriveServiceError> {
    let root = open_capability_root(&input.private_root_ref)?;
    let cursor = decode_cursor(
        input.page.cursor.as_deref(),
        &input.sandbox_id,
        &input.parent,
    )?;
    let directory = open_verified_directory(&root, &input.parent)?;

    let mut window = BTreeMap::<String, SandboxDirectoryEntry>::new();
    let mut skipped_symlinks = 0usize;
    let mut skipped_unrepresentable = 0usize;
    let mut skipped_unsupported = 0usize;
    let mut skipped_unreadable = 0usize;
    let mut snapshot_count = 0u64;
    let mut snapshot_digest = 0u128;
    let read_dir = directory
        .entries()
        .map_err(|error| map_read_error(error, "sandbox directory"))?;
    for item in read_dir {
        let Ok(item) = item else {
            skipped_unreadable += 1;
            continue;
        };
        let Ok(name) = item.file_name().into_string() else {
            skipped_unrepresentable += 1;
            continue;
        };
        let Ok(logical_path) = join_existing_logical_path(&input.parent, &name) else {
            skipped_unsupported += 1;
            continue;
        };
        let Ok(file_type) = item.file_type() else {
            skipped_unreadable += 1;
            continue;
        };
        if file_type.is_symlink() {
            skipped_symlinks += 1;
            continue;
        }
        let Ok(metadata) = directory.metadata(&name) else {
            skipped_unreadable += 1;
            continue;
        };
        let kind = if metadata.is_dir() {
            SandboxEntryKind::Directory
        } else if metadata.is_file() {
            SandboxEntryKind::File
        } else {
            continue;
        };
        snapshot_count = snapshot_count.saturating_add(1);
        snapshot_digest ^= entry_snapshot_digest(logical_path.as_str(), kind);
        if cursor
            .as_ref()
            .is_some_and(|cursor| name.as_str() <= cursor.name.as_str())
        {
            continue;
        }
        let entry = directory_entry(
            &input.sandbox_id,
            &input.root_entry_id,
            &input.parent,
            name.clone(),
            logical_path,
            kind,
            &metadata,
        );
        window.insert(name, entry);
        if window.len() > input.page.page_size + 1 {
            window.pop_last();
        }
    }

    let snapshot = format!("{snapshot_count:x}-{snapshot_digest:032x}");
    if cursor
        .as_ref()
        .is_some_and(|cursor| cursor.snapshot != snapshot)
    {
        return Err(DriveServiceError::Conflict(
            "sandbox directory changed; restart pagination".to_string(),
        ));
    }
    if skipped_symlinks > 0
        || skipped_unrepresentable > 0
        || skipped_unsupported > 0
        || skipped_unreadable > 0
    {
        tracing::warn!(
            target: "sdkwork.drive.sandbox",
            sandbox_id = %input.sandbox_id,
            skipped_symlinks,
            skipped_unrepresentable,
            skipped_unsupported,
            skipped_unreadable,
            "sandbox directory entries were excluded by containment and representation policy"
        );
    }

    let has_more = window.len() > input.page.page_size;
    if has_more {
        window.pop_last();
    }
    let next_cursor = if has_more {
        window
            .last_key_value()
            .map(|(name, _)| encode_cursor(&input.sandbox_id, &input.parent, &snapshot, name))
    } else {
        None
    };
    Ok(SandboxDirectoryPage {
        items: window.into_values().collect(),
        next_cursor,
        has_more,
    })
}

fn create_directory_sync(
    input: LocalCreateInput,
) -> Result<SandboxDirectoryEntry, DriveServiceError> {
    let root = open_capability_root(&input.private_root_ref)?;
    let parent = open_verified_directory(&root, &input.parent)?;
    let logical_path = input.parent.join(&input.name);
    parent
        .create_dir(input.name.as_str())
        .map_err(map_create_error)?;
    let metadata = parent
        .metadata(input.name.as_str())
        .map_err(|error| map_read_error(error, "created sandbox directory"))?;
    if !metadata.is_dir() {
        return Err(DriveServiceError::Internal(
            "created sandbox entry is not a directory".to_string(),
        ));
    }
    Ok(directory_entry(
        &input.sandbox_id,
        &input.root_entry_id,
        &input.parent,
        input.name.as_str().to_string(),
        logical_path,
        SandboxEntryKind::Directory,
        &metadata,
    ))
}

fn get_directory_sync(
    input: LocalCreateInput,
) -> Result<Option<SandboxDirectoryEntry>, DriveServiceError> {
    let root = open_capability_root(&input.private_root_ref)?;
    let logical_path = input.parent.join(&input.name);
    let parent = open_verified_directory(&root, &input.parent)?;
    let symlink_metadata = match parent.symlink_metadata(input.name.as_str()) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(map_read_error(error, "sandbox directory target")),
    };
    if symlink_metadata.file_type().is_symlink() {
        return Err(DriveServiceError::Conflict(
            "sandbox directory target is a symbolic link".to_string(),
        ));
    }
    let metadata = parent
        .metadata(input.name.as_str())
        .map_err(|error| map_read_error(error, "sandbox directory target"))?;
    if !metadata.is_dir() {
        return Err(DriveServiceError::Conflict(
            "sandbox directory target is not a directory".to_string(),
        ));
    }
    Ok(Some(directory_entry(
        &input.sandbox_id,
        &input.root_entry_id,
        &input.parent,
        input.name.as_str().to_string(),
        logical_path,
        SandboxEntryKind::Directory,
        &metadata,
    )))
}

fn get_entry_sync(
    input: LocalEntryInput,
) -> Result<Option<SandboxDirectoryEntry>, DriveServiceError> {
    let root = open_capability_root(&input.private_root_ref)?;
    let (parent_path, name) = split_entry_path(&input.logical_path)?;
    let parent = open_verified_directory(&root, &parent_path)?;
    let metadata = match entry_metadata(&parent, &name, "sandbox entry")? {
        Some(metadata) => metadata,
        None => return Ok(None),
    };
    let kind = supported_entry_kind(&metadata)?;
    Ok(Some(directory_entry(
        &input.sandbox_id,
        &input.root_entry_id,
        &parent_path,
        name.as_str().to_string(),
        input.logical_path,
        kind,
        &metadata,
    )))
}

fn read_file_sync(
    input: LocalEntryInput,
    max_bytes: usize,
) -> Result<SandboxFileContent, DriveServiceError> {
    let root = open_capability_root(&input.private_root_ref)?;
    let (parent_path, name) = split_entry_path(&input.logical_path)?;
    let parent = open_verified_directory(&root, &parent_path)?;
    let metadata = entry_metadata(&parent, &name, "sandbox file")?
        .ok_or_else(|| DriveServiceError::NotFound("sandbox file was not found".to_string()))?;
    if !metadata.is_file() {
        return Err(DriveServiceError::Validation(
            "sandbox entry is not a file".to_string(),
        ));
    }
    if metadata.len() > max_bytes as u64 {
        return Err(DriveServiceError::Validation(format!(
            "sandbox file exceeds the {max_bytes} byte read limit"
        )));
    }
    let mut file = parent
        .open(name.as_str())
        .map_err(|error| map_read_error(error, "sandbox file"))?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    Read::by_ref(&mut file)
        .take(max_bytes.saturating_add(1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| map_read_error(error, "sandbox file"))?;
    if bytes.len() > max_bytes {
        return Err(DriveServiceError::Validation(format!(
            "sandbox file exceeds the {max_bytes} byte read limit"
        )));
    }
    let entry = directory_entry(
        &input.sandbox_id,
        &input.root_entry_id,
        &parent_path,
        name.as_str().to_string(),
        input.logical_path,
        SandboxEntryKind::File,
        &metadata,
    );
    Ok(SandboxFileContent { entry, bytes })
}

fn create_file_sync(
    input: LocalCreateFileInput,
) -> Result<SandboxDirectoryEntry, DriveServiceError> {
    let _mutation_guard = lock_sandbox_mutation(&input.entry)?;
    let root = open_capability_root(&input.entry.private_root_ref)?;
    let (parent_path, name) = split_entry_path(&input.entry.logical_path)?;
    let parent = open_verified_directory(&root, &parent_path)?;
    if entry_metadata(&parent, &name, "sandbox file")?.is_some() {
        return Err(DriveServiceError::Conflict(
            "sandbox file already exists".to_string(),
        ));
    }
    let temporary_name = write_temporary_file(&parent, &input.bytes)?;
    let published = parent.hard_link(&temporary_name, &parent, name.as_str());
    let cleanup = parent.remove_file(&temporary_name);
    if let Err(error) = published {
        return Err(map_file_create_error(error));
    }
    cleanup.map_err(|_| {
        DriveServiceError::Internal("sandbox temporary file could not be removed".to_string())
    })?;
    created_file_entry(&input.entry, &parent_path, &name, &parent)
}

fn update_file_sync(
    input: LocalUpdateFileInput,
) -> Result<SandboxDirectoryEntry, DriveServiceError> {
    let _mutation_guard = lock_sandbox_mutation(&input.entry)?;
    let root = open_capability_root(&input.entry.private_root_ref)?;
    let (parent_path, name) = split_entry_path(&input.entry.logical_path)?;
    let parent = open_verified_directory(&root, &parent_path)?;
    let metadata = entry_metadata(&parent, &name, "sandbox file")?
        .ok_or_else(|| DriveServiceError::NotFound("sandbox file was not found".to_string()))?;
    if !metadata.is_file() {
        return Err(DriveServiceError::Validation(
            "sandbox entry is not a file".to_string(),
        ));
    }
    ensure_revision(&metadata, &input.expected_revision)?;
    let temporary_name = write_temporary_file(&parent, &input.bytes)?;
    if let Err(error) = parent.rename(&temporary_name, &parent, name.as_str()) {
        let _ = parent.remove_file(&temporary_name);
        return Err(map_file_update_error(error));
    }
    created_file_entry(&input.entry, &parent_path, &name, &parent)
}

fn move_entry_sync(input: LocalMoveInput) -> Result<SandboxDirectoryEntry, DriveServiceError> {
    let _mutation_guard = lock_sandbox_mutation(&input.entry)?;
    let root = open_capability_root(&input.entry.private_root_ref)?;
    let (source_parent_path, source_name) = split_entry_path(&input.entry.logical_path)?;
    let source_parent = open_verified_directory(&root, &source_parent_path)?;
    let source_metadata = entry_metadata(&source_parent, &source_name, "sandbox entry")?
        .ok_or_else(|| DriveServiceError::NotFound("sandbox entry was not found".to_string()))?;
    let kind = supported_entry_kind(&source_metadata)?;
    ensure_revision(&source_metadata, &input.expected_revision)?;
    let destination_path = input.destination_parent.join(&input.destination_name);
    if destination_path == input.entry.logical_path {
        return Ok(directory_entry(
            &input.entry.sandbox_id,
            &input.entry.root_entry_id,
            &source_parent_path,
            source_name.as_str().to_string(),
            input.entry.logical_path,
            kind,
            &source_metadata,
        ));
    }
    if kind == SandboxEntryKind::Directory
        && destination_path
            .as_str()
            .strip_prefix(input.entry.logical_path.as_str())
            .is_some_and(|suffix| suffix.starts_with('/'))
    {
        return Err(DriveServiceError::Validation(
            "sandbox directory cannot be moved inside itself".to_string(),
        ));
    }
    let destination_parent = open_verified_directory(&root, &input.destination_parent)?;
    if entry_metadata(
        &destination_parent,
        &input.destination_name,
        "sandbox destination entry",
    )?
    .is_some()
    {
        return Err(DriveServiceError::Conflict(
            "sandbox destination entry already exists".to_string(),
        ));
    }
    source_parent
        .rename(
            source_name.as_str(),
            &destination_parent,
            input.destination_name.as_str(),
        )
        .map_err(map_move_error)?;
    let metadata = entry_metadata(
        &destination_parent,
        &input.destination_name,
        "moved sandbox entry",
    )?
    .ok_or_else(|| {
        DriveServiceError::Internal("moved sandbox entry could not be read".to_string())
    })?;
    Ok(directory_entry(
        &input.entry.sandbox_id,
        &input.entry.root_entry_id,
        &input.destination_parent,
        input.destination_name.as_str().to_string(),
        destination_path,
        kind,
        &metadata,
    ))
}

fn delete_entry_sync(input: LocalDeleteInput) -> Result<(), DriveServiceError> {
    let _mutation_guard = lock_sandbox_mutation(&input.entry)?;
    let root = open_capability_root(&input.entry.private_root_ref)?;
    let (parent_path, name) = split_entry_path(&input.entry.logical_path)?;
    let parent = open_verified_directory(&root, &parent_path)?;
    let metadata = entry_metadata(&parent, &name, "sandbox entry")?
        .ok_or_else(|| DriveServiceError::NotFound("sandbox entry was not found".to_string()))?;
    let kind = supported_entry_kind(&metadata)?;
    ensure_revision(&metadata, &input.expected_revision)?;
    match kind {
        SandboxEntryKind::File => parent.remove_file(name.as_str()).map_err(map_delete_error),
        SandboxEntryKind::Directory if input.recursive => parent
            .remove_dir_all(name.as_str())
            .map_err(map_delete_error),
        SandboxEntryKind::Directory => parent.remove_dir(name.as_str()).map_err(map_delete_error),
    }
}

fn split_entry_path(
    logical_path: &SandboxLogicalPath,
) -> Result<(SandboxLogicalPath, SandboxEntryName), DriveServiceError> {
    logical_path.split_entry().map_err(|_| {
        DriveServiceError::Validation("sandbox root cannot be used as an entry".to_string())
    })
}

fn open_verified_directory(
    root: &Dir,
    logical_path: &SandboxLogicalPath,
) -> Result<Dir, DriveServiceError> {
    let mut current = root.try_clone().map_err(|_| {
        DriveServiceError::Internal("sandbox directory handle could not be cloned".to_string())
    })?;
    for segment in logical_path
        .as_str()
        .split('/')
        .filter(|value| !value.is_empty())
    {
        let metadata = current
            .symlink_metadata(segment)
            .map_err(|error| map_read_error(error, "sandbox directory"))?;
        if metadata.file_type().is_symlink() {
            return Err(DriveServiceError::PermissionDenied(
                "sandbox symbolic-link traversal is not permitted".to_string(),
            ));
        }
        if !metadata.is_dir() {
            return Err(DriveServiceError::Validation(
                "sandbox parent path is not a directory".to_string(),
            ));
        }
        current = current
            .open_dir(segment)
            .map_err(|error| map_read_error(error, "sandbox directory"))?;
    }
    Ok(current)
}

fn entry_metadata(
    parent: &Dir,
    name: &SandboxEntryName,
    resource: &str,
) -> Result<Option<Metadata>, DriveServiceError> {
    let symlink_metadata = match parent.symlink_metadata(name.as_str()) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(map_read_error(error, resource)),
    };
    if symlink_metadata.file_type().is_symlink() {
        return Err(DriveServiceError::PermissionDenied(
            "sandbox symbolic-link access is not permitted".to_string(),
        ));
    }
    parent
        .metadata(name.as_str())
        .map(Some)
        .map_err(|error| map_read_error(error, resource))
}

fn supported_entry_kind(metadata: &Metadata) -> Result<SandboxEntryKind, DriveServiceError> {
    if metadata.is_dir() {
        Ok(SandboxEntryKind::Directory)
    } else if metadata.is_file() {
        Ok(SandboxEntryKind::File)
    } else {
        Err(DriveServiceError::Validation(
            "sandbox entry type is not supported".to_string(),
        ))
    }
}

fn created_file_entry(
    input: &LocalEntryInput,
    parent_path: &SandboxLogicalPath,
    name: &SandboxEntryName,
    parent: &Dir,
) -> Result<SandboxDirectoryEntry, DriveServiceError> {
    let metadata = entry_metadata(parent, name, "sandbox file")?.ok_or_else(|| {
        DriveServiceError::Internal("sandbox file could not be read after mutation".to_string())
    })?;
    if !metadata.is_file() {
        return Err(DriveServiceError::Internal(
            "mutated sandbox entry is not a file".to_string(),
        ));
    }
    Ok(directory_entry(
        &input.sandbox_id,
        &input.root_entry_id,
        parent_path,
        name.as_str().to_string(),
        input.logical_path.clone(),
        SandboxEntryKind::File,
        &metadata,
    ))
}

fn write_temporary_file(parent: &Dir, bytes: &[u8]) -> Result<String, DriveServiceError> {
    let temporary_name = format!(".sdkwork-tmp-{}", uuid::Uuid::new_v4().simple());
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = parent
        .open_with(&temporary_name, &options)
        .map_err(map_file_update_error)?;
    if let Err(error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = parent.remove_file(&temporary_name);
        return Err(map_file_update_error(error));
    }
    Ok(temporary_name)
}

fn ensure_revision(metadata: &Metadata, expected_revision: &str) -> Result<(), DriveServiceError> {
    if metadata_revision(metadata) == expected_revision {
        Ok(())
    } else {
        Err(DriveServiceError::Conflict(
            "sandbox entry revision does not match If-Match".to_string(),
        ))
    }
}

fn lock_sandbox_mutation(
    input: &LocalEntryInput,
) -> Result<MutexGuard<'static, ()>, DriveServiceError> {
    static LOCKS: OnceLock<Vec<Mutex<()>>> = OnceLock::new();
    let locks = LOCKS.get_or_init(|| (0..MUTATION_LOCK_STRIPES).map(|_| Mutex::new(())).collect());
    let mut hasher = DefaultHasher::new();
    input.sandbox_id.hash(&mut hasher);
    input.private_root_ref.hash(&mut hasher);
    let index = hasher.finish() as usize % locks.len();
    locks[index].lock().map_err(|_| {
        DriveServiceError::Internal("sandbox mutation lock is unavailable".to_string())
    })
}

fn interrupted_provider_task() -> DriveServiceError {
    DriveServiceError::Internal("sandbox file-system provider task was interrupted".to_string())
}

fn open_capability_root(private_root_ref: &str) -> Result<Dir, DriveServiceError> {
    Dir::open_ambient_dir(private_root_ref, ambient_authority())
        .map_err(|error| map_read_error(error, "sandbox root"))
}

fn join_existing_logical_path(
    parent: &SandboxLogicalPath,
    name: &str,
) -> Result<SandboxLogicalPath, DriveServiceError> {
    let value = if parent.as_str().is_empty() {
        name.to_string()
    } else {
        format!("{}/{}", parent.as_str(), name)
    };
    SandboxLogicalPath::parse(&value).map_err(|_| {
        DriveServiceError::Internal(
            "sandbox directory contains an unsupported entry name".to_string(),
        )
    })
}

fn directory_entry(
    sandbox_id: &str,
    root_entry_id: &str,
    parent: &SandboxLogicalPath,
    name: String,
    logical_path: SandboxLogicalPath,
    kind: SandboxEntryKind,
    metadata: &Metadata,
) -> SandboxDirectoryEntry {
    let parent_id = if parent.as_str().is_empty() {
        root_entry_id.to_string()
    } else {
        stable_entry_id(sandbox_id, parent.as_str())
    };
    SandboxDirectoryEntry {
        id: stable_entry_id(sandbox_id, logical_path.as_str()),
        sandbox_id: sandbox_id.to_string(),
        parent_id,
        parent_logical_path: parent.as_str().to_string(),
        name,
        kind,
        logical_path: logical_path.as_str().to_string(),
        revision: metadata_revision(metadata),
    }
}

fn stable_entry_id(sandbox_id: &str, logical_path: &str) -> String {
    let digest = sha256_hash(format!("sandbox-entry\0{sandbox_id}\0{logical_path}").as_bytes());
    format!("sbe_{}", &digest[..32])
}

fn metadata_revision(metadata: &Metadata) -> String {
    let modified_nanos = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.into_std().duration_since(UNIX_EPOCH).ok())
        .map_or(0, |duration| duration.as_nanos());
    format!("{modified_nanos:x}-{:x}", metadata.len())
}

fn entry_snapshot_digest(logical_path: &str, kind: SandboxEntryKind) -> u128 {
    let kind = match kind {
        SandboxEntryKind::Directory => "directory",
        SandboxEntryKind::File => "file",
    };
    let digest = sha256_hash(format!("sandbox-snapshot\0{kind}\0{logical_path}").as_bytes());
    u128::from_str_radix(&digest[..32], 16).expect("sha256 hex prefix must be valid u128")
}

fn encode_cursor(
    sandbox_id: &str,
    parent: &SandboxLogicalPath,
    snapshot: &str,
    name: &str,
) -> String {
    let binding = cursor_binding(sandbox_id, parent);
    let mut payload =
        Vec::with_capacity(CURSOR_VERSION.len() + binding.len() + snapshot.len() + name.len() + 3);
    payload.extend_from_slice(CURSOR_VERSION);
    payload.push(0);
    payload.extend_from_slice(binding.as_bytes());
    payload.push(0);
    payload.extend_from_slice(snapshot.as_bytes());
    payload.push(0);
    payload.extend_from_slice(name.as_bytes());
    base64url_encode(&payload)
}

struct DecodedCursor {
    snapshot: String,
    name: String,
}

fn decode_cursor(
    cursor: Option<&str>,
    sandbox_id: &str,
    parent: &SandboxLogicalPath,
) -> Result<Option<DecodedCursor>, DriveServiceError> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };
    let decoded = base64url_decode(cursor).ok_or_else(invalid_cursor)?;
    let mut fields = decoded.splitn(4, |byte| *byte == 0);
    let version = fields.next().ok_or_else(invalid_cursor)?;
    let binding = fields.next().ok_or_else(invalid_cursor)?;
    let snapshot = fields.next().ok_or_else(invalid_cursor)?;
    let name = fields.next().ok_or_else(invalid_cursor)?;
    if version != CURSOR_VERSION || binding != cursor_binding(sandbox_id, parent).as_bytes() {
        return Err(invalid_cursor());
    }
    let snapshot = std::str::from_utf8(snapshot).map_err(|_| invalid_cursor())?;
    let name = std::str::from_utf8(name).map_err(|_| invalid_cursor())?;
    if snapshot.is_empty() || name.is_empty() {
        return Err(invalid_cursor());
    }
    Ok(Some(DecodedCursor {
        snapshot: snapshot.to_string(),
        name: name.to_string(),
    }))
}

fn cursor_binding(sandbox_id: &str, parent: &SandboxLogicalPath) -> String {
    sha256_hash(format!("sandbox-cursor\0{sandbox_id}\0{}", parent.as_str()).as_bytes())
}

fn invalid_cursor() -> DriveServiceError {
    DriveServiceError::Validation("sandbox cursor is invalid".to_string())
}

fn map_read_error(error: io::Error, resource: &str) -> DriveServiceError {
    match error.kind() {
        io::ErrorKind::NotFound => DriveServiceError::NotFound(format!("{resource} was not found")),
        io::ErrorKind::PermissionDenied => {
            DriveServiceError::PermissionDenied(format!("{resource} access is denied"))
        }
        _ => DriveServiceError::Internal(format!("{resource} could not be read")),
    }
}

fn map_create_error(error: io::Error) -> DriveServiceError {
    match error.kind() {
        io::ErrorKind::AlreadyExists => {
            DriveServiceError::Conflict("sandbox directory already exists".to_string())
        }
        io::ErrorKind::NotFound => {
            DriveServiceError::NotFound("sandbox parent directory was not found".to_string())
        }
        io::ErrorKind::PermissionDenied => DriveServiceError::PermissionDenied(
            "sandbox directory creation is not permitted".to_string(),
        ),
        _ => DriveServiceError::Internal("sandbox directory could not be created".to_string()),
    }
}

fn map_file_create_error(error: io::Error) -> DriveServiceError {
    match error.kind() {
        io::ErrorKind::AlreadyExists => {
            DriveServiceError::Conflict("sandbox file already exists".to_string())
        }
        io::ErrorKind::NotFound => {
            DriveServiceError::NotFound("sandbox parent directory was not found".to_string())
        }
        io::ErrorKind::PermissionDenied => DriveServiceError::PermissionDenied(
            "sandbox file creation is not permitted".to_string(),
        ),
        _ => DriveServiceError::Internal("sandbox file could not be created".to_string()),
    }
}

fn map_file_update_error(error: io::Error) -> DriveServiceError {
    match error.kind() {
        io::ErrorKind::NotFound => {
            DriveServiceError::NotFound("sandbox file was not found".to_string())
        }
        io::ErrorKind::PermissionDenied => {
            DriveServiceError::PermissionDenied("sandbox file update is not permitted".to_string())
        }
        _ => DriveServiceError::Internal("sandbox file could not be updated".to_string()),
    }
}

fn map_move_error(error: io::Error) -> DriveServiceError {
    match error.kind() {
        io::ErrorKind::AlreadyExists => {
            DriveServiceError::Conflict("sandbox destination entry already exists".to_string())
        }
        io::ErrorKind::NotFound => DriveServiceError::NotFound(
            "sandbox move source or destination was not found".to_string(),
        ),
        io::ErrorKind::PermissionDenied => {
            DriveServiceError::PermissionDenied("sandbox entry move is not permitted".to_string())
        }
        _ => DriveServiceError::Internal("sandbox entry could not be moved".to_string()),
    }
}

fn map_delete_error(error: io::Error) -> DriveServiceError {
    let directory_not_empty =
        error.kind() == io::ErrorKind::DirectoryNotEmpty || error.raw_os_error() == Some(145);
    if directory_not_empty {
        return DriveServiceError::Conflict(
            "sandbox directory is not empty; recursive deletion is required".to_string(),
        );
    }
    match error.kind() {
        io::ErrorKind::NotFound => {
            DriveServiceError::NotFound("sandbox entry was not found".to_string())
        }
        io::ErrorKind::PermissionDenied => DriveServiceError::PermissionDenied(
            "sandbox entry deletion is not permitted".to_string(),
        ),
        _ => DriveServiceError::Internal("sandbox entry could not be deleted".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::Path;

    fn list_input(
        root: &Path,
        parent: &str,
        page_size: usize,
        cursor: Option<String>,
    ) -> LocalListInput {
        LocalListInput {
            sandbox_id: "sandbox-1".to_string(),
            root_entry_id: "root-entry-1".to_string(),
            private_root_ref: root.to_string_lossy().into_owned(),
            parent: SandboxLogicalPath::parse(parent).expect("logical path"),
            page: SandboxDirectoryPageRequest::new(page_size, cursor).expect("page"),
        }
    }

    fn entry_input(root: &Path, logical_path: &str) -> LocalEntryInput {
        LocalEntryInput {
            sandbox_id: "sandbox-1".to_string(),
            root_entry_id: "root-entry-1".to_string(),
            private_root_ref: root.to_string_lossy().into_owned(),
            logical_path: SandboxLogicalPath::parse(logical_path).expect("logical path"),
        }
    }

    #[test]
    fn lists_children_with_stable_bounded_cursor_pages() {
        let temp = tempfile::tempdir().expect("tempdir");
        for name in ["zeta", "alpha", "middle"] {
            std::fs::create_dir(temp.path().join(name)).expect("directory");
        }
        std::fs::write(temp.path().join(".sdkwork-tmp-orphan"), b"internal")
            .expect("internal temp fixture");

        let first = list_children_sync(list_input(temp.path(), "", 2, None)).expect("first page");
        assert_eq!(
            first
                .items
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "middle"]
        );
        assert!(first.has_more);

        let second = list_children_sync(list_input(temp.path(), "", 2, first.next_cursor))
            .expect("second page");
        assert_eq!(
            second
                .items
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            vec!["zeta"]
        );
        assert!(!second.has_more);
        assert!(second.next_cursor.is_none());
    }

    #[test]
    fn lists_every_supported_entry_across_large_directory_pages() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut expected = BTreeSet::new();
        for index in 0..225 {
            let name = format!("entry-{index:03}");
            if index % 2 == 0 {
                std::fs::create_dir(temp.path().join(&name)).expect("directory");
            } else {
                std::fs::write(temp.path().join(&name), b"file").expect("file");
            }
            expected.insert(name);
        }
        for name in [
            ".git",
            "$Recycle.Bin",
            "folder with spaces",
            "\u{8d44}\u{6599}",
        ] {
            std::fs::create_dir(temp.path().join(name)).expect("special directory");
            expected.insert(name.to_string());
        }
        for name in [".hidden", "LICENSE", "caf\u{e9}.txt"] {
            std::fs::write(temp.path().join(name), b"file").expect("special file");
            expected.insert(name.to_string());
        }

        let mut actual = BTreeSet::new();
        let mut cursor = None;
        loop {
            let page = list_children_sync(list_input(temp.path(), "", 37, cursor))
                .expect("directory page");
            for entry in page.items {
                assert!(actual.insert(entry.name), "entry must appear exactly once");
            }
            if !page.has_more {
                assert!(page.next_cursor.is_none());
                break;
            }
            cursor = Some(page.next_cursor.expect("next cursor"));
        }

        assert_eq!(actual, expected);
    }

    #[test]
    fn cursor_is_bound_to_its_sandbox_directory() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(temp.path().join("alpha")).expect("alpha");
        std::fs::create_dir(temp.path().join("child")).expect("child");
        let first = list_children_sync(list_input(temp.path(), "", 1, None)).expect("page");
        let error = list_children_sync(list_input(temp.path(), "child", 1, first.next_cursor))
            .expect_err("cross-directory cursor must fail");
        assert!(matches!(error, DriveServiceError::Validation(_)));
    }

    #[test]
    fn cursor_rejects_a_directory_snapshot_changed_between_pages() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(temp.path().join("alpha")).expect("alpha");
        std::fs::create_dir(temp.path().join("zeta")).expect("zeta");
        let first = list_children_sync(list_input(temp.path(), "", 1, None)).expect("page");
        std::fs::create_dir(temp.path().join("middle")).expect("middle");

        let error = list_children_sync(list_input(temp.path(), "", 1, first.next_cursor))
            .expect_err("changed directory snapshot must invalidate cursor");
        assert!(matches!(error, DriveServiceError::Conflict(_)));
    }

    #[test]
    fn creates_a_directory_and_returns_only_logical_identity() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(temp.path().join("projects")).expect("projects");
        let entry = create_directory_sync(LocalCreateInput {
            sandbox_id: "sandbox-1".to_string(),
            root_entry_id: "root-entry-1".to_string(),
            private_root_ref: temp.path().to_string_lossy().into_owned(),
            parent: SandboxLogicalPath::parse("projects").expect("parent"),
            name: SandboxEntryName::parse("demo").expect("name"),
        })
        .expect("created directory");
        assert_eq!(entry.logical_path, "projects/demo");
        assert_eq!(entry.parent_logical_path, "projects");
        assert!(!format!("{entry:?}").contains(&temp.path().to_string_lossy().to_string()));
        assert!(temp.path().join("projects/demo").is_dir());
    }

    #[test]
    fn reads_an_existing_directory_for_idempotent_recovery() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(temp.path().join("recovered")).expect("recovered");
        let entry = get_directory_sync(LocalCreateInput {
            sandbox_id: "sandbox-1".to_string(),
            root_entry_id: "root-entry-1".to_string(),
            private_root_ref: temp.path().to_string_lossy().into_owned(),
            parent: SandboxLogicalPath::root(),
            name: SandboxEntryName::parse("recovered").expect("name"),
        })
        .expect("recovery read")
        .expect("directory");
        assert_eq!(entry.logical_path, "recovered");
    }

    #[test]
    fn atomically_creates_and_reads_binary_file_content() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(temp.path().join("projects")).expect("projects");
        let input = entry_input(temp.path(), "projects/data.bin");
        let bytes = vec![0, 159, 146, 150, 255];
        let entry = create_file_sync(LocalCreateFileInput {
            entry: input.clone(),
            bytes: bytes.clone(),
        })
        .expect("create file");
        assert_eq!(entry.kind, SandboxEntryKind::File);
        assert_eq!(entry.logical_path, "projects/data.bin");
        assert_eq!(
            std::fs::read(temp.path().join("projects/data.bin")).unwrap(),
            bytes
        );
        let content = read_file_sync(input, 32).expect("read file");
        assert_eq!(content.bytes, bytes);
        assert!(std::fs::read_dir(temp.path().join("projects"))
            .unwrap()
            .all(|item| !item
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".sdkwork-tmp-")));
    }

    #[test]
    fn update_is_atomic_and_rejects_a_stale_revision_without_overwrite() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(temp.path().join("main.rs"), b"old").expect("fixture");
        let input = entry_input(temp.path(), "main.rs");
        let current = get_entry_sync(input.clone())
            .expect("entry")
            .expect("existing");
        let stale = update_file_sync(LocalUpdateFileInput {
            entry: input.clone(),
            expected_revision: "stale-revision".to_string(),
            bytes: b"wrong".to_vec(),
        })
        .expect_err("stale revision");
        assert!(matches!(stale, DriveServiceError::Conflict(_)));
        assert_eq!(std::fs::read(temp.path().join("main.rs")).unwrap(), b"old");

        let updated = update_file_sync(LocalUpdateFileInput {
            entry: input,
            expected_revision: current.revision,
            bytes: b"new-content".to_vec(),
        })
        .expect("update file");
        assert_eq!(
            std::fs::read(temp.path().join("main.rs")).unwrap(),
            b"new-content"
        );
        assert_ne!(updated.revision, "stale-revision");
    }

    #[test]
    fn moves_an_entry_without_exposing_or_reusing_its_path_identity() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(temp.path().join("source")).expect("source");
        std::fs::create_dir(temp.path().join("destination")).expect("destination");
        std::fs::write(temp.path().join("source/file.txt"), b"content").expect("file");
        let input = entry_input(temp.path(), "source/file.txt");
        let before = get_entry_sync(input.clone())
            .expect("entry")
            .expect("existing");
        let moved = move_entry_sync(LocalMoveInput {
            entry: input,
            destination_parent: SandboxLogicalPath::parse("destination").unwrap(),
            destination_name: SandboxEntryName::parse("renamed.txt").unwrap(),
            expected_revision: before.revision,
        })
        .expect("move entry");
        assert_eq!(moved.logical_path, "destination/renamed.txt");
        assert_ne!(moved.id, before.id);
        assert!(!temp.path().join("source/file.txt").exists());
        assert_eq!(
            std::fs::read(temp.path().join("destination/renamed.txt")).unwrap(),
            b"content"
        );
    }

    #[test]
    fn directory_deletion_requires_explicit_recursive_policy() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(temp.path().join("tree")).expect("tree");
        std::fs::write(temp.path().join("tree/leaf.txt"), b"leaf").expect("leaf");
        let input = entry_input(temp.path(), "tree");
        let entry = get_entry_sync(input.clone())
            .expect("entry")
            .expect("existing");
        let error = delete_entry_sync(LocalDeleteInput {
            entry: input.clone(),
            expected_revision: entry.revision.clone(),
            recursive: false,
        })
        .expect_err("non-recursive delete must reject non-empty directory");
        assert!(matches!(error, DriveServiceError::Conflict(_)));
        assert!(temp.path().join("tree/leaf.txt").exists());
        delete_entry_sync(LocalDeleteInput {
            entry: input,
            expected_revision: entry.revision,
            recursive: true,
        })
        .expect("recursive delete");
        assert!(!temp.path().join("tree").exists());
    }

    #[test]
    fn file_reads_are_bounded_even_when_metadata_exceeds_the_limit() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(temp.path().join("large.bin"), vec![7; 33]).expect("large file");
        let error =
            read_file_sync(entry_input(temp.path(), "large.bin"), 32).expect_err("oversized read");
        assert!(matches!(error, DriveServiceError::Validation(_)));
    }

    #[cfg(unix)]
    #[test]
    fn excludes_symlinks_and_rejects_escape_navigation() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().expect("root");
        let outside = tempfile::tempdir().expect("outside");
        std::fs::create_dir(outside.path().join("secret")).expect("secret");
        symlink(outside.path(), root.path().join("escape")).expect("symlink");

        let page = list_children_sync(list_input(root.path(), "", 20, None)).expect("root page");
        assert!(page.items.iter().all(|entry| entry.name != "escape"));
        assert!(list_children_sync(list_input(root.path(), "escape", 20, None)).is_err());
        let write_error = create_file_sync(LocalCreateFileInput {
            entry: entry_input(root.path(), "escape/blocked.txt"),
            bytes: b"blocked".to_vec(),
        })
        .expect_err("symlink parent write must be rejected");
        assert!(matches!(
            write_error,
            DriveServiceError::PermissionDenied(_)
        ));
        assert!(!outside.path().join("blocked.txt").exists());
    }

    #[cfg(unix)]
    #[test]
    fn unrepresentable_entry_does_not_block_portable_siblings() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let root = tempfile::tempdir().expect("root");
        std::fs::create_dir(root.path().join("portable")).expect("portable directory");
        let non_unicode = OsString::from_vec(b"non-unicode-\xff".to_vec());
        std::fs::create_dir(root.path().join(non_unicode)).expect("non-Unicode directory");

        let page = list_children_sync(list_input(root.path(), "", 20, None)).expect("root page");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].name, "portable");
    }

    #[cfg(windows)]
    #[test]
    fn excludes_symlinks_and_rejects_escape_navigation() {
        use std::os::windows::fs::symlink_dir;

        let root = tempfile::tempdir().expect("root");
        let outside = tempfile::tempdir().expect("outside");
        std::fs::create_dir(outside.path().join("secret")).expect("secret");
        if symlink_dir(outside.path(), root.path().join("escape")).is_err() {
            return;
        }

        let page = list_children_sync(list_input(root.path(), "", 20, None)).expect("root page");
        assert!(page.items.iter().all(|entry| entry.name != "escape"));
        assert!(list_children_sync(list_input(root.path(), "escape", 20, None)).is_err());
        let write_error = create_file_sync(LocalCreateFileInput {
            entry: entry_input(root.path(), "escape/blocked.txt"),
            bytes: b"blocked".to_vec(),
        })
        .expect_err("symlink parent write must be rejected");
        assert!(matches!(
            write_error,
            DriveServiceError::PermissionDenied(_)
        ));
        assert!(!outside.path().join("blocked.txt").exists());
    }
}
