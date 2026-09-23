use crate::dto::{
    StorageProviderAccountResponse, StorageProviderCapabilitiesResponse, StorageProviderResponse,
};
use sdkwork_drive_workspace_service::application::storage_provider_service::StorageProviderCapabilities;
use sdkwork_drive_workspace_service::domain::storage_provider::{
    DriveStorageProvider, DriveStorageProviderKind,
};
use sdkwork_drive_workspace_service::DriveServiceError;
use sdkwork_iam_provider_account_service::ProviderAccount;

pub(crate) fn map_storage_provider(provider: DriveStorageProvider) -> StorageProviderResponse {
    let credential_configured =
        provider.credential_ref.is_some() || provider.provider_account_id.is_some();
    StorageProviderResponse {
        id: provider.id,
        provider_kind: provider.provider_kind.as_str().to_string(),
        name: provider.name,
        endpoint_url: provider.endpoint_url,
        region: provider.region,
        bucket: provider.bucket,
        path_style: provider.path_style,
        strict_tls: provider.strict_tls,
        credential_ref: provider.credential_ref.as_deref().map(mask_credential_ref),
        provider_account_id: provider.provider_account_id,
        server_side_encryption_mode: provider.server_side_encryption_mode,
        default_storage_class: provider.default_storage_class,
        status: provider.status,
        version: provider.version,
        credential_configured,
    }
}

/// Project an account-center row for the storage console. The projection is
/// intentionally narrow: scope, identity, lifecycle, and whether an active
/// credential exists — never credential material.
///
/// `scope_type` / `owner_user_id` / `is_default` travel to the client because
/// the console has to show *whose* account this is and which one a consumer
/// would pick by default; without them every row looks equally global.
pub(crate) fn map_storage_provider_account(
    account: ProviderAccount,
) -> StorageProviderAccountResponse {
    StorageProviderAccountResponse {
        id: account.id,
        scope_type: account.scope_type,
        owner_user_id: account.owner_user_id,
        is_default: account.is_default,
        vendor_code: account.vendor_code,
        account_code: account.account_code,
        display_name: account.display_name,
        account_type: account.account_type,
        environment: account.environment,
        external_account_id: account.external_account_id,
        capability_codes: account.capability_codes,
        region_code: account.region_code,
        status: account.status,
        credential_configured: account.credential_configured,
        credential_count: account.credential_count,
        version: account.version,
    }
}

pub(crate) fn map_storage_provider_capabilities(
    capabilities: StorageProviderCapabilities,
) -> StorageProviderCapabilitiesResponse {
    StorageProviderCapabilitiesResponse {
        provider_id: capabilities.provider_id,
        provider_kind: capabilities.provider_kind,
        supports_multipart_upload: capabilities.supports_multipart_upload,
        supports_presigned_upload_part: capabilities.supports_presigned_upload_part,
        supports_presigned_download: capabilities.supports_presigned_download,
        supports_server_side_encryption: capabilities.supports_server_side_encryption,
        supports_storage_class: capabilities.supports_storage_class,
        supports_credential_rotation: capabilities.supports_credential_rotation,
        supported_server_side_encryption_modes: capabilities.supported_server_side_encryption_modes,
        supported_storage_classes: capabilities.supported_storage_classes,
    }
}

pub(crate) fn mask_credential_ref(value: &str) -> String {
    match value.split_once(':') {
        Some((prefix, _)) if !prefix.trim().is_empty() => format!("{}:***", prefix.trim()),
        _ => "***".to_string(),
    }
}

pub(crate) fn parse_storage_provider_kind(
    raw: &str,
) -> Result<DriveStorageProviderKind, DriveServiceError> {
    DriveStorageProviderKind::try_from_str(raw).ok_or_else(|| {
        DriveServiceError::Validation(
            "provider_kind is invalid; allowed: local_filesystem, s3_compatible, google_cloud_storage, aliyun_oss, tencent_cos, huawei_obs, volcengine_tos, or custom:<vendor_key>"
                .to_string(),
        )
    })
}
