use async_trait::async_trait;

use crate::domain::root_scope_subscription::DriveRootScopeSubscription;
use crate::DriveServiceError;

pub const KNOWLEDGEBASE_RAW_CONSUMER_KIND: &str = "knowledgebase_raw";

#[derive(Debug, Clone)]
pub struct RegisterDriveRootScopeSubscription {
    pub tenant_id: String,
    pub space_id: String,
    pub consumer_resource_id: String,
    pub root_node_id: String,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct RegisterDriveRootScopeSubscriptionResult {
    pub subscription: DriveRootScopeSubscription,
    pub created: bool,
}

#[async_trait]
pub trait DriveRootScopeSubscriptionStore: Send + Sync {
    async fn register_knowledgebase_raw(
        &self,
        registration: &RegisterDriveRootScopeSubscription,
    ) -> Result<RegisterDriveRootScopeSubscriptionResult, DriveServiceError>;

    async fn get_by_uuid(
        &self,
        tenant_id: &str,
        subscription_uuid: &str,
    ) -> Result<DriveRootScopeSubscription, DriveServiceError>;
}
