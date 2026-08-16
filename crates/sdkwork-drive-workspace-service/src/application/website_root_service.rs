use crate::domain::website_root::{
    DriveWebsiteContentMode, DriveWebsiteRoot, DriveWebsiteSourceRootMode,
};
use crate::infrastructure::sql::website_root_store::SqlWebsiteRootStore;
use crate::ports::website_root_store::{
    CreateDriveWebsiteRoot, CreateDriveWebsiteRootResult, DriveWebsiteRootStore,
};
use crate::DriveServiceError;

const MAX_WEBSITE_ROOT_LIST_FETCH_LIMIT: i64 = sdkwork_utils_rust::MAX_LIST_PAGE_SIZE as i64 + 1;

#[derive(Debug, Clone)]
pub struct CreateWebsiteRootCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub root_key: String,
    pub display_name: String,
    pub source_root_mode: DriveWebsiteSourceRootMode,
    pub selected_folder_node_id: Option<String>,
    pub content_mode: DriveWebsiteContentMode,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct ListWebsiteRootsCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub offset: i64,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct GetWebsiteRootCommand {
    pub tenant_id: String,
    pub root_uuid: String,
}

#[derive(Debug, Clone)]
pub struct DriveWebsiteRootService<S>
where
    S: DriveWebsiteRootStore,
{
    store: S,
}

impl<S> DriveWebsiteRootService<S>
where
    S: DriveWebsiteRootStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn create_root(
        &self,
        command: CreateWebsiteRootCommand,
    ) -> Result<CreateDriveWebsiteRootResult, DriveServiceError> {
        let tenant_id = require_text(command.tenant_id, "tenant_id", 64)?;
        let space_id = require_text(command.space_id, "space_id", 64)?;
        let root_key = require_key(command.root_key, "root_key", 128)?;
        let display_name = require_text(command.display_name, "display_name", 255)?;
        let operator_id = require_text(command.operator_id, "operator_id", 128)?;
        let selected_folder_node_id = match command.source_root_mode {
            DriveWebsiteSourceRootMode::SpaceRoot => {
                if command.selected_folder_node_id.is_some() {
                    return Err(DriveServiceError::Validation(
                        "SPACE_ROOT must not include selected_folder_node_id".to_string(),
                    ));
                }
                None
            }
            DriveWebsiteSourceRootMode::Folder => Some(require_text(
                command.selected_folder_node_id.ok_or_else(|| {
                    DriveServiceError::Validation(
                        "FOLDER requires selected_folder_node_id".to_string(),
                    )
                })?,
                "selected_folder_node_id",
                64,
            )?),
        };

        self.store
            .create_or_get(&CreateDriveWebsiteRoot {
                tenant_id,
                space_id,
                root_key,
                display_name,
                source_root_mode: command.source_root_mode,
                selected_folder_node_id,
                content_mode: command.content_mode,
                operator_id,
            })
            .await
    }

    pub async fn get_root(
        &self,
        command: GetWebsiteRootCommand,
    ) -> Result<DriveWebsiteRoot, DriveServiceError> {
        self.store
            .get_by_uuid(
                &require_text(command.tenant_id, "tenant_id", 64)?,
                &require_text(command.root_uuid, "root_uuid", 64)?,
            )
            .await
    }

    pub async fn list_roots(
        &self,
        command: ListWebsiteRootsCommand,
    ) -> Result<Vec<DriveWebsiteRoot>, DriveServiceError> {
        if command.offset < 0 {
            return Err(DriveServiceError::Validation(
                "offset must not be negative".to_string(),
            ));
        }
        if !(1..=MAX_WEBSITE_ROOT_LIST_FETCH_LIMIT).contains(&command.limit) {
            return Err(DriveServiceError::Validation(format!(
                "limit must be between 1 and {MAX_WEBSITE_ROOT_LIST_FETCH_LIMIT}"
            )));
        }
        self.store
            .list_by_space(
                &require_text(command.tenant_id, "tenant_id", 64)?,
                &require_text(command.space_id, "space_id", 64)?,
                command.offset,
                command.limit,
            )
            .await
    }
}

pub type SqlDriveWebsiteRootService = DriveWebsiteRootService<SqlWebsiteRootStore>;

fn require_text(
    value: String,
    field_name: &str,
    max_length: usize,
) -> Result<String, DriveServiceError> {
    let value = value.trim().to_string();
    if value.is_empty() || value.len() > max_length {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} must contain between 1 and {max_length} characters"
        )));
    }
    Ok(value)
}

fn require_key(
    value: String,
    field_name: &str,
    max_length: usize,
) -> Result<String, DriveServiceError> {
    let value = require_text(value, field_name, max_length)?;
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} contains unsupported characters"
        )));
    }
    Ok(value)
}
