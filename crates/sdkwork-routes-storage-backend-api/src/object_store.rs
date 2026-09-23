use crate::config::{AdminStorageConfig, DriveAdminStorageObjectStoreAdapter};
use crate::error::{
    map_object_store_route_error, map_provider_account_error, map_service_error, ProblemDetail,
};
use crate::state::AdminStorageState;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_storage_contract::{DriveObjectStore, DriveStorageCredentialSnapshot};
#[cfg(not(feature = "opendal-s3-plugin"))]
use sdkwork_drive_storage_contract::{DriveObjectStoreError, DriveObjectStoreErrorKind};
#[cfg(feature = "opendal-s3-plugin")]
use sdkwork_drive_storage_opendal::{
    OpendalS3DriveObjectStore, OpendalS3ProviderParts, OpendalS3StoreConfig,
};
use sdkwork_drive_storage_s3::{S3DriveObjectStore, S3StoreConfig};
use sdkwork_drive_workspace_service::domain::storage_provider::{
    DriveStorageProvider, DriveStorageProviderKind,
};
use sdkwork_drive_workspace_service::DriveServiceError;
use sdkwork_iam_provider_account_service::{
    resolve_bound_credential_material, CREDENTIAL_KIND_ACCESS_KEY_PAIR,
};

/// Resolve the credential material a provider currently operates with.
///
/// Account-backed providers never store credential material locally: the
/// reusable service-provider account is the single source, so the material is
/// decrypted from the account center on every store construction. An account
/// rotation therefore takes effect for every consumer at once, which is the
/// reason providers reference accounts instead of embedding per-consumer
/// credential copies.
///
/// The lookup goes through the *pinned-reference* path rather than the
/// `user -> tenant -> platform` walk: the provider already names its account,
/// so walking would be wrong. It also has to be a pinned lookup because a
/// platform-scope account lives in the platform tenant (`100001`), which is not
/// the provider's own tenant and would therefore never match a tenant-filtered
/// read. The reference is authorised as "platform-wide, or owned by the
/// provider's tenant"; the wider per-operator and per-scope check happens where
/// the reference is written ([`crate::provider_handlers`]).
pub(crate) async fn resolve_provider_credentials(
    state: &AdminStorageState,
    provider: &DriveStorageProvider,
) -> Result<Option<DriveStorageCredentialSnapshot>, (StatusCode, Json<ProblemDetail>)> {
    let Some(account_id) = provider.provider_account_id.as_deref() else {
        return Ok(None);
    };
    let material = resolve_bound_credential_material(
        &state.pool,
        &provider.tenant_id,
        None,
        account_id,
        Some(CREDENTIAL_KIND_ACCESS_KEY_PAIR),
    )
    .await
    .map_err(map_provider_account_error)?;
    match (
        material.access_key_id,
        material.secret_access_key,
        material.session_token,
    ) {
        (Some(access_key_id), Some(secret_access_key), session_token) => {
            Ok(Some(DriveStorageCredentialSnapshot {
                access_key_id,
                secret_access_key,
                session_token,
            }))
        }
        _ => Err(map_service_error(DriveServiceError::Conflict(
            "provider account carries no usable access-key-pair credential for object storage"
                .to_string(),
        ))),
    }
}

pub(crate) async fn build_object_store_for_provider(
    state: &AdminStorageState,
    provider: &DriveStorageProvider,
) -> Result<Box<dyn DriveObjectStore>, (StatusCode, Json<ProblemDetail>)> {
    if !provider_supports_s3_object_store(&provider.provider_kind) {
        return Err(map_service_error(DriveServiceError::Conflict(
            "storage provider does not support s3-compatible object store operations".to_string(),
        )));
    }
    let credentials = resolve_provider_credentials(state, provider).await?;

    match state.config.object_store_adapter {
        DriveAdminStorageObjectStoreAdapter::AwsSdkS3 => {
            let boxed: Box<dyn DriveObjectStore> =
                Box::new(build_aws_sdk_object_store(provider, credentials.as_ref()).await?);
            Ok(boxed)
        }
        DriveAdminStorageObjectStoreAdapter::OpendalS3 => {
            build_opendal_object_store_for_provider(&state.config, provider, credentials.as_ref())
                .await
        }
    }
}

pub(crate) async fn build_full_s3_object_store_for_provider(
    state: &AdminStorageState,
    provider: &DriveStorageProvider,
) -> Result<S3DriveObjectStore, (StatusCode, Json<ProblemDetail>)> {
    if !provider_supports_s3_object_store(&provider.provider_kind) {
        return Err(map_service_error(DriveServiceError::Conflict(
            "storage provider does not support s3-compatible object store operations".to_string(),
        )));
    }
    let credentials = resolve_provider_credentials(state, provider).await?;
    build_aws_sdk_object_store(provider, credentials.as_ref()).await
}

async fn build_aws_sdk_object_store(
    provider: &DriveStorageProvider,
    credentials: Option<&DriveStorageCredentialSnapshot>,
) -> Result<S3DriveObjectStore, (StatusCode, Json<ProblemDetail>)> {
    let config = match credentials {
        Some(credentials) => S3StoreConfig::from_provider_parts_with_credentials(
            provider.provider_kind.as_str(),
            &provider.endpoint_url,
            provider.region.as_deref(),
            &provider.bucket,
            provider.path_style,
            credentials.clone(),
            Some(provider.strict_tls),
        )
        .map_err(map_object_store_route_error)?,
        None => S3StoreConfig::from_provider_parts(
            provider.provider_kind.as_str(),
            &provider.endpoint_url,
            provider.region.as_deref(),
            &provider.bucket,
            provider.path_style,
            provider.credential_ref.as_deref(),
            Some(provider.strict_tls),
        )
        .map_err(map_object_store_route_error)?,
    };
    S3DriveObjectStore::new(config)
        .await
        .map_err(map_object_store_route_error)
}

#[cfg(feature = "opendal-s3-plugin")]
async fn build_opendal_object_store_for_provider(
    _config: &AdminStorageConfig,
    provider: &DriveStorageProvider,
    credentials: Option<&DriveStorageCredentialSnapshot>,
) -> Result<Box<dyn DriveObjectStore>, (StatusCode, Json<ProblemDetail>)> {
    let store = OpendalS3DriveObjectStore::new(
        OpendalS3StoreConfig::from_provider_parts(OpendalS3ProviderParts {
            provider_kind: provider.provider_kind.as_str(),
            endpoint_url: &provider.endpoint_url,
            region: provider.region.as_deref(),
            default_bucket: &provider.bucket,
            force_path_style: Some(provider.path_style),
            credential_ref: provider.credential_ref.as_deref(),
            credentials,
            root: None,
            server_side_encryption: provider.server_side_encryption_mode.as_deref(),
            default_storage_class: provider.default_storage_class.as_deref(),
            strict_tls_override: Some(provider.strict_tls),
        })
        .map_err(map_object_store_route_error)?,
    )
    .map_err(map_object_store_route_error)?;
    let boxed: Box<dyn DriveObjectStore> = Box::new(store);
    Ok(boxed)
}

#[cfg(not(feature = "opendal-s3-plugin"))]
async fn build_opendal_object_store_for_provider(
    _config: &AdminStorageConfig,
    _provider: &DriveStorageProvider,
    _credentials: Option<&DriveStorageCredentialSnapshot>,
) -> Result<Box<dyn DriveObjectStore>, (StatusCode, Json<ProblemDetail>)> {
    Err(map_object_store_route_error(DriveObjectStoreError::new(
        DriveObjectStoreErrorKind::NotSupported,
        "OpenDAL S3 plugin is not enabled; build sdkwork-routes-storage-backend-api with the opendal-s3-plugin feature to use this adapter",
    )))
}

pub(crate) fn provider_supports_s3_object_store(provider_kind: &DriveStorageProviderKind) -> bool {
    matches!(
        provider_kind,
        DriveStorageProviderKind::S3Compatible
            | DriveStorageProviderKind::AliyunOss
            | DriveStorageProviderKind::TencentCos
            | DriveStorageProviderKind::HuaweiObs
            | DriveStorageProviderKind::VolcengineTos
            | DriveStorageProviderKind::GoogleCloudStorage
            | DriveStorageProviderKind::Custom(_)
    )
}
