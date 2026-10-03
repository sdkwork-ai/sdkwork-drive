use sdkwork_drive_storage_contract::{
    resolve_drive_storage_credentials, validate_s3_bucket_name, validate_s3_region_token,
    DriveObjectStoreError, DriveObjectStoreErrorKind, DriveStorageCredentialSnapshot,
    DriveStorageProviderKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum S3ProviderProfile {
    AwsS3,
    Minio,
    CloudflareR2,
    AliyunOss,
    AliyunOssInternational,
    TencentCos,
    TencentCosInternational,
    HuaweiObs,
    VolcengineTos,
    BaiduBos,
    KingsoftKs3,
    QiniuKodo,
    ChinaMobileEcloud,
    ChinaTelecomEos,
    ChinaUnicomWo,
    GoogleCloudStorage,
    BackblazeB2,
    Wasabi,
    DigitalOceanSpaces,
    LinodeObjectStorage,
    VultrObjectStorage,
    ScalewayObjectStorage,
    OracleCloudStorage,
    IbmCos,
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
            Self::AliyunOssInternational => "alibaba_cloud_international",
            Self::TencentCos => "tencent_cos",
            Self::TencentCosInternational => "tencent_cloud_international",
            Self::HuaweiObs => "huawei_obs",
            Self::VolcengineTos => "volcengine_tos",
            Self::BaiduBos => "baidu_bos",
            Self::KingsoftKs3 => "kingsoft_ks3",
            Self::QiniuKodo => "qiniu_kodo",
            Self::ChinaMobileEcloud => "china_mobile_ecloud",
            Self::ChinaTelecomEos => "china_telecom_eos",
            Self::ChinaUnicomWo => "china_unicom_wo",
            Self::GoogleCloudStorage => "google_cloud_storage",
            Self::BackblazeB2 => "backblaze_b2",
            Self::Wasabi => "wasabi",
            Self::DigitalOceanSpaces => "digitalocean_spaces",
            Self::LinodeObjectStorage => "linode_object_storage",
            Self::VultrObjectStorage => "vultr_object_storage",
            Self::ScalewayObjectStorage => "scaleway_object_storage",
            Self::OracleCloudStorage => "oracle_cloud_storage",
            Self::IbmCos => "ibm_cos",
            Self::GenericCompatible => "generic_s3_compatible",
        }
    }

    /// Region assumed when a configuration leaves `region` empty.
    ///
    /// `auto` on Cloudflare R2 and `us-east-1` elsewhere: R2 rejects any other
    /// literal, while the remaining vendors accept either `us-east-1` as their
    /// own default or ignore the field because the endpoint carries the real
    /// region.
    pub fn default_region(self) -> &'static str {
        match self {
            Self::CloudflareR2 => "auto",
            _ => "us-east-1",
        }
    }

    /// Whether the vendor serves bucket-in-path (`true`) or bucket-in-host
    /// (`false`) addressing by default.
    ///
    /// Self-hosted and generic endpoints default to path style; the named cloud
    /// vendors all publish virtual-hosted-style endpoints.
    pub fn default_force_path_style(self) -> bool {
        match self {
            Self::Minio
            | Self::CloudflareR2
            | Self::GenericCompatible
            | Self::ChinaMobileEcloud
            | Self::ChinaTelecomEos
            | Self::ChinaUnicomWo
            | Self::LinodeObjectStorage
            | Self::VultrObjectStorage
            | Self::IbmCos => true,
            _ => false,
        }
    }

    /// Account-level ("service") host the vendor answers its bucket inventory
    /// on, when that is not the regional endpoint a provider configuration
    /// stores.
    ///
    /// Tencent COS publishes the two separately: object operations are addressed
    /// to `cos.<region>.myqcloud.com`, while `GetService` — the S3 `ListBuckets`
    /// call, and the only account-level operation in the set — belongs to
    /// `service.cos.myqcloud.com`, the host the vendor's own SDKs target for
    /// that operation. Reading the inventory through the regional host is what
    /// made a COS account holding buckets in several regions answer with the
    /// single bucket that sits in the configured region.
    ///
    /// Vendors whose regional endpoint already answers the inventory (AWS S3,
    /// MinIO, Aliyun OSS, ...) return `None` and keep listing unchanged.
    pub fn service_endpoint(self) -> Option<&'static str> {
        match self {
            Self::TencentCos | Self::TencentCosInternational => {
                Some("https://service.cos.myqcloud.com")
            }
            _ => None,
        }
    }

    /// Whether `endpoint` is this vendor's own endpoint.
    ///
    /// The vendor split above may only be applied to the vendor's real domain: a
    /// proxy, a private gateway, or a test double configured as this provider
    /// kind answers the inventory itself, and redirecting that call to the
    /// public service host would break a deployment that deliberately fronts
    /// the vendor.
    #[must_use]
    pub fn owns_endpoint(self, endpoint: &str) -> bool {
        Self::from_endpoint(endpoint) == Some(self)
    }

    pub fn from_provider_kind(provider_kind: &str, endpoint: Option<&str>) -> Self {
        // `s3_compatible` is the "any S3 vendor" key: it intentionally does not
        // resolve to a named profile on its own, so the endpoint decides (or
        // falls back to the generic profile). Every other catalog key maps
        // straight to its profile.
        let normalized = provider_kind.trim().to_ascii_lowercase();
        if normalized == "s3_compatible" {
            if let Some(endpoint_value) = endpoint {
                if let Some(profile) = Self::from_endpoint(endpoint_value) {
                    return profile;
                }
            }
            return Self::GenericCompatible;
        }
        if let Some(profile) = Self::from_vendor_key(&normalized) {
            return profile;
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
            "aliyun_intl"
            | "aliyun_international"
            | "alibaba_cloud_international"
            | "alibabacloud_intl" => Some(Self::AliyunOssInternational),
            "cos" | "tencent" | "tencent_cos" => Some(Self::TencentCos),
            "tencent_intl"
            | "tencent_international"
            | "tencent_cloud_international"
            | "cos_intl" => Some(Self::TencentCosInternational),
            "obs" | "huawei" | "huawei_obs" => Some(Self::HuaweiObs),
            "tos" | "volc" | "volcengine" | "volcengine_tos" | "volcano" | "volcano_tos"
            | "bytedance_tos" => Some(Self::VolcengineTos),
            "bos" | "baidu" | "baidu_bos" => Some(Self::BaiduBos),
            "ks3" | "kingsoft" | "kingsoft_ks3" | "ksyun" => Some(Self::KingsoftKs3),
            "kodo" | "qiniu" | "qiniu_kodo" => Some(Self::QiniuKodo),
            "ecloud" | "china_mobile" | "china_mobile_ecloud" | "cmecloud" | "cmss" => {
                Some(Self::ChinaMobileEcloud)
            }
            "eos" | "china_telecom" | "china_telecom_eos" | "ctyun" | "ctyun_eos" => {
                Some(Self::ChinaTelecomEos)
            }
            "unicom_wo" | "china_unicom" | "china_unicom_wo" | "wocloud" | "wo" => {
                Some(Self::ChinaUnicomWo)
            }
            "gcs" | "google_cloud_storage" | "google_storage" => Some(Self::GoogleCloudStorage),
            "b2" | "backblaze" | "backblaze_b2" => Some(Self::BackblazeB2),
            "wasabi" => Some(Self::Wasabi),
            "spaces" | "digitalocean" | "digitalocean_spaces" | "do_spaces" => {
                Some(Self::DigitalOceanSpaces)
            }
            "linode" | "linode_object_storage" | "akamai" => Some(Self::LinodeObjectStorage),
            "vultr" | "vultr_object_storage" => Some(Self::VultrObjectStorage),
            "scaleway" | "scaleway_object_storage" | "scw" => Some(Self::ScalewayObjectStorage),
            "oci" | "oracle" | "oracle_cloud_storage" => Some(Self::OracleCloudStorage),
            "ibm" | "ibm_cos" | "cos_ibm" => Some(Self::IbmCos),
            "generic_s3" | "generic_s3_compatible" => Some(Self::GenericCompatible),
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
        // Cloudflare R2 also answers on the account-scoped `<account>.r2.cloudflarestorage.com`
        // form the console shows, which the suffix check above already covers;
        // the bare `r2.dev` public host is deliberately not a storage endpoint.
        if normalized.contains("aliyuncs.com") {
            // Both the mainland (`oss-cn-*`) and international (`oss-*-intl`)
            // hosts live under the same domain, so the region segment decides.
            if normalized.contains("-intl") {
                return Some(Self::AliyunOssInternational);
            }
            return Some(Self::AliyunOss);
        }
        if normalized.contains(".myqcloud.com") || normalized.contains(".cos.tencentcos") {
            if normalized.contains("intl") || normalized.contains("cos-intl") {
                return Some(Self::TencentCosInternational);
            }
            return Some(Self::TencentCos);
        }
        if normalized.contains(".myhuaweicloud.com") {
            return Some(Self::HuaweiObs);
        }
        if normalized.contains(".volces.com") || normalized.contains("volcengine") {
            return Some(Self::VolcengineTos);
        }
        if normalized.contains("bcebos.com") {
            return Some(Self::BaiduBos);
        }
        if normalized.contains("ks3-cn") || normalized.contains("ksyuncs.com") {
            return Some(Self::KingsoftKs3);
        }
        if normalized.contains("qiniucs.com") || normalized.contains("kodo") {
            return Some(Self::QiniuKodo);
        }
        if normalized.contains("cmecloud.cn") {
            return Some(Self::ChinaMobileEcloud);
        }
        if normalized.contains("ctyun.cn") || normalized.contains("ooscn.ctyunapi.cn") {
            return Some(Self::ChinaTelecomEos);
        }
        if normalized.contains("wocloud.com") || normalized.contains("wos.com.cn") {
            return Some(Self::ChinaUnicomWo);
        }
        if normalized.contains("storage.googleapis.com") {
            return Some(Self::GoogleCloudStorage);
        }
        if normalized.contains("backblazeb2.com") {
            return Some(Self::BackblazeB2);
        }
        if normalized.contains("wasabisys.com") {
            return Some(Self::Wasabi);
        }
        if normalized.contains("digitaloceanspaces.com") {
            return Some(Self::DigitalOceanSpaces);
        }
        if normalized.contains("linodeobjects.com") {
            return Some(Self::LinodeObjectStorage);
        }
        if normalized.contains("vultrobjects.com") {
            return Some(Self::VultrObjectStorage);
        }
        if normalized.contains("scw.cloud") {
            return Some(Self::ScalewayObjectStorage);
        }
        if normalized.contains("oraclecloud.com") {
            return Some(Self::OracleCloudStorage);
        }
        if normalized.contains("cloud-object-storage") || normalized.contains(".ibm.com") {
            return Some(Self::IbmCos);
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
        let credentials = Self::resolve_credentials(credential_ref)?;
        Self::from_provider_parts_with_credentials(
            provider_kind,
            endpoint_url,
            region,
            default_bucket,
            force_path_style,
            credentials,
            strict_tls_override,
        )
    }

    /// Build a store config from an explicit in-memory credential snapshot.
    ///
    /// Consuming domains that resolve credentials from the platform provider
    /// account center (reusable service-provider accounts) hold decrypted
    /// material only in memory; routing it through a `plain:` ref would both
    /// round-trip secrets through a string and stay gated by the production
    /// plain-ref policy, so this constructor exists instead.
    pub fn from_provider_parts_with_credentials(
        provider_kind: &str,
        endpoint_url: &str,
        region: Option<&str>,
        default_bucket: &str,
        force_path_style: bool,
        credentials: DriveStorageCredentialSnapshot,
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

    /// Endpoint the account-level bucket inventory (`ListBuckets`) is read from.
    ///
    /// Normally the configured endpoint. Vendors that publish the inventory on a
    /// separate service host answer it there — but only when the configuration
    /// actually points at that vendor's own domain, so a proxied or private
    /// endpoint keeps answering the inventory itself
    /// ([`S3ProviderProfile::owns_endpoint`]).
    #[must_use]
    pub fn bucket_inventory_endpoint(&self) -> Option<&str> {
        let configured = self.endpoint.as_deref()?;
        self.provider_profile
            .service_endpoint()
            .filter(|_| self.provider_profile.owns_endpoint(configured))
            .filter(|service| *service != configured)
            .or(Some(configured))
    }

    /// Endpoint for a bucket that lives in `region`.
    ///
    /// The bucket inventory is an account-level read that spans regions, so a row
    /// the operator picked can live somewhere other than the region the provider
    /// configuration was written for. S3-compatible vendors publish the region as
    /// part of the endpoint host (`cos.ap-guangzhou.myqcloud.com`,
    /// `oss-cn-hangzhou.aliyuncs.com`, `s3.us-east-1.amazonaws.com`, ...), so the
    /// bucket's own endpoint is a host swap on the configured one.
    ///
    /// The derivation is refused — the configured endpoint is kept — unless the
    /// endpoint is the vendor's own domain
    /// ([`S3ProviderProfile::owns_endpoint`]) and the configured region is part of
    /// its host. Proxies, private gateways, and single-endpoint vendors (Cloudflare
    /// R2, Google Cloud Storage, MinIO, ...) therefore keep addressing exactly what
    /// the operator configured.
    #[must_use]
    pub fn endpoint_for_region(&self, region: &str) -> Option<String> {
        let configured_region = self.region.trim();
        let region = region.trim();
        if region.is_empty() || region == configured_region {
            return None;
        }
        validate_s3_region_token(region, "region").ok()?;
        let endpoint = self.endpoint.as_deref()?;
        if !self.provider_profile.owns_endpoint(endpoint) {
            return None;
        }
        let (scheme, rest) = endpoint.split_once("://")?;
        let (host, path) = match rest.split_once('/') {
            Some((host, path)) => (host, Some(path)),
            None => (rest, None),
        };
        if configured_region.is_empty() || !host.contains(configured_region) {
            return None;
        }
        let host = host.replacen(configured_region, region, 1);
        Some(match path {
            Some(path) => format!("{scheme}://{host}/{path}"),
            None => format!("{scheme}://{host}"),
        })
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
