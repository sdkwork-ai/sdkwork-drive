use async_trait::async_trait;

use crate::domain::storage_provider::DriveStorageProvider;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct NewDriveStorageProvider {
    pub id: String,
    pub tenant_id: String,
    pub provider_kind: String,
    pub name: String,
    pub endpoint_url: String,
    pub region: Option<String>,
    pub bucket: String,
    pub path_style: bool,
    pub strict_tls: bool,
    pub credential_ref: Option<String>,
    pub provider_account_id: Option<String>,
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
    pub provider_account_id: Option<String>,
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

    /// List providers inside one window.
    ///
    /// `provider_kind` and `status` are *store-level* predicates, not
    /// post-filters: the console's provider-kind switch has to compose with the
    /// cursor window, so a kind filter that only matched rows on a later page
    /// must not be able to answer "empty" for an earlier one. `provider_kind`
    /// accepts a stored kind and additionally treats `custom` as the whole
    /// `custom:<vendor>` family.
    async fn list_storage_providers(
        &self,
        tenant_id: &str,
        provider_kind: Option<&str>,
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

    /// Count live objects that still resolve to this provider.
    ///
    /// A provider is not free to retire while it is still the physical home of
    /// live bytes: disabling or soft-deleting it would strand exactly the
    /// objects the locator contract promises to keep readable. Binding checks do
    /// not cover this, because a binding is a *routing* rule while
    /// `dr_drive_storage_object` rows are the historical record of where bytes
    /// actually live. Both have to be clear before a provider can be retired.
    ///
    /// Only `active` objects count: a soft-deleted object was already marked for
    /// removal and must not keep a provider alive forever.
    async fn count_active_storage_provider_objects(
        &self,
        provider_id: &str,
    ) -> Result<i64, DriveServiceError>;

    /// Resolve the single active provider of a tenant that addresses a bucket.
    ///
    /// Buckets are a *storage-level* namespace shared by every tenant on the
    /// same object store account, so `bucket` alone is not an identifier: two
    /// tenants can legitimately use `sdkwork-drive`, and resolving without a
    /// tenant filter would hand one tenant the other tenant's endpoint and
    /// credentials. Every lookup is therefore tenant-scoped.
    ///
    /// Within a tenant the mapping has to be a function, not a lottery. If two
    /// active providers of the same tenant address the same bucket, picking one
    /// by `updated_at` means a routine provider rename silently redirects
    /// subsequent writes to a different physical home. That is reported as
    /// [`DriveStorageProviderLookup::Ambiguous`] so the caller can fail loudly
    /// instead of sprouting a second, unreachable copy of the same object key.
    async fn find_active_storage_provider_by_bucket(
        &self,
        tenant_id: &str,
        bucket: &str,
    ) -> Result<DriveStorageProviderLookup, DriveServiceError>;

    async fn delete_storage_provider(&self, provider_id: &str) -> Result<bool, DriveServiceError>;
}

/// Outcome of resolving a `(tenant, bucket)` pair to a storage provider.
///
/// A three-way result rather than `Option` because "no provider" and "more than
/// one provider" are different operational problems with different remedies,
/// and collapsing both into `None` produces the useless "bucket is required"
/// message for an ambiguous configuration.
#[derive(Debug, Clone)]
pub enum DriveStorageProviderLookup {
    /// Exactly one active provider of the tenant addresses the bucket.
    Found(DriveStorageProvider),
    /// No active provider of the tenant addresses the bucket.
    Missing,
    /// Two or more active providers of the tenant address the bucket, so the
    /// resolution has no defined answer. Carries the competing provider ids so
    /// the error names the rows an operator has to disambiguate.
    Ambiguous(Vec<String>),
}

impl DriveStorageProviderLookup {
    /// The provider, or `None` for both `Missing` and `Ambiguous`.
    pub fn provider(self) -> Option<DriveStorageProvider> {
        match self {
            Self::Found(provider) => Some(provider),
            Self::Missing | Self::Ambiguous(_) => None,
        }
    }
}
