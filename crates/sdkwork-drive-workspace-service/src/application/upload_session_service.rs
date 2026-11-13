//! Upload-session lifecycle as an in-process service.
//!
//! Before this module the open/complete/cancel state machine for
//! `dr_drive_upload_session` existed only inside
//! `sdkwork-routes-drive-app-api`. That is exactly the shape
//! `MODULE_SPEC.md` §4.2 forbids for a sibling module: a backend module that
//! needs Drive to finalise an upload cannot reach a route handler. The logic
//! therefore lives here, on the host `PgPool`, and the route delegates to it.
//!
//! The storage protocol calls (create/complete/abort multipart, delete object)
//! are the only provider-dependent steps; they arrive through
//! [`DriveUploadMultipartStorage`]. Everything else - tenant scoping, node and
//! space validation, ACL, part accounting, the `completing` claim, version
//! allocation, node-version activation, the node head snapshot, change-log and
//! outbox recording - is owned here and shared by every caller.

use sqlx::{PgConnection, PgPool};

use crate::application::permission_service::SqlDrivePermissionService;
use crate::application::storage_target_service::{
    resolve_storage_target, ResolveDriveStorageTargetCommand,
};
use crate::application::upload_service::{CreateUploadSessionCommand, DriveUploadService};
use crate::application::upload_session_rules::{
    ensure_upload_session_not_expired, parse_storage_object_version_no_from_key,
    validate_completed_multipart_parts, validate_content_type, validate_mutable_upload_session,
    validate_sha256_checksum,
};
use crate::domain::node_version::{
    CreateDriveNodeVersionCommand, DriveNodeVersion, DriveNodeVersionChangeSource,
    DriveNodeVersionKind,
};
use crate::domain::upload::{DriveUploadSession, DriveUploadSessionState};
use crate::domain::uploader::{content_type_group_for, DriveUploadItem};
use crate::infrastructure::change_recorder::{
    record_drive_change, record_drive_node_version_committed_on_connection,
    RecordDriveChangeCommand, RecordDriveNodeVersionCommittedCommand,
};
use crate::infrastructure::outbox_dispatch::trigger_immediate_outbox_dispatch;
use crate::infrastructure::sql::begin_transaction_sql;
use crate::infrastructure::sql::managed_website_tree_guard::ensure_managed_website_node_mutation_allowed;
use crate::infrastructure::sql::node_head_metadata::{
    apply_file_node_head_snapshot_in_transaction, FileNodeHeadSnapshot,
};
use crate::infrastructure::sql::node_version_store::create_node_version_in_transaction;
use crate::infrastructure::sql::storage_object_store::{
    next_storage_object_version_no, read_node_content_state,
};
use crate::infrastructure::sql::upload_session_store::{
    update_session_state_on_connection, SqlUploadSessionStore,
};
use crate::infrastructure::sql::uploader_store::{sensitive_operation_id, SqlUploaderStore};
use crate::ports::permission_store::ResolveEffectiveNodeAccessCommand;
use crate::ports::upload_multipart_storage::{
    AbortDriveMultipartUploadCommand, CompleteDriveMultipartUploadCommand,
    CreateDriveMultipartUploadCommand, DeleteDriveStoredObjectCommand, DriveMultipartPart,
    DriveUploadMultipartStorage,
};
use crate::ports::upload_session_store::DriveUploadSessionStore;
use crate::ports::uploader_store::{DriveUploaderNodeRecord, DriveUploaderStore};
use crate::DriveServiceError;

/// Open an upload session.
///
/// The caller contract mirrors the app API endpoint it replaces:
///
/// * `tenant_id`, `space_id` and `node_id` must describe one active file node in
///   one active space of the caller's tenant; anything else is `NotFound`.
/// * `subject_type`/`subject_id` must resolve to at least `writer` on that node
///   through Drive's permission service. The service does not accept a
///   pre-computed role: the caller cannot assert access it does not have.
/// * `idempotency_key` is unique per `(tenant, space, node)`; replaying the same
///   key returns the already-open session instead of creating a second one.
/// * `storage_provider_id`/`bucket` are optional routing hints; when omitted the
///   tenant-scoped provider bindings of the space decide.
///
/// # Errors
///
/// - `Validation`: missing idempotency key, past expiry, file node required.
/// - `NotFound`: node/space absent or inactive in this tenant; explicit
///   provider not found in this tenant.
/// - `PermissionDenied`: subject lacks `writer`.
/// - `Conflict`: session id already used, provider inactive/ambiguous, or the
///   idempotency key was taken by a different locator.
/// - `Internal`: database or storage failure.
#[derive(Debug, Clone)]
pub struct PrepareDriveUploadSessionCommand {
    pub session_id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: String,
    pub requested_storage_provider_id: Option<String>,
    pub requested_bucket: Option<String>,
    pub idempotency_key: String,
    pub operator_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub expires_at_epoch_ms: i64,
}

/// Finalise an upload session whose parts are already in storage.
///
/// * `parts` must be the exact part list the client uploaded, sorted ascending.
/// * The session must belong to `tenant_id` and still be `created`/`uploading`
///   and unexpired; completion claims it (`completing`) so two concurrent
///   completions cannot both publish a version.
/// * `subject_type`/`subject_id` must resolve to `writer` on the session's node.
/// * Completion is idempotent at the database level: the storage object is
///   inserted under one version number, and a retried request that lost the
///   claim fails with `Conflict` instead of writing a second version.
#[derive(Debug, Clone)]
pub struct CompleteDriveUploadSessionCommand {
    pub tenant_id: String,
    pub upload_session_id: String,
    pub upload_id: Option<String>,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
    pub parts: Vec<DriveMultipartPart>,
    pub operator_id: String,
    pub subject_type: String,
    pub subject_id: String,
}

/// Abort an upload session and discard its multipart upload.
///
/// Aborting stays allowed after the session deadline: releasing a session that
/// can no longer be completed is cleanup, not a content mutation.
#[derive(Debug, Clone)]
pub struct CancelDriveUploadSessionCommand {
    pub tenant_id: String,
    pub upload_session_id: String,
    pub operator_id: String,
    pub subject_type: String,
    pub subject_id: String,
}

/// Drive's upload-session lifecycle over a `PgPool`.
///
/// All methods take the tenant explicitly and never read one from configuration
/// or a connection default: the pool is shared by every tenant of the process.
#[derive(Debug, Clone)]
pub struct DriveUploadSessionService<S>
where
    S: DriveUploadMultipartStorage,
{
    pool: PgPool,
    storage: S,
}

impl<S> DriveUploadSessionService<S>
where
    S: DriveUploadMultipartStorage,
{
    /// Build the service over the host pool and a storage adapter.
    pub fn new(pool: PgPool, storage: S) -> Self {
        Self { pool, storage }
    }

    fn sessions(&self) -> SqlUploadSessionStore {
        SqlUploadSessionStore::new(self.pool.clone())
    }

    fn uploader_store(&self) -> SqlUploaderStore {
        SqlUploaderStore::new(self.pool.clone())
    }

    /// Open a session, allocating its storage target and multipart upload.
    ///
    /// # Errors
    ///
    /// See [`PrepareDriveUploadSessionCommand`]; every variant is documented
    /// there. Authorization is delegated to Drive's own permission service.
    pub async fn prepare(
        &self,
        command: PrepareDriveUploadSessionCommand,
    ) -> Result<DriveUploadSession, DriveServiceError> {
        let tenant_id = require_text(&command.tenant_id, "tenant_id")?;
        let space_id = require_text(&command.space_id, "space_id")?;
        let node_id = require_text(&command.node_id, "node_id")?;
        let session_id = require_text(&command.session_id, "session_id")?;
        let idempotency_key = require_text(&command.idempotency_key, "idempotency_key")?;
        let operator_id = require_text(&command.operator_id, "operator_id")?;
        if command.expires_at_epoch_ms <= current_epoch_ms() {
            return Err(DriveServiceError::Validation(
                "expires_at_epoch_ms must be in the future".to_string(),
            ));
        }

        let node = self.load_upload_node(&tenant_id, &node_id).await?;
        if node.space_id != space_id {
            return Err(DriveServiceError::NotFound("node not found".to_string()));
        }
        if node.node_type != "file" {
            return Err(DriveServiceError::Validation(
                "nodeId must reference an active file".to_string(),
            ));
        }
        self.ensure_node_role(
            &tenant_id,
            &node.space_id,
            &node_id,
            &command.subject_type,
            &command.subject_id,
            "writer",
        )
        .await?;

        let sessions = self.sessions();
        if let Some(existing) = sessions
            .find_by_idempotency(&tenant_id, &space_id, &node_id, &idempotency_key)
            .await?
        {
            return Ok(existing);
        }
        sessions.ensure_id_available(&session_id).await?;

        let target_version_no =
            next_storage_object_version_no(&self.pool, &tenant_id, &node_id).await?;
        let storage_target = resolve_storage_target(
            &self.pool,
            ResolveDriveStorageTargetCommand {
                tenant_id: &tenant_id,
                space_id: &space_id,
                requested_provider_id: command.requested_storage_provider_id.as_deref(),
                requested_bucket: command.requested_bucket.as_deref(),
                node_id: &node_id,
                object_id: &session_id,
                version_no: target_version_no,
            },
        )
        .await?;
        let storage_upload_id = self
            .storage
            .create_multipart_upload(CreateDriveMultipartUploadCommand {
                storage_provider_id: storage_target.provider_id.clone(),
                bucket: storage_target.bucket.clone(),
                object_key: storage_target.object_key.clone(),
            })
            .await?;

        let created = DriveUploadService::new(self.sessions())
            .create_upload_session(CreateUploadSessionCommand {
                session_id,
                tenant_id: tenant_id.clone(),
                space_id: space_id.clone(),
                node_id: node_id.clone(),
                bucket: storage_target.bucket,
                object_key: storage_target.object_key,
                storage_provider_id: storage_target.provider_id,
                storage_upload_id: Some(storage_upload_id),
                idempotency_key,
                operator_id: operator_id.clone(),
                expires_at_epoch_ms: command.expires_at_epoch_ms,
            })
            .await?;

        record_drive_change(
            &self.pool,
            RecordDriveChangeCommand {
                tenant_id: &created.tenant_id,
                space_id: &created.space_id,
                node_id: Some(&created.node_id),
                event_type: sdkwork_drive_contract::drive::domain_events::upload_session::CREATED,
                actor_id: &operator_id,
            },
        )
        .await?;

        Ok(created)
    }

    /// Read one session for a caller that may read its node.
    ///
    /// # Errors
    ///
    /// `NotFound` when the session or its node/space is absent or inactive in
    /// this tenant; `PermissionDenied` when the subject lacks `reader`; and the
    /// repository's error kinds when a read fails.
    pub async fn retrieve(
        &self,
        tenant_id: &str,
        upload_session_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<DriveUploadSession, DriveServiceError> {
        let tenant_id = require_text(tenant_id, "tenant_id")?;
        let session = self.load_session(&tenant_id, upload_session_id).await?;
        let node = self.load_upload_node(&tenant_id, &session.node_id).await?;
        self.ensure_node_role(
            &tenant_id,
            &node.space_id,
            &session.node_id,
            subject_type,
            subject_id,
            "reader",
        )
        .await?;
        Ok(session)
    }

    /// Finalise the session's multipart upload into a new node version.
    ///
    /// # Errors
    ///
    /// - `Validation`: part list, content type, checksum or length invalid, or a
    ///   supplied `upload_id` is blank.
    /// - `NotFound`: session/node absent or inactive in this tenant.
    /// - `PermissionDenied`: subject lacks `writer`.
    /// - `Conflict`: session not mutable, expired, `upload_id` mismatch, version
    ///   race, storage-object id/locator collision, or storage-concurrency loss.
    /// - `Internal`: database or provider transport failure. A provider failure
    ///   releases the completion claim before returning, so the client can retry.
    pub async fn complete(
        &self,
        command: CompleteDriveUploadSessionCommand,
    ) -> Result<DriveUploadSession, DriveServiceError> {
        let tenant_id = require_text(&command.tenant_id, "tenant_id")?;
        let operator_id = require_text(&command.operator_id, "operator_id")?;
        validate_completed_multipart_parts(&command.parts)?;
        let content_type = validate_content_type(&command.content_type, "content_type")?;
        let checksum_sha256_hex =
            validate_sha256_checksum(&command.checksum_sha256_hex, "checksum_sha256_hex")?;
        let content_length = command.content_length;
        if content_length < 0 {
            return Err(DriveServiceError::Validation(
                "content_length must be greater than or equal to 0".to_string(),
            ));
        }
        let uploaded_parts_count = i64::try_from(command.parts.len()).map_err(|_| {
            DriveServiceError::Validation("parts must contain at most 10000 items".to_string())
        })?;

        let session = self
            .load_session(&tenant_id, &command.upload_session_id)
            .await?;
        validate_mutable_upload_session(session.state.as_str())?;
        ensure_upload_session_not_expired(session.expires_at_epoch_ms, current_epoch_ms())?;
        if let Some(upload_id) = command.upload_id.as_deref() {
            let upload_id = upload_id.trim();
            if upload_id.is_empty() {
                return Err(DriveServiceError::Validation(
                    "uploadId must not be empty".to_string(),
                ));
            }
            if upload_id != session.storage_upload_id {
                return Err(DriveServiceError::Conflict(
                    "uploadId does not match upload session storage upload id".to_string(),
                ));
            }
        }
        let node = self.load_upload_node(&tenant_id, &session.node_id).await?;
        self.ensure_node_role(
            &tenant_id,
            &node.space_id,
            &session.node_id,
            &command.subject_type,
            &command.subject_id,
            "writer",
        )
        .await?;

        let node_content_state_before_completion =
            read_node_content_state(&self.pool, &tenant_id, &session.node_id).await?;
        let uploader_upload_item = self
            .find_uploader_item_by_session(&tenant_id, &command.upload_session_id)
            .await?;
        let completed_object_plan = self.plan_completed_object(&tenant_id, &session).await?;
        self.sessions()
            .claim_completion(&session, &operator_id)
            .await?;

        if let Err(error) = self.complete_storage_upload(&session, &command.parts).await {
            if let Err(reset_error) = self
                .sessions()
                .release_completion_claim(&tenant_id, &command.upload_session_id, &operator_id)
                .await
            {
                tracing::error!(
                    event = "drive.upload.multipart_completion_session_reset_failed",
                    tenant_id = %tenant_id,
                    upload_session_id = %command.upload_session_id,
                    error = ?reset_error,
                    "failed to release upload session completion claim after storage failure"
                );
            }
            return Err(error);
        }

        let head_snapshot = match uploader_upload_item.as_ref() {
            Some(upload_item) => FileNodeHeadSnapshot {
                file_extension: upload_item.file_extension.clone(),
                content_type: content_type.to_string(),
                content_type_group: upload_item.content_type_group.clone(),
                content_length,
                version_no: completed_object_plan.version_no,
                checksum_sha256_hex: checksum_sha256_hex.to_string(),
            },
            None => FileNodeHeadSnapshot {
                file_extension: None,
                content_type: content_type.to_string(),
                content_type_group: content_type_group_for(content_type).to_string(),
                content_length,
                version_no: completed_object_plan.version_no,
                checksum_sha256_hex: checksum_sha256_hex.to_string(),
            },
        };

        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire upload completion transaction connection failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "begin upload completion transaction failed: {error}"
                ))
            })?;

        let tx_result: Result<(), DriveServiceError> = async {
            ensure_managed_website_node_mutation_allowed(
                &mut connection,
                &tenant_id,
                &session.node_id,
            )
            .await?;
            self.insert_storage_object(
                &mut connection,
                &tenant_id,
                &session,
                uploader_upload_item.as_ref(),
                &completed_object_plan,
                content_type,
                content_length,
                checksum_sha256_hex,
                &operator_id,
            )
            .await?;

            let node_version = self
                .insert_node_version(
                    &mut connection,
                    &tenant_id,
                    &node.space_id,
                    &session.node_id,
                    &completed_object_plan,
                    uploader_upload_item.as_ref(),
                    content_type,
                    content_length,
                    checksum_sha256_hex,
                    &operator_id,
                )
                .await?;

            if let Some(upload_item) = uploader_upload_item.as_ref() {
                self.complete_uploader_item(
                    &mut connection,
                    &tenant_id,
                    upload_item,
                    &completed_object_plan,
                    &session,
                    content_type,
                    content_length,
                    checksum_sha256_hex,
                    uploaded_parts_count,
                    &operator_id,
                    &node_content_state_before_completion,
                )
                .await?;
            }

            update_session_state_on_connection(
                &mut connection,
                &tenant_id,
                &command.upload_session_id,
                DriveUploadSessionState::Completed.as_str(),
                &operator_id,
            )
            .await?;

            apply_file_node_head_snapshot_in_transaction(
                &mut connection,
                &tenant_id,
                &session.node_id,
                &operator_id,
                &head_snapshot,
            )
            .await?;

            record_drive_node_version_committed_on_connection(
                &mut connection,
                RecordDriveNodeVersionCommittedCommand {
                    tenant_id: &tenant_id,
                    organization_id: uploader_upload_item
                        .as_ref()
                        .and_then(|item| item.organization_id.as_deref()),
                    space_id: &node.space_id,
                    node_id: &session.node_id,
                    node_version_id: &node_version.id,
                    version_no: completed_object_plan.version_no,
                    operation_id: uploader_upload_item
                        .as_ref()
                        .map(|item| item.id.as_str())
                        .unwrap_or(command.upload_session_id.as_str()),
                    content_type,
                    content_length,
                    checksum_sha256_hex,
                    actor_id: &operator_id,
                },
            )
            .await?;

            Ok(())
        }
        .await;

        match tx_result {
            Ok(()) => {
                sqlx::query("COMMIT")
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "commit upload completion transaction failed: {error}"
                        ))
                    })?;
            }
            Err(error) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                drop(connection);
                self.recover_completion_after_db_failure(&tenant_id, &session, &operator_id)
                    .await;
                return Err(error);
            }
        }
        drop(connection);

        sdkwork_drive_observability::metrics::record_outbox_pending();
        trigger_immediate_outbox_dispatch(self.pool.clone());

        self.load_session(&tenant_id, &command.upload_session_id)
            .await
    }

    /// Abort a session and its storage multipart upload.
    ///
    /// # Errors
    ///
    /// `NotFound` when the session/node is absent or inactive in this tenant;
    /// `PermissionDenied` when the subject lacks `writer`; `Conflict` when the
    /// session is `completing`/`completed`/`aborted`/`expired`; the provider's
    /// mapped error when the abort call fails; `Internal` for database failures.
    pub async fn cancel(
        &self,
        command: CancelDriveUploadSessionCommand,
    ) -> Result<DriveUploadSession, DriveServiceError> {
        let tenant_id = require_text(&command.tenant_id, "tenant_id")?;
        let operator_id = require_text(&command.operator_id, "operator_id")?;
        let session = self
            .load_session(&tenant_id, &command.upload_session_id)
            .await?;
        validate_mutable_upload_session(session.state.as_str())?;
        let node = self.load_upload_node(&tenant_id, &session.node_id).await?;
        self.ensure_node_role(
            &tenant_id,
            &node.space_id,
            &session.node_id,
            &command.subject_type,
            &command.subject_id,
            "writer",
        )
        .await?;

        self.storage
            .abort_multipart_upload(AbortDriveMultipartUploadCommand {
                storage_provider_id: session.storage_provider_id.clone(),
                bucket: session.bucket.clone(),
                object_key: session.object_key.clone(),
                storage_upload_id: session.storage_upload_id.clone(),
            })
            .await?;
        self.sessions()
            .update_state(
                &tenant_id,
                &command.upload_session_id,
                DriveUploadSessionState::Aborted.as_str(),
                &operator_id,
            )
            .await?;

        record_drive_change(
            &self.pool,
            RecordDriveChangeCommand {
                tenant_id: &tenant_id,
                space_id: &node.space_id,
                node_id: Some(&session.node_id),
                event_type: sdkwork_drive_contract::drive::domain_events::upload_session::ABORTED,
                actor_id: &operator_id,
            },
        )
        .await?;

        self.load_session(&tenant_id, &command.upload_session_id)
            .await
    }

    async fn load_session(
        &self,
        tenant_id: &str,
        upload_session_id: &str,
    ) -> Result<DriveUploadSession, DriveServiceError> {
        self.sessions()
            .find_by_id(tenant_id, upload_session_id)
            .await?
            .ok_or_else(|| DriveServiceError::NotFound("upload session not found".to_string()))
    }

    async fn load_upload_node(
        &self,
        tenant_id: &str,
        node_id: &str,
    ) -> Result<DriveUploaderNodeRecord, DriveServiceError> {
        let node = self
            .uploader_store()
            .find_active_node(tenant_id, node_id)
            .await?
            .ok_or_else(|| DriveServiceError::NotFound("node not found".to_string()))?;
        let space = self
            .uploader_store()
            .find_active_space(tenant_id, &node.space_id)
            .await?;
        if space.is_none() {
            return Err(DriveServiceError::NotFound("node not found".to_string()));
        }
        Ok(node)
    }

    async fn ensure_node_role(
        &self,
        tenant_id: &str,
        space_id: &str,
        node_id: &str,
        subject_type: &str,
        subject_id: &str,
        required_role: &str,
    ) -> Result<(), DriveServiceError> {
        let subject_type = require_text(subject_type, "subject_type")?;
        let subject_id = require_text(subject_id, "subject_id")?;
        let access = SqlDrivePermissionService::new(self.pool.clone())
            .resolve_effective_node_access(ResolveEffectiveNodeAccessCommand {
                tenant_id: tenant_id.to_string(),
                space_id: space_id.to_string(),
                node_id: node_id.to_string(),
                subject_type,
                subject_id,
            })
            .await?;
        if !access.allows_role(required_role) {
            return Err(drive_role_required_error());
        }
        Ok(())
    }

    async fn find_uploader_item_by_session(
        &self,
        tenant_id: &str,
        upload_session_id: &str,
    ) -> Result<Option<DriveUploadItem>, DriveServiceError> {
        self.uploader_store()
            .find_upload_item_by_session(tenant_id, upload_session_id)
            .await
    }

    async fn plan_completed_object(
        &self,
        tenant_id: &str,
        session: &DriveUploadSession,
    ) -> Result<CompletedStorageObjectInsertPlan, DriveServiceError> {
        let version_no = parse_storage_object_version_no_from_key(
            &session.object_key,
            tenant_id,
            &session.space_id,
            &session.node_id,
        )?;
        let next_version_no =
            next_storage_object_version_no(&self.pool, tenant_id, &session.node_id).await?;
        if version_no != next_version_no {
            return Err(DriveServiceError::Conflict(
                "upload session storage object version does not match next file version"
                    .to_string(),
            ));
        }
        let id = format!("{}-v{}", session.id, version_no);
        let id_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(1)
             FROM dr_drive_storage_object
             WHERE id=$1",
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "check dr_drive_storage_object id conflict failed: {error}"
            ))
        })?;
        if id_count > 0 {
            return Err(DriveServiceError::Conflict(
                "storage object id already exists".to_string(),
            ));
        }
        let active_locator_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(1)
             FROM dr_drive_storage_object
             WHERE tenant_id=$1
               AND node_id=$2
               AND bucket=$3
               AND object_key=$4
               AND lifecycle_status='active'",
        )
        .bind(tenant_id)
        .bind(&session.node_id)
        .bind(&session.bucket)
        .bind(&session.object_key)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "check dr_drive_storage_object active locator conflict failed: {error}"
            ))
        })?;
        if active_locator_count > 0 {
            return Err(DriveServiceError::Conflict(
                "active storage object locator already exists".to_string(),
            ));
        }
        Ok(CompletedStorageObjectInsertPlan { id, version_no })
    }

    async fn complete_storage_upload(
        &self,
        session: &DriveUploadSession,
        parts: &[DriveMultipartPart],
    ) -> Result<(), DriveServiceError> {
        self.storage
            .complete_multipart_upload(CompleteDriveMultipartUploadCommand {
                storage_provider_id: session.storage_provider_id.clone(),
                bucket: session.bucket.clone(),
                object_key: session.object_key.clone(),
                storage_upload_id: session.storage_upload_id.clone(),
                parts: parts.to_vec(),
            })
            .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_storage_object(
        &self,
        connection: &mut PgConnection,
        tenant_id: &str,
        session: &DriveUploadSession,
        uploader_upload_item: Option<&DriveUploadItem>,
        plan: &CompletedStorageObjectInsertPlan,
        content_type: &str,
        content_length: i64,
        checksum_sha256_hex: &str,
        operator_id: &str,
    ) -> Result<(), DriveServiceError> {
        sqlx::query(
            "INSERT INTO dr_drive_storage_object (
                id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
                scene, source, content_type, content_length, checksum_sha256_hex, lifecycle_status,
                created_by, updated_by
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'active', $13, $13)",
        )
        .bind(&plan.id)
        .bind(tenant_id)
        .bind(&session.node_id)
        .bind(plan.version_no)
        .bind(&session.storage_provider_id)
        .bind(&session.bucket)
        .bind(&session.object_key)
        .bind(uploader_upload_item.and_then(|item| item.scene.as_deref()))
        .bind(uploader_upload_item.and_then(|item| item.source.as_deref()))
        .bind(content_type)
        .bind(content_length)
        .bind(checksum_sha256_hex)
        .bind(operator_id)
        .execute(&mut *connection)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("insert dr_drive_storage_object failed: {error}"))
        })?;
        Ok(())
    }

    /// Insert the logical node version for a completed object.
    ///
    /// Reuses the node-version repository insert so the upload path, the
    /// uploader path and any stored-version completion share one column map.
    #[allow(clippy::too_many_arguments)]
    async fn insert_node_version(
        &self,
        connection: &mut PgConnection,
        tenant_id: &str,
        space_id: &str,
        node_id: &str,
        plan: &CompletedStorageObjectInsertPlan,
        uploader_upload_item: Option<&DriveUploadItem>,
        content_type: &str,
        content_length: i64,
        checksum_sha256_hex: &str,
        operator_id: &str,
    ) -> Result<DriveNodeVersion, DriveServiceError> {
        let version_id = crate::infrastructure::sql::next_drive_runtime_id("Drive node version")?;
        create_node_version_in_transaction(
            connection,
            &CreateDriveNodeVersionCommand {
                id: format!("ver_{version_id}"),
                tenant_id: tenant_id.to_string(),
                space_id: space_id.to_string(),
                node_id: node_id.to_string(),
                version_no: plan.version_no,
                storage_object_id: Some(plan.id.clone()),
                content_type: content_type.to_string(),
                content_length,
                checksum_sha256_hex: checksum_sha256_hex.to_string(),
                version_kind: DriveNodeVersionKind::Auto,
                version_label: None,
                change_source: if uploader_upload_item.is_some() {
                    DriveNodeVersionChangeSource::Uploader
                } else {
                    DriveNodeVersionChangeSource::AppApi
                },
                change_summary: None,
                restored_from_version_id: None,
                app_id: uploader_upload_item.map(|item| item.app_id.clone()),
                app_resource_type: uploader_upload_item.map(|item| item.app_resource_type.clone()),
                app_resource_id: uploader_upload_item.map(|item| item.app_resource_id.clone()),
                scene: uploader_upload_item.and_then(|item| item.scene.clone()),
                source: uploader_upload_item.and_then(|item| item.source.clone()),
                operator_id: operator_id.to_string(),
            },
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn complete_uploader_item(
        &self,
        connection: &mut PgConnection,
        tenant_id: &str,
        upload_item: &DriveUploadItem,
        storage_object: &CompletedStorageObjectInsertPlan,
        session: &DriveUploadSession,
        content_type: &str,
        content_length: i64,
        checksum_sha256_hex: &str,
        uploaded_parts_count: i64,
        operator_id: &str,
        before_content_state: &str,
    ) -> Result<(), DriveServiceError> {
        sqlx::query(
            "UPDATE dr_drive_upload_item
             SET status='completed',
                 checksum_sha256_hex=$1,
                 uploaded_bytes=$2,
                 uploaded_parts_count=$3,
                 updated_by=$4,
                 updated_at=CURRENT_TIMESTAMP
             WHERE tenant_id=$5
               AND id=$6
               AND upload_session_id=$7",
        )
        .bind(checksum_sha256_hex)
        .bind(content_length)
        .bind(uploaded_parts_count)
        .bind(operator_id)
        .bind(tenant_id)
        .bind(&upload_item.id)
        .bind(&session.id)
        .execute(&mut *connection)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("complete dr_drive_upload_item failed: {error}"))
        })?;

        sqlx::query(
            "INSERT INTO dr_drive_file_sensitive_operation (
                id, tenant_id, organization_id, user_id, space_id, node_id,
                storage_object_id, upload_item_id, operation_type, operation_reason,
                content_type, content_type_group, content_length, checksum_sha256_hex,
                object_bucket, object_key, before_lifecycle_status, after_lifecycle_status,
                operator_id, maintenance_job_id, request_id, trace_id, object_delete_status
             ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, 'upload_completed', 'user_request',
                $9, $10, $11, $12,
                $13, $14, $15, 'active',
                $16, NULL, NULL, NULL, 'not_required'
             )",
        )
        .bind(sensitive_operation_id(
            tenant_id,
            "upload_completed",
            &upload_item.id,
            &storage_object.id,
        ))
        .bind(tenant_id)
        .bind(upload_item.organization_id.as_deref().unwrap_or("0"))
        .bind(&upload_item.user_id)
        .bind(&upload_item.space_id)
        .bind(&upload_item.node_id)
        .bind(&storage_object.id)
        .bind(&upload_item.id)
        .bind(content_type)
        .bind(&upload_item.content_type_group)
        .bind(content_length)
        .bind(checksum_sha256_hex)
        .bind(&session.bucket)
        .bind(&session.object_key)
        .bind(before_content_state)
        .bind(operator_id)
        .execute(&mut *connection)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "insert dr_drive_file_sensitive_operation upload_completed failed: {error}"
            ))
        })?;
        Ok(())
    }

    /// Reset the session and best-effort delete an orphaned object.
    ///
    /// Runs only after storage finalisation succeeded but the database
    /// transaction failed. Nothing here may fail the caller: the original
    /// database error is the one surfaced, and every cleanup problem is logged.
    async fn recover_completion_after_db_failure(
        &self,
        tenant_id: &str,
        session: &DriveUploadSession,
        operator_id: &str,
    ) {
        match self
            .sessions()
            .release_completion_claim(tenant_id, &session.id, operator_id)
            .await
        {
            Ok(true) => {}
            Ok(false) => tracing::warn!(
                event = "drive.upload.completion_db_failure_session_reset_skipped",
                tenant_id = %tenant_id,
                upload_session_id = %session.id,
                "upload session left the completing state concurrently; reset skipped"
            ),
            Err(error) => tracing::error!(
                event = "drive.upload.completion_db_failure_session_reset_failed",
                tenant_id = %tenant_id,
                upload_session_id = %session.id,
                error = ?error,
                "failed to reset upload session after DB completion failure"
            ),
        }

        match self
            .storage
            .delete_object(DeleteDriveStoredObjectCommand {
                storage_provider_id: session.storage_provider_id.clone(),
                bucket: session.bucket.clone(),
                object_key: session.object_key.clone(),
            })
            .await
        {
            Ok(true) => {}
            Ok(false) => tracing::warn!(
                event = "drive.upload.completion_db_failure_orphan_delete_skipped",
                tenant_id = %tenant_id,
                upload_session_id = %session.id,
                "storage provider does not support object delete"
            ),
            Err(error) => tracing::warn!(
                event = "drive.upload.completion_db_failure_orphan_delete_failed",
                tenant_id = %tenant_id,
                upload_session_id = %session.id,
                object_key = %session.object_key,
                error = ?error,
                "failed to delete orphaned storage object after DB completion failure"
            ),
        }

        tracing::warn!(
            event = "drive.upload.completion_db_failure_recovered",
            tenant_id = %tenant_id,
            upload_session_id = %session.id,
            object_key = %session.object_key,
            "upload DB commit failed after storage finalize; session reset and orphan cleanup attempted"
        );
    }
}

/// The storage-object row a completion will insert, decided before any write.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CompletedStorageObjectInsertPlan {
    id: String,
    version_no: i64,
}

fn require_text(value: &str, field_name: &str) -> Result<String, DriveServiceError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} is required"
        )));
    }
    Ok(trimmed.to_string())
}

fn current_epoch_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

/// The detail string the app API returns when a node role check fails.
fn drive_role_required_error() -> DriveServiceError {
    DriveServiceError::PermissionDenied(
        "subject does not have required access to the drive node".to_string(),
    )
}
