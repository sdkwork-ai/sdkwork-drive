use sqlx::PgPool;

use crate::infrastructure::sql::permission_store::SqlDrivePermissionStore;
use crate::ports::permission_store::DrivePermissionStore;
use crate::DriveServiceError;

pub use crate::ports::permission_store::{
    CheckDriveNodePermissionCommand, DriveEffectiveNodeAccess, DriveNodePermissionCheck,
    DriveNodePermissionGrant, DriveNodePermissionList, GrantDriveNodePermissionCommand,
    ListDriveNodePermissionsCommand, ResolveEffectiveNodeAccessCommand,
    RevokeDriveNodePermissionCommand,
};

#[derive(Debug, Clone)]
pub struct SqlDrivePermissionService {
    store: SqlDrivePermissionStore,
}

impl SqlDrivePermissionService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            store: SqlDrivePermissionStore::new(pool),
        }
    }

    pub async fn resolve_space_permission_anchor_node(
        &self,
        tenant_id: &str,
        space_id: &str,
    ) -> Result<String, DriveServiceError> {
        self.store
            .resolve_space_permission_anchor_node(tenant_id, space_id)
            .await
    }

    pub async fn grant_node_permission(
        &self,
        command: GrantDriveNodePermissionCommand,
    ) -> Result<DriveNodePermissionGrant, DriveServiceError> {
        self.store.grant_node_permission(command).await
    }

    pub async fn revoke_node_permission(
        &self,
        command: RevokeDriveNodePermissionCommand,
    ) -> Result<(), DriveServiceError> {
        self.store.revoke_node_permission(command).await
    }

    pub async fn list_node_permissions(
        &self,
        command: ListDriveNodePermissionsCommand,
    ) -> Result<DriveNodePermissionList, DriveServiceError> {
        self.store.list_node_permissions(command).await
    }

    pub async fn check_node_permission(
        &self,
        command: CheckDriveNodePermissionCommand,
    ) -> Result<DriveNodePermissionCheck, DriveServiceError> {
        self.store.check_node_permission(command).await
    }

    pub async fn resolve_effective_node_access(
        &self,
        command: ResolveEffectiveNodeAccessCommand,
    ) -> Result<DriveEffectiveNodeAccess, DriveServiceError> {
        self.store.resolve_effective_node_access(command).await
    }

    pub async fn is_space_owner(
        &self,
        tenant_id: &str,
        space_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<bool, DriveServiceError> {
        self.store
            .is_space_owner(tenant_id, space_id, subject_type, subject_id)
            .await
    }
}
