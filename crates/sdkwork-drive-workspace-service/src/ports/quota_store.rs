use async_trait::async_trait;

use crate::domain::quota::{DriveQuotaSummary, DriveTenantQuotaPolicy};
use crate::DriveServiceError;

#[async_trait]
pub trait DriveQuotaStore: Send + Sync {
    async fn summarize_tenant_quota(
        &self,
        tenant_id: &str,
    ) -> Result<DriveQuotaSummary, DriveServiceError>;

    async fn get_tenant_quota_policy(
        &self,
        tenant_id: &str,
    ) -> Result<Option<DriveTenantQuotaPolicy>, DriveServiceError>;

    async fn upsert_tenant_quota_policy(
        &self,
        tenant_id: &str,
        max_bytes: Option<i64>,
        updated_by: &str,
    ) -> Result<DriveTenantQuotaPolicy, DriveServiceError>;

    async fn clear_tenant_quota_policy(&self, tenant_id: &str) -> Result<(), DriveServiceError>;
}
