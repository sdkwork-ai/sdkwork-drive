use async_trait::async_trait;

use crate::DriveServiceError;

/// Resolves every authorization subject effective for a verified app-api actor.
/// Implementations belong at the IAM/workspace adapter boundary, never in HTTP handlers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveSandboxPrincipal {
    pub subject_type: String,
    pub subject_id: String,
}

#[async_trait]
pub trait EffectiveSandboxPrincipalsPort: Send + Sync {
    async fn resolve_effective_principals(
        &self,
        tenant_id: &str,
        user_id: &str,
        organization_id: Option<&str>,
    ) -> Result<Vec<EffectiveSandboxPrincipal>, DriveServiceError>;
}
