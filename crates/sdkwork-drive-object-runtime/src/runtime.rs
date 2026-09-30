use std::sync::Arc;

use sqlx::PgPool;

use sdkwork_drive_storage_contract::{DriveObjectStore, DriveObjectStoreError};
use sdkwork_drive_workspace_service::domain::storage_provider::DriveStorageProvider;

use crate::provider_store::{DriveProviderObjectStoreFactory, ProviderAccessIntent};

/// Locator-aware façade over [`DriveProviderObjectStoreFactory`].
///
/// This is the entry point for services that hold a *locator* - the persisted
/// `(storage_provider_id, bucket)` a Drive object was written to - rather than a
/// provider row. It keeps that locator authoritative: the provider is looked up
/// fresh, the read intent is enforced, and the provider is required to still
/// address the bucket the object was written to.
///
/// Write-path callers that mint URLs or accept new bytes resolve through the
/// same factory with [`ProviderAccessIntent::Write`]; there is exactly one
/// provider -> object store pipeline in Drive.
#[derive(Clone)]
pub struct DriveObjectStoreRuntime {
    factory: DriveProviderObjectStoreFactory,
}

impl std::fmt::Debug for DriveObjectStoreRuntime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DriveObjectStoreRuntime")
            .finish_non_exhaustive()
    }
}

impl DriveObjectStoreRuntime {
    pub fn new(pool: PgPool) -> Self {
        Self {
            factory: DriveProviderObjectStoreFactory::new(pool),
        }
    }

    pub fn factory(&self) -> &DriveProviderObjectStoreFactory {
        &self.factory
    }

    /// Resolve the adapter a historical locator points at.
    ///
    /// Reads tolerate a provider that has since been disabled - disabling is how
    /// an operator drains a provider while its objects stay reachable - but fail
    /// closed on a deleted provider and on a provider that has been repointed at
    /// a different bucket.
    pub async fn resolve_locator(
        &self,
        provider_id: &str,
        bucket: &str,
    ) -> Result<Arc<dyn DriveObjectStore>, DriveObjectStoreError> {
        let provider = self
            .factory
            .load_provider_for(provider_id, ProviderAccessIntent::Read)
            .await?;
        self.factory
            .resolve_for_locator(&provider, bucket, ProviderAccessIntent::Read)
            .await
    }

    /// Resolve the adapter for a provider by id under an explicit intent.
    pub async fn resolve(
        &self,
        provider_id: &str,
        intent: ProviderAccessIntent,
    ) -> Result<Arc<dyn DriveObjectStore>, DriveObjectStoreError> {
        self.factory.resolve_by_id(provider_id, intent).await
    }

    /// Load a provider row for callers that need the record itself.
    pub async fn load_provider(
        &self,
        provider_id: &str,
    ) -> Result<DriveStorageProvider, DriveObjectStoreError> {
        self.factory.load_provider(provider_id).await
    }
}
