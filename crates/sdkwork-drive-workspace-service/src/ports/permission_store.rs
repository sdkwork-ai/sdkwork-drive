use async_trait::async_trait;

use crate::DriveServiceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveNodePermissionGrant {
    pub id: String,
    pub node_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveNodePermissionCheck {
    pub allowed: bool,
    pub effective_role: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveNodePermissionList {
    pub items: Vec<DriveNodePermissionGrant>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantDriveNodePermissionCommand {
    pub tenant_id: String,
    pub node_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub role: String,
    pub operator_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevokeDriveNodePermissionCommand {
    pub tenant_id: String,
    pub node_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub operator_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListDriveNodePermissionsCommand {
    pub tenant_id: String,
    pub node_id: String,
    pub page_size: Option<u32>,
    pub page_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckDriveNodePermissionCommand {
    pub tenant_id: String,
    pub node_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub required_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveEffectiveNodeAccessCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: String,
    pub subject_type: String,
    pub subject_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveEffectiveNodeAccess {
    pub role: String,
    pub source: String,
    pub permission_id: Option<String>,
    pub inherited: bool,
    pub inherited_from_node_id: Option<String>,
}

impl DriveEffectiveNodeAccess {
    pub fn allows_role(&self, required_role: &str) -> bool {
        crate::domain::permission_role::drive_role_satisfies(&self.role, required_role)
    }
}

#[async_trait]
pub trait DrivePermissionStore: Send + Sync {
    async fn resolve_space_permission_anchor_node(
        &self,
        tenant_id: &str,
        space_id: &str,
    ) -> Result<String, DriveServiceError>;

    async fn grant_node_permission(
        &self,
        command: GrantDriveNodePermissionCommand,
    ) -> Result<DriveNodePermissionGrant, DriveServiceError>;

    async fn revoke_node_permission(
        &self,
        command: RevokeDriveNodePermissionCommand,
    ) -> Result<(), DriveServiceError>;

    async fn list_node_permissions(
        &self,
        command: ListDriveNodePermissionsCommand,
    ) -> Result<DriveNodePermissionList, DriveServiceError>;

    async fn check_node_permission(
        &self,
        command: CheckDriveNodePermissionCommand,
    ) -> Result<DriveNodePermissionCheck, DriveServiceError>;

    async fn resolve_effective_node_access(
        &self,
        command: ResolveEffectiveNodeAccessCommand,
    ) -> Result<DriveEffectiveNodeAccess, DriveServiceError>;

    async fn is_space_owner(
        &self,
        tenant_id: &str,
        space_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<bool, DriveServiceError>;
}
