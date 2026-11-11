use crate::domain::node_version::DriveNodeVersion;
use crate::domain::upload::{DriveUploadSession, DriveUploadSessionState};
use crate::ports::stored_version_store::DriveStoredVersionStore;
use crate::ports::upload_session_store::{DriveUploadSessionStore, NewDriveUploadSession};
use crate::DriveServiceError;
use std::time::{SystemTime, UNIX_EPOCH};

pub use crate::ports::stored_version_store::CompleteStoredVersionUploadCommand;

#[derive(Debug, Clone)]
pub struct CreateUploadSessionCommand {
    pub session_id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: String,
    pub bucket: String,
    pub object_key: String,
    pub storage_provider_id: String,
    pub storage_upload_id: Option<String>,
    pub idempotency_key: String,
    pub operator_id: String,
    pub expires_at_epoch_ms: i64,
}

#[derive(Debug, Clone)]
pub struct DriveUploadService<S>
where
    S: DriveUploadSessionStore,
{
    store: S,
}

impl<S> DriveUploadService<S>
where
    S: DriveUploadSessionStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn create_upload_session(
        &self,
        command: CreateUploadSessionCommand,
    ) -> Result<DriveUploadSession, DriveServiceError> {
        if command.session_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "session_id is required".to_string(),
            ));
        }
        if command.idempotency_key.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "idempotency_key is required".to_string(),
            ));
        }
        validate_runtime_bucket(&command.bucket)?;
        validate_object_key(&command.object_key)?;
        if command.storage_provider_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "storage_provider_id is required".to_string(),
            ));
        }
        if command.expires_at_epoch_ms <= current_epoch_ms() {
            return Err(DriveServiceError::Validation(
                "expires_at_epoch_ms must be in the future".to_string(),
            ));
        }
        let storage_upload_id = command
            .storage_upload_id
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| command.session_id.trim().to_string());

        if let Some(existing) = self
            .store
            .find_by_idempotency(
                &command.tenant_id,
                &command.space_id,
                &command.node_id,
                &command.idempotency_key,
            )
            .await?
        {
            return Ok(existing);
        }

        let new_session = NewDriveUploadSession {
            id: command.session_id,
            tenant_id: command.tenant_id,
            space_id: command.space_id,
            node_id: command.node_id,
            bucket: command.bucket,
            object_key: command.object_key,
            idempotency_key: command.idempotency_key,
            storage_provider_id: command.storage_provider_id,
            storage_upload_id,
            state: DriveUploadSessionState::Created.as_str().to_string(),
            expires_at_epoch_ms: command.expires_at_epoch_ms,
            created_by: command.operator_id.clone(),
            updated_by: command.operator_id,
        };
        self.store.insert_upload_session(&new_session).await
    }

    /// Publish an already-stored object as the next version of an existing node.
    ///
    /// This is the in-process replacement for the app API's upload-session
    /// finalisation when the caller wrote the bytes itself (a document renderer
    /// publishing a page body, for example): it records the
    /// `dr_drive_storage_object` row, the `dr_drive_node_version` row and the
    /// node head snapshot in one transaction and emits the
    /// `node.version_committed` change/outbox event.
    ///
    /// # Caller contract
    ///
    /// * `tenant_id` scopes the node/space lookup; a node of another tenant is
    ///   `NotFound` and is never written.
    /// * The caller MUST have enforced at least `writer` on the node through
    ///   Drive's permission service before calling. This method performs no
    ///   authorization and no session bookkeeping.
    /// * `storage_provider_id`/`bucket`/`object_key` must be the locator the
    ///   bytes already occupy; the object is not read or verified against
    ///   `checksum_sha256_hex`.
    /// * `idempotency_key` makes the call replay-safe: replaying the same locator
    ///   returns the version registered the first time (or `Conflict` when the
    ///   same locator was completed with different content metadata).
    ///
    /// # Errors
    ///
    /// - `Validation` for a blank identifier, invalid content type, negative
    ///   length, malformed checksum, or an invalid object key.
    /// - `NotFound` when the node or its space is absent or inactive in this
    ///   tenant.
    /// - `Conflict` when the locator was already completed with different
    ///   metadata, when the version number raced another writer, or when the
    ///   managed-website guard rejects the mutation.
    /// - `Internal` when a database operation fails.
    pub async fn complete_stored_version(
        &self,
        command: CompleteStoredVersionUploadCommand,
    ) -> Result<DriveNodeVersion, DriveServiceError>
    where
        S: DriveStoredVersionStore,
    {
        self.store.complete_stored_version(&command).await
    }
}

fn validate_runtime_bucket(bucket: &str) -> Result<(), DriveServiceError> {
    let trimmed = bucket.trim();
    if bucket != trimmed {
        return Err(DriveServiceError::Validation(
            "bucket must be trimmed".to_string(),
        ));
    }
    if trimmed.is_empty() {
        return Err(DriveServiceError::Validation(
            "bucket is required".to_string(),
        ));
    }
    if trimmed.len() > 255 {
        return Err(DriveServiceError::Validation(
            "bucket must be at most 255 characters".to_string(),
        ));
    }
    if !trimmed
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(DriveServiceError::Validation(
            "bucket may only contain ASCII letters, digits, dot, underscore, or hyphen".to_string(),
        ));
    }
    Ok(())
}

fn validate_object_key(object_key: &str) -> Result<(), DriveServiceError> {
    let trimmed = object_key.trim();
    if object_key != trimmed {
        return Err(DriveServiceError::Validation(
            "object_key must be trimmed".to_string(),
        ));
    }
    if trimmed.is_empty() {
        return Err(DriveServiceError::Validation(
            "object_key is required".to_string(),
        ));
    }
    if trimmed.len() > 1024 {
        return Err(DriveServiceError::Validation(
            "object_key must be at most 1024 UTF-8 bytes".to_string(),
        ));
    }
    if trimmed.as_bytes().contains(&0) {
        return Err(DriveServiceError::Validation(
            "object_key must not contain NUL bytes".to_string(),
        ));
    }
    if trimmed.starts_with('/') || trimmed.ends_with('/') {
        return Err(DriveServiceError::Validation(
            "object_key must not start or end with slash".to_string(),
        ));
    }
    for segment in trimmed.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(DriveServiceError::Validation(
                "object_key must not contain empty or period-only path segments".to_string(),
            ));
        }
    }
    Ok(())
}

fn current_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
