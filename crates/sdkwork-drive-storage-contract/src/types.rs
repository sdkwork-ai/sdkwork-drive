use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriveStorageProviderKind {
    LocalFilesystem,
    S3Compatible,
    GoogleCloudStorage,
    AliyunOss,
    TencentCos,
    HuaweiObs,
    VolcengineTos,
    Custom(String),
}

impl DriveStorageProviderKind {
    pub const CUSTOM_PREFIX: &'static str = "custom:";

    pub fn as_str(&self) -> &str {
        match self {
            Self::LocalFilesystem => "local_filesystem",
            Self::S3Compatible => "s3_compatible",
            Self::GoogleCloudStorage => "google_cloud_storage",
            Self::AliyunOss => "aliyun_oss",
            Self::TencentCos => "tencent_cos",
            Self::HuaweiObs => "huawei_obs",
            Self::VolcengineTos => "volcengine_tos",
            Self::Custom(value) => value.as_str(),
        }
    }

    pub fn try_from_str(raw: &str) -> Option<Self> {
        let normalized = raw.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "local_filesystem" => Some(Self::LocalFilesystem),
            "s3_compatible" => Some(Self::S3Compatible),
            "google_cloud_storage" => Some(Self::GoogleCloudStorage),
            "aliyun_oss" => Some(Self::AliyunOss),
            "tencent_cos" => Some(Self::TencentCos),
            "huawei_obs" => Some(Self::HuaweiObs),
            "volcengine_tos" => Some(Self::VolcengineTos),
            _ => {
                let suffix = normalized.strip_prefix(Self::CUSTOM_PREFIX)?;
                if is_valid_custom_suffix(suffix) {
                    Some(Self::Custom(normalized))
                } else {
                    None
                }
            }
        }
    }
}

fn is_valid_custom_suffix(raw: &str) -> bool {
    if raw.len() < 2 || raw.len() > 32 {
        return false;
    }
    raw.chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '-'))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriveStorageProviderCapabilities {
    pub supports_multipart_upload: bool,
    pub supports_presigned_upload_part: bool,
    pub supports_presigned_download: bool,
    pub supports_range_read: bool,
    pub supports_server_side_copy: bool,
    pub supports_versioning: bool,
}

impl DriveStorageProviderCapabilities {
    pub const fn default_s3_compatible() -> Self {
        Self {
            supports_multipart_upload: true,
            supports_presigned_upload_part: true,
            supports_presigned_download: true,
            supports_range_read: true,
            supports_server_side_copy: true,
            supports_versioning: true,
        }
    }

    pub const fn default_local_filesystem() -> Self {
        Self {
            supports_multipart_upload: false,
            supports_presigned_upload_part: false,
            supports_presigned_download: false,
            supports_range_read: true,
            supports_server_side_copy: false,
            supports_versioning: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriveObjectLocator {
    pub bucket: String,
    pub object_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriveByteRange {
    pub start_inclusive: u64,
    pub end_inclusive: u64,
}

pub type DriveObjectHeaders = BTreeMap<String, String>;
pub type DriveObjectMetadata = BTreeMap<String, String>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PutObjectRequest {
    pub locator: DriveObjectLocator,
    pub content_type: Option<String>,
    pub metadata: DriveObjectMetadata,
    pub body: Vec<u8>,
    pub checksum_sha256_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PutObjectResponse {
    pub locator: DriveObjectLocator,
    pub etag: Option<String>,
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadObjectRequest {
    pub locator: DriveObjectLocator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadObjectResponse {
    pub locator: DriveObjectLocator,
    pub content_length: u64,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub version_id: Option<String>,
    pub checksum_sha256_hex: Option<String>,
    pub metadata: DriveObjectMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteObjectRequest {
    pub locator: DriveObjectLocator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteObjectResponse {
    pub locator: DriveObjectLocator,
    pub deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadBucketRequest {
    pub bucket: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadBucketResponse {
    pub bucket: String,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListBucketsRequest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListedBucket {
    pub bucket: String,
    pub creation_date_epoch_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListBucketsResponse {
    pub items: Vec<ListedBucket>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateBucketRequest {
    pub bucket: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateBucketResponse {
    pub bucket: String,
    pub created: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteBucketRequest {
    pub bucket: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteBucketResponse {
    pub bucket: String,
    pub deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListObjectsRequest {
    pub bucket: String,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub continuation_token: Option<String>,
    pub max_keys: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListedObject {
    pub object_key: String,
    pub content_length: u64,
    pub etag: Option<String>,
    pub storage_class: Option<String>,
    pub last_modified_epoch_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListObjectsResponse {
    pub bucket: String,
    pub prefix: Option<String>,
    pub items: Vec<ListedObject>,
    pub prefixes: Vec<String>,
    pub next_continuation_token: Option<String>,
    pub is_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopyObjectRequest {
    pub source: DriveObjectLocator,
    pub destination: DriveObjectLocator,
    pub metadata_directive: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopyObjectResponse {
    pub locator: DriveObjectLocator,
    pub etag: Option<String>,
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateMultipartUploadRequest {
    pub locator: DriveObjectLocator,
    pub content_type: Option<String>,
    pub metadata: DriveObjectMetadata,
    pub checksum_sha256_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateMultipartUploadResponse {
    pub locator: DriveObjectLocator,
    pub upload_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresignUploadPartRequest {
    pub locator: DriveObjectLocator,
    pub upload_id: String,
    pub part_number: u16,
    pub expires_in_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresignedUploadPartResponse {
    pub method: String,
    pub url: String,
    pub headers: DriveObjectHeaders,
    pub expires_at_epoch_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletedMultipartPart {
    pub part_number: u16,
    pub etag: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompleteMultipartUploadRequest {
    pub locator: DriveObjectLocator,
    pub upload_id: String,
    pub parts: Vec<CompletedMultipartPart>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompleteMultipartUploadResponse {
    pub locator: DriveObjectLocator,
    pub etag: Option<String>,
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbortMultipartUploadRequest {
    pub locator: DriveObjectLocator,
    pub upload_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresignDownloadRequest {
    pub locator: DriveObjectLocator,
    pub expires_in_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresignedDownloadResponse {
    pub method: String,
    pub url: String,
    pub headers: DriveObjectHeaders,
    pub expires_at_epoch_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadObjectRangeRequest {
    pub locator: DriveObjectLocator,
    pub range: DriveByteRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadObjectRangeResponse {
    pub locator: DriveObjectLocator,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub content_length: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriveObjectStoreErrorKind {
    NotFound,
    InvalidRequest,
    Conflict,
    RateLimited,
    PermissionDenied,
    Timeout,
    Unavailable,
    IntegrityFailed,
    UpstreamError,
    NotSupported,
    Internal,
}

impl DriveObjectStoreErrorKind {
    pub fn as_code(self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::InvalidRequest => "invalid_request",
            Self::Conflict => "conflict",
            Self::RateLimited => "rate_limited",
            Self::PermissionDenied => "permission_denied",
            Self::Timeout => "timeout",
            Self::Unavailable => "unavailable",
            Self::IntegrityFailed => "integrity_failed",
            Self::UpstreamError => "upstream_error",
            Self::NotSupported => "not_supported",
            Self::Internal => "internal_error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriveObjectStoreError {
    pub kind: DriveObjectStoreErrorKind,
    pub message: String,
}

impl DriveObjectStoreError {
    pub fn new(kind: DriveObjectStoreErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn upstream(message: impl Into<String>) -> Self {
        Self::new(DriveObjectStoreErrorKind::UpstreamError, message)
    }

    pub fn code(&self) -> &'static str {
        self.kind.as_code()
    }
}

impl Display for DriveObjectStoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.message)
    }
}

impl Error for DriveObjectStoreError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriveStorageCredentialSnapshot {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
}

pub const S3_BUCKET_NAME_PATTERN: &str =
    "^(?!xn--)(?!sthree-)(?!.*\\.\\.)(?!.*\\.-)(?!.*-\\.)(?!\\d+\\.\\d+\\.\\d+\\.\\d+$)(?!.*(-s3alias|--ol-s3|\\.mrap|--x-s3)$)[a-z0-9][a-z0-9.-]{1,61}[a-z0-9]$";

pub const S3_BUCKET_NAME_DESCRIPTION: &str =
    "S3-compatible bucket name. DNS-compatible 3-63 characters; lowercase letters, digits, dots, and hyphens only; must start and end with a letter or digit; no IPv4-looking names, adjacent dots, dot-hyphen adjacency, or reserved S3 affixes.";

pub fn validate_s3_bucket_name(raw: &str, field_name: &str) -> Result<(), DriveObjectStoreError> {
    const RESERVED_BUCKET_PREFIXES: [&str; 2] = ["xn--", "sthree-"];
    const RESERVED_BUCKET_SUFFIXES: [&str; 4] = ["-s3alias", "--ol-s3", ".mrap", "--x-s3"];

    let bucket = raw.trim();
    if raw != bucket {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("{field_name} must be trimmed"),
        ));
    }
    if !(3..=63).contains(&bucket.len()) {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("{field_name} must be between 3 and 63 characters"),
        ));
    }
    if !bucket.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
    }) {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!(
                "{field_name} may only contain lowercase ASCII letters, digits, dot, or hyphen"
            ),
        ));
    }
    let starts_with_alnum = bucket
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
    let ends_with_alnum = bucket
        .bytes()
        .last()
        .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
    if !starts_with_alnum || !ends_with_alnum {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("{field_name} must start and end with a letter or digit"),
        ));
    }
    if bucket.contains("..") {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("{field_name} must not contain adjacent dots"),
        ));
    }
    if bucket.contains(".-") || bucket.contains("-.") {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("{field_name} must not contain dot-hyphen adjacency"),
        ));
    }
    if is_s3_bucket_ipv4_address_like(bucket) {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("{field_name} must not be formatted as an IPv4 address"),
        ));
    }
    if RESERVED_BUCKET_PREFIXES
        .iter()
        .any(|prefix| bucket.starts_with(prefix))
        || RESERVED_BUCKET_SUFFIXES
            .iter()
            .any(|suffix| bucket.ends_with(suffix))
    {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("{field_name} uses a reserved S3 bucket name affix"),
        ));
    }
    Ok(())
}

fn is_s3_bucket_ipv4_address_like(bucket: &str) -> bool {
    let mut parts = bucket.split('.');
    let mut count = 0;
    for part in &mut parts {
        count += 1;
        if part.is_empty() || part.len() > 3 || !part.bytes().all(|byte| byte.is_ascii_digit()) {
            return false;
        }
        if part.parse::<u8>().is_err() {
            return false;
        }
    }
    count == 4
}

pub fn resolve_drive_storage_credentials(
    credential_ref: Option<&str>,
    default_access_key_env: &str,
    default_secret_key_env: &str,
    default_session_token_env: &str,
    default_error_context: &str,
) -> Result<DriveStorageCredentialSnapshot, DriveObjectStoreError> {
    if let Some(raw) = credential_ref {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "credential_ref must not be empty",
            ));
        }
        if let Some(payload) = trimmed.strip_prefix("plain:") {
            if !sdkwork_drive_config::allows_plain_credential_refs() {
                return Err(DriveObjectStoreError::new(
                    DriveObjectStoreErrorKind::InvalidRequest,
                    "plain credential_ref is disabled for the current runtime profile",
                ));
            }
            return resolve_plain_credential_ref(payload);
        }
        if let Some(payload) = trimmed.strip_prefix("env:") {
            return resolve_env_credential_ref(payload);
        }
        for scheme in ["secret", "kms", "vault"] {
            let prefix = format!("{scheme}:");
            if let Some(payload) = trimmed.strip_prefix(&prefix) {
                return resolve_external_materialized_credential_ref(scheme, payload);
            }
        }
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            "credential_ref must start with plain:, env:, secret:, kms:, or vault:",
        ));
    }

    let access_key_id = read_required_credential_env(default_access_key_env).map_err(|_| {
        DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("missing {default_access_key_env} for {default_error_context}"),
        )
    })?;
    let secret_access_key = read_required_credential_env(default_secret_key_env).map_err(|_| {
        DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("missing {default_secret_key_env} for {default_error_context}"),
        )
    })?;
    let session_token = read_optional_credential_env(default_session_token_env);
    Ok(DriveStorageCredentialSnapshot {
        access_key_id,
        secret_access_key,
        session_token,
    })
}

fn resolve_plain_credential_ref(
    payload: &str,
) -> Result<DriveStorageCredentialSnapshot, DriveObjectStoreError> {
    let parts: Vec<&str> = payload.split(':').collect();
    if !(2..=3).contains(&parts.len()) {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            "credential_ref plain format is invalid",
        ));
    }
    let access_key_id = parts[0].trim().to_string();
    let secret_access_key = parts[1].trim().to_string();
    if access_key_id.is_empty() || secret_access_key.is_empty() {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            "credential_ref plain credentials are empty",
        ));
    }
    let session_token = parts
        .get(2)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    Ok(DriveStorageCredentialSnapshot {
        access_key_id,
        secret_access_key,
        session_token,
    })
}

fn resolve_env_credential_ref(
    payload: &str,
) -> Result<DriveStorageCredentialSnapshot, DriveObjectStoreError> {
    let parts: Vec<&str> = payload.split(':').collect();
    if !(2..=3).contains(&parts.len()) {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            "credential_ref env format is invalid",
        ));
    }
    let access_key_name = parts[0].trim();
    let secret_key_name = parts[1].trim();
    if access_key_name.is_empty() || secret_key_name.is_empty() {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            "credential_ref env variable names are empty",
        ));
    }
    let access_key_id = read_required_credential_env(access_key_name).map_err(|_| {
        DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("missing env variable for credential_ref access key: {access_key_name}"),
        )
    })?;
    let secret_access_key = read_required_credential_env(secret_key_name).map_err(|_| {
        DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("missing env variable for credential_ref secret key: {secret_key_name}"),
        )
    })?;
    let session_token = parts.get(2).and_then(|name| {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            None
        } else {
            read_optional_credential_env(trimmed)
        }
    });
    Ok(DriveStorageCredentialSnapshot {
        access_key_id,
        secret_access_key,
        session_token,
    })
}

fn resolve_external_materialized_credential_ref(
    scheme: &str,
    payload: &str,
) -> Result<DriveStorageCredentialSnapshot, DriveObjectStoreError> {
    let key = materialized_credential_env_key(payload).ok_or_else(|| {
        DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("credential_ref {scheme} payload is invalid"),
        )
    })?;
    let access_key_name = format!("SDKWORK_DRIVE_STORAGE_CREDENTIAL__{key}__ACCESS_KEY_ID");
    let secret_key_name = format!("SDKWORK_DRIVE_STORAGE_CREDENTIAL__{key}__SECRET_ACCESS_KEY");
    let session_token_name = format!("SDKWORK_DRIVE_STORAGE_CREDENTIAL__{key}__SESSION_TOKEN");

    let access_key_id = read_required_credential_env(&access_key_name).map_err(|_| {
        DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!(
                "missing materialized env variable for credential_ref {scheme} access key: {access_key_name}"
            ),
        )
    })?;
    let secret_access_key = read_required_credential_env(&secret_key_name).map_err(|_| {
        DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!(
                "missing materialized env variable for credential_ref {scheme} secret key: {secret_key_name}"
            ),
        )
    })?;
    let session_token = read_optional_credential_env(&session_token_name);
    Ok(DriveStorageCredentialSnapshot {
        access_key_id,
        secret_access_key,
        session_token,
    })
}

fn materialized_credential_env_key(payload: &str) -> Option<String> {
    let mut key = String::with_capacity(payload.len());
    let mut previous_was_separator = false;
    for byte in payload.trim().bytes() {
        if byte.is_ascii_alphanumeric() {
            key.push(byte as char);
            previous_was_separator = false;
        } else if !previous_was_separator {
            key.push('_');
            previous_was_separator = true;
        }
    }
    let key = key.trim_matches('_').to_string();
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

fn read_required_credential_env(key: &str) -> Result<String, ()> {
    let value = std::env::var(key).map_err(|_| ())?;
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return Err(());
    }
    Ok(trimmed)
}

fn read_optional_credential_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
