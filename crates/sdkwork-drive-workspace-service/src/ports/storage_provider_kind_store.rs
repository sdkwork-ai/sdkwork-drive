use async_trait::async_trait;

use crate::domain::storage_provider_kind::DriveStorageProviderKindRegistry;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct NewDriveStorageProviderKind {
    pub provider_kind: String,
    pub display_name: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageProviderKindConfigCount {
    pub provider_kind: String,
    pub config_count: i64,
}

#[async_trait]
pub trait DriveStorageProviderKindStore: Send + Sync {
    async fn list_storage_provider_kinds(
        &self,
    ) -> Result<Vec<DriveStorageProviderKindRegistry>, DriveServiceError>;

    async fn find_storage_provider_kind(
        &self,
        provider_kind: &str,
    ) -> Result<Option<DriveStorageProviderKindRegistry>, DriveServiceError>;

    /// Idempotent upsert of the built-in kind catalog. Existing rows keep
    /// their operator-managed enabled flag; missing rows are inserted.
    async fn initialize_storage_provider_kinds(
        &self,
        kinds: &[NewDriveStorageProviderKind],
    ) -> Result<Vec<DriveStorageProviderKindRegistry>, DriveServiceError>;

    async fn set_storage_provider_kind_enabled(
        &self,
        provider_kind: &str,
        enabled: bool,
    ) -> Result<DriveStorageProviderKindRegistry, DriveServiceError>;

    async fn count_storage_provider_configs_by_kind(
        &self,
    ) -> Result<Vec<StorageProviderKindConfigCount>, DriveServiceError>;
}
