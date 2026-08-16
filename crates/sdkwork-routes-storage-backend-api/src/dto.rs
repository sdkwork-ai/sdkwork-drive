use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateStorageProviderRequest {
    pub(crate) id: String,
    pub(crate) provider_kind: String,
    pub(crate) name: String,
    pub(crate) endpoint_url: String,
    pub(crate) region: Option<String>,
    pub(crate) bucket: String,
    pub(crate) path_style: Option<bool>,
    pub(crate) strict_tls: Option<bool>,
    pub(crate) credential_ref: Option<String>,
    pub(crate) server_side_encryption_mode: Option<String>,
    pub(crate) default_storage_class: Option<String>,
    pub(crate) status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateStorageProviderRequest {
    pub(crate) name: Option<String>,
    pub(crate) endpoint_url: Option<String>,
    pub(crate) region: Option<String>,
    pub(crate) bucket: Option<String>,
    pub(crate) path_style: Option<bool>,
    pub(crate) strict_tls: Option<bool>,
    pub(crate) credential_ref: Option<String>,
    pub(crate) server_side_encryption_mode: Option<String>,
    pub(crate) default_storage_class: Option<String>,
    pub(crate) status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RotateStorageProviderCredentialRequest {
    pub(crate) credential_ref: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SetStorageProviderKindEnabledRequest {
    pub(crate) enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListStorageProvidersQuery {
    pub(crate) status: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DefaultStorageProviderBindingQuery {
    pub(crate) space_id: Option<String>,
    pub(crate) space_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListStorageProviderBindingsQuery {
    pub(crate) space_id: Option<String>,
    pub(crate) provider_id: Option<String>,
    pub(crate) lifecycle_status: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DeleteDefaultStorageProviderBindingQuery {
    pub(crate) space_id: Option<String>,
    pub(crate) space_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SetDefaultStorageProviderBindingRequest {
    pub(crate) space_id: Option<String>,
    pub(crate) space_type: Option<String>,
    pub(crate) provider_id: String,
    pub(crate) storage_root_prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListProviderBucketsQuery {
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListProviderObjectsQuery {
    pub(crate) prefix: Option<String>,
    pub(crate) delimiter: Option<String>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<u16>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CopyProviderObjectRequest {
    pub(crate) source_object_key: String,
    pub(crate) destination_object_key: String,
    pub(crate) destination_bucket: Option<String>,
    pub(crate) metadata_directive: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageProviderResponse {
    pub(crate) id: String,
    pub(crate) provider_kind: String,
    pub(crate) name: String,
    pub(crate) endpoint_url: String,
    pub(crate) region: Option<String>,
    pub(crate) bucket: String,
    pub(crate) path_style: bool,
    pub(crate) strict_tls: bool,
    pub(crate) credential_ref: Option<String>,
    pub(crate) server_side_encryption_mode: Option<String>,
    pub(crate) default_storage_class: Option<String>,
    pub(crate) status: String,
    pub(crate) version: i64,
    pub(crate) credential_configured: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageProviderCapabilitiesResponse {
    pub(crate) provider_id: String,
    pub(crate) provider_kind: String,
    pub(crate) supports_multipart_upload: bool,
    pub(crate) supports_presigned_upload_part: bool,
    pub(crate) supports_presigned_download: bool,
    pub(crate) supports_server_side_encryption: bool,
    pub(crate) supports_storage_class: bool,
    pub(crate) supports_credential_rotation: bool,
    pub(crate) supported_server_side_encryption_modes: Vec<String>,
    pub(crate) supported_storage_classes: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageProviderKindResponse {
    pub(crate) provider_kind: String,
    pub(crate) display_name: String,
    pub(crate) enabled: bool,
    pub(crate) sort_order: i64,
    pub(crate) version: i64,
    pub(crate) config_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TestStorageProviderResponse {
    pub(crate) provider_id: String,
    pub(crate) reachable: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderBucketResponse {
    pub(crate) provider_id: String,
    pub(crate) bucket: String,
    pub(crate) exists: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderBucketListItemResponse {
    pub(crate) bucket: String,
    pub(crate) configured: bool,
    pub(crate) creation_date_epoch_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderBucketMutationResponse {
    pub(crate) provider_id: String,
    pub(crate) bucket: String,
    pub(crate) changed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderObjectResponse {
    pub(crate) provider_id: String,
    pub(crate) bucket: String,
    pub(crate) object_kind: String,
    pub(crate) object_key: String,
    pub(crate) content_length: u64,
    pub(crate) content_type: Option<String>,
    pub(crate) etag: Option<String>,
    pub(crate) version_id: Option<String>,
    pub(crate) storage_class: Option<String>,
    pub(crate) last_modified_epoch_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderObjectMutationResponse {
    pub(crate) provider_id: String,
    pub(crate) bucket: String,
    pub(crate) object_key: String,
    pub(crate) changed: bool,
}

/// 对象内容读取响应：内容以 base64 传输（任意字节安全），配套大小与校验和。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderObjectContentResponse {
    pub(crate) provider_id: String,
    pub(crate) bucket: String,
    pub(crate) object_key: String,
    pub(crate) content_type: Option<String>,
    pub(crate) size_bytes: u64,
    pub(crate) encoding: String,
    pub(crate) content: String,
    pub(crate) checksum_sha256: String,
}

/// 对象内容写入请求：`encoding` 为 `utf8`（默认）或 `base64`；
/// `object_key` 以 `/` 结尾且内容为空时创建目录占位对象。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateProviderObjectContentRequest {
    pub(crate) content: String,
    pub(crate) encoding: Option<String>,
    pub(crate) content_type: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageProviderBindingResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) space_id: Option<String>,
    pub(crate) provider_id: String,
    pub(crate) binding_scope: String,
    pub(crate) purpose: String,
    pub(crate) storage_root_prefix: String,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
    pub(crate) storage_provider: StorageProviderResponse,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct OffsetPage {
    pub(crate) limit: i64,
    pub(crate) offset: i64,
}
