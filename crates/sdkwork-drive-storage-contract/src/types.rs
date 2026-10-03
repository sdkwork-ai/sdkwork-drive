use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Every provider kind the platform treats as first class.
///
/// The first seven entries are the original catalog. Everything after them is a
/// named S3-compatible vendor added so an operator picks "Wasabi" instead of
/// hand-writing `custom:wasabi`: a named kind carries its own default region,
/// path-style default and endpoint template, which a free-form custom key
/// cannot. `Custom` stays the escape hatch for vendors not listed here.
///
/// `China mainland` and `rest of world` vendors are both represented, so a
/// tenant in either market can complete the console's provider picker without
/// falling back to a custom key.
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
    // --- Mainland China vendors -------------------------------------------
    BaiduBos,
    KingsoftKs3,
    QiniuKodo,
    ChinaMobileEcloud,
    ChinaTelecomEos,
    ChinaUnicomWo,
    // --- Rest-of-world vendors --------------------------------------------
    Minio,
    CloudflareR2,
    BackblazeB2,
    Wasabi,
    DigitalOceanSpaces,
    LinodeObjectStorage,
    VultrObjectStorage,
    ScalewayObjectStorage,
    OracleCloudStorage,
    IbmCos,
    AlibabaCloudInternational,
    TencentCloudInternational,
    Custom(String),
}

impl DriveStorageProviderKind {
    pub const CUSTOM_PREFIX: &'static str = "custom:";

    /// Provider kinds with an independent, named catalog entry, in catalog
    /// order. `Custom` is excluded: it is not a catalog row, it is the
    /// operator-defined fallback.
    pub const BUILTIN: &'static [Self] = &[
        Self::LocalFilesystem,
        Self::S3Compatible,
        Self::GoogleCloudStorage,
        Self::AliyunOss,
        Self::TencentCos,
        Self::HuaweiObs,
        Self::VolcengineTos,
        Self::BaiduBos,
        Self::KingsoftKs3,
        Self::QiniuKodo,
        Self::ChinaMobileEcloud,
        Self::ChinaTelecomEos,
        Self::ChinaUnicomWo,
        Self::Minio,
        Self::CloudflareR2,
        Self::BackblazeB2,
        Self::Wasabi,
        Self::DigitalOceanSpaces,
        Self::LinodeObjectStorage,
        Self::VultrObjectStorage,
        Self::ScalewayObjectStorage,
        Self::OracleCloudStorage,
        Self::IbmCos,
        Self::AlibabaCloudInternational,
        Self::TencentCloudInternational,
    ];

    pub fn as_str(&self) -> &str {
        match self {
            Self::LocalFilesystem => "local_filesystem",
            Self::S3Compatible => "s3_compatible",
            Self::GoogleCloudStorage => "google_cloud_storage",
            Self::AliyunOss => "aliyun_oss",
            Self::TencentCos => "tencent_cos",
            Self::HuaweiObs => "huawei_obs",
            Self::VolcengineTos => "volcengine_tos",
            Self::BaiduBos => "baidu_bos",
            Self::KingsoftKs3 => "kingsoft_ks3",
            Self::QiniuKodo => "qiniu_kodo",
            Self::ChinaMobileEcloud => "china_mobile_ecloud",
            Self::ChinaTelecomEos => "china_telecom_eos",
            Self::ChinaUnicomWo => "china_unicom_wo",
            Self::Minio => "minio",
            Self::CloudflareR2 => "cloudflare_r2",
            Self::BackblazeB2 => "backblaze_b2",
            Self::Wasabi => "wasabi",
            Self::DigitalOceanSpaces => "digitalocean_spaces",
            Self::LinodeObjectStorage => "linode_object_storage",
            Self::VultrObjectStorage => "vultr_object_storage",
            Self::ScalewayObjectStorage => "scaleway_object_storage",
            Self::OracleCloudStorage => "oracle_cloud_storage",
            Self::IbmCos => "ibm_cos",
            Self::AlibabaCloudInternational => "alibaba_cloud_international",
            Self::TencentCloudInternational => "tencent_cloud_international",
            Self::Custom(value) => value.as_str(),
        }
    }

    /// Whether the kind addresses storage over an S3-compatible HTTP API.
    /// `LocalFilesystem` is the only kind that does not.
    pub fn is_s3_compatible(&self) -> bool {
        !matches!(self, Self::LocalFilesystem)
    }

    pub fn try_from_str(raw: &str) -> Option<Self> {
        let normalized = raw.trim().to_ascii_lowercase();
        // Aliases let an operator type the vendor's own short name where a
        // catalog key exists, mirroring the endpoint-side detection in
        // `S3ProviderProfile::from_vendor_key`.
        let resolved = match normalized.as_str() {
            "local_filesystem" | "local" => Self::LocalFilesystem,
            "s3_compatible" | "s3" | "aws_s3" | "amazon_s3" => Self::S3Compatible,
            "google_cloud_storage" | "gcs" => Self::GoogleCloudStorage,
            "aliyun_oss" | "oss" | "aliyun" | "alibaba_oss" => Self::AliyunOss,
            "tencent_cos" | "cos" | "tencent" => Self::TencentCos,
            "huawei_obs" | "obs" | "huawei" => Self::HuaweiObs,
            "volcengine_tos" | "tos" | "volcengine" | "volcano" | "bytedance_tos" => {
                Self::VolcengineTos
            }
            "baidu_bos" | "bos" | "baidu" => Self::BaiduBos,
            "kingsoft_ks3" | "ks3" | "kingsoft" => Self::KingsoftKs3,
            "qiniu_kodo" | "kodo" | "qiniu" => Self::QiniuKodo,
            "china_mobile_ecloud" | "ecloud" | "china_mobile" | "cmecloud" => {
                Self::ChinaMobileEcloud
            }
            "china_telecom_eos" | "eos" | "china_telecom" | "ctyun_eos" => Self::ChinaTelecomEos,
            "china_unicom_wo" | "unicom_wo" | "china_unicom" | "wocloud" => Self::ChinaUnicomWo,
            "minio" => Self::Minio,
            "cloudflare_r2" | "r2" | "cloudflare" => Self::CloudflareR2,
            "backblaze_b2" | "b2" | "backblaze" => Self::BackblazeB2,
            "wasabi" => Self::Wasabi,
            "digitalocean_spaces" | "spaces" | "digitalocean" | "do_spaces" => {
                Self::DigitalOceanSpaces
            }
            "linode_object_storage" | "linode" | "akamai" => Self::LinodeObjectStorage,
            "vultr_object_storage" | "vultr" => Self::VultrObjectStorage,
            "scaleway_object_storage" | "scaleway" => Self::ScalewayObjectStorage,
            "oracle_cloud_storage" | "oci" | "oracle" => Self::OracleCloudStorage,
            "ibm_cos" | "ibm" | "cos_ibm" => Self::IbmCos,
            "alibaba_cloud_international" | "aliyun_intl" | "alibabacloud_intl" => {
                Self::AlibabaCloudInternational
            }
            "tencent_cloud_international" | "tencent_intl" | "cos_intl" => {
                Self::TencentCloudInternational
            }
            _ => {
                let suffix = normalized.strip_prefix(Self::CUSTOM_PREFIX)?;
                if is_valid_custom_suffix(suffix) {
                    Self::Custom(normalized)
                } else {
                    return None;
                }
            }
        };
        Some(resolved)
    }

    /// Whether the kind is addressed by a catalog key rather than a
    /// `custom:<vendor>` key.
    pub fn is_builtin(&self) -> bool {
        !matches!(self, Self::Custom(_))
    }

    /// Default region when a configuration leaves `region` empty.
    ///
    /// `auto` on Cloudflare R2 (which rejects any other literal); `us-east-1`
    /// elsewhere, which the remaining vendors either treat as their own default
    /// or ignore because the endpoint carries the real region.
    ///
    /// A `Custom` key resolves through [`Self::try_from_str`] so that
    /// `custom:r2` gets the same answer as the named `cloudflare_r2` kind.
    pub fn default_region(&self) -> &'static str {
        match self.named_vendor() {
            Some(Self::CloudflareR2) => "auto",
            _ => "us-east-1",
        }
    }

    /// Whether the vendor serves bucket-in-path (`true`) or bucket-in-host
    /// (`false`) addressing by default.
    ///
    /// Self-hosted and generic endpoints default to path style; the named cloud
    /// vendors publish virtual-hosted-style endpoints. `S3Compatible` is the
    /// "any S3 vendor" key, so it cannot assume a virtual-hosted endpoint and
    /// takes the generic answer. `Custom` follows its resolved vendor when the
    /// key names one, and otherwise assumes path style, which is the safe
    /// default for an unknown endpoint.
    pub fn default_force_path_style(&self) -> bool {
        match self.named_vendor() {
            Some(
                Self::S3Compatible
                | Self::Minio
                | Self::CloudflareR2
                | Self::ChinaMobileEcloud
                | Self::ChinaTelecomEos
                | Self::ChinaUnicomWo
                | Self::LinodeObjectStorage
                | Self::VultrObjectStorage
                | Self::IbmCos,
            ) => true,
            Some(_) => false,
            // Unknown custom vendor: path style is the permissive default.
            None => true,
        }
    }

    /// Resolve a `Custom("<vendor>")` key to the catalog kind it names, so
    /// vendor defaults are shared between `custom:wasabi` and `wasabi` instead
    /// of being duplicated per call site.
    ///
    /// Returns `None` when the key does not name a known vendor. A known vendor
    /// always comes back as its non-`Custom` variant, which is what makes the
    /// caller's `Option<Self>` match exhaustive without a second `Custom` arm.
    fn named_vendor(&self) -> Option<Self> {
        match self {
            Self::Custom(value) => {
                let suffix = value.strip_prefix(Self::CUSTOM_PREFIX)?;
                let resolved = Self::try_from_str(suffix)?;
                if matches!(resolved, Self::Custom(_)) {
                    return None;
                }
                Some(resolved)
            }
            _ => Some(self.clone()),
        }
    }

    /// The server-side-encryption modes this vendor actually accepts.
    ///
    /// Returned in preference order, so callers may take `[0]` as the default a
    /// fresh configuration should carry. This is a *vendor* property, not a
    /// protocol one: an S3-compatible store each names its own encryption
    /// values (`aws:kms` on AWS, `KMS` on Aliyun/Tencent/Huawei, `AES256`
    /// everywhere), so answering with one generic list hands Tencent's editor a
    /// value Tencent rejects.
    ///
    /// A `Custom` key follows the vendor it names; an unknown one gets the
    /// conservative S3 baseline (`AES256`), which every S3-compatible store
    /// accepts.
    pub fn supported_sse_modes(&self) -> &'static [&'static str] {
        match self.named_vendor() {
            Some(Self::S3Compatible) => &["AES256", "aws:kms", "aws:kms:dsse"],
            Some(Self::AliyunOss | Self::AlibabaCloudInternational) => &["KMS", "AES256"],
            Some(Self::TencentCos | Self::TencentCloudInternational) => &["AES256", "KMS"],
            // TOS happens to sit on the same two tokens as Tencent today, but it
            // is its own vendor: keeping it in a separate arm means a later change
            // to either vendor's SSE vocabulary cannot silently drag the other
            // along. The storage-class table below splits it for the same reason
            // (there the values genuinely differ: `IA` vs `STANDARD_IA`).
            Some(Self::VolcengineTos) => &["AES256", "KMS"],
            Some(Self::HuaweiObs) => &["kms", "AES256"],
            Some(Self::GoogleCloudStorage) => &["AES256", "GOOGLE_DEFAULT_ENCRYPTION"],
            Some(Self::LocalFilesystem) => &[],
            // Every remaining vendor encrypts at rest with server-managed keys
            // and exposes the plain S3 `AES256` token for it. `named_vendor`
            // never yields `Custom`, so this arm also absorbs any future vendor.
            Some(_) => &["AES256"],
            None => &["AES256"],
        }
    }

    /// The storage classes this vendor actually accepts, in preference order
    /// (so `[0]` is the default a fresh configuration should carry).
    ///
    /// Same vendor-not-protocol rule as [`Self::supported_sse_modes`]: Aliyun's
    /// tiers are `Standard`/`IA`, IBM's are `VAULT`/`COLD`, Oracle's are
    /// `InfrequentAccess` — none of which AWS's list would name.
    ///
    /// An empty slice means "this vendor exposes no storage-class choice", which
    /// is honest for a local filesystem and safe for an unknown custom vendor
    /// (the console then leaves the field unset rather than offering a tier the
    /// store would reject).
    pub fn supported_storage_classes(&self) -> &'static [&'static str] {
        match self.named_vendor() {
            Some(Self::S3Compatible) => &[
                "STANDARD",
                "STANDARD_IA",
                "ONEZONE_IA",
                "INTELLIGENT_TIERING",
                "GLACIER",
                "DEEP_ARCHIVE",
            ],
            Some(Self::AliyunOss | Self::AlibabaCloudInternational) => &[
                "Standard",
                "IA",
                "Archive",
                "ColdArchive",
                "DeepColdArchive",
            ],
            Some(Self::TencentCos | Self::TencentCloudInternational) => {
                &["STANDARD", "STANDARD_IA", "ARCHIVE", "DEEP_ARCHIVE"]
            }
            // Volcengine TOS names its infrequent-access tier `IA`, not AWS's
            // `STANDARD_IA`, and rejects the latter — the same vendor-not-protocol
            // trap as Aliyun's `Standard`. It is deliberately not grouped with
            // Tencent above for exactly that reason.
            Some(Self::VolcengineTos) => &["STANDARD", "IA", "ARCHIVE", "DEEP_ARCHIVE"],
            Some(Self::HuaweiObs) => &["STANDARD", "WARM", "COLD"],
            Some(Self::GoogleCloudStorage) => &["STANDARD", "NEARLINE", "COLDLINE", "ARCHIVE"],
            Some(Self::BaiduBos) => &["STANDARD", "STANDARD_IA", "COLD", "ARCHIVE"],
            Some(Self::KingsoftKs3) => &["STANDARD", "STANDARD_IA", "ARCHIVE"],
            Some(Self::QiniuKodo) => &["STANDARD", "LINE", "ARCHIVE", "ARCHIVE_IA"],
            Some(Self::ChinaMobileEcloud | Self::ChinaTelecomEos | Self::ChinaUnicomWo) => {
                &["STANDARD", "STANDARD_IA", "ARCHIVE"]
            }
            // Wasabi is single-tier by design: "there are no archival or warm
            // storage tiers; it is all low-latency hot storage". Offering it
            // Glacier-style tiers would present choices the store rejects.
            Some(Self::Wasabi) => &["STANDARD"],
            Some(Self::ScalewayObjectStorage) => &["STANDARD", "ONEZONE_IA", "GLACIER"],
            Some(Self::OracleCloudStorage) => &["Standard", "InfrequentAccess", "Archive"],
            Some(Self::IbmCos) => &["STANDARD", "VAULT", "COLD", "SMART"],
            Some(
                Self::Minio
                | Self::CloudflareR2
                | Self::BackblazeB2
                | Self::DigitalOceanSpaces
                | Self::LinodeObjectStorage
                | Self::VultrObjectStorage,
            ) => &["STANDARD"],
            Some(Self::LocalFilesystem) => &[],
            // Any vendor that reaches here (a future one, or a `Custom` key that
            // names no catalog entry) gets no storage-class choice rather than a
            // fabricated tier the store would reject.
            Some(_) => &[],
            None => &[],
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

/// Stage an object whose bytes are already written to a local file.
///
/// `source_path` is a runtime-local path, never persisted as Drive state: it is
/// the staging file a caller produced (for example a spooled download-package
/// archive) and is deleted by that caller once the upload returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PutObjectFromPathRequest {
    pub locator: DriveObjectLocator,
    pub content_type: Option<String>,
    pub metadata: DriveObjectMetadata,
    pub source_path: std::path::PathBuf,
    pub checksum_sha256_hex: Option<String>,
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
    /// Region the vendor reports for this bucket, when it reports one.
    ///
    /// The bucket inventory is an account-level read that spans regions, so the
    /// region is a property of the *row*, not of the provider configuration: it
    /// is what tells an operator where a bucket lives and what lets a later
    /// object operation address that bucket's own regional endpoint.
    pub region: Option<String>,
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

pub const S3_REGION_TOKEN_PATTERN: &str = "^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?$";

pub const S3_REGION_TOKEN_DESCRIPTION: &str =
    "Vendor region code as the provider publishes it (for example `ap-guangzhou`, `cn-hangzhou`, `us-east-1`). Letters, digits, and single hyphens; 1-63 characters; must start and end with a letter or digit.";

/// Validate a vendor region code before it is allowed to take part in endpoint
/// derivation.
///
/// The code reaches the endpoint host, so it is validated exactly like a bucket
/// name is: letters, digits, and inner hyphens only. Anything else — a dot, a
/// slash, a colon, an empty string — is rejected rather than substituted, which
/// keeps a caller from steering a request at a host of its own choosing.
pub fn validate_s3_region_token(raw: &str, field_name: &str) -> Result<(), DriveObjectStoreError> {
    let region = raw.trim();
    if raw != region {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("{field_name} must be trimmed"),
        ));
    }
    if region.is_empty() || region.len() > 63 {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("{field_name} must be between 1 and 63 characters"),
        ));
    }
    let mut bytes = region.bytes();
    let starts_with_alnum = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphanumeric());
    let ends_with_alnum = region
        .bytes()
        .last()
        .is_some_and(|byte| byte.is_ascii_alphanumeric());
    if !starts_with_alnum || !ends_with_alnum {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("{field_name} must start and end with a letter or digit"),
        ));
    }
    if !region
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(DriveObjectStoreError::new(
            DriveObjectStoreErrorKind::InvalidRequest,
            format!("{field_name} may only contain letters, digits, or hyphen"),
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

#[cfg(test)]
mod vendor_capability_defaults_tests {
    use super::DriveStorageProviderKind;

    /// Every built-in cloud kind must offer at least one SSE mode and at least
    /// one storage class, or the console renders an empty dropdown and the
    /// bootstrap has no default to write — the "傻瓜式" guarantee fails silently
    /// for that vendor only.
    #[test]
    fn every_cloud_vendor_offers_a_default_sse_mode_and_storage_class() {
        let cloud_kinds = [
            "s3_compatible",
            "aliyun_oss",
            "tencent_cos",
            "huawei_obs",
            "volcengine_tos",
            "google_cloud_storage",
            "baidu_bos",
            "kingsoft_ks3",
            "qiniu_kodo",
            "china_mobile_ecloud",
            "china_telecom_eos",
            "china_unicom_wo",
            "minio",
            "cloudflare_r2",
            "backblaze_b2",
            "wasabi",
            "digitalocean_spaces",
            "linode_object_storage",
            "vultr_object_storage",
            "scaleway_object_storage",
            "oracle_cloud_storage",
            "ibm_cos",
            "alibaba_cloud_international",
            "tencent_cloud_international",
        ];
        for key in cloud_kinds {
            let kind = DriveStorageProviderKind::try_from_str(key)
                .unwrap_or_else(|| panic!("{key} is not a catalogued kind"));
            assert!(
                !kind.supported_sse_modes().is_empty(),
                "{key} offers no SSE mode, so its editor dropdown would be empty"
            );
            assert!(
                !kind.supported_storage_classes().is_empty(),
                "{key} offers no storage class, so its editor dropdown would be empty"
            );
        }
    }

    #[test]
    fn the_local_filesystem_claims_no_cloud_tiers() {
        let kind = DriveStorageProviderKind::LocalFilesystem;
        assert!(kind.supported_sse_modes().is_empty());
        assert!(kind.supported_storage_classes().is_empty());
    }

    /// The vendor tables must be genuinely per-vendor: if every kind answered
    /// with AWS's list, the drift this table exists to prevent would be back.
    #[test]
    fn vendors_do_not_all_share_one_generic_list() {
        let aws = DriveStorageProviderKind::try_from_str("s3_compatible").unwrap();
        let aliyun = DriveStorageProviderKind::try_from_str("aliyun_oss").unwrap();
        let ibm = DriveStorageProviderKind::try_from_str("ibm_cos").unwrap();

        assert_ne!(
            aws.supported_storage_classes(),
            aliyun.supported_storage_classes(),
            "Aliyun must not answer with AWS's storage classes"
        );
        assert_ne!(
            aws.supported_sse_modes(),
            aliyun.supported_sse_modes(),
            "Aliyun names its KMS mode differently from AWS"
        );
        assert!(ibm.supported_storage_classes().contains(&"VAULT"));
        assert!(
            !aws.supported_storage_classes().contains(&"VAULT"),
            "AWS has no VAULT tier; a shared list would have leaked it"
        );
    }

    /// A `custom:<vendor>` key resolves to its named vendor, so a custom
    /// Wasabi entry inherits Wasabi's tiers instead of the generic baseline.
    #[test]
    fn custom_keys_follow_the_vendor_they_name() {
        let named = DriveStorageProviderKind::try_from_str("wasabi").unwrap();
        let custom = DriveStorageProviderKind::try_from_str("custom:wasabi").unwrap();
        assert_eq!(
            named.supported_storage_classes(),
            custom.supported_storage_classes()
        );
        assert_eq!(named.supported_sse_modes(), custom.supported_sse_modes());

        // An unknown custom vendor still gets the conservative S3 baseline for
        // SSE, and no fabricated storage class.
        let unknown = DriveStorageProviderKind::try_from_str("custom:my-own-store").unwrap();
        assert_eq!(unknown.supported_sse_modes(), &["AES256"]);
        assert!(unknown.supported_storage_classes().is_empty());
    }

    /// The default a fresh configuration carries is the *first* entry, so the
    /// tables are ordered lists, not sets. Each of these vendors leads with the
    /// standard tier it actually names (`Standard` for Aliyun/OCI, `STANDARD`
    /// for the AWS-derived ones).
    #[test]
    fn the_first_entry_is_the_vendors_standard_tier() {
        let expectations = [
            ("s3_compatible", "STANDARD"),
            ("tencent_cos", "STANDARD"),
            ("minio", "STANDARD"),
            ("ibm_cos", "STANDARD"),
            ("aliyun_oss", "Standard"),
            ("oracle_cloud_storage", "Standard"),
        ];
        for (key, expected) in expectations {
            let kind = DriveStorageProviderKind::try_from_str(key).unwrap();
            assert_eq!(
                kind.supported_storage_classes().first().copied(),
                Some(expected),
                "{key} must lead with its standard tier"
            );
        }
    }

    /// Vendors that name the *same* concept differently must not be grouped into
    /// one match arm. This is the exact slip the console parity test exposed:
    /// Volcengine TOS had been folded in with Tencent and inherited AWS's
    /// `STANDARD_IA`, a token TOS rejects (it calls the tier `IA`).
    ///
    /// Asserting membership rather than inequality is what makes it bite: two
    /// lists can differ in length and still both carry the wrong token.
    #[test]
    fn vendors_that_rename_a_tier_are_not_grouped_with_its_other_name() {
        let tos = DriveStorageProviderKind::try_from_str("volcengine_tos").unwrap();
        let classes = tos.supported_storage_classes();
        assert!(
            classes.contains(&"IA"),
            "TOS names its infrequent-access tier IA, got {classes:?}"
        );
        assert!(
            !classes.contains(&"STANDARD_IA"),
            "TOS rejects AWS's STANDARD_IA spelling, got {classes:?}"
        );

        // Aliyun is the same trap on the standard tier itself.
        let aliyun = DriveStorageProviderKind::try_from_str("aliyun_oss").unwrap();
        assert!(aliyun.supported_storage_classes().contains(&"Standard"));
        assert!(
            !aliyun.supported_storage_classes().contains(&"STANDARD"),
            "Aliyun spells its standard tier Standard, not STANDARD"
        );
    }

    /// A vendor whose two vocabularies happen to coincide with a sibling's today
    /// must still resolve through its own arm, or a future divergence on either
    /// side silently drags the other along.
    ///
    /// This is the *other* half of the slip above: TOS's storage classes were
    /// split off Tencent but its SSE list was folded back in, and no test
    /// noticed because both vendors currently sit on `["AES256", "KMS"]`. The
    /// check is that each named kind resolves to its own `named_vendor()` — not
    /// to a shared match arm — which is observable by round-tripping the kind
    /// through its own `Custom` spelling: an arm shared with a sibling would
    /// still answer, but a kind that stopped being catalogued would not.
    #[test]
    fn a_vendor_resolves_through_its_own_entry_not_a_siblings() {
        // Every catalogued cloud kind answers identically whether reached by its
        // own variant or by the `custom:<key>` alias that names it. A kind whose
        // arm was deleted (or whose key stopped parsing) breaks this pairing.
        for key in [
            "volcengine_tos",
            "tencent_cos",
            "tencent_cloud_international",
            "baidu_bos",
            "kingsoft_ks3",
            "qiniu_kodo",
            "wasabi",
            "ibm_cos",
            "oracle_cloud_storage",
            "scaleway_object_storage",
        ] {
            let named = DriveStorageProviderKind::try_from_str(key).unwrap();
            let aliased = DriveStorageProviderKind::Custom(format!("custom:{key}"));
            assert_eq!(
                named.named_vendor(),
                Some(named.clone()),
                "{key} no longer resolves to itself"
            );
            assert_eq!(
                aliased.supported_sse_modes(),
                named.supported_sse_modes(),
                "{key} answers differently through its custom alias"
            );
            assert_eq!(
                aliased.supported_storage_classes(),
                named.supported_storage_classes(),
                "{key} answers differently through its custom alias"
            );
        }

        // And the concrete invariant that pins TOS apart from Tencent: their
        // storage classes must not be the same list, because TOS spells the
        // infrequent tier `IA` where Tencent spells it `STANDARD_IA`. If a
        // refactor ever merges the arms, this goes red.
        let tos = DriveStorageProviderKind::VolcengineTos;
        let tencent = DriveStorageProviderKind::TencentCos;
        assert_ne!(
            tos.supported_storage_classes(),
            tencent.supported_storage_classes(),
            "TOS and Tencent spell their tiers differently and must not share an arm"
        );
        assert!(
            tos.supported_storage_classes().len() == tencent.supported_storage_classes().len(),
            "the two vendors expose the same number of tiers, so only the spelling differs"
        );
    }
}
