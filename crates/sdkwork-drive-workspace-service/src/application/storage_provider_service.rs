use crate::domain::storage_provider::{DriveStorageProvider, DriveStorageProviderKind};
use crate::ports::storage_provider_store::{
    DriveStorageProviderStore, NewDriveStorageProvider, UpdateDriveStorageProvider,
};
use crate::DriveServiceError;
use sdkwork_drive_config::allows_plain_credential_refs;

#[derive(Debug, Clone)]
pub struct CreateStorageProviderCommand {
    pub id: String,
    pub provider_kind: DriveStorageProviderKind,
    pub name: String,
    pub endpoint_url: String,
    pub region: Option<String>,
    pub bucket: String,
    pub path_style: Option<bool>,
    pub strict_tls: Option<bool>,
    pub credential_ref: Option<String>,
    pub server_side_encryption_mode: Option<String>,
    pub default_storage_class: Option<String>,
    pub status: Option<String>,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct ListStorageProvidersCommand {
    pub status: Option<String>,
    pub offset: i64,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct GetStorageProviderCommand {
    pub provider_id: String,
}

#[derive(Debug, Clone)]
pub struct UpdateStorageProviderCommand {
    pub provider_id: String,
    pub name: Option<String>,
    pub endpoint_url: Option<String>,
    pub region: Option<String>,
    pub bucket: Option<String>,
    pub path_style: Option<bool>,
    pub strict_tls: Option<bool>,
    pub credential_ref: Option<String>,
    pub server_side_encryption_mode: Option<String>,
    pub default_storage_class: Option<String>,
    pub status: Option<String>,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteStorageProviderCommand {
    pub provider_id: String,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct TestStorageProviderCommand {
    pub provider_id: String,
}

#[derive(Debug, Clone)]
pub struct StorageProviderCapabilitiesCommand {
    pub provider_id: String,
}

#[derive(Debug, Clone)]
pub struct SetStorageProviderStatusCommand {
    pub provider_id: String,
    pub status: String,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct RotateStorageProviderCredentialCommand {
    pub provider_id: String,
    pub credential_ref: String,
    pub operator_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteStorageProviderResult {
    pub deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestStorageProviderResult {
    pub provider_id: String,
    pub reachable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageProviderCapabilities {
    pub provider_id: String,
    pub provider_kind: String,
    pub supports_multipart_upload: bool,
    pub supports_presigned_upload_part: bool,
    pub supports_presigned_download: bool,
    pub supports_server_side_encryption: bool,
    pub supports_storage_class: bool,
    pub supports_credential_rotation: bool,
    pub supported_server_side_encryption_modes: Vec<String>,
    pub supported_storage_classes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DriveStorageProviderService<S>
where
    S: DriveStorageProviderStore,
{
    store: S,
}

impl<S> DriveStorageProviderService<S>
where
    S: DriveStorageProviderStore,
{
    const ACTIVE_BINDING_DELETE_CONFLICT: &'static str =
        "storage provider has active bindings; remove or disable bindings before deletion";
    const DELETED_PROVIDER_REACTIVATION_CONFLICT: &'static str =
        "deleted storage provider cannot be reactivated; create a new provider";
    const DELETED_PROVIDER_MODIFICATION_CONFLICT: &'static str =
        "deleted storage provider cannot be modified; create a new provider";

    fn default_path_style_for_provider(kind: &DriveStorageProviderKind) -> bool {
        match kind {
            DriveStorageProviderKind::AliyunOss
            | DriveStorageProviderKind::TencentCos
            | DriveStorageProviderKind::HuaweiObs
            | DriveStorageProviderKind::VolcengineTos
            | DriveStorageProviderKind::GoogleCloudStorage => false,
            DriveStorageProviderKind::Custom(value) => {
                let suffix = value
                    .trim()
                    .to_ascii_lowercase()
                    .strip_prefix(DriveStorageProviderKind::CUSTOM_PREFIX)
                    .map(str::to_string);
                !matches!(
                    suffix.as_deref(),
                    Some("aws")
                        | Some("aws_s3")
                        | Some("amazon_s3")
                        | Some("oss")
                        | Some("aliyun")
                        | Some("aliyun_oss")
                        | Some("cos")
                        | Some("tencent")
                        | Some("tencent_cos")
                        | Some("obs")
                        | Some("huawei")
                        | Some("huawei_obs")
                        | Some("tos")
                        | Some("volc")
                        | Some("volcengine")
                        | Some("volcengine_tos")
                        | Some("volcano")
                        | Some("volcano_tos")
                        | Some("gcs")
                        | Some("google_cloud_storage")
                        | Some("google_storage")
                        | Some("b2")
                        | Some("backblaze")
                        | Some("backblaze_b2")
                )
            }
            _ => true,
        }
    }

    fn default_strict_tls_for_endpoint(endpoint_url: &str) -> bool {
        !endpoint_url
            .trim()
            .to_ascii_lowercase()
            .starts_with("http://")
    }

    pub fn new(store: S) -> Self {
        Self { store }
    }

    async fn ensure_storage_provider_can_transition_to_deleted(
        &self,
        provider_id: &str,
        current_status: &str,
        target_status: &str,
    ) -> Result<(), DriveServiceError> {
        if current_status == "deleted" && target_status != "deleted" {
            return Err(DriveServiceError::Conflict(
                Self::DELETED_PROVIDER_REACTIVATION_CONFLICT.to_string(),
            ));
        }
        if current_status == "deleted" || target_status != "deleted" {
            return Ok(());
        }
        if self
            .store
            .has_active_storage_provider_bindings(provider_id)
            .await?
        {
            return Err(DriveServiceError::Conflict(
                Self::ACTIVE_BINDING_DELETE_CONFLICT.to_string(),
            ));
        }
        Ok(())
    }

    fn ensure_deleted_provider_is_not_modified(
        current_status: &str,
    ) -> Result<(), DriveServiceError> {
        if current_status == "deleted" {
            return Err(DriveServiceError::Conflict(
                Self::DELETED_PROVIDER_MODIFICATION_CONFLICT.to_string(),
            ));
        }
        Ok(())
    }

    async fn ensure_active_bindings_allow_location_update(
        &self,
        provider_id: &str,
        current: &DriveStorageProvider,
        endpoint_url: &str,
        bucket: &str,
        path_style: bool,
        strict_tls: bool,
    ) -> Result<(), DriveServiceError> {
        let changed_field = if current.endpoint_url != endpoint_url {
            Some("endpoint_url")
        } else if current.bucket != bucket {
            Some("bucket")
        } else if current.path_style != path_style {
            Some("path_style")
        } else if current.strict_tls != strict_tls {
            Some("strict_tls")
        } else {
            None
        };
        let Some(field_name) = changed_field else {
            return Ok(());
        };
        if self
            .store
            .has_active_storage_provider_bindings(provider_id)
            .await?
        {
            return Err(DriveServiceError::Conflict(format!(
                "{field_name} cannot be changed while storage provider has active bindings"
            )));
        }
        Ok(())
    }

    pub async fn create_storage_provider(
        &self,
        command: CreateStorageProviderCommand,
    ) -> Result<DriveStorageProvider, DriveServiceError> {
        if command.id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "storage provider id is required".to_string(),
            ));
        }
        if command.name.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "name is required".to_string(),
            ));
        }
        let endpoint_url = validate_endpoint_url(&command.provider_kind, &command.endpoint_url)?;
        let bucket = validate_bucket_name(&command.provider_kind, &command.bucket)?;
        if command.operator_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "operator_id is required".to_string(),
            ));
        }

        let status = command
            .status
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("active")
            .to_string();
        let status = normalize_status(&status)?;
        let region = command
            .region
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        let server_side_encryption_mode = command
            .server_side_encryption_mode
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        let default_storage_class = command
            .default_storage_class
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        let strict_tls = command
            .strict_tls
            .unwrap_or_else(|| Self::default_strict_tls_for_endpoint(&endpoint_url));
        validate_strict_tls_endpoint(strict_tls, &endpoint_url)?;

        if let Some(ref credential_ref) = command.credential_ref {
            validate_credential_ref(credential_ref)?;
        }

        self.store
            .insert_storage_provider(&NewDriveStorageProvider {
                id: command.id,
                provider_kind: command.provider_kind.as_str().to_string(),
                name: command.name.trim().to_string(),
                endpoint_url,
                region,
                bucket,
                path_style: command.path_style.unwrap_or_else(|| {
                    Self::default_path_style_for_provider(&command.provider_kind)
                }),
                strict_tls,
                credential_ref: command.credential_ref,
                server_side_encryption_mode,
                default_storage_class,
                status,
                created_by: command.operator_id.clone(),
                updated_by: command.operator_id,
            })
            .await
    }

    pub async fn list_storage_providers(
        &self,
        command: ListStorageProvidersCommand,
    ) -> Result<Vec<DriveStorageProvider>, DriveServiceError> {
        let status = match command.status.as_deref() {
            Some(status) if !status.trim().is_empty() => Some(normalize_status(status)?),
            _ => None,
        };
        self.store
            .list_storage_providers(status.as_deref(), command.offset, command.limit)
            .await
    }

    pub async fn get_storage_provider(
        &self,
        command: GetStorageProviderCommand,
    ) -> Result<DriveStorageProvider, DriveServiceError> {
        let provider_id = command.provider_id.trim();
        if provider_id.is_empty() {
            return Err(DriveServiceError::Validation(
                "provider_id is required".to_string(),
            ));
        }
        self.store
            .find_storage_provider(provider_id)
            .await?
            .ok_or_else(|| DriveServiceError::NotFound("storage provider not found".to_string()))
    }

    pub async fn update_storage_provider(
        &self,
        command: UpdateStorageProviderCommand,
    ) -> Result<DriveStorageProvider, DriveServiceError> {
        let provider_id = command.provider_id.trim();
        if provider_id.is_empty() {
            return Err(DriveServiceError::Validation(
                "provider_id is required".to_string(),
            ));
        }
        if command.operator_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "operator_id is required".to_string(),
            ));
        }

        let current = self
            .store
            .find_storage_provider(provider_id)
            .await?
            .ok_or_else(|| DriveServiceError::NotFound("storage provider not found".to_string()))?;
        let name = command
            .name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(current.name.as_str())
            .to_string();

        let endpoint_url = match command.endpoint_url.as_deref() {
            Some(value) if !value.trim().is_empty() => {
                validate_endpoint_url(&current.provider_kind, value)?
            }
            _ => current.endpoint_url.clone(),
        };
        let bucket = match command.bucket.as_deref() {
            Some(value) if !value.trim().is_empty() => {
                validate_bucket_name(&current.provider_kind, value)?
            }
            _ => current.bucket.clone(),
        };
        let status = command
            .status
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(current.status.as_str())
            .to_string();
        let status = normalize_status(&status)?;
        self.ensure_storage_provider_can_transition_to_deleted(
            provider_id,
            &current.status,
            &status,
        )
        .await?;
        let path_style = command.path_style.unwrap_or(current.path_style);
        let strict_tls = command.strict_tls.unwrap_or(current.strict_tls);
        self.ensure_active_bindings_allow_location_update(
            provider_id,
            &current,
            &endpoint_url,
            &bucket,
            path_style,
            strict_tls,
        )
        .await?;
        validate_strict_tls_endpoint(strict_tls, &endpoint_url)?;
        let region = match command.region {
            Some(region) if region.trim().is_empty() => None,
            Some(region) => Some(region.trim().to_string()),
            None => current.region,
        };
        let server_side_encryption_mode = match command.server_side_encryption_mode {
            Some(mode) if mode.trim().is_empty() => None,
            Some(mode) => Some(mode.trim().to_string()),
            None => current.server_side_encryption_mode,
        };
        let default_storage_class = match command.default_storage_class {
            Some(class) if class.trim().is_empty() => None,
            Some(class) => Some(class.trim().to_string()),
            None => current.default_storage_class,
        };

        let credential_ref = match command.credential_ref {
            Some(ref value) if value.trim().is_empty() => None,
            Some(value) => {
                validate_credential_ref(&value)?;
                Some(value.clone())
            }
            None => current.credential_ref,
        };

        self.store
            .update_storage_provider(
                provider_id,
                &UpdateDriveStorageProvider {
                    name,
                    endpoint_url,
                    region,
                    bucket,
                    path_style,
                    strict_tls,
                    credential_ref,
                    server_side_encryption_mode,
                    default_storage_class,
                    status,
                    updated_by: command.operator_id,
                },
            )
            .await
    }

    pub async fn get_storage_provider_capabilities(
        &self,
        command: StorageProviderCapabilitiesCommand,
    ) -> Result<StorageProviderCapabilities, DriveServiceError> {
        let provider = self
            .get_storage_provider(GetStorageProviderCommand {
                provider_id: command.provider_id,
            })
            .await?;
        Ok(capabilities_for_provider(&provider))
    }

    pub async fn set_storage_provider_status(
        &self,
        command: SetStorageProviderStatusCommand,
    ) -> Result<DriveStorageProvider, DriveServiceError> {
        let provider_id = command.provider_id.trim();
        if provider_id.is_empty() {
            return Err(DriveServiceError::Validation(
                "provider_id is required".to_string(),
            ));
        }
        if command.operator_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "operator_id is required".to_string(),
            ));
        }
        let status = normalize_status(&command.status)?;
        let current = self
            .store
            .find_storage_provider(provider_id)
            .await?
            .ok_or_else(|| DriveServiceError::NotFound("storage provider not found".to_string()))?;
        self.ensure_storage_provider_can_transition_to_deleted(
            provider_id,
            &current.status,
            &status,
        )
        .await?;
        self.store
            .update_storage_provider(
                provider_id,
                &UpdateDriveStorageProvider {
                    name: current.name,
                    endpoint_url: current.endpoint_url,
                    region: current.region,
                    bucket: current.bucket,
                    path_style: current.path_style,
                    strict_tls: current.strict_tls,
                    credential_ref: current.credential_ref,
                    server_side_encryption_mode: current.server_side_encryption_mode,
                    default_storage_class: current.default_storage_class,
                    status,
                    updated_by: command.operator_id,
                },
            )
            .await
    }

    pub async fn rotate_storage_provider_credential(
        &self,
        command: RotateStorageProviderCredentialCommand,
    ) -> Result<DriveStorageProvider, DriveServiceError> {
        let provider_id = command.provider_id.trim();
        if provider_id.is_empty() {
            return Err(DriveServiceError::Validation(
                "provider_id is required".to_string(),
            ));
        }
        if command.operator_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "operator_id is required".to_string(),
            ));
        }
        let credential_ref = command.credential_ref.trim();
        if credential_ref.is_empty() {
            return Err(DriveServiceError::Validation(
                "credential_ref is required".to_string(),
            ));
        }
        if !is_supported_credential_ref(credential_ref) {
            return Err(DriveServiceError::Validation(
                "credential_ref must start with env:, secret:, kms:, vault:, or plain:".to_string(),
            ));
        }
        validate_credential_ref(credential_ref)?;
        let current = self
            .store
            .find_storage_provider(provider_id)
            .await?
            .ok_or_else(|| DriveServiceError::NotFound("storage provider not found".to_string()))?;
        Self::ensure_deleted_provider_is_not_modified(&current.status)?;
        self.store
            .update_storage_provider(
                provider_id,
                &UpdateDriveStorageProvider {
                    name: current.name,
                    endpoint_url: current.endpoint_url,
                    region: current.region,
                    bucket: current.bucket,
                    path_style: current.path_style,
                    strict_tls: current.strict_tls,
                    credential_ref: Some(credential_ref.to_string()),
                    server_side_encryption_mode: current.server_side_encryption_mode,
                    default_storage_class: current.default_storage_class,
                    status: current.status,
                    updated_by: command.operator_id,
                },
            )
            .await
    }

    pub async fn test_storage_provider(
        &self,
        command: TestStorageProviderCommand,
    ) -> Result<TestStorageProviderResult, DriveServiceError> {
        let provider_id = command.provider_id.trim();
        if provider_id.is_empty() {
            return Err(DriveServiceError::Validation(
                "provider_id is required".to_string(),
            ));
        }
        let Some(provider) = self.store.find_storage_provider(provider_id).await? else {
            return Err(DriveServiceError::NotFound(
                "storage provider not found".to_string(),
            ));
        };

        Ok(TestStorageProviderResult {
            provider_id: provider.id,
            reachable: provider.status == "active" || provider.status == "disabled",
        })
    }

    pub async fn delete_storage_provider(
        &self,
        command: DeleteStorageProviderCommand,
    ) -> Result<DeleteStorageProviderResult, DriveServiceError> {
        if command.provider_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "provider_id is required".to_string(),
            ));
        }
        if command.operator_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "operator_id is required".to_string(),
            ));
        }
        let current = self
            .store
            .find_storage_provider(command.provider_id.trim())
            .await?
            .ok_or_else(|| DriveServiceError::NotFound("storage provider not found".to_string()))?;
        self.ensure_storage_provider_can_transition_to_deleted(
            command.provider_id.trim(),
            &current.status,
            "deleted",
        )
        .await?;
        let deleted = self
            .store
            .delete_storage_provider(command.provider_id.trim())
            .await?;
        if !deleted {
            return Err(DriveServiceError::NotFound(
                "storage provider not found".to_string(),
            ));
        }
        Ok(DeleteStorageProviderResult { deleted })
    }
}

fn normalize_status(raw: &str) -> Result<String, DriveServiceError> {
    let status = raw.trim().to_ascii_lowercase();
    match status.as_str() {
        "active" | "disabled" | "deleted" => Ok(status),
        _ => Err(DriveServiceError::Validation(
            "status is invalid; allowed: active, disabled, deleted".to_string(),
        )),
    }
}

fn validate_endpoint_url(
    provider_kind: &DriveStorageProviderKind,
    raw: &str,
) -> Result<String, DriveServiceError> {
    let endpoint_url = raw.trim();
    if endpoint_url.is_empty() {
        return Err(DriveServiceError::Validation(
            "endpoint_url is required".to_string(),
        ));
    }
    if endpoint_url.chars().any(char::is_whitespace) {
        return Err(DriveServiceError::Validation(
            "endpoint_url must not contain whitespace".to_string(),
        ));
    }
    let lower = endpoint_url.to_ascii_lowercase();
    if matches!(provider_kind, DriveStorageProviderKind::LocalFilesystem) {
        let Some(path) = lower.strip_prefix("file://") else {
            return Err(DriveServiceError::Validation(
                "endpoint_url must use file scheme for local filesystem providers".to_string(),
            ));
        };
        if path.is_empty() {
            return Err(DriveServiceError::Validation(
                "endpoint_url must include a local filesystem path".to_string(),
            ));
        }
        return Ok(endpoint_url.to_string());
    }
    let Some(after_scheme) = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
    else {
        return Err(DriveServiceError::Validation(
            "endpoint_url must use http or https scheme".to_string(),
        ));
    };
    let host = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .trim_matches('[')
        .trim_matches(']');
    if host.is_empty() || host.starts_with(':') || host.contains('@') {
        return Err(DriveServiceError::Validation(
            "endpoint_url must include a valid host".to_string(),
        ));
    }
    Ok(endpoint_url.to_string())
}

fn validate_strict_tls_endpoint(
    strict_tls: bool,
    endpoint_url: &str,
) -> Result<(), DriveServiceError> {
    if strict_tls
        && endpoint_url
            .trim()
            .to_ascii_lowercase()
            .starts_with("http://")
    {
        return Err(DriveServiceError::Validation(
            "strict_tls=true requires an https endpoint".to_string(),
        ));
    }
    Ok(())
}

fn validate_bucket_name(
    provider_kind: &DriveStorageProviderKind,
    raw: &str,
) -> Result<String, DriveServiceError> {
    let bucket = raw.trim();
    if bucket.is_empty() {
        return Err(DriveServiceError::Validation(
            "bucket is required".to_string(),
        ));
    }
    if matches!(provider_kind, DriveStorageProviderKind::LocalFilesystem) {
        return validate_local_bucket_name(bucket);
    }
    validate_object_store_bucket_name(bucket)
}

fn validate_local_bucket_name(bucket: &str) -> Result<String, DriveServiceError> {
    if bucket.len() > 255 {
        return Err(DriveServiceError::Validation(
            "bucket must be at most 255 characters".to_string(),
        ));
    }
    if bucket.starts_with(['/', '\\']) || bucket.ends_with(['/', '\\']) {
        return Err(DriveServiceError::Validation(
            "bucket must not be a path".to_string(),
        ));
    }
    if !bucket
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        return Err(DriveServiceError::Validation(
            "bucket may only contain ASCII letters, digits, dot, underscore, or hyphen".to_string(),
        ));
    }
    Ok(bucket.to_string())
}

fn validate_object_store_bucket_name(bucket: &str) -> Result<String, DriveServiceError> {
    const RESERVED_PREFIXES: [&str; 2] = ["xn--", "sthree-"];
    const RESERVED_SUFFIXES: [&str; 4] = ["-s3alias", "--ol-s3", ".mrap", "--x-s3"];

    if !(3..=63).contains(&bucket.len()) {
        return Err(DriveServiceError::Validation(
            "bucket must be between 3 and 63 characters for object storage providers".to_string(),
        ));
    }
    if !bucket.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
    }) {
        return Err(DriveServiceError::Validation(
            "bucket may only contain lowercase ASCII letters, digits, dot, or hyphen for object storage providers".to_string(),
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
        return Err(DriveServiceError::Validation(
            "bucket must start and end with a letter or digit for object storage providers"
                .to_string(),
        ));
    }
    if bucket.contains("..") {
        return Err(DriveServiceError::Validation(
            "bucket must not contain adjacent dots for object storage providers".to_string(),
        ));
    }
    if bucket.contains(".-") || bucket.contains("-.") {
        return Err(DriveServiceError::Validation(
            "bucket must not contain dot-hyphen adjacency for object storage providers".to_string(),
        ));
    }
    if is_ipv4_address_like(bucket) {
        return Err(DriveServiceError::Validation(
            "bucket must not be formatted as an IPv4 address for object storage providers"
                .to_string(),
        ));
    }
    if RESERVED_PREFIXES
        .iter()
        .any(|prefix| bucket.starts_with(prefix))
        || RESERVED_SUFFIXES
            .iter()
            .any(|suffix| bucket.ends_with(suffix))
    {
        return Err(DriveServiceError::Validation(
            "bucket uses a reserved S3 bucket name affix".to_string(),
        ));
    }
    Ok(bucket.to_string())
}

fn is_ipv4_address_like(bucket: &str) -> bool {
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

fn is_supported_credential_ref(raw: &str) -> bool {
    ["env:", "secret:", "kms:", "vault:", "plain:"]
        .iter()
        .any(|prefix| raw.starts_with(prefix))
}

fn validate_credential_ref(raw: &str) -> Result<(), DriveServiceError> {
    let trimmed = raw.trim();
    if !is_supported_credential_ref(trimmed) {
        return Err(DriveServiceError::Validation(
            "credential_ref must start with env:, secret:, kms:, vault:, or plain:".to_string(),
        ));
    }
    if trimmed.starts_with("plain:") && !allows_plain_credential_refs() {
        return Err(DriveServiceError::Validation(
            "plain credential_ref is not allowed in production runtime; use env:, secret:, kms:, or vault:"
                .to_string(),
        ));
    }
    Ok(())
}

fn capabilities_for_provider(provider: &DriveStorageProvider) -> StorageProviderCapabilities {
    match &provider.provider_kind {
        DriveStorageProviderKind::S3Compatible
        | DriveStorageProviderKind::AliyunOss
        | DriveStorageProviderKind::TencentCos
        | DriveStorageProviderKind::HuaweiObs
        | DriveStorageProviderKind::VolcengineTos
        | DriveStorageProviderKind::GoogleCloudStorage
        | DriveStorageProviderKind::Custom(_) => StorageProviderCapabilities {
            provider_id: provider.id.clone(),
            provider_kind: provider.provider_kind.as_str().to_string(),
            supports_multipart_upload: true,
            supports_presigned_upload_part: true,
            supports_presigned_download: true,
            supports_server_side_encryption: true,
            supports_storage_class: true,
            supports_credential_rotation: true,
            supported_server_side_encryption_modes: vec![
                "AES256".to_string(),
                "aws:kms".to_string(),
                "none".to_string(),
            ],
            supported_storage_classes: vec![
                "STANDARD".to_string(),
                "STANDARD_IA".to_string(),
                "INTELLIGENT_TIERING".to_string(),
                "GLACIER_IR".to_string(),
            ],
        },
        DriveStorageProviderKind::LocalFilesystem => StorageProviderCapabilities {
            provider_id: provider.id.clone(),
            provider_kind: provider.provider_kind.as_str().to_string(),
            supports_multipart_upload: false,
            supports_presigned_upload_part: false,
            supports_presigned_download: false,
            supports_server_side_encryption: false,
            supports_storage_class: false,
            supports_credential_rotation: false,
            supported_server_side_encryption_modes: Vec::new(),
            supported_storage_classes: Vec::new(),
        },
    }
}
