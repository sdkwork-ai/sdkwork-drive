use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use sdkwork_drive_storage_contract::{
    DriveObjectStore, DriveObjectStoreError, DriveObjectStoreErrorKind, DriveStorageProviderKind,
};
use sdkwork_drive_storage_local::LocalDriveObjectStore;
use sdkwork_drive_storage_s3::{S3DriveObjectStore, S3StoreConfig};
use sdkwork_drive_workspace_service::domain::storage_provider::DriveStorageProvider;
use sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_store::SqlStorageProviderStore;
use sdkwork_drive_workspace_service::ports::storage_provider_store::DriveStorageProviderStore;
use sqlx::PgPool;
use tokio::sync::RwLock;
use url::Url;

const MAX_CACHED_PROVIDER_ADAPTERS: usize = 1024;
type ProviderCacheKey = (String, i64);
type ProviderObjectStore = Arc<dyn DriveObjectStore>;
type ProviderObjectStoreCache = Arc<RwLock<HashMap<ProviderCacheKey, ProviderObjectStore>>>;

#[derive(Clone)]
pub struct DriveObjectStoreRuntime {
    provider_store: SqlStorageProviderStore,
    cache: ProviderObjectStoreCache,
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
            provider_store: SqlStorageProviderStore::new(pool),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn resolve(
        &self,
        provider_id: &str,
        provider_version: i64,
    ) -> Result<Arc<dyn DriveObjectStore>, DriveObjectStoreError> {
        if provider_id.trim().is_empty() || provider_version < 1 {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "storage provider identity is invalid",
            ));
        }
        let cache_key = (provider_id.to_string(), provider_version);
        if let Some(store) = self.cache.read().await.get(&cache_key).cloned() {
            return Ok(store);
        }

        let provider = self
            .provider_store
            .find_storage_provider(provider_id)
            .await
            .map_err(map_service_error)?
            .ok_or_else(|| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::NotFound,
                    "storage provider was not found",
                )
            })?;
        if provider.status != "active" || provider.version != provider_version {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Conflict,
                "storage provider changed while resolving content",
            ));
        }
        let store = build_store(&provider).await?;

        let mut cache = self.cache.write().await;
        if let Some(existing) = cache.get(&cache_key).cloned() {
            return Ok(existing);
        }
        cache.retain(|(cached_provider_id, _), _| cached_provider_id != provider_id);
        if cache.len() < MAX_CACHED_PROVIDER_ADAPTERS {
            cache.insert(cache_key, store.clone());
        }
        Ok(store)
    }
}

async fn build_store(
    provider: &DriveStorageProvider,
) -> Result<Arc<dyn DriveObjectStore>, DriveObjectStoreError> {
    match &provider.provider_kind {
        DriveStorageProviderKind::LocalFilesystem => {
            let root = local_root_from_endpoint(&provider.endpoint_url)?;
            Ok(Arc::new(LocalDriveObjectStore::new(root)))
        }
        DriveStorageProviderKind::S3Compatible
        | DriveStorageProviderKind::GoogleCloudStorage
        | DriveStorageProviderKind::AliyunOss
        | DriveStorageProviderKind::TencentCos
        | DriveStorageProviderKind::HuaweiObs
        | DriveStorageProviderKind::VolcengineTos
        | DriveStorageProviderKind::Custom(_) => {
            let config = S3StoreConfig::from_provider_parts(
                provider.provider_kind.as_str(),
                &provider.endpoint_url,
                provider.region.as_deref(),
                &provider.bucket,
                provider.path_style,
                provider.credential_ref.as_deref(),
                Some(provider.strict_tls),
            )?;
            S3DriveObjectStore::new(config)
                .await
                .map(|store| Arc::new(store) as Arc<dyn DriveObjectStore>)
        }
    }
}

fn local_root_from_endpoint(endpoint_url: &str) -> Result<PathBuf, DriveObjectStoreError> {
    let url = Url::parse(endpoint_url).map_err(|_| {
        DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            "local storage endpoint must be a valid file URL",
        )
    })?;
    if url.scheme() != "file" || (url.has_host() && url.host_str() != Some("localhost")) {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            "local storage endpoint must use a local file URL",
        ));
    }
    url.to_file_path().map_err(|_| {
        DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            "local storage file URL cannot be converted to a filesystem path",
        )
    })
}

fn map_service_error(
    error: sdkwork_drive_workspace_service::DriveServiceError,
) -> DriveObjectStoreError {
    use sdkwork_drive_workspace_service::DriveServiceError;

    let (kind, message) = match error {
        DriveServiceError::Validation(message) => {
            (DriveObjectStoreErrorKind::InvalidRequest, message)
        }
        DriveServiceError::Conflict(message) => (DriveObjectStoreErrorKind::Conflict, message),
        DriveServiceError::NotFound(message) => (DriveObjectStoreErrorKind::NotFound, message),
        DriveServiceError::PermissionDenied(message) => {
            (DriveObjectStoreErrorKind::PermissionDenied, message)
        }
        DriveServiceError::Internal(message) => (DriveObjectStoreErrorKind::Internal, message),
    };
    DriveObjectStoreError::new(kind, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_endpoint_rejects_non_file_and_remote_file_urls() {
        assert!(local_root_from_endpoint("https://example.test/storage").is_err());
        assert!(local_root_from_endpoint("file://remote-host/storage").is_err());
        let local_url = Url::from_directory_path(std::env::temp_dir())
            .expect("system temp directory should convert to a file URL");
        assert!(local_root_from_endpoint(local_url.as_str()).is_ok());
    }
}
