use async_trait::async_trait;

use crate::domain::sandbox::{AuthorizedSandboxMount, DriveSandboxGrant, DriveSandboxVolume};
use crate::ports::sandbox_principal_resolver::EffectiveSandboxPrincipal;
use crate::DriveServiceError;

#[async_trait]
pub trait DriveSandboxStore: Send + Sync {
    async fn list_accessible_for_principals(
        &self,
        tenant_id: &str,
        principals: &[EffectiveSandboxPrincipal],
        offset: i64,
        limit: i64,
    ) -> Result<(Vec<DriveSandboxVolume>, i64), DriveServiceError>;

    async fn list_accessible(
        &self,
        tenant_id: &str,
        subject_type: &str,
        subject_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DriveSandboxVolume>, DriveServiceError>;

    async fn get_grant(
        &self,
        tenant_id: &str,
        sandbox_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<Option<DriveSandboxGrant>, DriveServiceError>;

    /// Returns the private provider binding only when at least one supplied principal has
    /// an explicit grant in the same tenant. Callers must never project this value to HTTP.
    async fn get_authorized_mount_for_principals(
        &self,
        tenant_id: &str,
        sandbox_id: &str,
        principals: &[EffectiveSandboxPrincipal],
    ) -> Result<Option<AuthorizedSandboxMount>, DriveServiceError>;
}
