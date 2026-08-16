use crate::domain::space::{DriveSpace, DriveSpaceType};
use crate::infrastructure::sql::space_store::SqlSpaceStore;
use crate::ports::space_store::{DriveSpaceStore, ListAccessibleSpacesQuery, NewDriveSpace};
use crate::DriveServiceError;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct CreateSpaceCommand {
    pub id: String,
    pub tenant_id: String,
    pub owner_subject_type: String,
    pub owner_subject_id: String,
    pub display_name: String,
    pub space_type: DriveSpaceType,
    pub presentation_icon: Option<String>,
    pub presentation_color: Option<String>,
    pub description: Option<String>,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct SqlDriveSpaceService {
    service: DriveSpaceService<SqlSpaceStore>,
}

impl SqlDriveSpaceService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            service: DriveSpaceService::new(SqlSpaceStore::new(pool)),
        }
    }

    pub async fn create_space(
        &self,
        command: CreateSpaceCommand,
    ) -> Result<DriveSpace, DriveServiceError> {
        self.service.create_space(command).await
    }

    pub async fn list_spaces(
        &self,
        command: ListSpacesCommand,
    ) -> Result<Vec<DriveSpace>, DriveServiceError> {
        self.service.list_spaces(command).await
    }

    pub async fn list_accessible_spaces(
        &self,
        command: ListAccessibleSpacesCommand,
    ) -> Result<Vec<DriveSpace>, DriveServiceError> {
        self.service.list_accessible_spaces(command).await
    }

    pub async fn get_space(
        &self,
        command: GetSpaceCommand,
    ) -> Result<DriveSpace, DriveServiceError> {
        self.service.get_space(command).await
    }

    pub async fn update_space(
        &self,
        command: UpdateSpaceCommand,
    ) -> Result<DriveSpace, DriveServiceError> {
        self.service.update_space(command).await
    }

    pub async fn delete_space(
        &self,
        command: DeleteSpaceCommand,
    ) -> Result<DriveSpace, DriveServiceError> {
        self.service.delete_space(command).await
    }
}

#[derive(Debug, Clone)]
pub struct GetSpaceCommand {
    pub tenant_id: String,
    pub space_id: String,
}

#[derive(Debug, Clone)]
pub struct UpdateSpaceCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub display_name: Option<String>,
    pub presentation_icon: Option<String>,
    pub presentation_color: Option<String>,
    pub description: Option<String>,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteSpaceCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct DriveSpaceService<S>
where
    S: DriveSpaceStore,
{
    store: S,
}

impl<S> DriveSpaceService<S>
where
    S: DriveSpaceStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn create_space(
        &self,
        command: CreateSpaceCommand,
    ) -> Result<DriveSpace, DriveServiceError> {
        let id = require_non_empty(command.id, "space id")?;
        let tenant_id = require_non_empty(command.tenant_id, "tenant_id")?;
        let owner_subject_type = require_owner_subject_type(command.owner_subject_type)?;
        let owner_subject_id = require_non_empty(command.owner_subject_id, "owner_subject_id")?;
        let display_name = require_non_empty(command.display_name, "display_name")?;
        let operator_id = require_non_empty(command.operator_id, "operator_id")?;
        if command.space_type.requires_user_owner() && owner_subject_type != "user" {
            return Err(DriveServiceError::Validation(format!(
                "{} must be owned by a user",
                command.space_type.display_label()
            )));
        }

        let new_space = NewDriveSpace {
            id,
            tenant_id,
            owner_subject_type,
            owner_subject_id,
            display_name,
            space_type: command.space_type.as_str().to_string(),
            lifecycle_status: "active".to_string(),
            presentation_icon: normalize_optional_text(command.presentation_icon),
            presentation_color: normalize_optional_text(command.presentation_color),
            description: normalize_optional_text(command.description),
            created_by: operator_id.clone(),
            updated_by: operator_id,
        };
        if command.space_type == DriveSpaceType::Website {
            self.store.insert_website_space(&new_space).await
        } else {
            self.store.insert_space(&new_space).await
        }
    }

    pub async fn list_spaces(
        &self,
        command: ListSpacesCommand,
    ) -> Result<Vec<DriveSpace>, DriveServiceError> {
        if command.tenant_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "tenant_id is required".to_string(),
            ));
        }
        let owner_type = command.owner_subject_type.as_deref();
        let owner_id = command.owner_subject_id.as_deref();
        if (owner_type.is_some() && owner_id.is_none())
            || (owner_type.is_none() && owner_id.is_some())
        {
            return Err(DriveServiceError::Validation(
                "owner_subject_type and owner_subject_id must be provided together".to_string(),
            ));
        }

        let space_type = validate_optional_space_type(command.space_type.as_deref())?;

        self.store
            .list_spaces(
                command.tenant_id.trim(),
                owner_type,
                owner_id,
                space_type.as_deref(),
                command.offset,
                command.limit,
            )
            .await
    }

    pub async fn list_accessible_spaces(
        &self,
        command: ListAccessibleSpacesCommand,
    ) -> Result<Vec<DriveSpace>, DriveServiceError> {
        let tenant_id = require_non_empty(command.tenant_id, "tenant_id")?;
        let viewer_subject_type =
            require_non_empty(command.viewer_subject_type, "viewer_subject_type")?;
        let viewer_subject_id = require_non_empty(command.viewer_subject_id, "viewer_subject_id")?;
        let owner_type = command.owner_subject_type.as_deref();
        let owner_id = command.owner_subject_id.as_deref();
        if (owner_type.is_some() && owner_id.is_none())
            || (owner_type.is_none() && owner_id.is_some())
        {
            return Err(DriveServiceError::Validation(
                "owner_subject_type and owner_subject_id must be provided together".to_string(),
            ));
        }
        let space_type = validate_optional_space_type(command.space_type.as_deref())?;

        self.store
            .list_accessible_spaces(ListAccessibleSpacesQuery {
                tenant_id: &tenant_id,
                viewer_subject_type: &viewer_subject_type,
                viewer_subject_id: &viewer_subject_id,
                owner_subject_type: owner_type,
                owner_subject_id: owner_id,
                space_type: space_type.as_deref(),
                offset: command.offset,
                limit: command.limit,
            })
            .await
    }

    pub async fn get_space(
        &self,
        command: GetSpaceCommand,
    ) -> Result<DriveSpace, DriveServiceError> {
        let tenant_id = require_non_empty(command.tenant_id, "tenant_id")?;
        let space_id = require_non_empty(command.space_id, "space_id")?;
        self.store.get_space(&tenant_id, &space_id).await
    }

    pub async fn update_space(
        &self,
        command: UpdateSpaceCommand,
    ) -> Result<DriveSpace, DriveServiceError> {
        let tenant_id = require_non_empty(command.tenant_id, "tenant_id")?;
        let space_id = require_non_empty(command.space_id, "space_id")?;
        let operator_id = require_non_empty(command.operator_id, "operator_id")?;
        if command.display_name.is_none()
            && command.presentation_icon.is_none()
            && command.presentation_color.is_none()
            && command.description.is_none()
        {
            return self.store.get_space(&tenant_id, &space_id).await;
        }
        let display_name = match command.display_name {
            Some(value) => Some(require_non_empty(value, "display_name")?),
            None => None,
        };
        self.store
            .update_space(
                &tenant_id,
                &space_id,
                display_name.as_deref(),
                normalize_optional_text_ref(command.presentation_icon.as_deref()),
                normalize_optional_text_ref(command.presentation_color.as_deref()),
                normalize_optional_text_ref(command.description.as_deref()),
                &operator_id,
            )
            .await
    }

    pub async fn delete_space(
        &self,
        command: DeleteSpaceCommand,
    ) -> Result<DriveSpace, DriveServiceError> {
        let tenant_id = require_non_empty(command.tenant_id, "tenant_id")?;
        let space_id = require_non_empty(command.space_id, "space_id")?;
        let operator_id = require_non_empty(command.operator_id, "operator_id")?;
        let existing = self.store.get_space(&tenant_id, &space_id).await?;
        if existing.space_type.is_non_deletable() {
            return Err(DriveServiceError::Validation(format!(
                "{} cannot be deleted",
                existing.space_type.display_label()
            )));
        }
        self.store
            .delete_space(&tenant_id, &space_id, &operator_id)
            .await
    }
}

#[derive(Debug, Clone)]
pub struct ListSpacesCommand {
    pub tenant_id: String,
    pub owner_subject_type: Option<String>,
    pub owner_subject_id: Option<String>,
    pub space_type: Option<String>,
    pub offset: i64,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct ListAccessibleSpacesCommand {
    pub tenant_id: String,
    pub viewer_subject_type: String,
    pub viewer_subject_id: String,
    pub owner_subject_type: Option<String>,
    pub owner_subject_id: Option<String>,
    pub space_type: Option<String>,
    pub offset: i64,
    pub limit: i64,
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

fn require_owner_subject_type(value: String) -> Result<String, DriveServiceError> {
    let owner_subject_type = require_non_empty(value, "owner_subject_type")?;
    match owner_subject_type.as_str() {
        "app" | "user" | "group" | "organization" => Ok(owner_subject_type),
        _ => Err(DriveServiceError::Validation(
            "owner_subject_type must be app, user, group, or organization".to_string(),
        )),
    }
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|trimmed| !trimmed.is_empty())
}

fn normalize_optional_text_ref(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|trimmed| !trimmed.is_empty())
}

fn validate_optional_space_type(value: Option<&str>) -> Result<Option<String>, DriveServiceError> {
    let Some(raw) = normalize_optional_text_ref(value) else {
        return Ok(None);
    };
    if DriveSpaceType::try_from_str(raw).is_none() {
        return Err(DriveServiceError::Validation(
            "space_type is invalid".to_string(),
        ));
    }
    Ok(Some(raw.to_string()))
}
