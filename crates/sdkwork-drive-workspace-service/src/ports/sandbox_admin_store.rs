use async_trait::async_trait;

use crate::domain::sandbox_admin::{SandboxAdminGrant, SandboxAdminPage, SandboxAdminVolume};
use crate::DriveServiceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxAdminAuditContext {
    pub tenant_id: String,
    pub organization_id: String,
    pub operator_id: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSandboxAdminVolumesQuery {
    pub tenant_id: String,
    pub organization_id: String,
    pub lifecycle_status: Option<String>,
    pub provider_kind: Option<String>,
    pub page: i64,
    pub page_size: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSandboxAdminGrant {
    pub id: String,
    pub sandbox_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub access_level: String,
    pub granted_by: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct NewSandboxAdminVolume {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub display_name: String,
    pub root_entry_id: String,
    pub provider_kind: String,
    pub provider_root_ref: String,
    pub lifecycle_status: String,
    pub default_access: String,
    pub created_by: String,
    pub initial_grant: Option<NewSandboxAdminGrant>,
}

impl std::fmt::Debug for NewSandboxAdminVolume {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NewSandboxAdminVolume")
            .field("id", &self.id)
            .field("tenant_id", &self.tenant_id)
            .field("organization_id", &self.organization_id)
            .field("display_name", &self.display_name)
            .field("root_entry_id", &self.root_entry_id)
            .field("provider_kind", &self.provider_kind)
            .field("provider_root_ref", &"[REDACTED]")
            .field("lifecycle_status", &self.lifecycle_status)
            .field("default_access", &self.default_access)
            .field("created_by", &self.created_by)
            .field("initial_grant", &self.initial_grant)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct UpdateSandboxAdminVolume {
    pub tenant_id: String,
    pub organization_id: String,
    pub sandbox_id: String,
    pub display_name: String,
    pub provider_root_ref: String,
    pub lifecycle_status: String,
    pub default_access: String,
    pub expected_version: i64,
    pub updated_by: String,
}

impl std::fmt::Debug for UpdateSandboxAdminVolume {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UpdateSandboxAdminVolume")
            .field("tenant_id", &self.tenant_id)
            .field("sandbox_id", &self.sandbox_id)
            .field("display_name", &self.display_name)
            .field("provider_root_ref", &"[REDACTED]")
            .field("lifecycle_status", &self.lifecycle_status)
            .field("default_access", &self.default_access)
            .field("expected_version", &self.expected_version)
            .field("updated_by", &self.updated_by)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSandboxAdminGrantsQuery {
    pub tenant_id: String,
    pub organization_id: String,
    pub sandbox_id: String,
    pub page: i64,
    pub page_size: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSandboxAdminGrant {
    pub tenant_id: String,
    pub organization_id: String,
    pub sandbox_id: String,
    pub grant_id: String,
    pub access_level: String,
}

#[async_trait]
pub trait SandboxAdminStore: Send + Sync {
    async fn list_volumes(
        &self,
        query: &ListSandboxAdminVolumesQuery,
    ) -> Result<SandboxAdminPage<SandboxAdminVolume>, DriveServiceError>;

    async fn get_volume(
        &self,
        tenant_id: &str,
        organization_id: &str,
        sandbox_id: &str,
    ) -> Result<Option<SandboxAdminVolume>, DriveServiceError>;

    async fn create_volume(
        &self,
        volume: &NewSandboxAdminVolume,
        audit: &SandboxAdminAuditContext,
    ) -> Result<SandboxAdminVolume, DriveServiceError>;

    async fn update_volume(
        &self,
        volume: &UpdateSandboxAdminVolume,
        audit: &SandboxAdminAuditContext,
    ) -> Result<SandboxAdminVolume, DriveServiceError>;

    async fn delete_volume(
        &self,
        tenant_id: &str,
        organization_id: &str,
        sandbox_id: &str,
        audit: &SandboxAdminAuditContext,
    ) -> Result<(), DriveServiceError>;

    async fn list_grants(
        &self,
        query: &ListSandboxAdminGrantsQuery,
    ) -> Result<SandboxAdminPage<SandboxAdminGrant>, DriveServiceError>;

    async fn create_grant(
        &self,
        tenant_id: &str,
        organization_id: &str,
        grant: &NewSandboxAdminGrant,
        audit: &SandboxAdminAuditContext,
    ) -> Result<SandboxAdminGrant, DriveServiceError>;

    async fn get_grant(
        &self,
        tenant_id: &str,
        organization_id: &str,
        sandbox_id: &str,
        grant_id: &str,
    ) -> Result<Option<SandboxAdminGrant>, DriveServiceError>;

    async fn update_grant(
        &self,
        grant: &UpdateSandboxAdminGrant,
        audit: &SandboxAdminAuditContext,
    ) -> Result<SandboxAdminGrant, DriveServiceError>;

    async fn delete_grant(
        &self,
        tenant_id: &str,
        organization_id: &str,
        sandbox_id: &str,
        grant_id: &str,
        audit: &SandboxAdminAuditContext,
    ) -> Result<(), DriveServiceError>;
}
