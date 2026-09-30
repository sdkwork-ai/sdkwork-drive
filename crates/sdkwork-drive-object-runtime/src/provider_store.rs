//! Single provider -> object store resolution pipeline for the whole Drive.
//!
//! Every Drive surface that needs physical object access (app-api upload and
//! signing, internal-api content reads, admin-storage object tooling, install
//! worker maintenance) resolves through this module. The pipeline is the only
//! place that knows about the three things a provider needs before it can serve
//! bytes:
//!
//! 1. **Credential source.** A provider reads its material from exactly one of
//!    two places: a reusable account-center entry (`provider_account_id`) or a
//!    local `credential_ref`. The account-center path has to be resolved here
//!    and not only on the admin surface - a provider created from the console
//!    carries `credential_ref = NULL`, and a store built from `credential_ref`
//!    alone would silently fall through to the process-wide
//!    `SDKWORK_DRIVE_S3_*` environment credentials. That is one tenant serving
//!    another tenant's objects, so the resolution is part of the contract
//!    rather than a per-surface detail.
//!
//! 2. **Location immutability.** A provider is not an immutable entity: its
//!    `name`, `status` and storage class are all expected to change during
//!    normal operation. Only the fields that decide *where the bytes live* -
//!    `endpoint_url`, `bucket`, `path_style` - are content-addressing inputs.
//!    Resolving a historical object therefore validates that the provider still
//!    points at the same physical location, instead of demanding that the whole
//!    row be untouched. Pinning the row version would make a harmless rename
//!    break reads of every object the provider already holds.
//!
//! 3. **State.** Reading historical bytes has to survive a provider that was
//!    disabled or soft-deleted after the bytes were written. New writes still
//!    require an `active` provider; historical reads report a precise,
//!    operational error instead of a misleading "not found".

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::RwLock;
use url::Url;

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

/// Upper bound on cached adapters.
///
/// A cache entry is keyed by `(provider, location fingerprint, credential
/// generation)`, so an estate with many providers stays well below this. The
/// bound exists to stop an unbounded map from growing under repeated credential
/// rotation, not to evict working providers.
const MAX_CACHED_PROVIDER_ADAPTERS: usize = 1024;

/// Physical location a provider addresses, used as the content-addressing
/// fingerprint.
///
/// Two providers that agree on these three fields address the same bytes, so a
/// change to any of them is the only thing that invalidates a historical
/// object's locator. Everything else about the provider is operational
/// metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DriveProviderLocation {
    pub endpoint_url: String,
    pub bucket: String,
    pub path_style: bool,
}

impl DriveProviderLocation {
    pub fn from_provider(provider: &DriveStorageProvider) -> Self {
        Self {
            endpoint_url: provider.endpoint_url.clone(),
            bucket: provider.bucket.clone(),
            path_style: provider.path_style,
        }
    }

    /// Build the location a historical locator was written against.
    pub fn from_locator(endpoint_url: &str, bucket: &str) -> Self {
        Self {
            endpoint_url: endpoint_url.to_string(),
            bucket: bucket.to_string(),
            path_style: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderAccessIntent {
    /// Minting new presigned URLs or accepting new bytes. Requires `active`.
    Write,
    /// Reading bytes that were written earlier. Any non-deleted provider is
    /// allowed; a `disabled` provider is an operational state, not a licence to
    /// hide data that still exists.
    Read,
}

/// Cache key of one provider adapter.
///
/// The third element is the *credential generation*: `0` for a provider that
/// carries its own `credential_ref`, and the account-center credential version
/// for a provider bound to a reusable account. It has to be part of the key -
/// rotating an account credential inserts a new credential row and leaves both
/// `dr_drive_storage_provider.version` and the account row untouched, so a
/// provider-version-only key would keep serving the rotated-away secret.
type ProviderCacheKey = (String, DriveProviderLocation, i64);
type ProviderObjectStore = Arc<dyn DriveObjectStore>;
type ProviderObjectStoreCache = Arc<RwLock<HashMap<ProviderCacheKey, ProviderObjectStore>>>;

/// Credential material a provider operates with, plus the generation it came
/// from.
pub struct ResolvedProviderCredentials {
    pub snapshot: DriveStorageCredentialSnapshot,
    pub credential_generation: i64,
}

#[derive(Clone)]
pub struct DriveProviderObjectStoreFactory {
    pool: PgPool,
    provider_store: SqlStorageProviderStore,
    cache: ProviderObjectStoreCache,
}

impl std::fmt::Debug for DriveProviderObjectStoreFactory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DriveProviderObjectStoreFactory")
            .finish_non_exhaustive()
    }
}

impl DriveProviderObjectStoreFactory {
    pub fn new(pool: PgPool) -> Self {
        Self {
            provider_store: SqlStorageProviderStore::new(pool.clone()),
            pool,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load a provider row by id, failing closed when it is unreadable.
    pub async fn load_provider(
        &self,
        provider_id: &str,
    ) -> Result<DriveStorageProvider, DriveObjectStoreError> {
        if provider_id.trim().is_empty() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "storage provider id is empty",
            ));
        }
        self.provider_store
            .find_storage_provider(provider_id)
            .await
            .map_err(map_service_error)?
            .ok_or_else(|| {
                DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::NotFound,
                    "storage provider was not found",
                )
            })
    }

    /// Load a provider and enforce the access intent of the caller.
    ///
    /// A `deleted` provider is unusable for both reading and writing: its
    /// bytes were scheduled for removal. A `disabled` provider may still be
    /// read from, because disabling is how an operator drains a provider while
    /// historical objects remain reachable.
    pub async fn load_provider_for(
        &self,
        provider_id: &str,
        intent: ProviderAccessIntent,
    ) -> Result<DriveStorageProvider, DriveObjectStoreError> {
        let provider = self.load_provider(provider_id).await?;
        ensure_provider_state_allows(&provider, intent)?;
        Ok(provider)
    }

    /// Resolve an adapter for a provider the caller already loaded, enforcing
    /// that the provider still addresses the location a historical locator was
    /// written against.
    pub async fn resolve_for_locator(
        &self,
        provider: &DriveStorageProvider,
        expected_bucket: &str,
        intent: ProviderAccessIntent,
    ) -> Result<Arc<dyn DriveObjectStore>, DriveObjectStoreError> {
        ensure_provider_state_allows(provider, intent)?;
        if provider.bucket != expected_bucket {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::Conflict,
                format!(
                    "storage provider {} no longer addresses bucket {}; the object was written to a different location",
                    provider.id, expected_bucket
                ),
            ));
        }
        self.resolve(provider).await
    }

    /// Resolve an adapter for a provider by id, enforcing the access intent.
    pub async fn resolve_by_id(
        &self,
        provider_id: &str,
        intent: ProviderAccessIntent,
    ) -> Result<Arc<dyn DriveObjectStore>, DriveObjectStoreError> {
        let provider = self.load_provider_for(provider_id, intent).await?;
        self.resolve(&provider).await
    }

    /// Resolve (and cache) the adapter for a concrete provider row.
    pub async fn resolve(
        &self,
        provider: &DriveStorageProvider,
    ) -> Result<Arc<dyn DriveObjectStore>, DriveObjectStoreError> {
        let credentials = self.resolve_credentials(provider).await?;
        let cache_key = (
            provider.id.clone(),
            DriveProviderLocation::from_provider(provider),
            credentials
                .as_ref()
                .map(|resolved| resolved.credential_generation)
                .unwrap_or(0),
        );
        if let Some(store) = self.cache.read().await.get(&cache_key).cloned() {
            return Ok(store);
        }

        let store = build_store(provider, credentials.as_ref()).await?;

        let mut cache = self.cache.write().await;
        if let Some(existing) = cache.get(&cache_key).cloned() {
            return Ok(existing);
        }
        // Drop adapters for the same provider whose location or credential
        // generation has moved on: they can never be asked for again, because
        // the key is recomputed from the live row on every resolution.
        cache.retain(|(cached_provider_id, _, _), _| cached_provider_id != &provider.id);
        if cache.len() < MAX_CACHED_PROVIDER_ADAPTERS {
            cache.insert(cache_key, store.clone());
        }
        Ok(store)
    }

    /// Resolve the credential material of a provider.
    ///
    /// Returns `None` when the provider carries its own `credential_ref`, which
    /// the S3 config builder resolves lazily. A provider bound to an account
    /// center entry returns the decrypted material and the generation it came
    /// from, so a rotation invalidates the cached adapter.
    pub async fn resolve_credentials(
        &self,
        provider: &DriveStorageProvider,
    ) -> Result<Option<ResolvedProviderCredentials>, DriveObjectStoreError> {
        let Some(account_id) = provider
            .provider_account_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };

        let material = resolve_bound_credential_material(
            &self.pool,
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

        Ok(Some(ResolvedProviderCredentials {
            snapshot: DriveStorageCredentialSnapshot {
                access_key_id,
                secret_access_key,
                session_token: material.session_token,
            },
            credential_generation: material.credential_version,
        }))
    }
}

/// Enforce the provider state a given access intent requires.
pub fn ensure_provider_state_allows(
    provider: &DriveStorageProvider,
    intent: ProviderAccessIntent,
) -> Result<(), DriveObjectStoreError> {
    match (provider.status.as_str(), intent) {
        ("active", _) => Ok(()),
        ("disabled", ProviderAccessIntent::Read) => Ok(()),
        ("disabled", ProviderAccessIntent::Write) => Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::Conflict,
            format!(
                "storage provider {} is disabled; new content cannot be written to it",
                provider.id
            ),
        )),
        ("deleted", _) => Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::NotFound,
            format!("storage provider {} was deleted", provider.id),
        )),
        (other, _) => Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::Internal,
            format!(
                "storage provider {} has an unknown status: {other}",
                provider.id
            ),
        )),
    }
}

/// Build the provider-neutral object store for a provider row.
///
/// Branching is on the *protocol*, not on a vendor list: every catalog kind
/// except the local filesystem speaks the S3 API, so a newly catalogued vendor
/// is served here automatically instead of silently falling through.
pub async fn build_store(
    provider: &DriveStorageProvider,
    credentials: Option<&ResolvedProviderCredentials>,
) -> Result<Arc<dyn DriveObjectStore>, DriveObjectStoreError> {
    if matches!(
        provider.provider_kind,
        DriveStorageProviderKind::LocalFilesystem
    ) {
        let store: Arc<dyn DriveObjectStore> = Arc::new(build_local_store_for_provider(provider)?);
        return Ok(store);
    }

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
    let store: Arc<dyn DriveObjectStore> = Arc::new(S3DriveObjectStore::new(config).await?);
    Ok(store)
}

/// Whether this provider kind is served by the S3 object store.
///
/// Delegates to the storage contract so a newly catalogued vendor is covered
/// the moment it is added there, instead of needing a second edit per surface.
pub fn provider_supports_s3_object_store(provider_kind: &DriveStorageProviderKind) -> bool {
    provider_kind.is_s3_compatible()
}

/// Build the local-filesystem object store for a provider row.
///
/// Admin bucket/object management routes reuse this so they share one
/// root-resolution rule with the runtime: the provider endpoint must be a
/// local `file://` URL.
pub fn build_local_store_for_provider(
    provider: &DriveStorageProvider,
) -> Result<LocalDriveObjectStore, DriveObjectStoreError> {
    let root = local_root_from_endpoint(&provider.endpoint_url)?;
    Ok(LocalDriveObjectStore::new(root))
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

pub fn map_provider_account_error(error: ProviderAccountError) -> DriveObjectStoreError {
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

pub fn map_service_error(
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

    fn provider(status: &str) -> DriveStorageProvider {
        DriveStorageProvider {
            id: "prv_1".to_string(),
            tenant_id: "100001".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "primary".to_string(),
            endpoint_url: "https://s3.example.test".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "sdkwork-drive".to_string(),
            path_style: true,
            strict_tls: true,
            credential_ref: Some("env:A:B".to_string()),
            provider_account_id: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: status.to_string(),
            version: 1,
        }
    }

    #[test]
    fn disabled_provider_is_readable_but_not_writable() {
        assert!(
            ensure_provider_state_allows(&provider("disabled"), ProviderAccessIntent::Read).is_ok()
        );
        assert!(
            ensure_provider_state_allows(&provider("disabled"), ProviderAccessIntent::Write)
                .is_err()
        );
    }

    #[test]
    fn deleted_provider_is_neither_readable_nor_writable() {
        for intent in [ProviderAccessIntent::Read, ProviderAccessIntent::Write] {
            assert!(ensure_provider_state_allows(&provider("deleted"), intent).is_err());
        }
    }

    #[test]
    fn active_provider_allows_both_intents() {
        for intent in [ProviderAccessIntent::Read, ProviderAccessIntent::Write] {
            assert!(ensure_provider_state_allows(&provider("active"), intent).is_ok());
        }
    }

    #[test]
    fn location_fingerprint_ignores_operational_fields() {
        let base = provider("active");
        let mut renamed = base.clone();
        renamed.name = "renamed".to_string();
        renamed.status = "disabled".to_string();
        renamed.default_storage_class = Some("STANDARD_IA".to_string());
        assert_eq!(
            DriveProviderLocation::from_provider(&base),
            DriveProviderLocation::from_provider(&renamed)
        );

        let mut relocated = base.clone();
        relocated.bucket = "other-bucket".to_string();
        assert_ne!(
            DriveProviderLocation::from_provider(&base),
            DriveProviderLocation::from_provider(&relocated)
        );
    }

    #[test]
    fn local_endpoint_rejects_non_file_and_remote_file_urls() {
        assert!(local_root_from_endpoint("https://example.test/storage").is_err());
        assert!(local_root_from_endpoint("file://remote-host/storage").is_err());
        let local_url = Url::from_directory_path(std::env::temp_dir())
            .expect("system temp directory should convert to a file URL");
        assert!(local_root_from_endpoint(local_url.as_str()).is_ok());
    }
}
