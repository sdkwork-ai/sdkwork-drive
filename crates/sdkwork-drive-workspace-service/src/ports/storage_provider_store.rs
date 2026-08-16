use async_trait::async_trait;

use crate::domain::storage_provider::DriveStorageProvider;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct NewDriveStorageProvider {
    pub id: String,
    pub provider_kind: String,
    pub name: String,
    pub endpoint_url: String,
    pub region: Option<String>,
    pub bucket: String,
    pub path_style: bool,
    pub strict_tls: bool,
    pub credential_ref: Option<String>,
    pub server_side_encryption_mode: Option<String>,
    pub default_storage_class: Option<String>,
    pub status: String,
    pub created_by: String,
    pub updated_by: String,
}

#[derive(Debug, Clone)]
pub struct UpdateDriveStorageProvider {
    pub name: String,
    pub endpoint_url: String,
    pub region: Option<String>,
    pub bucket: String,
    pub path_style: bool,
    pub strict_tls: bool,
    pub credential_ref: Option<String>,
    pub server_side_encryption_mode: Option<String>,
    pub default_storage_class: Option<String>,
    pub status: String,
    pub updated_by: String,
}

#[async_trait]
pub trait DriveStorageProviderStore: Send + Sync {
    async fn insert_storage_provider(
        &self,
        new_provider: &NewDriveStorageProvider,
    ) -> Result<DriveStorageProvider, DriveServiceError>;

    async fn list_storage_providers(
        &self,
        status: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DriveStorageProvider>, DriveServiceError>;

    async fn find_storage_provider(
        &self,
        provider_id: &str,
    ) -> Result<Option<DriveStorageProvider>, DriveServiceError>;

    async fn update_storage_provider(
        &self,
        provider_id: &str,
        patch: &UpdateDriveStorageProvider,
    ) -> Result<DriveStorageProvider, DriveServiceError>;

    async fn has_active_storage_provider_bindings(
        &self,
        provider_id: &str,
    ) -> Result<bool, DriveServiceError>;

    async fn delete_storage_provider(&self, provider_id: &str) -> Result<bool, DriveServiceError>;
}
