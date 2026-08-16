use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_utils_rust::MAX_LIST_PAGE_SIZE;
use sqlx::PgPool;

use crate::infrastructure::sql::workspace_store::SqlDriveWorkspaceStore;
use crate::ports::workspace_store::{
    DriveWorkspaceNodeRecord, DriveWorkspaceStore, NewDriveWorkspaceNodeRecord,
    NewDriveWorkspaceObjectRecord,
};
use crate::DriveServiceError;

const SDKWORK_SNOWFLAKE_EPOCH_MS: u64 = 1_609_459_200_000;
const SDKWORK_DRIVE_WORKER_ID: u64 = 17;
static LAST_WORKSPACE_SNOWFLAKE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveWorkspaceNodeKind {
    File,
    Folder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveWorkspaceObjectRef {
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnsureDriveWorkspaceNode {
    Folder {
        logical_path: String,
    },
    File {
        logical_path: String,
        object_ref: DriveWorkspaceObjectRef,
    },
}

impl EnsureDriveWorkspaceNode {
    pub fn folder(logical_path: impl Into<String>) -> Self {
        Self::Folder {
            logical_path: logical_path.into(),
        }
    }

    pub fn file(logical_path: impl Into<String>, object_ref: DriveWorkspaceObjectRef) -> Self {
        Self::File {
            logical_path: logical_path.into(),
            object_ref,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnsureDriveWorkspaceNodesCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub operator_id: String,
    pub nodes: Vec<EnsureDriveWorkspaceNode>,
}

#[derive(Debug, Clone)]
pub struct ResolveDriveWorkspacePathCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub logical_path: String,
}

#[derive(Debug, Clone)]
pub struct GetDriveWorkspaceNodeCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: String,
}

#[derive(Debug, Clone)]
pub struct ListDriveWorkspaceChildrenCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub parent_node_id: Option<String>,
    pub offset: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveWorkspaceNode {
    pub id: String,
    pub parent_node_id: Option<String>,
    pub name: String,
    pub path: String,
    pub kind: DriveWorkspaceNodeKind,
    pub updated_at: String,
    pub content_type: Option<String>,
    pub content_length: Option<i64>,
    pub children_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveWorkspaceChildrenPage {
    pub nodes: Vec<DriveWorkspaceNode>,
    pub next_offset: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct SqlDriveWorkspaceService {
    service: DriveWorkspaceService<SqlDriveWorkspaceStore>,
    pool: PgPool,
}

impl SqlDriveWorkspaceService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            service: DriveWorkspaceService::new(SqlDriveWorkspaceStore::new(pool.clone())),
            pool,
        }
    }

    pub async fn ensure_nodes(
        &self,
        command: EnsureDriveWorkspaceNodesCommand,
    ) -> Result<(), DriveServiceError> {
        self.service.ensure_nodes(command).await
    }

    pub async fn resolve_path(
        &self,
        command: ResolveDriveWorkspacePathCommand,
    ) -> Result<Option<DriveWorkspaceNode>, DriveServiceError> {
        self.service.resolve_path(command).await
    }

    pub async fn get_node(
        &self,
        command: GetDriveWorkspaceNodeCommand,
    ) -> Result<Option<DriveWorkspaceNode>, DriveServiceError> {
        self.service.get_node(command).await
    }

    pub async fn list_children(
        &self,
        command: ListDriveWorkspaceChildrenCommand,
    ) -> Result<DriveWorkspaceChildrenPage, DriveServiceError> {
        self.service.list_children(command).await
    }

    pub async fn find_latest_active_storage_object_by_node(
        &self,
        tenant_id: &str,
        node_id: &str,
    ) -> Result<Option<crate::ports::storage_object_store::DriveStorageObject>, DriveServiceError>
    {
        use crate::infrastructure::sql::storage_object_store::SqlStorageObjectStore;
        use crate::ports::storage_object_store::DriveStorageObjectStore;

        SqlStorageObjectStore::new(self.pool.clone())
            .find_latest_active_by_node(tenant_id, node_id)
            .await
    }
}

#[derive(Debug, Clone)]
pub struct DriveWorkspaceService<S>
where
    S: DriveWorkspaceStore,
{
    store: S,
}

struct WorkspaceChildNodeSpec<'a> {
    parent_node_id: Option<&'a str>,
    node_name: &'a str,
    node_type: &'a str,
    content_state: &'a str,
}

impl<S> DriveWorkspaceService<S>
where
    S: DriveWorkspaceStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn ensure_nodes(
        &self,
        command: EnsureDriveWorkspaceNodesCommand,
    ) -> Result<(), DriveServiceError> {
        let tenant_id = require_non_empty(command.tenant_id, "tenant_id")?;
        let space_id = require_non_empty(command.space_id, "space_id")?;
        let operator_id = require_non_empty(command.operator_id, "operator_id")?;

        for node in command.nodes {
            match node {
                EnsureDriveWorkspaceNode::Folder { logical_path } => {
                    let segments = parse_logical_path(&logical_path)?;
                    self.ensure_folder_path(&tenant_id, &space_id, &operator_id, &segments)
                        .await?;
                }
                EnsureDriveWorkspaceNode::File {
                    logical_path,
                    object_ref,
                } => {
                    let segments = parse_logical_path(&logical_path)?;
                    if segments.is_empty() {
                        return Err(DriveServiceError::Validation(
                            "file logical_path is required".to_string(),
                        ));
                    }
                    let (parent_segments, file_name) = segments.split_at(segments.len() - 1);
                    let Some(file_name) = file_name.first() else {
                        return Err(DriveServiceError::Validation(
                            "file logical_path is required".to_string(),
                        ));
                    };
                    let parent_id = self
                        .ensure_folder_path(&tenant_id, &space_id, &operator_id, parent_segments)
                        .await?;
                    let file = self
                        .ensure_child_node(
                            &tenant_id,
                            &space_id,
                            &operator_id,
                            WorkspaceChildNodeSpec {
                                parent_node_id: parent_id.as_deref(),
                                node_name: file_name,
                                node_type: "file",
                                content_state: "uploading",
                            },
                        )
                        .await?;
                    self.ensure_object_ref(&tenant_id, &file.id, object_ref, &operator_id)
                        .await?;
                    self.store
                        .mark_node_content_ready(&tenant_id, &file.id, &operator_id)
                        .await?;
                }
            }
        }

        Ok(())
    }

    pub async fn resolve_path(
        &self,
        command: ResolveDriveWorkspacePathCommand,
    ) -> Result<Option<DriveWorkspaceNode>, DriveServiceError> {
        let tenant_id = require_non_empty(command.tenant_id, "tenant_id")?;
        let space_id = require_non_empty(command.space_id, "space_id")?;
        let segments = parse_logical_path(&command.logical_path)?;
        let mut parent_id = None::<String>;
        let mut records = Vec::new();

        for segment in segments {
            let node = self
                .store
                .find_child_node(&tenant_id, &space_id, parent_id.as_deref(), &segment)
                .await?;
            let Some(node) = node else {
                return Ok(None);
            };
            parent_id = Some(node.id.clone());
            records.push(node);
        }

        let Some(last) = records.last() else {
            return Ok(None);
        };
        Ok(Some(workspace_node_from_record(
            last,
            &records
                .iter()
                .map(|record| record.node_name.clone())
                .collect::<Vec<_>>()
                .join("/"),
        )?))
    }

    pub async fn get_node(
        &self,
        command: GetDriveWorkspaceNodeCommand,
    ) -> Result<Option<DriveWorkspaceNode>, DriveServiceError> {
        let tenant_id = require_non_empty(command.tenant_id, "tenant_id")?;
        let space_id = require_non_empty(command.space_id, "space_id")?;
        let node_id = require_non_empty(command.node_id, "node_id")?;

        let Some(record) = self
            .store
            .find_node(&tenant_id, &space_id, &node_id)
            .await?
        else {
            return Ok(None);
        };
        let path = self
            .resolve_record_path(&tenant_id, &space_id, record.clone())
            .await?;

        Ok(Some(workspace_node_from_record(&record, &path)?))
    }

    pub async fn list_children(
        &self,
        command: ListDriveWorkspaceChildrenCommand,
    ) -> Result<DriveWorkspaceChildrenPage, DriveServiceError> {
        let tenant_id = require_non_empty(command.tenant_id, "tenant_id")?;
        let space_id = require_non_empty(command.space_id, "space_id")?;
        if command.offset < 0 {
            return Err(DriveServiceError::Validation(
                "offset must be greater than or equal to 0".to_string(),
            ));
        }
        if command.page_size <= 0 || command.page_size > i64::from(MAX_LIST_PAGE_SIZE) {
            return Err(DriveServiceError::Validation(format!(
                "page_size must be between 1 and {MAX_LIST_PAGE_SIZE}"
            )));
        }

        let parent_path = match command.parent_node_id.as_deref() {
            Some(parent_node_id) => {
                let Some(parent) = self
                    .store
                    .find_node(&tenant_id, &space_id, parent_node_id)
                    .await?
                else {
                    return Ok(DriveWorkspaceChildrenPage {
                        nodes: Vec::new(),
                        next_offset: None,
                    });
                };
                self.resolve_record_path(&tenant_id, &space_id, parent)
                    .await?
            }
            None => String::new(),
        };

        let mut nodes = self
            .store
            .list_children(
                &tenant_id,
                &space_id,
                command.parent_node_id.as_deref(),
                command.page_size + 1,
                command.offset,
            )
            .await?
            .into_iter()
            .map(|child| {
                let child_path = join_path(&parent_path, &child.node_name);
                workspace_node_from_record(&child, &child_path)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let next_offset = if nodes.len() > command.page_size as usize {
            nodes.pop();
            Some(command.offset + command.page_size)
        } else {
            None
        };
        Ok(DriveWorkspaceChildrenPage { nodes, next_offset })
    }

    async fn ensure_folder_path(
        &self,
        tenant_id: &str,
        space_id: &str,
        operator_id: &str,
        segments: &[String],
    ) -> Result<Option<String>, DriveServiceError> {
        let mut parent_id = None::<String>;
        for segment in segments {
            let folder = self
                .ensure_child_node(
                    tenant_id,
                    space_id,
                    operator_id,
                    WorkspaceChildNodeSpec {
                        parent_node_id: parent_id.as_deref(),
                        node_name: segment,
                        node_type: "folder",
                        content_state: "ready",
                    },
                )
                .await?;
            parent_id = Some(folder.id);
        }
        Ok(parent_id)
    }

    async fn ensure_child_node(
        &self,
        tenant_id: &str,
        space_id: &str,
        operator_id: &str,
        spec: WorkspaceChildNodeSpec<'_>,
    ) -> Result<DriveWorkspaceNodeRecord, DriveServiceError> {
        let record = NewDriveWorkspaceNodeRecord {
            id: next_workspace_id("wsn"),
            tenant_id: tenant_id.to_string(),
            space_id: space_id.to_string(),
            parent_node_id: spec.parent_node_id.map(ToString::to_string),
            node_type: spec.node_type.to_string(),
            node_name: spec.node_name.to_string(),
            content_state: spec.content_state.to_string(),
            operator_id: operator_id.to_string(),
        };
        let existing = self.store.ensure_node(record).await?;
        if existing.node_type != spec.node_type {
            return Err(DriveServiceError::Conflict(format!(
                "workspace path segment already exists with different node_type: {}",
                spec.node_name
            )));
        }
        Ok(existing)
    }

    async fn ensure_object_ref(
        &self,
        tenant_id: &str,
        node_id: &str,
        object_ref: DriveWorkspaceObjectRef,
        operator_id: &str,
    ) -> Result<(), DriveServiceError> {
        let storage_provider_id =
            require_non_empty(object_ref.storage_provider_id, "storage_provider_id")?;
        let bucket = require_non_empty(object_ref.bucket, "bucket")?;
        let object_key = require_valid_object_key(object_ref.object_key)?;
        let content_type = normalize_content_type(object_ref.content_type)?;
        if object_ref.content_length < 0 {
            return Err(DriveServiceError::Validation(
                "content_length must be greater than or equal to 0".to_string(),
            ));
        }
        let checksum_sha256_hex = normalize_sha256_checksum(object_ref.checksum_sha256_hex)?;
        self.store
            .ensure_object_ref(NewDriveWorkspaceObjectRecord {
                id: next_workspace_id("wso"),
                tenant_id: tenant_id.to_string(),
                node_id: node_id.to_string(),
                storage_provider_id,
                bucket,
                object_key,
                content_type,
                content_length: object_ref.content_length,
                checksum_sha256_hex,
                operator_id: operator_id.to_string(),
            })
            .await
    }

    async fn resolve_record_path(
        &self,
        tenant_id: &str,
        space_id: &str,
        mut record: DriveWorkspaceNodeRecord,
    ) -> Result<String, DriveServiceError> {
        let mut names = vec![record.node_name.clone()];
        let mut guard = 0;
        while let Some(parent_id) = record.parent_node_id.as_deref() {
            guard += 1;
            if guard > 128 {
                return Err(DriveServiceError::Conflict(
                    "workspace parent chain is too deep".to_string(),
                ));
            }
            let Some(parent) = self.store.find_node(tenant_id, space_id, parent_id).await? else {
                return Err(DriveServiceError::Internal(format!(
                    "workspace parent node is missing: {parent_id}"
                )));
            };
            names.push(parent.node_name.clone());
            record = parent;
        }
        names.reverse();
        Ok(names.join("/"))
    }
}

fn workspace_node_from_record(
    record: &DriveWorkspaceNodeRecord,
    path: &str,
) -> Result<DriveWorkspaceNode, DriveServiceError> {
    let kind = match record.node_type.as_str() {
        "folder" => DriveWorkspaceNodeKind::Folder,
        "file" => DriveWorkspaceNodeKind::File,
        other => {
            return Err(DriveServiceError::Internal(format!(
                "workspace node_type is unsupported: {other}"
            )))
        }
    };
    Ok(DriveWorkspaceNode {
        id: record.id.clone(),
        parent_node_id: record.parent_node_id.clone(),
        name: record.node_name.clone(),
        path: path.to_string(),
        kind,
        updated_at: record.updated_at.clone(),
        content_type: record.content_type.clone(),
        content_length: record.content_length,
        children_count: record.children_count,
    })
}

fn parse_logical_path(path: &str) -> Result<Vec<String>, DriveServiceError> {
    let path = path.trim();
    if path.is_empty() {
        return Err(DriveServiceError::Validation(
            "logical_path is required".to_string(),
        ));
    }
    if path.starts_with('/') || path.ends_with('/') || path.contains('\\') {
        return Err(DriveServiceError::Validation(
            "logical_path must be a relative slash-delimited path".to_string(),
        ));
    }

    let mut segments = Vec::new();
    for segment in path.split('/') {
        if segment.is_empty()
            || segment == "."
            || segment == ".."
            || segment != segment.trim()
            || segment.contains('\0')
        {
            return Err(DriveServiceError::Validation(format!(
                "logical_path contains invalid segment: {segment}"
            )));
        }
        if segment.len() > 255 {
            return Err(DriveServiceError::Validation(
                "logical_path segment length must be <= 255".to_string(),
            ));
        }
        segments.push(segment.to_string());
    }
    Ok(segments)
}

fn require_non_empty(value: String, field_name: &str) -> Result<String, DriveServiceError> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} is required"
        )));
    }
    Ok(trimmed)
}

fn require_valid_object_key(value: String) -> Result<String, DriveServiceError> {
    let value = require_non_empty(value, "object_key")?;
    if value.starts_with('/')
        || value.ends_with('/')
        || value.contains("//")
        || value
            .split('/')
            .any(|segment| segment == "." || segment == "..")
        || value.contains('\0')
    {
        return Err(DriveServiceError::Validation(
            "object_key is invalid".to_string(),
        ));
    }
    Ok(value)
}

fn normalize_content_type(value: String) -> Result<String, DriveServiceError> {
    let content_type = value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if content_type.len() < 3
        || content_type.len() > 255
        || content_type.matches('/').count() != 1
        || content_type.chars().any(char::is_whitespace)
    {
        return Err(DriveServiceError::Validation(
            "content_type must be a valid media type".to_string(),
        ));
    }
    Ok(content_type)
}

fn normalize_sha256_checksum(value: String) -> Result<String, DriveServiceError> {
    let checksum = value.trim().to_ascii_lowercase();
    let hex = checksum.strip_prefix("sha256:").unwrap_or(&checksum);
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(DriveServiceError::Validation(
            "checksum_sha256_hex must be sha256:<64 lowercase hex> or 64 hex".to_string(),
        ));
    }
    Ok(format!("sha256:{hex}"))
}

fn join_path(parent_path: &str, node_name: &str) -> String {
    if parent_path.is_empty() {
        node_name.to_string()
    } else {
        format!("{parent_path}/{node_name}")
    }
}

fn next_workspace_id(prefix: &str) -> String {
    format!("{prefix}_{}", next_snowflake_id())
}

fn next_snowflake_id() -> u64 {
    next_snowflake_id_from_time(SystemTime::now())
}

fn next_snowflake_id_from_time(now: SystemTime) -> u64 {
    let now_ms = now
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0);
    let timestamp_part = now_ms
        .saturating_sub(SDKWORK_SNOWFLAKE_EPOCH_MS)
        .min((1_u64 << 41) - 1);
    let base = (timestamp_part << 22) | (SDKWORK_DRIVE_WORKER_ID << 12);
    loop {
        let previous = LAST_WORKSPACE_SNOWFLAKE_ID.load(Ordering::Relaxed);
        let candidate = if base > previous { base } else { previous + 1 };
        if LAST_WORKSPACE_SNOWFLAKE_ID
            .compare_exchange(previous, candidate, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            return candidate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{next_snowflake_id_from_time, UNIX_EPOCH};
    use std::time::Duration;

    #[test]
    fn snowflake_generation_tolerates_clock_before_unix_epoch() {
        let before_epoch = UNIX_EPOCH - Duration::from_millis(1);
        let id = next_snowflake_id_from_time(before_epoch);

        assert!(id > 0);
    }
}
