use chrono::{DateTime, Duration, SecondsFormat, Utc};

use crate::domain::website_sync::{
    validate_website_sync_tree, DriveWebsiteManifestSummary, DriveWebsiteSync,
    DriveWebsiteSyncStatus, MAX_WEBSITE_SYNC_FILES, MAX_WEBSITE_SYNC_TOTAL_BYTES,
};
use crate::infrastructure::sql::website_sync_store::SqlWebsiteSyncStore;
use crate::ports::website_sync_store::{
    AbortDriveWebsiteSync, ActivateDriveWebsiteGeneration, ActivateValidatedWebsiteSync,
    CreateDriveWebsiteSync, CreateDriveWebsiteSyncResult, DriveWebsiteGenerationActivation,
    DriveWebsiteSyncActivation, DriveWebsiteSyncStore, MarkDriveWebsiteSyncFailed,
    ValidateDriveWebsiteSync,
};
use crate::DriveServiceError;

const MAX_SYNC_LIFETIME_HOURS: i64 = 24;

#[derive(Debug, Clone)]
pub struct CreateWebsiteSyncCommand {
    pub tenant_id: String,
    pub website_root_uuid: String,
    pub idempotency_key: String,
    pub expected_root_version: i64,
    pub expected_generation: i64,
    pub manifest_sha256: String,
    pub manifest_file_count: i64,
    pub manifest_total_bytes: i64,
    pub expires_at: String,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct GetWebsiteSyncCommand {
    pub tenant_id: String,
    pub website_root_uuid: String,
    pub sync_id: String,
}

#[derive(Debug, Clone)]
pub struct FinalizeWebsiteSyncCommand {
    pub tenant_id: String,
    pub website_root_uuid: String,
    pub sync_id: String,
    pub expected_sync_version: i64,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct AbortWebsiteSyncCommand {
    pub tenant_id: String,
    pub website_root_uuid: String,
    pub sync_id: String,
    pub expected_sync_version: i64,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct ActivateWebsiteGenerationCommand {
    pub tenant_id: String,
    pub website_root_uuid: String,
    pub target_generation: i64,
    pub expected_root_version: i64,
    pub expected_generation: i64,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct DriveWebsiteSyncService<S>
where
    S: DriveWebsiteSyncStore,
{
    store: S,
}

impl<S> DriveWebsiteSyncService<S>
where
    S: DriveWebsiteSyncStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn create_sync(
        &self,
        command: CreateWebsiteSyncCommand,
    ) -> Result<CreateDriveWebsiteSyncResult, DriveServiceError> {
        let manifest = validate_declared_manifest(
            command.manifest_sha256,
            command.manifest_file_count,
            command.manifest_total_bytes,
        )?;
        let expires_at = validate_expiry(command.expires_at)?;
        self.store
            .create_or_get(&CreateDriveWebsiteSync {
                tenant_id: require_text(command.tenant_id, "tenant_id", 64)?,
                website_root_uuid: require_text(
                    command.website_root_uuid,
                    "website_root_uuid",
                    64,
                )?,
                idempotency_key: require_idempotency_key(command.idempotency_key)?,
                expected_root_version: require_positive(
                    command.expected_root_version,
                    "expected_root_version",
                )?,
                expected_generation: require_positive(
                    command.expected_generation,
                    "expected_generation",
                )?,
                manifest_sha256: manifest.sha256,
                manifest_file_count: manifest.file_count,
                manifest_total_bytes: manifest.total_bytes,
                expires_at,
                operator_id: require_text(command.operator_id, "operator_id", 128)?,
            })
            .await
    }

    pub async fn get_sync(
        &self,
        command: GetWebsiteSyncCommand,
    ) -> Result<DriveWebsiteSync, DriveServiceError> {
        self.store
            .get(
                &require_text(command.tenant_id, "tenant_id", 64)?,
                &require_text(command.website_root_uuid, "website_root_uuid", 64)?,
                &require_text(command.sync_id, "sync_id", 64)?,
            )
            .await
    }

    pub async fn finalize_sync(
        &self,
        command: FinalizeWebsiteSyncCommand,
    ) -> Result<DriveWebsiteSyncActivation, DriveServiceError> {
        let validation = ValidateDriveWebsiteSync {
            tenant_id: require_text(command.tenant_id, "tenant_id", 64)?,
            website_root_uuid: require_text(command.website_root_uuid, "website_root_uuid", 64)?,
            sync_id: require_text(command.sync_id, "sync_id", 64)?,
            expected_sync_version: require_positive(
                command.expected_sync_version,
                "expected_sync_version",
            )?,
            operator_id: require_text(command.operator_id, "operator_id", 128)?,
        };
        let validation_lease = self.store.begin_validation(&validation).await?;
        let sync = validation_lease.sync;
        let observed_manifest = if sync.status == DriveWebsiteSyncStatus::Completed {
            declared_manifest(&sync)
        } else {
            let entries = self
                .store
                .list_staging_tree(
                    &validation.tenant_id,
                    &validation.website_root_uuid,
                    &validation.sync_id,
                )
                .await?;
            match validate_website_sync_tree(&entries) {
                Ok(manifest) => manifest,
                Err(error) => {
                    self.persist_validation_failure(
                        &validation,
                        sync.version,
                        validation_lease.lease_token.as_deref(),
                        &error,
                    )
                    .await;
                    return Err(error);
                }
            }
        };
        if observed_manifest != declared_manifest(&sync) {
            let error = DriveServiceError::Validation("WEBSITE_SYNC_MANIFEST_MISMATCH".to_string());
            self.persist_validation_failure(
                &validation,
                sync.version,
                validation_lease.lease_token.as_deref(),
                &error,
            )
            .await;
            return Err(error);
        }

        let activation = self
            .store
            .activate_validated(&ActivateValidatedWebsiteSync {
                tenant_id: validation.tenant_id.clone(),
                website_root_uuid: validation.website_root_uuid.clone(),
                sync_id: validation.sync_id.clone(),
                expected_sync_version: sync.version,
                lease_token: validation_lease.lease_token.clone(),
                observed_manifest,
                operator_id: validation.operator_id.clone(),
            })
            .await;
        if let Err(error) = &activation {
            if matches!(error, DriveServiceError::Validation(_)) {
                self.persist_validation_failure(
                    &validation,
                    sync.version,
                    validation_lease.lease_token.as_deref(),
                    error,
                )
                .await;
            }
        }
        activation
    }

    pub async fn abort_sync(
        &self,
        command: AbortWebsiteSyncCommand,
    ) -> Result<DriveWebsiteSync, DriveServiceError> {
        self.store
            .abort(&AbortDriveWebsiteSync {
                tenant_id: require_text(command.tenant_id, "tenant_id", 64)?,
                website_root_uuid: require_text(
                    command.website_root_uuid,
                    "website_root_uuid",
                    64,
                )?,
                sync_id: require_text(command.sync_id, "sync_id", 64)?,
                expected_sync_version: require_positive(
                    command.expected_sync_version,
                    "expected_sync_version",
                )?,
                operator_id: require_text(command.operator_id, "operator_id", 128)?,
            })
            .await
    }

    pub async fn activate_generation(
        &self,
        command: ActivateWebsiteGenerationCommand,
    ) -> Result<DriveWebsiteGenerationActivation, DriveServiceError> {
        self.store
            .activate_generation(&ActivateDriveWebsiteGeneration {
                tenant_id: require_text(command.tenant_id, "tenant_id", 64)?,
                website_root_uuid: require_text(
                    command.website_root_uuid,
                    "website_root_uuid",
                    64,
                )?,
                target_generation: require_positive(
                    command.target_generation,
                    "target_generation",
                )?,
                expected_root_version: require_positive(
                    command.expected_root_version,
                    "expected_root_version",
                )?,
                expected_generation: require_positive(
                    command.expected_generation,
                    "expected_generation",
                )?,
                operator_id: require_text(command.operator_id, "operator_id", 128)?,
            })
            .await
    }

    async fn persist_validation_failure(
        &self,
        validation: &ValidateDriveWebsiteSync,
        sync_version: i64,
        lease_token: Option<&str>,
        error: &DriveServiceError,
    ) {
        let Some(lease_token) = lease_token else {
            return;
        };
        let code = match error {
            DriveServiceError::Validation(code) => code.as_str(),
            _ => "WEBSITE_SYNC_VALIDATION_FAILED",
        };
        let _ = self
            .store
            .mark_failed(&MarkDriveWebsiteSyncFailed {
                tenant_id: validation.tenant_id.clone(),
                website_root_uuid: validation.website_root_uuid.clone(),
                sync_id: validation.sync_id.clone(),
                expected_sync_version: sync_version,
                lease_token: lease_token.to_string(),
                error_code: code.to_string(),
                error_summary: "Website sync validation failed".to_string(),
                operator_id: validation.operator_id.clone(),
            })
            .await;
    }
}

pub type SqlDriveWebsiteSyncService = DriveWebsiteSyncService<SqlWebsiteSyncStore>;

fn declared_manifest(sync: &DriveWebsiteSync) -> DriveWebsiteManifestSummary {
    DriveWebsiteManifestSummary {
        sha256: sync.manifest_sha256.clone(),
        file_count: sync.manifest_file_count,
        total_bytes: sync.manifest_total_bytes,
    }
}

fn validate_declared_manifest(
    sha256: String,
    file_count: i64,
    total_bytes: i64,
) -> Result<DriveWebsiteManifestSummary, DriveServiceError> {
    let sha256 = sha256.trim().to_string();
    if sha256.len() != 71
        || !sha256.starts_with("sha256:")
        || !sha256[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(DriveServiceError::Validation(
            "manifest_sha256 must use sha256:<64 lowercase hex>".to_string(),
        ));
    }
    if !(1..=MAX_WEBSITE_SYNC_FILES as i64).contains(&file_count) {
        return Err(DriveServiceError::Validation(format!(
            "manifest_file_count must be between 1 and {MAX_WEBSITE_SYNC_FILES}"
        )));
    }
    if !(0..=MAX_WEBSITE_SYNC_TOTAL_BYTES).contains(&total_bytes) {
        return Err(DriveServiceError::Validation(format!(
            "manifest_total_bytes must be between 0 and {MAX_WEBSITE_SYNC_TOTAL_BYTES}"
        )));
    }
    Ok(DriveWebsiteManifestSummary {
        sha256,
        file_count,
        total_bytes,
    })
}

fn validate_expiry(value: String) -> Result<String, DriveServiceError> {
    let parsed = DateTime::parse_from_rfc3339(value.trim()).map_err(|_| {
        DriveServiceError::Validation("expires_at must be an RFC 3339 timestamp".to_string())
    })?;
    if parsed.offset().local_minus_utc() != 0 {
        return Err(DriveServiceError::Validation(
            "expires_at must use UTC".to_string(),
        ));
    }
    let expires_at = parsed.with_timezone(&Utc);
    let now = Utc::now();
    if expires_at <= now || expires_at > now + Duration::hours(MAX_SYNC_LIFETIME_HOURS) {
        return Err(DriveServiceError::Validation(format!(
            "expires_at must be in the future and no more than {MAX_SYNC_LIFETIME_HOURS} hours away"
        )));
    }
    Ok(expires_at.to_rfc3339_opts(SecondsFormat::Millis, true))
}

fn require_text(
    value: String,
    field_name: &str,
    maximum: usize,
) -> Result<String, DriveServiceError> {
    let value = value.trim().to_string();
    if value.is_empty() || value.len() > maximum {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} must contain between 1 and {maximum} characters"
        )));
    }
    Ok(value)
}

fn require_idempotency_key(value: String) -> Result<String, DriveServiceError> {
    let value = require_text(value, "idempotency_key", 128)?;
    if !value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
    }) {
        return Err(DriveServiceError::Validation(
            "idempotency_key contains unsupported characters".to_string(),
        ));
    }
    Ok(value)
}

fn require_positive(value: i64, field_name: &str) -> Result<i64, DriveServiceError> {
    if value < 1 {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} must be positive"
        )));
    }
    Ok(value)
}
