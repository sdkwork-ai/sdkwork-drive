use sqlx::PgPool;
use std::collections::HashMap;

use crate::infrastructure::sql::permission_store::SqlDrivePermissionStore;
use crate::ports::permission_store::DrivePermissionStore;
use crate::DriveServiceError;

pub use crate::infrastructure::sql::permission_store::MAX_NODE_ACCESS_BATCH;
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

    /// Resolve effective access for many nodes in one set-based query.
    ///
    /// Use this instead of calling [`Self::resolve_effective_node_access`] in a
    /// loop when a caller already holds the node ids (the material-save path).
    /// Each entry equals the single-node answer for that node.
    ///
    /// # Contract
    ///
    /// * `tenant_id` scopes every read; ids from another tenant are absent.
    /// * At most [`MAX_NODE_ACCESS_BATCH`] ids per call.
    /// * An id missing from the result has no resolvable access (deleted node or
    ///   broken ancestry); treat it as no access.
    ///
    /// # Errors
    ///
    /// `Validation` for an over-sized batch and `Internal` when the query fails.
    pub async fn resolve_node_access_batch(
        &self,
        tenant_id: &str,
        node_ids: &[String],
        subject_type: &str,
        subject_id: &str,
    ) -> Result<HashMap<String, DriveEffectiveNodeAccess>, DriveServiceError> {
        self.store
            .resolve_node_access_batch(tenant_id, node_ids, subject_type, subject_id)
            .await
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
