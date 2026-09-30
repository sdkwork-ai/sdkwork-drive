use crate::error::map_object_store_error;
use crate::state::AppState;
use crate::time::signing_ttl_seconds;
use async_trait::async_trait;
use sdkwork_drive_object_runtime::{
    provider_supports_s3_object_store as provider_supports_s3_object_store_shared,
    DriveObjectStoreRuntime, ProviderAccessIntent,
};
use sdkwork_drive_storage_contract::{
    DriveObjectLocator, DriveObjectStore, PresignDownloadRequest,
    PresignUploadPartRequest as ObjectStorePresignUploadPartRequest,
};
use sdkwork_drive_workspace_service::application::download_service::DriveDownloadService;
use sdkwork_drive_workspace_service::domain::storage_provider::{
    DriveStorageProvider, DriveStorageProviderKind,
};
use sdkwork_drive_workspace_service::infrastructure::sql::storage_object_store::SqlStorageObjectStore;
use sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_store::SqlStorageProviderStore;
use sdkwork_drive_workspace_service::ports::storage_object_store::{
    DownloadSignCommand, DriveDownloadSigner, SignedDownloadPayload,
};
use sdkwork_drive_workspace_service::ports::storage_provider_store::{
    DriveStorageProviderLookup, DriveStorageProviderStore,
};
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::PgPool;
use sqlx::Row;
use std::collections::BTreeMap;
use std::sync::Arc;

pub(crate) fn build_download_service(
    state: &AppState,
) -> DriveDownloadService<SqlStorageObjectStore, AppDownloadSigner> {
    DriveDownloadService::new(
        SqlStorageObjectStore::new(state.pool.clone()),
        AppDownloadSigner::new(state.pool.clone()),
    )
}
#[derive(Debug, Clone)]
pub(crate) struct AppDownloadSigner {
    object_runtime: DriveObjectStoreRuntime,
}

#[derive(Debug, Clone)]
pub(crate) struct UploadPartSignCommand {
    pub(crate) storage_provider_id: String,
    pub(crate) bucket: String,
    pub(crate) object_key: String,
    pub(crate) upload_id: String,
    pub(crate) part_no: u16,
    pub(crate) expires_at_epoch_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SignedUploadPartPayload {
    pub(crate) method: String,
    pub(crate) raw_url: String,
    pub(crate) headers: BTreeMap<String, String>,
    pub(crate) expires_at_epoch_ms: i64,
}

impl AppDownloadSigner {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self {
            object_runtime: DriveObjectStoreRuntime::new(pool),
        }
    }

    /// Resolve the S3 adapter a signing command addresses.
    ///
    /// Signing is a write-path grant (it mints a URL that can create or read
    /// bytes), so the provider must be `active`. Resolution goes through the
    /// shared runtime so an account-backed provider signs with the account's
    /// credential rather than the process-wide environment fallback.
    async fn resolve_signing_store(
        &self,
        provider_id: &str,
        bucket: &str,
    ) -> Result<Arc<dyn DriveObjectStore>, DriveServiceError> {
        let store = self
            .object_runtime
            .factory()
            .resolve_by_id(provider_id, ProviderAccessIntent::Write)
            .await
            .map_err(map_object_store_error)?;
        let provider = self
            .object_runtime
            .load_provider(provider_id)
            .await
            .map_err(map_object_store_error)?;
        if provider.bucket != bucket {
            return Err(missing_signing_provider_error(bucket));
        }
        Ok(store)
    }

    pub(crate) async fn sign_upload_part(
        &self,
        command: UploadPartSignCommand,
    ) -> Result<SignedUploadPartPayload, DriveServiceError> {
        let object_store = self
            .resolve_signing_store(&command.storage_provider_id, &command.bucket)
            .await?;
        if !provider_supports_s3_object_store(&object_store.provider_kind()) {
            return Err(unsupported_signing_provider_error(&command.bucket));
        }
        let ttl_seconds = signing_ttl_seconds(command.expires_at_epoch_ms)?;
        let signed = object_store
            .presign_upload_part(ObjectStorePresignUploadPartRequest {
                locator: DriveObjectLocator {
                    bucket: command.bucket,
                    object_key: command.object_key,
                },
                upload_id: command.upload_id,
                part_number: command.part_no,
                expires_in_seconds: ttl_seconds,
            })
            .await
            .map_err(map_object_store_error)?;
        Ok(SignedUploadPartPayload {
            method: signed.method,
            raw_url: signed.url,
            headers: signed.headers,
            expires_at_epoch_ms: signed.expires_at_epoch_ms,
        })
    }
}

#[async_trait]
impl DriveDownloadSigner for AppDownloadSigner {
    async fn sign_download(
        &self,
        command: DownloadSignCommand,
    ) -> Result<SignedDownloadPayload, DriveServiceError> {
        let object_store = self
            .resolve_signing_store(&command.storage_provider_id, &command.bucket)
            .await?;
        if !provider_supports_s3_object_store(&object_store.provider_kind()) {
            return Err(unsupported_signing_provider_error(&command.bucket));
        }
        let ttl_seconds = signing_ttl_seconds(command.expires_at_epoch_ms)?;
        let signed = object_store
            .presign_download(PresignDownloadRequest {
                locator: DriveObjectLocator {
                    bucket: command.bucket,
                    object_key: command.object_key,
                },
                expires_in_seconds: ttl_seconds,
            })
            .await
            .map_err(map_object_store_error)?;
        Ok(SignedDownloadPayload {
            method: signed.method,
            raw_url: signed.url,
            headers: signed.headers,
            expires_at_epoch_ms: signed.expires_at_epoch_ms,
        })
    }
}

pub(crate) fn missing_signing_provider_error(bucket: &str) -> DriveServiceError {
    DriveServiceError::Conflict(format!(
        "active storage provider is required for bucket {bucket} to sign object store URLs"
    ))
}

pub(crate) fn unsupported_signing_provider_error(bucket: &str) -> DriveServiceError {
    DriveServiceError::Conflict(format!(
        "active storage provider for bucket {bucket} does not support object store URL signing"
    ))
}

#[derive(Debug, Clone)]
pub(crate) struct ActiveStorageProviderRecord {
    pub(crate) id: String,
    pub(crate) provider_kind: DriveStorageProviderKind,
    pub(crate) endpoint_url: String,
    pub(crate) region: Option<String>,
    pub(crate) bucket: String,
    pub(crate) status: String,
    pub(crate) path_style: bool,
    pub(crate) strict_tls: bool,
    pub(crate) credential_ref: Option<String>,
    /// Reusable service-provider account the credential is read from.
    ///
    /// Mutually exclusive with `credential_ref`. It has to travel with the
    /// record because an account-backed provider stores no local material: a
    /// store built without this field falls through to the process-wide
    /// `SDKWORK_DRIVE_S3_*` credentials, which is one tenant serving another
    /// tenant's objects.
    pub(crate) provider_account_id: Option<String>,
    pub(crate) tenant_id: String,
}

/// Resolve the active provider a bucket name refers to for one tenant.
///
/// Delegates to the canonical tenant-scoped resolver rather than carrying a
/// local `SELECT ... LIMIT 1`. Two defects used to live in that local query:
/// it filtered on `bucket` without `tenant_id` (buckets are a storage-level
/// namespace, so a foreign tenant's provider could be picked up), and it broke
/// ties on `updated_at DESC` (a provider rename could silently repoint writes).
pub(crate) async fn find_active_storage_provider_by_bucket(
    pool: &PgPool,
    tenant_id: &str,
    bucket: &str,
) -> Result<Option<ActiveStorageProviderRecord>, DriveServiceError> {
    let store = SqlStorageProviderStore::new(pool.clone());
    match store
        .find_active_storage_provider_by_bucket(tenant_id, bucket)
        .await?
    {
        DriveStorageProviderLookup::Found(provider) => Ok(Some(
            ActiveStorageProviderRecord::from_domain_provider(provider),
        )),
        DriveStorageProviderLookup::Missing => Ok(None),
        DriveStorageProviderLookup::Ambiguous(provider_ids) => {
            Err(ambiguous_provider_error(tenant_id, bucket, &provider_ids))
        }
    }
}

/// Report a `(tenant, bucket)` pair that resolves to more than one provider.
///
/// Failing loudly is the point: silently choosing one would publish objects
/// under a key that another active provider also claims, producing two copies
/// of the same logical object and a read that depends on which row happened to
/// sort first.
pub(crate) fn ambiguous_provider_error(
    tenant_id: &str,
    bucket: &str,
    provider_ids: &[String],
) -> DriveServiceError {
    DriveServiceError::Conflict(format!(
        "bucket {bucket} of tenant {tenant_id} is addressed by {} active storage providers ({}); \
         disable or repoint all but one before writing to this bucket",
        provider_ids.len(),
        provider_ids.join(", ")
    ))
}

pub(crate) async fn find_storage_provider_by_id(
    pool: &PgPool,
    provider_id: &str,
) -> Result<Option<ActiveStorageProviderRecord>, DriveServiceError> {
    let row = sqlx::query(
        "SELECT id, tenant_id, provider_kind, endpoint_url, region, bucket, status, path_style,
                strict_tls, credential_ref, provider_account_id
         FROM dr_drive_storage_provider
         WHERE id=$1
         LIMIT 1",
    )
    .bind(provider_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "query dr_drive_storage_provider by id failed: {error}"
        ))
    })?;

    let Some(row) = row else {
        return Ok(None);
    };
    let status: String = row.get("status");
    map_active_provider_row(&row, &status).map(Some)
}

/// Decode one `dr_drive_storage_provider` row into the record the app surface
/// builds signers and stores from.
fn map_active_provider_row(
    row: &sqlx::postgres::PgRow,
    status: &str,
) -> Result<ActiveStorageProviderRecord, DriveServiceError> {
    let raw_kind: String = row.get("provider_kind");
    let provider_kind = DriveStorageProviderKind::try_from_str(&raw_kind).ok_or_else(|| {
        DriveServiceError::Internal(format!("storage provider kind is invalid: {raw_kind}"))
    })?;
    Ok(ActiveStorageProviderRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        provider_kind,
        endpoint_url: row.get("endpoint_url"),
        region: row.get("region"),
        bucket: row.get("bucket"),
        status: status.to_string(),
        path_style: get_bool(row, "path_style")?,
        strict_tls: get_bool(row, "strict_tls")?,
        credential_ref: row.get("credential_ref"),
        provider_account_id: row.get("provider_account_id"),
    })
}

impl ActiveStorageProviderRecord {
    /// Project a domain provider onto this app-local record.
    ///
    /// The canonical resolver returns the domain type; the app surface keeps
    /// its own record so the row decoders stay in one place, so the two have to
    /// meet here instead of every caller re-deriving the fields.
    pub(crate) fn from_domain_provider(provider: DriveStorageProvider) -> Self {
        Self {
            id: provider.id,
            provider_kind: provider.provider_kind,
            endpoint_url: provider.endpoint_url,
            region: provider.region,
            bucket: provider.bucket,
            status: provider.status,
            path_style: provider.path_style,
            strict_tls: provider.strict_tls,
            credential_ref: provider.credential_ref,
            provider_account_id: provider.provider_account_id,
            tenant_id: provider.tenant_id,
        }
    }

    /// Project the record into the domain provider the shared runtime consumes.
    ///
    /// The runtime keys its adapter cache and its location check on provider
    /// identity, so it needs the domain type rather than the app-local record.
    pub(crate) fn to_domain_provider(&self) -> DriveStorageProvider {
        DriveStorageProvider {
            id: self.id.clone(),
            tenant_id: self.tenant_id.clone(),
            provider_kind: self.provider_kind.clone(),
            name: self.id.clone(),
            endpoint_url: self.endpoint_url.clone(),
            region: self.region.clone(),
            bucket: self.bucket.clone(),
            path_style: self.path_style,
            strict_tls: self.strict_tls,
            credential_ref: self.credential_ref.clone(),
            provider_account_id: self.provider_account_id.clone(),
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: self.status.clone(),
            version: 1,
        }
    }
}

pub(crate) fn require_active_storage_provider(
    provider: ActiveStorageProviderRecord,
    bucket: &str,
) -> Result<ActiveStorageProviderRecord, DriveServiceError> {
    if provider.status == "active" {
        Ok(provider)
    } else {
        Err(missing_signing_provider_error(bucket))
    }
}

fn get_bool(row: &sqlx::postgres::PgRow, column: &str) -> Result<bool, DriveServiceError> {
    row.try_get::<bool, _>(column)
        .or_else(|_| row.try_get::<i64, _>(column).map(|value| value != 0))
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "decode dr_drive_storage_provider.{column} as bool failed: {error}"
            ))
        })
}

/// Resolve the S3 adapter a provider row addresses.
///
/// Delegates to the shared provider runtime so the app surface cannot drift from
/// the pipeline the rest of Drive uses. That pipeline is what resolves an
/// account-center credential for a provider whose `credential_ref` is NULL -
/// building an S3 config straight from `credential_ref` here used to fall
/// through to the process-wide `SDKWORK_DRIVE_S3_*` credentials instead.
///
/// Returns `None` for the local filesystem provider kind, which the shared
/// runtime still serves but which does not speak the S3 protocol.
pub(crate) async fn build_s3_object_store_for_provider(
    state: &AppState,
    provider: &DriveStorageProvider,
) -> Result<Option<Arc<dyn DriveObjectStore>>, DriveServiceError> {
    if !provider_supports_s3_object_store(&provider.provider_kind) {
        return Ok(None);
    }
    state
        .object_runtime
        .factory()
        .resolve(provider)
        .await
        .map(Some)
        .map_err(map_object_store_error)
}

/// Whether this kind is served by the S3 object store.
///
/// Delegates to the shared provider runtime, which in turn delegates to the
/// storage contract, so a newly catalogued vendor is covered the moment it is
/// added there instead of needing a second edit per surface.
fn provider_supports_s3_object_store(provider_kind: &DriveStorageProviderKind) -> bool {
    provider_supports_s3_object_store_shared(provider_kind)
}
