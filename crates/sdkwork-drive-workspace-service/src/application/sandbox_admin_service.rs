use std::path::PathBuf;

use sdkwork_drive_security::{
    has_drive_admin_capability, DriveAppContext, DRIVE_SANDBOXES_ADMIN_PERMISSION,
};
use sdkwork_utils_rust::{uuid, validated_offset_list_params, DEFAULT_LIST_PAGE_SIZE};

use crate::domain::sandbox_admin::{
    SandboxAdminAccessLevel, SandboxAdminGrant, SandboxAdminGrantSubjectType,
    SandboxAdminLifecycleStatus, SandboxAdminPage, SandboxAdminProviderKind, SandboxAdminVolume,
};
use crate::ports::sandbox_admin_store::{
    ListSandboxAdminGrantsQuery, ListSandboxAdminVolumesQuery, NewSandboxAdminGrant,
    NewSandboxAdminVolume, SandboxAdminAuditContext, SandboxAdminStore, UpdateSandboxAdminGrant,
    UpdateSandboxAdminVolume,
};
use crate::DriveServiceError;

const MAX_DISPLAY_NAME_LEN: usize = 255;
const MAX_IDENTIFIER_LEN: usize = 128;
const MAX_PROVIDER_ROOT_REF_LEN: usize = 4096;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListSandboxAdminVolumesCommand {
    pub lifecycle_status: Option<String>,
    pub provider_kind: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialSandboxUserGrant {
    pub enabled: bool,
    pub access_level: Option<String>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct CreateSandboxAdminVolumeCommand {
    pub display_name: String,
    pub provider_kind: Option<String>,
    pub provider_root_ref: String,
    pub default_access: Option<String>,
    pub initial_user_grant: Option<InitialSandboxUserGrant>,
}

impl std::fmt::Debug for CreateSandboxAdminVolumeCommand {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CreateSandboxAdminVolumeCommand")
            .field("display_name", &self.display_name)
            .field("provider_kind", &self.provider_kind)
            .field("provider_root_ref", &"[REDACTED]")
            .field("default_access", &self.default_access)
            .field("initial_user_grant", &self.initial_user_grant)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct UpdateSandboxAdminVolumeCommand {
    pub sandbox_id: String,
    pub display_name: Option<String>,
    pub provider_root_ref: Option<String>,
    pub lifecycle_status: Option<String>,
    pub default_access: Option<String>,
    pub expected_version: i64,
}

impl std::fmt::Debug for UpdateSandboxAdminVolumeCommand {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UpdateSandboxAdminVolumeCommand")
            .field("sandbox_id", &self.sandbox_id)
            .field("display_name", &self.display_name)
            .field(
                "provider_root_ref",
                &self.provider_root_ref.as_ref().map(|_| "[REDACTED]"),
            )
            .field("lifecycle_status", &self.lifecycle_status)
            .field("default_access", &self.default_access)
            .field("expected_version", &self.expected_version)
            .finish()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListSandboxAdminGrantsCommand {
    pub sandbox_id: String,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateSandboxAdminGrantCommand {
    pub sandbox_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub access_level: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSandboxAdminGrantCommand {
    pub sandbox_id: String,
    pub grant_id: String,
    pub access_level: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SandboxAdminIdentity {
    tenant_id: String,
    organization_id: String,
    user_id: String,
    operator_id: String,
    actor_kind: String,
    request_id: Option<String>,
    trace_id: Option<String>,
}

pub struct DriveSandboxAdminService<S> {
    store: S,
}

impl<S: SandboxAdminStore> DriveSandboxAdminService<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn list_volumes(
        &self,
        context: &DriveAppContext,
        command: ListSandboxAdminVolumesCommand,
    ) -> Result<SandboxAdminPage<SandboxAdminVolume>, DriveServiceError> {
        let identity = authorize(context)?;
        let params = parse_page(command.page, command.page_size)?;
        let lifecycle_status = command
            .lifecycle_status
            .map(|value| parse_lifecycle_status(&value).map(|status| status.as_str().to_string()))
            .transpose()?;
        let provider_kind = command
            .provider_kind
            .map(|value| parse_provider_kind(&value).map(|kind| kind.as_str().to_string()))
            .transpose()?;

        self.store
            .list_volumes(&ListSandboxAdminVolumesQuery {
                tenant_id: identity.tenant_id,
                organization_id: identity.organization_id,
                lifecycle_status,
                provider_kind,
                page: params.page,
                page_size: params.page_size,
                offset: params.offset,
            })
            .await
    }

    pub async fn get_volume(
        &self,
        context: &DriveAppContext,
        sandbox_id: &str,
    ) -> Result<SandboxAdminVolume, DriveServiceError> {
        let identity = authorize(context)?;
        let sandbox_id = normalize_identifier("sandbox_id", sandbox_id)?;
        self.store
            .get_volume(&identity.tenant_id, &identity.organization_id, &sandbox_id)
            .await?
            .ok_or_else(|| DriveServiceError::NotFound("sandbox volume not found".to_string()))
    }

    pub async fn create_volume(
        &self,
        context: &DriveAppContext,
        command: CreateSandboxAdminVolumeCommand,
    ) -> Result<SandboxAdminVolume, DriveServiceError> {
        let identity = authorize(context)?;
        let display_name = normalize_display_name(command.display_name)?;
        let provider_kind = parse_provider_kind(
            command
                .provider_kind
                .as_deref()
                .unwrap_or(SandboxAdminProviderKind::LocalFilesystem.as_str()),
        )?;
        let provider_root_ref =
            normalize_provider_root_ref(provider_kind, command.provider_root_ref).await?;
        let default_access = parse_access_level(
            command
                .default_access
                .as_deref()
                .unwrap_or(SandboxAdminAccessLevel::Full.as_str()),
        )?;

        let sandbox_id = uuid();
        let initial_grant =
            resolve_initial_grant(&sandbox_id, &identity, command.initial_user_grant)?;
        let volume = NewSandboxAdminVolume {
            id: sandbox_id,
            tenant_id: identity.tenant_id.clone(),
            organization_id: identity.organization_id.clone(),
            display_name,
            root_entry_id: uuid(),
            provider_kind: provider_kind.as_str().to_string(),
            provider_root_ref,
            lifecycle_status: SandboxAdminLifecycleStatus::Active.as_str().to_string(),
            default_access: default_access.as_str().to_string(),
            created_by: identity.operator_id.clone(),
            initial_grant,
        };

        self.store
            .create_volume(&volume, &audit_context(&identity))
            .await
    }

    pub async fn update_volume(
        &self,
        context: &DriveAppContext,
        command: UpdateSandboxAdminVolumeCommand,
    ) -> Result<SandboxAdminVolume, DriveServiceError> {
        let identity = authorize(context)?;
        let sandbox_id = normalize_identifier("sandbox_id", &command.sandbox_id)?;
        if command.expected_version < 1 {
            return Err(DriveServiceError::Validation(
                "expected_version must be greater than 0".to_string(),
            ));
        }
        if command.display_name.is_none()
            && command.provider_root_ref.is_none()
            && command.lifecycle_status.is_none()
            && command.default_access.is_none()
        {
            return Err(DriveServiceError::Validation(
                "at least one sandbox volume field must be provided".to_string(),
            ));
        }

        let current = self
            .store
            .get_volume(&identity.tenant_id, &identity.organization_id, &sandbox_id)
            .await?
            .ok_or_else(|| DriveServiceError::NotFound("sandbox volume not found".to_string()))?;
        let provider_kind =
            SandboxAdminProviderKind::parse(&current.provider_kind).ok_or_else(|| {
                DriveServiceError::Internal("sandbox provider kind is invalid".to_string())
            })?;
        let display_name = command
            .display_name
            .map(normalize_display_name)
            .transpose()?
            .unwrap_or(current.display_name);
        let lifecycle_status = match command.lifecycle_status.as_deref() {
            Some(value) => parse_lifecycle_status(value)?,
            None => {
                SandboxAdminLifecycleStatus::parse(&current.lifecycle_status).ok_or_else(|| {
                    DriveServiceError::Internal(
                        "persisted sandbox lifecycle status is invalid".to_string(),
                    )
                })?
            }
        };
        let default_access = match command.default_access.as_deref() {
            Some(value) => parse_access_level(value)?,
            None => SandboxAdminAccessLevel::parse(&current.default_access).ok_or_else(|| {
                DriveServiceError::Internal(
                    "persisted sandbox default access is invalid".to_string(),
                )
            })?,
        };
        let should_revalidate_root = command.provider_root_ref.is_some()
            || (current.lifecycle_status == SandboxAdminLifecycleStatus::Disabled.as_str()
                && lifecycle_status != SandboxAdminLifecycleStatus::Disabled);
        let provider_root_ref = if should_revalidate_root {
            normalize_provider_root_ref(
                provider_kind,
                command
                    .provider_root_ref
                    .unwrap_or(current.provider_root_ref),
            )
            .await?
        } else {
            current.provider_root_ref
        };

        self.store
            .update_volume(
                &UpdateSandboxAdminVolume {
                    tenant_id: identity.tenant_id.clone(),
                    organization_id: identity.organization_id.clone(),
                    sandbox_id,
                    display_name,
                    provider_root_ref,
                    lifecycle_status: lifecycle_status.as_str().to_string(),
                    default_access: default_access.as_str().to_string(),
                    expected_version: command.expected_version,
                    updated_by: identity.operator_id.clone(),
                },
                &audit_context(&identity),
            )
            .await
    }

    pub async fn delete_volume(
        &self,
        context: &DriveAppContext,
        sandbox_id: &str,
    ) -> Result<(), DriveServiceError> {
        let identity = authorize(context)?;
        self.store
            .delete_volume(
                &identity.tenant_id,
                &identity.organization_id,
                &normalize_identifier("sandbox_id", sandbox_id)?,
                &audit_context(&identity),
            )
            .await
    }

    pub async fn list_grants(
        &self,
        context: &DriveAppContext,
        command: ListSandboxAdminGrantsCommand,
    ) -> Result<SandboxAdminPage<SandboxAdminGrant>, DriveServiceError> {
        let identity = authorize(context)?;
        let sandbox_id = normalize_identifier("sandbox_id", &command.sandbox_id)?;
        let params = parse_page(command.page, command.page_size)?;
        self.store
            .list_grants(&ListSandboxAdminGrantsQuery {
                tenant_id: identity.tenant_id,
                organization_id: identity.organization_id,
                sandbox_id,
                page: params.page,
                page_size: params.page_size,
                offset: params.offset,
            })
            .await
    }

    pub async fn create_grant(
        &self,
        context: &DriveAppContext,
        command: CreateSandboxAdminGrantCommand,
    ) -> Result<SandboxAdminGrant, DriveServiceError> {
        let identity = authorize(context)?;
        let sandbox_id = normalize_identifier("sandbox_id", &command.sandbox_id)?;
        let subject_type = parse_subject_type(&command.subject_type)?;
        let subject_id = normalize_identifier("subject_id", &command.subject_id)?;
        if subject_type == SandboxAdminGrantSubjectType::Organization && subject_id == "0" {
            return Err(DriveServiceError::Validation(
                "organization grant subject_id must be non-zero".to_string(),
            ));
        }
        let access_level = parse_access_level(&command.access_level)?;
        let grant = NewSandboxAdminGrant {
            id: uuid(),
            sandbox_id,
            subject_type: subject_type.as_str().to_string(),
            subject_id,
            access_level: access_level.as_str().to_string(),
            granted_by: identity.operator_id.clone(),
        };
        self.store
            .create_grant(
                &identity.tenant_id,
                &identity.organization_id,
                &grant,
                &audit_context(&identity),
            )
            .await
    }

    pub async fn get_grant(
        &self,
        context: &DriveAppContext,
        sandbox_id: &str,
        grant_id: &str,
    ) -> Result<SandboxAdminGrant, DriveServiceError> {
        let identity = authorize(context)?;
        self.store
            .get_grant(
                &identity.tenant_id,
                &identity.organization_id,
                &normalize_identifier("sandbox_id", sandbox_id)?,
                &normalize_identifier("grant_id", grant_id)?,
            )
            .await?
            .ok_or_else(|| DriveServiceError::NotFound("sandbox grant not found".to_string()))
    }

    pub async fn update_grant(
        &self,
        context: &DriveAppContext,
        command: UpdateSandboxAdminGrantCommand,
    ) -> Result<SandboxAdminGrant, DriveServiceError> {
        let identity = authorize(context)?;
        let access_level = parse_access_level(&command.access_level)?;
        self.store
            .update_grant(
                &UpdateSandboxAdminGrant {
                    tenant_id: identity.tenant_id.clone(),
                    organization_id: identity.organization_id.clone(),
                    sandbox_id: normalize_identifier("sandbox_id", &command.sandbox_id)?,
                    grant_id: normalize_identifier("grant_id", &command.grant_id)?,
                    access_level: access_level.as_str().to_string(),
                },
                &audit_context(&identity),
            )
            .await
    }

    pub async fn delete_grant(
        &self,
        context: &DriveAppContext,
        sandbox_id: &str,
        grant_id: &str,
    ) -> Result<(), DriveServiceError> {
        let identity = authorize(context)?;
        self.store
            .delete_grant(
                &identity.tenant_id,
                &identity.organization_id,
                &normalize_identifier("sandbox_id", sandbox_id)?,
                &normalize_identifier("grant_id", grant_id)?,
                &audit_context(&identity),
            )
            .await
    }
}

fn authorize(context: &DriveAppContext) -> Result<SandboxAdminIdentity, DriveServiceError> {
    if !has_drive_admin_capability(context, DRIVE_SANDBOXES_ADMIN_PERMISSION) {
        return Err(DriveServiceError::PermissionDenied(
            "sandbox administration permission is required".to_string(),
        ));
    }
    let tenant_id = required_context_value("tenant", &context.tenant_id)?;
    let organization_id = context
        .organization_id
        .as_deref()
        .map(|value| required_context_value("organization", value))
        .transpose()?
        .ok_or_else(|| {
            DriveServiceError::PermissionDenied(
                "organization-scoped backend session is required".to_string(),
            )
        })?;
    if organization_id == "0" {
        return Err(DriveServiceError::PermissionDenied(
            "organization-scoped backend session requires a non-zero organization".to_string(),
        ));
    }
    let user_id = required_context_value("user", &context.user_id)?;
    let operator_id = required_context_value("actor", &context.actor_id)?;
    let actor_kind = required_context_value("actor kind", &context.actor_kind)?;
    Ok(SandboxAdminIdentity {
        tenant_id,
        organization_id,
        user_id,
        operator_id,
        actor_kind,
        request_id: non_empty(context.request_id.clone()),
        trace_id: non_empty(context.trace_id.clone()),
    })
}

fn required_context_value(field: &str, value: &str) -> Result<String, DriveServiceError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(DriveServiceError::PermissionDenied(format!(
            "verified {field} context is required"
        )));
    }
    Ok(value.to_string())
}

fn audit_context(identity: &SandboxAdminIdentity) -> SandboxAdminAuditContext {
    SandboxAdminAuditContext {
        tenant_id: identity.tenant_id.clone(),
        organization_id: identity.organization_id.clone(),
        operator_id: identity.operator_id.clone(),
        request_id: identity.request_id.clone(),
        trace_id: identity.trace_id.clone(),
    }
}

fn resolve_initial_grant(
    sandbox_id: &str,
    identity: &SandboxAdminIdentity,
    configuration: Option<InitialSandboxUserGrant>,
) -> Result<Option<NewSandboxAdminGrant>, DriveServiceError> {
    let configuration = configuration.unwrap_or(InitialSandboxUserGrant {
        enabled: true,
        access_level: None,
    });
    if !configuration.enabled {
        if configuration.access_level.is_some() {
            return Err(DriveServiceError::Validation(
                "initial_user_grant.access_level requires enabled=true".to_string(),
            ));
        }
        return Ok(None);
    }
    if identity.actor_kind != "user" {
        return Err(DriveServiceError::Validation(
            "initial_user_grant requires a verified user actor; service and system actors must set enabled=false"
                .to_string(),
        ));
    }
    let access_level = parse_access_level(
        configuration
            .access_level
            .as_deref()
            .unwrap_or(SandboxAdminAccessLevel::Full.as_str()),
    )?;
    Ok(Some(NewSandboxAdminGrant {
        id: uuid(),
        sandbox_id: sandbox_id.to_string(),
        subject_type: SandboxAdminGrantSubjectType::User.as_str().to_string(),
        subject_id: identity.user_id.clone(),
        access_level: access_level.as_str().to_string(),
        granted_by: identity.operator_id.clone(),
    }))
}

fn parse_page(
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<sdkwork_utils_rust::OffsetListPageParams, DriveServiceError> {
    let resolved_page = page.unwrap_or(1);
    let resolved_page_size = page_size.unwrap_or(i64::from(DEFAULT_LIST_PAGE_SIZE));
    if resolved_page
        .checked_sub(1)
        .and_then(|value| value.checked_mul(resolved_page_size))
        .is_none()
    {
        return Err(DriveServiceError::Validation(
            "page and page_size produce an offset outside the supported range".to_string(),
        ));
    }
    validated_offset_list_params(page, page_size).map_err(|_| {
        DriveServiceError::Validation(
            "page must be greater than 0 and page_size must be in range [1, 200]".to_string(),
        )
    })
}

fn parse_provider_kind(value: &str) -> Result<SandboxAdminProviderKind, DriveServiceError> {
    SandboxAdminProviderKind::parse(value.trim()).ok_or_else(|| {
        DriveServiceError::Validation(
            "provider_kind is invalid; only local_filesystem is available".to_string(),
        )
    })
}

fn parse_lifecycle_status(value: &str) -> Result<SandboxAdminLifecycleStatus, DriveServiceError> {
    SandboxAdminLifecycleStatus::parse(value.trim()).ok_or_else(|| {
        DriveServiceError::Validation(
            "lifecycle_status must be active, read_only, or disabled".to_string(),
        )
    })
}

fn parse_access_level(value: &str) -> Result<SandboxAdminAccessLevel, DriveServiceError> {
    SandboxAdminAccessLevel::parse(value.trim()).ok_or_else(|| {
        DriveServiceError::Validation("access_level must be full or read_only".to_string())
    })
}

fn parse_subject_type(value: &str) -> Result<SandboxAdminGrantSubjectType, DriveServiceError> {
    let value = value.trim();
    if matches!(value, "workspace" | "role") {
        return Err(DriveServiceError::Validation(format!(
            "subject_type {value} is unavailable until an authoritative membership resolver exists"
        )));
    }
    SandboxAdminGrantSubjectType::parse(value).ok_or_else(|| {
        DriveServiceError::Validation("subject_type must be user or organization".to_string())
    })
}

fn normalize_display_name(value: String) -> Result<String, DriveServiceError> {
    let value = value.trim().to_string();
    if value.is_empty() || value.len() > MAX_DISPLAY_NAME_LEN || value.chars().any(char::is_control)
    {
        return Err(DriveServiceError::Validation(
            "display_name must be non-empty, contain no control characters, and be at most 255 bytes"
                .to_string(),
        ));
    }
    Ok(value)
}

fn normalize_identifier(field: &str, value: &str) -> Result<String, DriveServiceError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > MAX_IDENTIFIER_LEN
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'@' | b'-')
        })
    {
        return Err(DriveServiceError::Validation(format!("{field} is invalid")));
    }
    Ok(value.to_string())
}

async fn normalize_provider_root_ref(
    provider_kind: SandboxAdminProviderKind,
    value: String,
) -> Result<String, DriveServiceError> {
    let value = value.trim().to_string();
    if value.is_empty()
        || value.len() > MAX_PROVIDER_ROOT_REF_LEN
        || value.chars().any(char::is_control)
    {
        return Err(DriveServiceError::Validation(
            "provider_root_ref is invalid".to_string(),
        ));
    }
    debug_assert_eq!(provider_kind, SandboxAdminProviderKind::LocalFilesystem);
    tokio::task::spawn_blocking(move || canonicalize_local_root(PathBuf::from(value)))
        .await
        .map_err(|_| {
            DriveServiceError::Internal("local filesystem root validation failed".to_string())
        })?
}

fn canonicalize_local_root(path: PathBuf) -> Result<String, DriveServiceError> {
    let canonical = std::fs::canonicalize(path).map_err(|_| {
        DriveServiceError::Validation(
            "local filesystem root must already exist and be accessible".to_string(),
        )
    })?;
    let metadata = std::fs::metadata(&canonical).map_err(|_| {
        DriveServiceError::Validation(
            "local filesystem root must already exist and be accessible".to_string(),
        )
    })?;
    if !metadata.is_dir() {
        return Err(DriveServiceError::Validation(
            "local filesystem root must be a directory".to_string(),
        ));
    }
    std::fs::read_dir(&canonical).map_err(|_| {
        DriveServiceError::Validation(
            "local filesystem root must already exist and be accessible".to_string(),
        )
    })?;
    let canonical = canonical.into_os_string().into_string().map_err(|_| {
        DriveServiceError::Validation(
            "local filesystem root must be representable as UTF-8".to_string(),
        )
    })?;
    if canonical.len() > MAX_PROVIDER_ROOT_REF_LEN || canonical.chars().any(char::is_control) {
        return Err(DriveServiceError::Validation(
            "canonical local filesystem root is invalid".to_string(),
        ));
    }
    Ok(canonical)
}

fn non_empty(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}
