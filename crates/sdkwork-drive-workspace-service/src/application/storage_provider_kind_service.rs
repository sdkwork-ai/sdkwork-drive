use std::collections::BTreeMap;

use crate::domain::storage_provider::DriveStorageProviderKind;
use crate::domain::storage_provider_kind::DriveStorageProviderKindRegistry;
use crate::ports::storage_provider_kind_store::{
    DriveStorageProviderKindStore, NewDriveStorageProviderKind,
};
use crate::DriveServiceError;

/// Built-in provider kind catalog used for registry initialization.
/// Display names mirror the operator admin surfaces.
///
/// `sort_order` groups the catalog the way an operator scans it: local first
/// (the no-credential default), then Amazon S3 as the reference implementation,
/// then the mainland-China vendors, then the rest-of-world vendors, then Google
/// Cloud Storage, which closes the group because its S3 surface is an
/// interoperability layer rather than the vendor's native protocol.
///
/// Kept in lockstep with `DriveStorageProviderKind::BUILTIN` and with the
/// Postgres `provider_kind` whitelist; the three describe the same set.
/// Table holding the built-in provider kind catalog.
///
/// Exported so infrastructure that resets Drive state (test fixtures, dev seed
/// scripts) can treat the catalog as reference data and skip it. The rows are
/// installed by the baseline DDL and migration `0008`, so truncating the table
/// would silently disable every built-in kind.
pub const STORAGE_PROVIDER_KIND_TABLE: &str = "dr_drive_storage_provider_kind";

pub const BUILTIN_STORAGE_PROVIDER_KIND_CATALOG: [(&str, &str, i32); 25] = [
    ("local_filesystem", "Local Filesystem", 1),
    ("s3_compatible", "Amazon S3 / S3 Compatible", 2),
    // --- Mainland China ---------------------------------------------------
    ("aliyun_oss", "Alibaba Cloud OSS", 10),
    ("tencent_cos", "Tencent Cloud COS", 11),
    ("huawei_obs", "Huawei Cloud OBS", 12),
    ("volcengine_tos", "Volcengine TOS", 13),
    ("baidu_bos", "Baidu Cloud BOS", 14),
    ("kingsoft_ks3", "Kingsoft Cloud KS3", 15),
    ("qiniu_kodo", "Qiniu Kodo", 16),
    ("china_mobile_ecloud", "China Mobile Ecloud", 17),
    ("china_telecom_eos", "China Telecom EOS", 18),
    ("china_unicom_wo", "China Unicom Wo Cloud", 19),
    // --- Rest of world ----------------------------------------------------
    ("minio", "MinIO", 30),
    ("cloudflare_r2", "Cloudflare R2", 31),
    ("backblaze_b2", "Backblaze B2", 32),
    ("wasabi", "Wasabi", 33),
    ("digitalocean_spaces", "DigitalOcean Spaces", 34),
    (
        "linode_object_storage",
        "Akamai / Linode Object Storage",
        35,
    ),
    ("vultr_object_storage", "Vultr Object Storage", 36),
    ("scaleway_object_storage", "Scaleway Object Storage", 37),
    (
        "oracle_cloud_storage",
        "Oracle Cloud Infrastructure Object Storage",
        38,
    ),
    ("ibm_cos", "IBM Cloud Object Storage", 39),
    (
        "alibaba_cloud_international",
        "Alibaba Cloud OSS (International)",
        40,
    ),
    (
        "tencent_cloud_international",
        "Tencent Cloud COS (International)",
        41,
    ),
    ("google_cloud_storage", "Google Cloud Storage", 50),
];

#[derive(Debug, Clone)]
pub struct SetStorageProviderKindEnabledCommand {
    pub provider_kind: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageProviderKindSummary {
    pub kind: DriveStorageProviderKindRegistry,
    pub config_count: i64,
}

#[derive(Debug, Clone)]
pub struct DriveStorageProviderKindService<S>
where
    S: DriveStorageProviderKindStore,
{
    store: S,
}

impl<S> DriveStorageProviderKindService<S>
where
    S: DriveStorageProviderKindStore,
{
    const DISABLED_KIND_CONFLICT: &'static str =
        "storage provider kind is disabled; enable the provider kind before creating or activating its configurations";

    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Idempotently (re)initialize the built-in provider kind catalog.
    /// Existing rows keep their operator-managed enabled flag; missing rows
    /// are inserted. Safe to call at any time.
    pub async fn initialize_storage_provider_kinds(
        &self,
    ) -> Result<Vec<DriveStorageProviderKindRegistry>, DriveServiceError> {
        let kinds = BUILTIN_STORAGE_PROVIDER_KIND_CATALOG
            .iter()
            .map(
                |(provider_kind, display_name, sort_order)| NewDriveStorageProviderKind {
                    provider_kind: (*provider_kind).to_string(),
                    display_name: (*display_name).to_string(),
                    sort_order: *sort_order,
                },
            )
            .collect::<Vec<_>>();
        self.store.initialize_storage_provider_kinds(&kinds).await
    }

    /// List the provider kind catalog with per-kind configuration counts
    /// (non-deleted `dr_drive_storage_provider` rows).
    ///
    /// `locale` is a normalized BCP 47 tag from the supported set (`zh-CN`,
    /// `en-US`); when present, the seeded translation for that locale replaces
    /// the locale-neutral base display name, and a missing translation falls
    /// back to the base name.
    pub async fn list_storage_provider_kinds(
        &self,
        locale: Option<&str>,
    ) -> Result<Vec<StorageProviderKindSummary>, DriveServiceError> {
        let kinds = self.store.list_storage_provider_kinds(locale).await?;
        let counts = self
            .store
            .count_storage_provider_configs_by_kind()
            .await?
            .into_iter()
            .map(|count| (count.provider_kind, count.config_count))
            .collect::<BTreeMap<_, _>>();
        Ok(kinds
            .into_iter()
            .map(|kind| StorageProviderKindSummary {
                config_count: counts.get(&kind.provider_kind).copied().unwrap_or(0),
                kind,
            })
            .collect())
    }

    pub async fn get_storage_provider_kind(
        &self,
        provider_kind: &str,
    ) -> Result<DriveStorageProviderKindRegistry, DriveServiceError> {
        let provider_kind = provider_kind.trim();
        if provider_kind.is_empty() {
            return Err(DriveServiceError::Validation(
                "provider_kind is required".to_string(),
            ));
        }
        let kind = parse_builtin_provider_kind_key(provider_kind)?;
        self.store
            .find_storage_provider_kind(kind)
            .await?
            .ok_or_else(|| {
                DriveServiceError::NotFound("storage provider kind not found".to_string())
            })
    }

    pub async fn set_storage_provider_kind_enabled(
        &self,
        command: SetStorageProviderKindEnabledCommand,
    ) -> Result<DriveStorageProviderKindRegistry, DriveServiceError> {
        let kind = parse_builtin_provider_kind_key(&command.provider_kind)?;
        if self.store.find_storage_provider_kind(kind).await?.is_none() {
            return Err(DriveServiceError::NotFound(
                "storage provider kind not found".to_string(),
            ));
        }
        self.store
            .set_storage_provider_kind_enabled(kind, command.enabled)
            .await
    }

    /// Ensure a provider kind may host new or reactivated configurations.
    /// Custom kinds are operator-defined and always available; built-in kinds
    /// must be registered and enabled.
    pub async fn ensure_storage_provider_kind_available(
        &self,
        provider_kind: &DriveStorageProviderKind,
    ) -> Result<(), DriveServiceError> {
        let kind_key = match provider_kind {
            DriveStorageProviderKind::Custom(_) => return Ok(()),
            _ => provider_kind.as_str(),
        };
        let registered = self
            .store
            .find_storage_provider_kind(kind_key)
            .await?
            .ok_or_else(|| {
                DriveServiceError::NotFound(
                    "storage provider kind is not initialized; initialize the provider kind catalog first"
                        .to_string(),
                )
            })?;
        if !registered.enabled {
            return Err(DriveServiceError::Conflict(
                Self::DISABLED_KIND_CONFLICT.to_string(),
            ));
        }
        Ok(())
    }
}

fn parse_builtin_provider_kind_key(raw: &str) -> Result<&str, DriveServiceError> {
    let normalized = raw.trim();
    match DriveStorageProviderKind::try_from_str(normalized) {
        Some(DriveStorageProviderKind::Custom(_)) => Err(DriveServiceError::Validation(
            "custom provider kinds cannot be managed as built-in kinds".to_string(),
        )),
        Some(_) => Ok(normalized),
        None => Err(DriveServiceError::Validation(
            "provider_kind is invalid".to_string(),
        )),
    }
}
