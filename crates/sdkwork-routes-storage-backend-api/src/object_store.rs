use crate::config::{AdminStorageConfig, DriveAdminStorageObjectStoreAdapter};
use crate::error::{map_object_store_route_error, map_service_error, ProblemDetail};
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_storage_contract::{
    DriveObjectStore, DriveObjectStoreError, DriveObjectStoreErrorKind,
};
#[cfg(feature = "opendal-s3-plugin")]
use sdkwork_drive_storage_opendal::{
    OpendalS3DriveObjectStore, OpendalS3ProviderParts, OpendalS3StoreConfig,
};
use sdkwork_drive_storage_s3::{S3DriveObjectStore, S3StoreConfig};
use sdkwork_drive_workspace_service::domain::storage_provider::{
    DriveStorageProvider, DriveStorageProviderKind,
};
use sdkwork_drive_workspace_service::DriveServiceError;

pub(crate) async fn build_object_store_for_provider(
    config: &AdminStorageConfig,
    provider: &DriveStorageProvider,
) -> Result<Box<dyn DriveObjectStore>, (StatusCode, Json<ProblemDetail>)> {
    if !provider_supports_s3_object_store(&provider.provider_kind) {
        return Err(map_service_error(DriveServiceError::Conflict(
            "storage provider does not support s3-compatible object store operations".to_string(),
        )));
    }

    match config.object_store_adapter {
        DriveAdminStorageObjectStoreAdapter::AwsSdkS3 => S3DriveObjectStore::new(
            S3StoreConfig::from_provider_parts(
                provider.provider_kind.as_str(),
                &provider.endpoint_url,
                provider.region.as_deref(),
                &provider.bucket,
                provider.path_style,
                provider.credential_ref.as_deref(),
                Some(provider.strict_tls),
            )
            .map_err(map_object_store_route_error)?,
        )
        .await
        .map(|store| Box::new(store) as Box<dyn DriveObjectStore>)
        .map_err(map_object_store_route_error),
        DriveAdminStorageObjectStoreAdapter::OpendalS3 => {
            build_opendal_object_store_for_provider(provider).await
        }
    }
}

pub(crate) async fn build_full_s3_object_store_for_provider(
    provider: &DriveStorageProvider,
) -> Result<S3DriveObjectStore, (StatusCode, Json<ProblemDetail>)> {
    if !provider_supports_s3_object_store(&provider.provider_kind) {
        return Err(map_service_error(DriveServiceError::Conflict(
            "storage provider does not support s3-compatible object store operations".to_string(),
        )));
    }
    S3DriveObjectStore::new(
        S3StoreConfig::from_provider_parts(
            provider.provider_kind.as_str(),
            &provider.endpoint_url,
            provider.region.as_deref(),
            &provider.bucket,
            provider.path_style,
            provider.credential_ref.as_deref(),
            Some(provider.strict_tls),
        )
        .map_err(map_object_store_route_error)?,
    )
    .await
    .map_err(map_object_store_route_error)
}

#[cfg(feature = "opendal-s3-plugin")]
async fn build_opendal_object_store_for_provider(
    provider: &DriveStorageProvider,
) -> Result<Box<dyn DriveObjectStore>, (StatusCode, Json<ProblemDetail>)> {
    OpendalS3DriveObjectStore::new(
        OpendalS3StoreConfig::from_provider_parts(OpendalS3ProviderParts {
            provider_kind: provider.provider_kind.as_str(),
            endpoint_url: &provider.endpoint_url,
            region: provider.region.as_deref(),
            default_bucket: &provider.bucket,
            force_path_style: Some(provider.path_style),
            credential_ref: provider.credential_ref.as_deref(),
            root: None,
            server_side_encryption: provider.server_side_encryption_mode.as_deref(),
            default_storage_class: provider.default_storage_class.as_deref(),
            strict_tls_override: Some(provider.strict_tls),
        })
        .map_err(map_object_store_route_error)?,
    )
    .map(|store| Box::new(store) as Box<dyn DriveObjectStore>)
    .map_err(map_object_store_route_error)
}

#[cfg(not(feature = "opendal-s3-plugin"))]
async fn build_opendal_object_store_for_provider(
    _provider: &DriveStorageProvider,
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
