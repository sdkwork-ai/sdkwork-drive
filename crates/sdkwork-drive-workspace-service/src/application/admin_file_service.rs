use crate::ports::admin_file_store::{
    DriveAdminFileRecord, DriveAdminFileStore, ListAdminFilesQuery,
};
use crate::DriveServiceError;

/// Upper bound for one admin file page.
///
/// Mirrors the backend list page cap: the admin table renders server pages, so
/// the read model must never be asked to collect the whole tenant in process.
pub const MAX_ADMIN_FILE_PAGE_SIZE: i64 = 200;

const SPACE_TYPES: &[&str] = &[
    "personal",
    "team",
    "knowledge_base",
    "ai_generated",
    "git_repository",
    "deployment",
    "app_upload",
    "im",
    "rtc",
    "notary",
    "website",
    "documents",
];

const NODE_TYPES: &[&str] = &["file", "folder", "shortcut", "virtual_reference"];

const NODE_LIFECYCLE_STATUSES: &[&str] = &["active", "trashed", "deleted"];

const CONTENT_TYPE_GROUPS: &[&str] = &[
    "image", "video", "audio", "text", "document", "archive", "binary",
];

const MAX_KEYWORD_CHARS: usize = 128;

/// Filters for one admin file page.
#[derive(Debug, Clone)]
pub struct ListAdminFilesCommand {
    pub tenant_id: String,
    pub space_id: Option<String>,
    pub space_type: Option<String>,
    pub node_type: Option<String>,
    pub lifecycle_status: Option<String>,
    pub content_type_group: Option<String>,
    pub keyword: Option<String>,
    pub owner_subject_id: Option<String>,
    pub offset: i64,
    pub limit: i64,
}

/// One page of the admin file list.
#[derive(Debug, Clone)]
pub struct DriveAdminFilePage {
    pub items: Vec<DriveAdminFileRecord>,
    /// Offset of the next page; `None` when the current page is the last one.
    pub next_offset: Option<i64>,
}

/// Tenant-scoped admin file listing over the drive node metadata plane.
///
/// The service owns filter validation and the `limit + 1` look-ahead that
/// produces the continuation offset; the store owns SQL. The node workspace
/// service stays the only writer of `dr_drive_node`, so this is a read-only
/// projection and never mutates file state.
pub struct DriveAdminFileService<S>
where
    S: DriveAdminFileStore,
{
    store: S,
}

impl<S> DriveAdminFileService<S>
where
    S: DriveAdminFileStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn list_files(
        &self,
        command: ListAdminFilesCommand,
    ) -> Result<DriveAdminFilePage, DriveServiceError> {
        let tenant_id = require_non_empty(command.tenant_id, "tenant_id")?;
        if command.offset < 0 {
            return Err(DriveServiceError::Validation(
                "offset must be greater than or equal to 0".to_string(),
            ));
        }
        if command.limit <= 0 || command.limit > MAX_ADMIN_FILE_PAGE_SIZE {
            return Err(DriveServiceError::Validation(format!(
                "limit must be between 1 and {MAX_ADMIN_FILE_PAGE_SIZE}"
            )));
        }

        let space_id = normalize_optional(command.space_id);
        let space_type = validate_choice(
            normalize_optional(command.space_type),
            SPACE_TYPES,
            "spaceType is invalid",
        )?;
        let node_type = validate_choice(
            normalize_optional(command.node_type),
            NODE_TYPES,
            "nodeType is invalid",
        )?;
        let lifecycle_status = validate_choice(
            normalize_optional(command.lifecycle_status),
            NODE_LIFECYCLE_STATUSES,
            "lifecycleStatus is invalid",
        )?
        // The admin list follows the workspace default: an omitted status means
        // live files. Trashed and deleted nodes stay reachable through an
        // explicit filter instead of appearing between active files.
        .or_else(|| Some("active".to_string()));
        let content_type_group = validate_choice(
            normalize_optional(command.content_type_group),
            CONTENT_TYPE_GROUPS,
            "contentTypeGroup is invalid",
        )?;
        let owner_subject_id = normalize_optional(command.owner_subject_id);
        let keyword = normalize_optional(command.keyword);
        if let Some(keyword) = keyword.as_deref() {
            if keyword.chars().count() > MAX_KEYWORD_CHARS {
                return Err(DriveServiceError::Validation(format!(
                    "keyword must be at most {MAX_KEYWORD_CHARS} characters"
                )));
            }
        }

        let mut items = self
            .store
            .list_admin_files(&ListAdminFilesQuery {
                tenant_id,
                space_id,
                space_type,
                node_type,
                lifecycle_status,
                content_type_group,
                keyword,
                owner_subject_id,
                offset: command.offset,
                limit: command.limit + 1,
            })
            .await?;

        let next_offset = if items.len() as i64 > command.limit {
            items.pop();
            Some(command.offset + command.limit)
        } else {
            None
        };

        Ok(DriveAdminFilePage { items, next_offset })
    }
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|trimmed| !trimmed.is_empty())
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

fn validate_choice(
    value: Option<String>,
    allowed: &[&str],
    message: &str,
) -> Result<Option<String>, DriveServiceError> {
    match value {
        Some(value) if !allowed.contains(&value.as_str()) => {
            Err(DriveServiceError::Validation(message.to_string()))
        }
        other => Ok(other),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DriveAdminFilePage, DriveAdminFileService, ListAdminFilesCommand, MAX_ADMIN_FILE_PAGE_SIZE,
    };
    use crate::ports::admin_file_store::{
        DriveAdminFileRecord, DriveAdminFileStore, ListAdminFilesQuery,
    };
    use crate::DriveServiceError;

    #[derive(Clone, Default)]
    struct RecordingStore {
        queries: std::sync::Arc<std::sync::Mutex<Vec<ListAdminFilesQuery>>>,
        items: Vec<DriveAdminFileRecord>,
    }

    #[async_trait::async_trait]
    impl DriveAdminFileStore for RecordingStore {
        async fn list_admin_files(
            &self,
            query: &ListAdminFilesQuery,
        ) -> Result<Vec<DriveAdminFileRecord>, DriveServiceError> {
            self.queries
                .lock()
                .expect("recording store mutex should not be poisoned")
                .push(query.clone());
            Ok(self.items.clone())
        }
    }

    fn record(id: &str) -> DriveAdminFileRecord {
        DriveAdminFileRecord {
            id: id.to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: "space-001".to_string(),
            space_name: "Main".to_string(),
            space_type: "personal".to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-001".to_string(),
            parent_node_id: None,
            node_type: "file".to_string(),
            node_name: format!("{id}.txt"),
            file_path: format!("{id}.txt"),
            file_extension: Some("txt".to_string()),
            content_type: Some("text/plain".to_string()),
            content_type_group: Some("text".to_string()),
            content_length: 12,
            lifecycle_status: "active".to_string(),
            version: 1,
            created_by: "user-001".to_string(),
            updated_by: "user-001".to_string(),
            created_at: "2026-11-14T00:00:00Z".to_string(),
            updated_at: "2026-11-14T00:00:00Z".to_string(),
        }
    }

    fn command(limit: i64) -> ListAdminFilesCommand {
        ListAdminFilesCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: None,
            space_type: None,
            node_type: None,
            lifecycle_status: None,
            content_type_group: None,
            keyword: None,
            owner_subject_id: None,
            offset: 0,
            limit,
        }
    }

    #[tokio::test]
    async fn reports_next_offset_only_when_the_look_ahead_row_exists() {
        let store = RecordingStore {
            queries: Default::default(),
            items: vec![record("node-1"), record("node-2"), record("node-3")],
        };

        let page: DriveAdminFilePage = DriveAdminFileService::new(store.clone())
            .list_files(command(2))
            .await
            .expect("page should load");
        assert_eq!(2, page.items.len());
        assert_eq!(Some(2), page.next_offset);
        assert_eq!(
            3,
            store
                .queries
                .lock()
                .expect("queries")
                .first()
                .expect("query")
                .limit
        );
        assert_eq!(
            Some("active".to_string()),
            store
                .queries
                .lock()
                .expect("queries")
                .first()
                .expect("query")
                .lifecycle_status
        );
    }

    #[tokio::test]
    async fn rejects_unknown_filters_and_oversized_pages() {
        let service = DriveAdminFileService::new(RecordingStore::default());

        let mut invalid_node_type = command(20);
        invalid_node_type.node_type = Some("symlink".to_string());
        let error = service
            .list_files(invalid_node_type)
            .await
            .expect_err("unknown node types must be rejected");
        assert!(
            matches!(error, DriveServiceError::Validation(message) if message.contains("nodeType"))
        );

        let mut oversized_keyword = command(20);
        oversized_keyword.keyword = Some("x".repeat(129));
        let error = service
            .list_files(oversized_keyword)
            .await
            .expect_err("oversized keywords must be rejected");
        assert!(
            matches!(error, DriveServiceError::Validation(message) if message.contains("keyword"))
        );

        let error = service
            .list_files(command(MAX_ADMIN_FILE_PAGE_SIZE + 1))
            .await
            .expect_err("oversized pages must be rejected");
        assert!(
            matches!(error, DriveServiceError::Validation(message) if message.contains("limit"))
        );
    }
}
