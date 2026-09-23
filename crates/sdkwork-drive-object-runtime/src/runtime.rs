use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use sdkwork_drive_storage_contract::{
    DriveObjectStore, DriveObjectStoreError, DriveObjectStoreErrorKind,
    DriveStorageCredentialSnapshot, DriveStorageProviderKind,
};
use sdkwork_drive_storage_local::LocalDriveObjectStore;
use sdkwork_drive_storage_s3::{S3DriveObjectStore, S3StoreConfig};
use sdkwork_drive_workspace_service::domain::storage_provider::DriveStorageProvider;
use sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_store::SqlStorageProviderStore;
use sdkwork_drive_workspace_service::ports::storage_provider_store::DriveStorageProviderStore;
use sdkwork_iam_provider_account_service::{
    resolve_bound_credential_material, ProviderAccountError, CREDENTIAL_KIND_ACCESS_KEY_PAIR,
};
use sqlx::PgPool;
use tokio::sync::RwLock;
use url::Url;

const MAX_CACHED_PROVIDER_ADAPTERS: usize = 1024;
/// Cache key of one provider adapter.
///
/// The third element is the *credential generation*: `0` for a provider that
/// carries its own `credential_ref`, and the account-center credential version
/// for a provider bound to a reusable account. It has to be part of the key —
/// rotating an account credential inserts a new credential row and leaves both
/// `dr_drive_storage_provider.version` and the account row untouched, so a
/// provider-version-only key would keep serving the rotated-away secret until
/// some unrelated edit bumped the provider.
type ProviderCacheKey = (String, i64, i64);
type ProviderObjectStore = Arc<dyn DriveObjectStore>;
type ProviderObjectStoreCache = Arc<RwLock<HashMap<ProviderCacheKey, ProviderObjectStore>>>;

#[derive(Clone)]
pub struct DriveObjectStoreRuntime {
    pool: PgPool,
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
            provider_store: SqlStorageProviderStore::new(pool.clone()),
            pool,
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

        let credentials = resolve_provider_credentials(&self.pool, &provider).await?;
        let cache_key = (
            provider_id.to_string(),
            provider_version,
            credentials
                .as_ref()
                .map(|snapshot| snapshot.credential_version)
                .unwrap_or(0),
        );
        if let Some(store) = self.cache.read().await.get(&cache_key).cloned() {
            return Ok(store);
        }

        let store = build_store(&provider, credentials.as_ref()).await?;

        let mut cache = self.cache.write().await;
        if let Some(existing) = cache.get(&cache_key).cloned() {
            return Ok(existing);
        }
        cache.retain(|(cached_provider_id, _, _), _| cached_provider_id != provider_id);
        if cache.len() < MAX_CACHED_PROVIDER_ADAPTERS {
            cache.insert(cache_key, store.clone());
        }
        Ok(store)
    }
}

/// Credential material a provider operates with, plus the version it came from.
struct ResolvedCredentials {
    snapshot: DriveStorageCredentialSnapshot,
    credential_version: i64,
}

/// Resolve the credential material of a provider that reads from the account
/// center; `None` means the provider carries its own `credential_ref`.
///
/// Content operations used to build their store from `credential_ref` alone,
/// which left account-backed providers (the default the admin console offers,
/// and the only way "reuse one cloud account across resources" can hold) with
/// `credential_ref = NULL` — so every read and write silently fell through to
/// the process-wide `SDKWORK_DRIVE_S3_*` environment credentials instead of the
/// account's. That is a tenant provider serving another tenant's objects, so the
/// resolution has to happen here and not only on the admin surface.
///
/// The lookup is the *pinned* one: the provider names its account, and a
/// platform-scope account lives in the platform tenant, which a tenant-filtered
/// walk would never reach. Who may bind an account is decided where the
/// reference is written (`ensure_bindable_provider_account`), not here — the
/// content path has no operator on hand.
async fn resolve_provider_credentials(
    pool: &PgPool,
    provider: &DriveStorageProvider,
) -> Result<Option<ResolvedCredentials>, DriveObjectStoreError> {
    let Some(account_id) = provider
        .provider_account_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    let material = resolve_bound_credential_material(
        pool,
        &provider.tenant_id,
        None,
        account_id,
        Some(CREDENTIAL_KIND_ACCESS_KEY_PAIR),
    )
    .await
    .map_err(map_provider_account_error)?;

    let (Some(access_key_id), Some(secret_access_key)) =
        (material.access_key_id, material.secret_access_key)
    else {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::Conflict,
            "provider account carries no usable access-key-pair credential for object storage",
        ));
    };

    Ok(Some(ResolvedCredentials {
        snapshot: DriveStorageCredentialSnapshot {
            access_key_id,
            secret_access_key,
            session_token: material.session_token,
        },
        credential_version: material.credential_version,
    }))
}

fn map_provider_account_error(error: ProviderAccountError) -> DriveObjectStoreError {
    let (kind, message) = match error {
        ProviderAccountError::Validation(message) => {
            (DriveObjectStoreErrorKind::InvalidRequest, message)
        }
        ProviderAccountError::NotFound(message) => (DriveObjectStoreErrorKind::NotFound, message),
        ProviderAccountError::Conflict(message) => (DriveObjectStoreErrorKind::Conflict, message),
        ProviderAccountError::Unavailable(message) => {
            (DriveObjectStoreErrorKind::Internal, message)
        }
        ProviderAccountError::Cipher(message) => (DriveObjectStoreErrorKind::Internal, message),
    };
    DriveObjectStoreError::new(kind, message)
}

async fn build_store(
    provider: &DriveStorageProvider,
    credentials: Option<&ResolvedCredentials>,
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
            let config = match credentials {
                Some(credentials) => S3StoreConfig::from_provider_parts_with_credentials(
                    provider.provider_kind.as_str(),
                    &provider.endpoint_url,
                    provider.region.as_deref(),
                    &provider.bucket,
                    provider.path_style,
                    credentials.snapshot.clone(),
                    Some(provider.strict_tls),
                )?,
                None => S3StoreConfig::from_provider_parts(
                    provider.provider_kind.as_str(),
                    &provider.endpoint_url,
                    provider.region.as_deref(),
                    &provider.bucket,
                    provider.path_style,
                    provider.credential_ref.as_deref(),
                    Some(provider.strict_tls),
                )?,
            };
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
