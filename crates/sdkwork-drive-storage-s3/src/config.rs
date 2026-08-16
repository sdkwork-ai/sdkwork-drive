use sdkwork_drive_storage_contract::{
    resolve_drive_storage_credentials, validate_s3_bucket_name, DriveObjectStoreError,
    DriveObjectStoreErrorKind, DriveStorageCredentialSnapshot, DriveStorageProviderKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum S3ProviderProfile {
    AwsS3,
    Minio,
    CloudflareR2,
    AliyunOss,
    TencentCos,
    HuaweiObs,
    VolcengineTos,
    GoogleCloudStorage,
    BackblazeB2,
    GenericCompatible,
}

impl S3ProviderProfile {
    const CUSTOM_PREFIX: &'static str = "custom:";

    pub fn as_str(self) -> &'static str {
        match self {
            Self::AwsS3 => "aws_s3",
            Self::Minio => "minio",
            Self::CloudflareR2 => "cloudflare_r2",
            Self::AliyunOss => "aliyun_oss",
            Self::TencentCos => "tencent_cos",
            Self::HuaweiObs => "huawei_obs",
            Self::VolcengineTos => "volcengine_tos",
            Self::GoogleCloudStorage => "google_cloud_storage",
            Self::BackblazeB2 => "backblaze_b2",
            Self::GenericCompatible => "generic_s3_compatible",
        }
    }

    pub fn default_region(self) -> &'static str {
        match self {
            Self::CloudflareR2 => "auto",
            _ => "us-east-1",
        }
    }

    pub fn default_force_path_style(self) -> bool {
        match self {
            Self::AwsS3 => false,
            Self::GoogleCloudStorage => false,
            Self::AliyunOss => false,
            Self::TencentCos => false,
            Self::HuaweiObs => false,
            Self::VolcengineTos => false,
            Self::BackblazeB2 => false,
            Self::CloudflareR2 => true,
            Self::Minio => true,
            Self::GenericCompatible => true,
        }
    }

    pub fn from_provider_kind(provider_kind: &str, endpoint: Option<&str>) -> Self {
        let normalized = provider_kind.trim().to_ascii_lowercase();
        if normalized == "s3_compatible" {
            if let Some(endpoint_value) = endpoint {
                if let Some(profile) = Self::from_endpoint(endpoint_value) {
                    return profile;
                }
            }
            return Self::GenericCompatible;
        }
        if normalized == "aliyun_oss" {
            return Self::AliyunOss;
        }
        if normalized == "tencent_cos" {
            return Self::TencentCos;
        }
        if normalized == "huawei_obs" {
            return Self::HuaweiObs;
        }
        if normalized == "volcengine_tos" {
            return Self::VolcengineTos;
        }
        if normalized == "google_cloud_storage" {
            return Self::GoogleCloudStorage;
        }
        if let Some(suffix) = normalized.strip_prefix(Self::CUSTOM_PREFIX) {
            if let Some(profile) = Self::from_vendor_key(suffix) {
                return profile;
            }
        }
        if let Some(endpoint_value) = endpoint {
            if let Some(profile) = Self::from_endpoint(endpoint_value) {
                return profile;
            }
        }
        Self::GenericCompatible
    }

    fn from_vendor_key(raw: &str) -> Option<Self> {
        let normalized = raw.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "aws" | "aws_s3" | "amazon_s3" => Some(Self::AwsS3),
            "minio" => Some(Self::Minio),
            "r2" | "cloudflare" | "cloudflare_r2" => Some(Self::CloudflareR2),
            "oss" | "aliyun" | "aliyun_oss" | "alibaba_oss" => Some(Self::AliyunOss),
            "cos" | "tencent" | "tencent_cos" => Some(Self::TencentCos),
            "obs" | "huawei" | "huawei_obs" => Some(Self::HuaweiObs),
            "tos" | "volc" | "volcengine" | "volcengine_tos" | "volcano" | "volcano_tos"
            | "bytedance_tos" => Some(Self::VolcengineTos),
            "gcs" | "google_cloud_storage" | "google_storage" => Some(Self::GoogleCloudStorage),
            "b2" | "backblaze" | "backblaze_b2" => Some(Self::BackblazeB2),
            "s3_compatible" | "generic_s3" | "generic_s3_compatible" => {
                Some(Self::GenericCompatible)
            }
            _ => None,
        }
    }

    fn from_endpoint(raw: &str) -> Option<Self> {
        let normalized = raw.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return None;
        }
        if normalized.contains(".r2.cloudflarestorage.com") {
            return Some(Self::CloudflareR2);
        }
        if normalized.contains("aliyuncs.com") {
            return Some(Self::AliyunOss);
        }
        if normalized.contains(".myqcloud.com") {
            return Some(Self::TencentCos);
        }
        if normalized.contains(".myhuaweicloud.com") {
            return Some(Self::HuaweiObs);
        }
        if normalized.contains(".volces.com") || normalized.contains("volcengine") {
            return Some(Self::VolcengineTos);
        }
        if normalized.contains("storage.googleapis.com") {
            return Some(Self::GoogleCloudStorage);
        }
        if normalized.contains("backblazeb2.com") {
            return Some(Self::BackblazeB2);
        }
        if normalized.contains("amazonaws.com") {
            return Some(Self::AwsS3);
        }
        if normalized.contains("minio") || normalized.contains("127.0.0.1:9000") {
            return Some(Self::Minio);
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S3StoreConfig {
    pub provider_kind: DriveStorageProviderKind,
    pub provider_profile: S3ProviderProfile,
    pub endpoint: Option<String>,
    pub region: String,
    pub default_bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
    pub force_path_style: bool,
    pub strict_tls: bool,
}

impl S3StoreConfig {
    pub fn from_provider_parts(
        provider_kind: &str,
        endpoint_url: &str,
        region: Option<&str>,
        default_bucket: &str,
        force_path_style: bool,
        credential_ref: Option<&str>,
        strict_tls_override: Option<bool>,
    ) -> Result<Self, DriveObjectStoreError> {
        let endpoint = endpoint_url.trim();
        if endpoint_url != endpoint {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "endpoint_url must be trimmed",
            ));
        }
        if endpoint.is_empty() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "endpoint_url must not be empty",
            ));
        }
        if endpoint.chars().any(char::is_whitespace) {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "endpoint_url must not contain whitespace",
            ));
        }

        let provider_kind = parse_provider_kind(provider_kind)?;
        let provider_profile =
            S3ProviderProfile::from_provider_kind(provider_kind.as_str(), Some(endpoint));
        let region = region
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .or_else(|| {
                std::env::var("SDKWORK_DRIVE_S3_REGION")
                    .ok()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_else(|| provider_profile.default_region().to_string());
        let credentials = Self::resolve_credentials(credential_ref)?;
        let strict_tls = strict_tls_override.unwrap_or_else(|| {
            Self::read_bool_env(
                "SDKWORK_DRIVE_S3_STRICT_TLS",
                !endpoint.to_ascii_lowercase().starts_with("http://"),
            )
        });

        let config = Self {
            provider_kind,
            provider_profile,
            endpoint: Some(endpoint.to_string()),
            region,
            default_bucket: default_bucket.to_string(),
            access_key_id: credentials.access_key_id,
            secret_access_key: credentials.secret_access_key,
            session_token: credentials.session_token,
            force_path_style,
            strict_tls,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), DriveObjectStoreError> {
        if matches!(
            self.provider_kind,
            DriveStorageProviderKind::LocalFilesystem
        ) {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "s3 store only supports s3-compatible provider kinds",
            ));
        }
        if self.region.trim().is_empty() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "region must not be empty",
            ));
        }
        validate_s3_bucket_name(&self.default_bucket, "default_bucket")?;
        if self.access_key_id.trim().is_empty() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "access_key_id must not be empty",
            ));
        }
        if self.secret_access_key.trim().is_empty() {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "secret_access_key must not be empty",
            ));
        }
        if self.strict_tls
            && self
                .endpoint
                .as_ref()
                .is_some_and(|endpoint| endpoint.to_ascii_lowercase().starts_with("http://"))
        {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "strict_tls=true requires an https endpoint",
            ));
        }
        Ok(())
    }

    pub fn resolve_bucket(&self, requested_bucket: &str) -> Result<String, DriveObjectStoreError> {
        let trimmed = requested_bucket.trim();
        if trimmed.is_empty() {
            return Ok(self.default_bucket.clone());
        }
        validate_s3_bucket_name(trimmed, "bucket")?;
        Ok(trimmed.to_string())
    }

    fn resolve_credentials(
        credential_ref: Option<&str>,
    ) -> Result<DriveStorageCredentialSnapshot, DriveObjectStoreError> {
        resolve_drive_storage_credentials(
            credential_ref,
            "SDKWORK_DRIVE_S3_ACCESS_KEY_ID",
            "SDKWORK_DRIVE_S3_SECRET_ACCESS_KEY",
            "SDKWORK_DRIVE_S3_SESSION_TOKEN",
            "s3-compatible object store",
        )
    }

    fn read_bool_env(key: &str, default_value: bool) -> bool {
        let Ok(value) = std::env::var(key) else {
            return default_value;
        };
        match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default_value,
        }
    }
}

impl Default for S3StoreConfig {
    fn default() -> Self {
        Self {
            provider_kind: DriveStorageProviderKind::S3Compatible,
            provider_profile: S3ProviderProfile::GenericCompatible,
            endpoint: None,
            region: "us-east-1".to_string(),
            default_bucket: "sdkwork-drive-default".to_string(),
            access_key_id: String::new(),
            secret_access_key: String::new(),
            session_token: None,
            force_path_style: true,
            strict_tls: true,
        }
    }
}

fn parse_provider_kind(raw: &str) -> Result<DriveStorageProviderKind, DriveObjectStoreError> {
    DriveStorageProviderKind::try_from_str(raw).ok_or_else(|| {
        DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            "provider_kind is invalid for s3 store",
        )
    })
}
