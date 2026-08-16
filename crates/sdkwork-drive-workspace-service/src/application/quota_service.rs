use sdkwork_drive_config::TenantQuotaPolicy;

use crate::domain::quota::{DriveQuotaSummary, DriveTenantQuotaPolicy};
use crate::ports::quota_store::DriveQuotaStore;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct GetTenantQuotaSummaryCommand {
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct UpsertTenantQuotaPolicyCommand {
    pub tenant_id: String,
    pub max_bytes: Option<i64>,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct ClearTenantQuotaPolicyCommand {
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct DriveQuotaService<S>
where
    S: DriveQuotaStore,
{
    store: S,
}

impl<S> DriveQuotaService<S>
where
    S: DriveQuotaStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn get_tenant_quota_summary(
        &self,
        command: GetTenantQuotaSummaryCommand,
    ) -> Result<DriveQuotaSummary, DriveServiceError> {
        if command.tenant_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "tenant_id is required".to_string(),
            ));
        }
        self.store
            .summarize_tenant_quota(command.tenant_id.trim())
            .await
    }

    pub async fn resolve_effective_max_bytes(
        &self,
        tenant_id: &str,
    ) -> Result<Option<i64>, DriveServiceError> {
        if tenant_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "tenant_id is required".to_string(),
            ));
        }
        if let Some(policy) = self.store.get_tenant_quota_policy(tenant_id.trim()).await? {
            return Ok(policy.max_bytes);
        }
        Ok(TenantQuotaPolicy::from_env().max_bytes)
    }

    pub async fn upsert_tenant_quota_policy(
        &self,
        command: UpsertTenantQuotaPolicyCommand,
    ) -> Result<DriveTenantQuotaPolicy, DriveServiceError> {
        if command.tenant_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "tenant_id is required".to_string(),
            ));
        }
        if command.operator_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "operator_id is required".to_string(),
            ));
        }
        self.store
            .upsert_tenant_quota_policy(
                command.tenant_id.trim(),
                command.max_bytes,
                command.operator_id.trim(),
            )
            .await
    }

    pub async fn clear_tenant_quota_policy(
        &self,
        command: ClearTenantQuotaPolicyCommand,
    ) -> Result<(), DriveServiceError> {
        if command.tenant_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "tenant_id is required".to_string(),
            ));
        }
        self.store
            .clear_tenant_quota_policy(command.tenant_id.trim())
            .await
    }
}
