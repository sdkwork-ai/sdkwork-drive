use crate::application::sandbox_service::DriveSandboxService;
use crate::domain::sandbox::AuthorizedSandboxMount;
use crate::domain::sandbox_directory::{
    sandbox_idempotency_key_hash, SandboxDirectoryAccess, SandboxDirectoryEntry, SandboxEntryKind,
    SandboxEntryName, SandboxFileContent, SandboxIdempotencyKey, SandboxLogicalPath,
    MAX_SANDBOX_FILE_CONTENT_BYTES,
};
use crate::ports::sandbox_directory_provider::DriveSandboxFileSystemProvider;
use crate::ports::sandbox_mutation_operation_store::{
    BeginSandboxMutationOperation, CompleteSandboxMutationOperation,
    DriveSandboxMutationOperationStore, SandboxMutationOperationBeginResult, SandboxMutationResult,
};
use crate::ports::sandbox_principal_resolver::EffectiveSandboxPrincipal;
use crate::ports::sandbox_store::DriveSandboxStore;
use crate::DriveServiceError;
use sdkwork_utils_rust::sha256_hash;
use uuid::Uuid;

pub const SANDBOX_FILE_CREATED_AUDIT_ACTION: &str = "drive.sandbox.file.created";
pub const SANDBOX_FILE_UPDATED_AUDIT_ACTION: &str = "drive.sandbox.file.updated";
pub const SANDBOX_ENTRY_MOVED_AUDIT_ACTION: &str = "drive.sandbox.entry.moved";
pub const SANDBOX_ENTRY_DELETED_AUDIT_ACTION: &str = "drive.sandbox.entry.deleted";
pub const SANDBOX_ENTRY_AUDIT_RESOURCE_TYPE: &str = "sandbox_entry";

const SANDBOX_MUTATION_LEASE_MS: i64 = 30_000;

struct BeginSandboxOperationInput<'a> {
    tenant_id: &'a str,
    sandbox_id: &'a str,
    mutation: &'a SandboxMutationContext,
    mutation_kind: &'static str,
    result_parent: &'a SandboxLogicalPath,
    result_name: &'a SandboxEntryName,
    request_fingerprint: String,
}

#[derive(Debug, Clone)]
pub struct ReadSandboxFileCommand {
    pub tenant_id: String,
    pub sandbox_id: String,
    pub principals: Vec<EffectiveSandboxPrincipal>,
    pub entry_id: String,
    pub logical_path: String,
}

#[derive(Debug, Clone)]
pub struct CreateSandboxFileCommand {
    pub tenant_id: String,
    pub sandbox_id: String,
    pub principals: Vec<EffectiveSandboxPrincipal>,
    pub parent_logical_path: String,
    pub name: String,
    pub bytes: Vec<u8>,
    pub mutation: SandboxMutationContext,
}

#[derive(Debug, Clone)]
pub struct UpdateSandboxFileCommand {
    pub tenant_id: String,
    pub sandbox_id: String,
    pub principals: Vec<EffectiveSandboxPrincipal>,
    pub entry_id: String,
    pub logical_path: String,
    pub expected_revision: String,
    pub bytes: Vec<u8>,
    pub mutation: SandboxMutationContext,
}

#[derive(Debug, Clone)]
pub struct MoveSandboxEntryCommand {
    pub tenant_id: String,
    pub sandbox_id: String,
    pub principals: Vec<EffectiveSandboxPrincipal>,
    pub entry_id: String,
    pub logical_path: String,
    pub destination_parent_logical_path: String,
    pub destination_name: String,
    pub expected_revision: String,
    pub mutation: SandboxMutationContext,
}

#[derive(Debug, Clone)]
pub struct DeleteSandboxEntryCommand {
    pub tenant_id: String,
    pub sandbox_id: String,
    pub principals: Vec<EffectiveSandboxPrincipal>,
    pub entry_id: String,
    pub logical_path: String,
    pub expected_revision: String,
    pub recursive: bool,
    pub mutation: SandboxMutationContext,
}

#[derive(Debug, Clone)]
pub struct SandboxMutationContext {
    pub operator_id: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub idempotency_key: String,
}

pub struct DriveSandboxFileSystemService<S, P, O>
where
    S: DriveSandboxStore,
    P: DriveSandboxFileSystemProvider,
    O: DriveSandboxMutationOperationStore,
{
    authorization: DriveSandboxService<S>,
    provider: P,
    operations: O,
}

impl<S, P, O> DriveSandboxFileSystemService<S, P, O>
where
    S: DriveSandboxStore,
    P: DriveSandboxFileSystemProvider,
    O: DriveSandboxMutationOperationStore,
{
    pub fn new(store: S, provider: P, operation_store: O) -> Self {
        Self {
            authorization: DriveSandboxService::new(store),
            provider,
            operations: operation_store,
        }
    }

    pub async fn read_file(
        &self,
        command: ReadSandboxFileCommand,
    ) -> Result<SandboxFileContent, DriveServiceError> {
        let logical_path = parse_entry_path(&command.logical_path)?;
        validate_entry_id(&command.entry_id)?;
        let mount = self
            .authorize(
                &command.tenant_id,
                &command.sandbox_id,
                &command.principals,
                SandboxDirectoryAccess::Read,
            )
            .await?;
        let content = self
            .provider
            .read_file(&mount, &logical_path, MAX_SANDBOX_FILE_CONTENT_BYTES)
            .await?;
        ensure_entry_identity(&content.entry, &command.entry_id)?;
        Ok(content)
    }

    pub async fn create_file(
        &self,
        command: CreateSandboxFileCommand,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        validate_content(&command.bytes)?;
        validate_mutation_context(&command.mutation)?;
        let parent = parse_path(&command.parent_logical_path)?;
        let name = parse_name(&command.name)?;
        let mount = self
            .authorize(
                &command.tenant_id,
                &command.sandbox_id,
                &command.principals,
                SandboxDirectoryAccess::Full,
            )
            .await?;
        let content_hash = sha256_hash(&command.bytes);
        let fingerprint = mutation_fingerprint(&[
            "sandbox-file-create-v1",
            parent.as_str(),
            name.as_str(),
            &content_hash,
        ]);
        let begin = self
            .begin_operation(BeginSandboxOperationInput {
                tenant_id: &command.tenant_id,
                sandbox_id: &command.sandbox_id,
                mutation: &command.mutation,
                mutation_kind: "create_file",
                result_parent: &parent,
                result_name: &name,
                request_fingerprint: fingerprint,
            })
            .await?;
        match begin {
            SandboxMutationOperationBeginResult::Completed(result) => result.into_entry(),
            SandboxMutationOperationBeginResult::FailedConflict => previous_conflict(),
            SandboxMutationOperationBeginResult::Started {
                operation_id,
                lease_token,
            } => {
                self.execute_create_file(
                    operation_id,
                    lease_token,
                    &mount,
                    &parent,
                    &name,
                    &command,
                )
                .await
            }
            SandboxMutationOperationBeginResult::Pending {
                operation_id,
                lease_expires_at_ms,
            } => {
                self.ensure_expired(lease_expires_at_ms)?;
                let target = parent.join(&name);
                if let Some(entry) = self.provider.get_entry(&mount, &target).await? {
                    if entry.kind == SandboxEntryKind::File {
                        let content = self
                            .provider
                            .read_file(&mount, &target, MAX_SANDBOX_FILE_CONTENT_BYTES)
                            .await?;
                        if content.bytes == command.bytes {
                            return self
                                .complete_entry(
                                    operation_id,
                                    None,
                                    content.entry,
                                    SANDBOX_FILE_CREATED_AUDIT_ACTION,
                                    &command.tenant_id,
                                    &command.mutation,
                                )
                                .await;
                        }
                    }
                    return self
                        .claim_and_fail_entry_conflict(
                            operation_id,
                            &command.tenant_id,
                            "sandbox file already exists",
                        )
                        .await;
                }
                let lease_token = self.claim_expired(operation_id, &command.tenant_id).await?;
                self.execute_create_file(
                    operation_id,
                    lease_token,
                    &mount,
                    &parent,
                    &name,
                    &command,
                )
                .await
            }
        }
    }

    pub async fn update_file(
        &self,
        command: UpdateSandboxFileCommand,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        validate_content(&command.bytes)?;
        validate_mutation_context(&command.mutation)?;
        validate_entry_id(&command.entry_id)?;
        validate_revision(&command.expected_revision)?;
        let logical_path = parse_entry_path(&command.logical_path)?;
        let (parent, name) = split_entry(&logical_path)?;
        let mount = self
            .authorize(
                &command.tenant_id,
                &command.sandbox_id,
                &command.principals,
                SandboxDirectoryAccess::Full,
            )
            .await?;
        let content_hash = sha256_hash(&command.bytes);
        let fingerprint = mutation_fingerprint(&[
            "sandbox-file-update-v1",
            &command.entry_id,
            logical_path.as_str(),
            &command.expected_revision,
            &content_hash,
        ]);
        let begin = self
            .begin_operation(BeginSandboxOperationInput {
                tenant_id: &command.tenant_id,
                sandbox_id: &command.sandbox_id,
                mutation: &command.mutation,
                mutation_kind: "update_file",
                result_parent: &parent,
                result_name: &name,
                request_fingerprint: fingerprint,
            })
            .await?;
        match begin {
            SandboxMutationOperationBeginResult::Completed(result) => result.into_entry(),
            SandboxMutationOperationBeginResult::FailedConflict => previous_conflict(),
            SandboxMutationOperationBeginResult::Started {
                operation_id,
                lease_token,
            } => {
                self.execute_update_file(operation_id, lease_token, &mount, &logical_path, &command)
                    .await
            }
            SandboxMutationOperationBeginResult::Pending {
                operation_id,
                lease_expires_at_ms,
            } => {
                self.ensure_expired(lease_expires_at_ms)?;
                match self
                    .provider
                    .read_file(&mount, &logical_path, MAX_SANDBOX_FILE_CONTENT_BYTES)
                    .await
                {
                    Ok(content) => {
                        if ensure_entry_identity(&content.entry, &command.entry_id).is_err() {
                            return self
                                .claim_and_fail_entry_conflict(
                                    operation_id,
                                    &command.tenant_id,
                                    "sandbox entry id does not match logical path",
                                )
                                .await;
                        }
                        if content.bytes == command.bytes {
                            return self
                                .complete_entry(
                                    operation_id,
                                    None,
                                    content.entry,
                                    SANDBOX_FILE_UPDATED_AUDIT_ACTION,
                                    &command.tenant_id,
                                    &command.mutation,
                                )
                                .await;
                        }
                    }
                    Err(error @ DriveServiceError::PermissionDenied(_))
                    | Err(error @ DriveServiceError::Internal(_)) => return Err(error),
                    Err(_) => {}
                }
                let lease_token = self.claim_expired(operation_id, &command.tenant_id).await?;
                self.execute_update_file(operation_id, lease_token, &mount, &logical_path, &command)
                    .await
            }
        }
    }

    pub async fn move_entry(
        &self,
        command: MoveSandboxEntryCommand,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        validate_mutation_context(&command.mutation)?;
        validate_entry_id(&command.entry_id)?;
        validate_revision(&command.expected_revision)?;
        let logical_path = parse_entry_path(&command.logical_path)?;
        let destination_parent = parse_path(&command.destination_parent_logical_path)?;
        let destination_name = parse_name(&command.destination_name)?;
        let destination_path = destination_parent.join(&destination_name);
        let mount = self
            .authorize(
                &command.tenant_id,
                &command.sandbox_id,
                &command.principals,
                SandboxDirectoryAccess::Full,
            )
            .await?;
        let fingerprint = mutation_fingerprint(&[
            "sandbox-entry-move-v1",
            &command.entry_id,
            logical_path.as_str(),
            destination_parent.as_str(),
            destination_name.as_str(),
            &command.expected_revision,
        ]);
        let begin = self
            .begin_operation(BeginSandboxOperationInput {
                tenant_id: &command.tenant_id,
                sandbox_id: &command.sandbox_id,
                mutation: &command.mutation,
                mutation_kind: "move_entry",
                result_parent: &destination_parent,
                result_name: &destination_name,
                request_fingerprint: fingerprint,
            })
            .await?;
        match begin {
            SandboxMutationOperationBeginResult::Completed(result) => result.into_entry(),
            SandboxMutationOperationBeginResult::FailedConflict => previous_conflict(),
            SandboxMutationOperationBeginResult::Started {
                operation_id,
                lease_token,
            } => {
                self.execute_move_entry(
                    operation_id,
                    lease_token,
                    &mount,
                    &logical_path,
                    &destination_parent,
                    &destination_name,
                    &command,
                )
                .await
            }
            SandboxMutationOperationBeginResult::Pending {
                operation_id,
                lease_expires_at_ms,
            } => {
                self.ensure_expired(lease_expires_at_ms)?;
                let source = self.provider.get_entry(&mount, &logical_path).await?;
                let destination = self.provider.get_entry(&mount, &destination_path).await?;
                if let Some(entry) = destination {
                    if source.is_none() || destination_path == logical_path {
                        return self
                            .complete_entry(
                                operation_id,
                                None,
                                entry,
                                SANDBOX_ENTRY_MOVED_AUDIT_ACTION,
                                &command.tenant_id,
                                &command.mutation,
                            )
                            .await;
                    }
                    return self
                        .claim_and_fail_entry_conflict(
                            operation_id,
                            &command.tenant_id,
                            "sandbox destination entry already exists",
                        )
                        .await;
                }
                if source.is_none() {
                    return self
                        .claim_and_fail_entry_conflict(
                            operation_id,
                            &command.tenant_id,
                            "sandbox move source was not found",
                        )
                        .await;
                }
                let lease_token = self.claim_expired(operation_id, &command.tenant_id).await?;
                self.execute_move_entry(
                    operation_id,
                    lease_token,
                    &mount,
                    &logical_path,
                    &destination_parent,
                    &destination_name,
                    &command,
                )
                .await
            }
        }
    }

    pub async fn delete_entry(
        &self,
        command: DeleteSandboxEntryCommand,
    ) -> Result<(), DriveServiceError> {
        validate_mutation_context(&command.mutation)?;
        validate_entry_id(&command.entry_id)?;
        validate_revision(&command.expected_revision)?;
        let logical_path = parse_entry_path(&command.logical_path)?;
        let (parent, name) = split_entry(&logical_path)?;
        let mount = self
            .authorize(
                &command.tenant_id,
                &command.sandbox_id,
                &command.principals,
                SandboxDirectoryAccess::Full,
            )
            .await?;
        let fingerprint = mutation_fingerprint(&[
            "sandbox-entry-delete-v1",
            &command.entry_id,
            logical_path.as_str(),
            &command.expected_revision,
            if command.recursive {
                "recursive"
            } else {
                "empty"
            },
        ]);
        let begin = self
            .begin_operation(BeginSandboxOperationInput {
                tenant_id: &command.tenant_id,
                sandbox_id: &command.sandbox_id,
                mutation: &command.mutation,
                mutation_kind: "delete_entry",
                result_parent: &parent,
                result_name: &name,
                request_fingerprint: fingerprint,
            })
            .await?;
        match begin {
            SandboxMutationOperationBeginResult::Completed(SandboxMutationResult::Deleted) => {
                Ok(())
            }
            SandboxMutationOperationBeginResult::Completed(SandboxMutationResult::Entry(_)) => {
                invalid_completed_result()
            }
            SandboxMutationOperationBeginResult::FailedConflict => previous_conflict(),
            SandboxMutationOperationBeginResult::Started {
                operation_id,
                lease_token,
            } => {
                self.execute_delete_entry(
                    operation_id,
                    lease_token,
                    &mount,
                    &logical_path,
                    &command,
                )
                .await
            }
            SandboxMutationOperationBeginResult::Pending {
                operation_id,
                lease_expires_at_ms,
            } => {
                self.ensure_expired(lease_expires_at_ms)?;
                if self
                    .provider
                    .get_entry(&mount, &logical_path)
                    .await?
                    .is_none()
                {
                    return self.complete_deleted(operation_id, None, &command).await;
                }
                let lease_token = self.claim_expired(operation_id, &command.tenant_id).await?;
                self.execute_delete_entry(
                    operation_id,
                    lease_token,
                    &mount,
                    &logical_path,
                    &command,
                )
                .await
            }
        }
    }

    async fn authorize(
        &self,
        tenant_id: &str,
        sandbox_id: &str,
        principals: &[EffectiveSandboxPrincipal],
        access: SandboxDirectoryAccess,
    ) -> Result<AuthorizedSandboxMount, DriveServiceError> {
        let mount = self
            .authorization
            .authorize_mount_for_principals(tenant_id, sandbox_id, principals, access)
            .await?;
        if !self.provider.supports(mount.provider_kind()) {
            return Err(DriveServiceError::Internal(
                "sandbox file-system provider is unavailable".to_string(),
            ));
        }
        Ok(mount)
    }

    async fn begin_operation(
        &self,
        input: BeginSandboxOperationInput<'_>,
    ) -> Result<SandboxMutationOperationBeginResult, DriveServiceError> {
        let key = SandboxIdempotencyKey::parse(&input.mutation.idempotency_key)
            .map_err(|error| DriveServiceError::Validation(error.to_string()))?;
        let now_ms = chrono::Utc::now().timestamp_millis();
        let lease_token = new_lease_token();
        self.operations
            .begin_or_load(&BeginSandboxMutationOperation {
                tenant_id: input.tenant_id.to_string(),
                sandbox_id: input.sandbox_id.to_string(),
                actor_id: input.mutation.operator_id.clone(),
                idempotency_key_hash: sandbox_idempotency_key_hash(input.mutation_kind, &key),
                request_fingerprint: input.request_fingerprint,
                mutation_kind: input.mutation_kind.to_string(),
                parent_logical_path: input.result_parent.as_str().to_string(),
                entry_name: input.result_name.as_str().to_string(),
                lease_token: lease_token.clone(),
                lease_expires_at_ms: now_ms + SANDBOX_MUTATION_LEASE_MS,
            })
            .await
    }

    async fn execute_create_file(
        &self,
        operation_id: i64,
        lease_token: String,
        mount: &AuthorizedSandboxMount,
        parent: &SandboxLogicalPath,
        name: &SandboxEntryName,
        command: &CreateSandboxFileCommand,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        match self
            .provider
            .create_file(mount, parent, name, &command.bytes)
            .await
        {
            Ok(entry) => {
                self.complete_entry(
                    operation_id,
                    Some(lease_token),
                    entry,
                    SANDBOX_FILE_CREATED_AUDIT_ACTION,
                    &command.tenant_id,
                    &command.mutation,
                )
                .await
            }
            Err(error) => {
                self.persist_entry_failure(operation_id, &command.tenant_id, &lease_token, error)
                    .await
            }
        }
    }

    async fn execute_update_file(
        &self,
        operation_id: i64,
        lease_token: String,
        mount: &AuthorizedSandboxMount,
        logical_path: &SandboxLogicalPath,
        command: &UpdateSandboxFileCommand,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        let current = match self.provider.get_entry(mount, logical_path).await {
            Ok(Some(entry)) => entry,
            Ok(None) => {
                return self
                    .persist_entry_failure(
                        operation_id,
                        &command.tenant_id,
                        &lease_token,
                        DriveServiceError::NotFound("sandbox file was not found".to_string()),
                    )
                    .await
            }
            Err(error) => {
                return self
                    .persist_entry_failure(operation_id, &command.tenant_id, &lease_token, error)
                    .await
            }
        };
        if let Err(error) = ensure_entry_identity(&current, &command.entry_id) {
            return self
                .persist_entry_failure(operation_id, &command.tenant_id, &lease_token, error)
                .await;
        }
        match self
            .provider
            .update_file(
                mount,
                logical_path,
                &command.expected_revision,
                &command.bytes,
            )
            .await
        {
            Ok(entry) => {
                self.complete_entry(
                    operation_id,
                    Some(lease_token),
                    entry,
                    SANDBOX_FILE_UPDATED_AUDIT_ACTION,
                    &command.tenant_id,
                    &command.mutation,
                )
                .await
            }
            Err(error) => {
                self.persist_entry_failure(operation_id, &command.tenant_id, &lease_token, error)
                    .await
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn execute_move_entry(
        &self,
        operation_id: i64,
        lease_token: String,
        mount: &AuthorizedSandboxMount,
        logical_path: &SandboxLogicalPath,
        destination_parent: &SandboxLogicalPath,
        destination_name: &SandboxEntryName,
        command: &MoveSandboxEntryCommand,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        let current = match self.provider.get_entry(mount, logical_path).await {
            Ok(Some(entry)) => entry,
            Ok(None) => {
                return self
                    .persist_entry_failure(
                        operation_id,
                        &command.tenant_id,
                        &lease_token,
                        DriveServiceError::NotFound("sandbox entry was not found".to_string()),
                    )
                    .await
            }
            Err(error) => {
                return self
                    .persist_entry_failure(operation_id, &command.tenant_id, &lease_token, error)
                    .await
            }
        };
        if let Err(error) = ensure_entry_identity(&current, &command.entry_id) {
            return self
                .persist_entry_failure(operation_id, &command.tenant_id, &lease_token, error)
                .await;
        }
        match self
            .provider
            .move_entry(
                mount,
                logical_path,
                destination_parent,
                destination_name,
                &command.expected_revision,
            )
            .await
        {
            Ok(entry) => {
                self.complete_entry(
                    operation_id,
                    Some(lease_token),
                    entry,
                    SANDBOX_ENTRY_MOVED_AUDIT_ACTION,
                    &command.tenant_id,
                    &command.mutation,
                )
                .await
            }
            Err(error) => {
                self.persist_entry_failure(operation_id, &command.tenant_id, &lease_token, error)
                    .await
            }
        }
    }

    async fn execute_delete_entry(
        &self,
        operation_id: i64,
        lease_token: String,
        mount: &AuthorizedSandboxMount,
        logical_path: &SandboxLogicalPath,
        command: &DeleteSandboxEntryCommand,
    ) -> Result<(), DriveServiceError> {
        let current = match self.provider.get_entry(mount, logical_path).await {
            Ok(Some(entry)) => entry,
            Ok(None) => {
                return self
                    .persist_delete_failure(
                        operation_id,
                        &command.tenant_id,
                        &lease_token,
                        DriveServiceError::NotFound("sandbox entry was not found".to_string()),
                    )
                    .await
            }
            Err(error) => {
                return self
                    .persist_delete_failure(operation_id, &command.tenant_id, &lease_token, error)
                    .await
            }
        };
        if let Err(error) = ensure_entry_identity(&current, &command.entry_id) {
            return self
                .persist_delete_failure(operation_id, &command.tenant_id, &lease_token, error)
                .await;
        }
        match self
            .provider
            .delete_entry(
                mount,
                logical_path,
                &command.expected_revision,
                command.recursive,
            )
            .await
        {
            Ok(()) => {
                self.complete_deleted(operation_id, Some(lease_token), command)
                    .await
            }
            Err(error) => {
                self.persist_delete_failure(operation_id, &command.tenant_id, &lease_token, error)
                    .await
            }
        }
    }

    async fn complete_entry(
        &self,
        operation_id: i64,
        lease_token: Option<String>,
        entry: SandboxDirectoryEntry,
        audit_action: &str,
        tenant_id: &str,
        mutation: &SandboxMutationContext,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        let resource_id = entry.id.clone();
        self.operations
            .complete_with_audit(&CompleteSandboxMutationOperation {
                operation_id,
                tenant_id: tenant_id.to_string(),
                lease_token,
                result: SandboxMutationResult::Entry(entry),
                audit_action: audit_action.to_string(),
                audit_resource_type: SANDBOX_ENTRY_AUDIT_RESOURCE_TYPE.to_string(),
                audit_resource_id: resource_id,
                operator_id: mutation.operator_id.clone(),
                request_id: mutation.request_id.clone(),
                trace_id: mutation.trace_id.clone(),
            })
            .await?
            .into_entry()
    }

    async fn complete_deleted(
        &self,
        operation_id: i64,
        lease_token: Option<String>,
        command: &DeleteSandboxEntryCommand,
    ) -> Result<(), DriveServiceError> {
        match self
            .operations
            .complete_with_audit(&CompleteSandboxMutationOperation {
                operation_id,
                tenant_id: command.tenant_id.clone(),
                lease_token,
                result: SandboxMutationResult::Deleted,
                audit_action: SANDBOX_ENTRY_DELETED_AUDIT_ACTION.to_string(),
                audit_resource_type: SANDBOX_ENTRY_AUDIT_RESOURCE_TYPE.to_string(),
                audit_resource_id: command.entry_id.clone(),
                operator_id: command.mutation.operator_id.clone(),
                request_id: command.mutation.request_id.clone(),
                trace_id: command.mutation.trace_id.clone(),
            })
            .await?
        {
            SandboxMutationResult::Deleted => Ok(()),
            SandboxMutationResult::Entry(_) => invalid_completed_result(),
        }
    }

    async fn claim_expired(
        &self,
        operation_id: i64,
        tenant_id: &str,
    ) -> Result<String, DriveServiceError> {
        let now_ms = chrono::Utc::now().timestamp_millis();
        let lease_token = new_lease_token();
        if !self
            .operations
            .try_claim_pending(
                operation_id,
                tenant_id,
                now_ms,
                &lease_token,
                now_ms + SANDBOX_MUTATION_LEASE_MS,
            )
            .await?
        {
            return Err(DriveServiceError::Conflict(
                "sandbox mutation is in progress".to_string(),
            ));
        }
        Ok(lease_token)
    }

    async fn claim_and_fail_entry_conflict(
        &self,
        operation_id: i64,
        tenant_id: &str,
        detail: &str,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        let lease_token = self.claim_expired(operation_id, tenant_id).await?;
        match self
            .operations
            .mark_conflict(operation_id, tenant_id, &lease_token)
            .await?
        {
            Some(SandboxMutationResult::Entry(entry)) => Ok(entry),
            Some(SandboxMutationResult::Deleted) => invalid_completed_result(),
            None => Err(DriveServiceError::Conflict(detail.to_string())),
        }
    }

    async fn persist_entry_failure(
        &self,
        operation_id: i64,
        tenant_id: &str,
        lease_token: &str,
        error: DriveServiceError,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        if matches!(
            error,
            DriveServiceError::Validation(_)
                | DriveServiceError::Conflict(_)
                | DriveServiceError::NotFound(_)
        ) {
            if let Some(result) = self
                .operations
                .mark_conflict(operation_id, tenant_id, lease_token)
                .await?
            {
                return match result {
                    SandboxMutationResult::Entry(entry) => Ok(entry),
                    SandboxMutationResult::Deleted => invalid_completed_result(),
                };
            }
        }
        Err(error)
    }

    async fn persist_delete_failure(
        &self,
        operation_id: i64,
        tenant_id: &str,
        lease_token: &str,
        error: DriveServiceError,
    ) -> Result<(), DriveServiceError> {
        if matches!(
            error,
            DriveServiceError::Validation(_)
                | DriveServiceError::Conflict(_)
                | DriveServiceError::NotFound(_)
        ) {
            if let Some(result) = self
                .operations
                .mark_conflict(operation_id, tenant_id, lease_token)
                .await?
            {
                return match result {
                    SandboxMutationResult::Deleted => Ok(()),
                    SandboxMutationResult::Entry(_) => invalid_completed_result(),
                };
            }
        }
        Err(error)
    }

    fn ensure_expired(&self, lease_expires_at_ms: i64) -> Result<(), DriveServiceError> {
        if lease_expires_at_ms > chrono::Utc::now().timestamp_millis() {
            Err(DriveServiceError::Conflict(
                "sandbox mutation is in progress".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

trait SandboxMutationResultExt {
    fn into_entry(self) -> Result<SandboxDirectoryEntry, DriveServiceError>;
}

impl SandboxMutationResultExt for SandboxMutationResult {
    fn into_entry(self) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        match self {
            Self::Entry(entry) => Ok(entry),
            Self::Deleted => invalid_completed_result(),
        }
    }
}

fn validate_mutation_context(context: &SandboxMutationContext) -> Result<(), DriveServiceError> {
    if context.operator_id.trim().is_empty() {
        return Err(DriveServiceError::Validation(
            "sandbox mutation operator is required".to_string(),
        ));
    }
    SandboxIdempotencyKey::parse(&context.idempotency_key)
        .map(|_| ())
        .map_err(|error| DriveServiceError::Validation(error.to_string()))
}

fn validate_entry_id(value: &str) -> Result<(), DriveServiceError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(DriveServiceError::Validation(
            "sandbox entry id is invalid".to_string(),
        ));
    }
    Ok(())
}

fn validate_revision(value: &str) -> Result<(), DriveServiceError> {
    if value.is_empty() || value.len() > 128 || value.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(DriveServiceError::Validation(
            "sandbox entry revision is invalid".to_string(),
        ));
    }
    Ok(())
}

fn validate_content(bytes: &[u8]) -> Result<(), DriveServiceError> {
    if bytes.len() > MAX_SANDBOX_FILE_CONTENT_BYTES {
        return Err(DriveServiceError::Validation(format!(
            "sandbox file content exceeds the {MAX_SANDBOX_FILE_CONTENT_BYTES} byte limit"
        )));
    }
    Ok(())
}

fn parse_path(value: &str) -> Result<SandboxLogicalPath, DriveServiceError> {
    SandboxLogicalPath::parse(value)
        .map_err(|error| DriveServiceError::Validation(error.to_string()))
}

fn parse_entry_path(value: &str) -> Result<SandboxLogicalPath, DriveServiceError> {
    let path = parse_path(value)?;
    path.split_entry()
        .map(|_| path)
        .map_err(|_| DriveServiceError::Validation("sandbox entry path is required".to_string()))
}

fn parse_name(value: &str) -> Result<SandboxEntryName, DriveServiceError> {
    SandboxEntryName::parse(value).map_err(|error| DriveServiceError::Validation(error.to_string()))
}

fn split_entry(
    path: &SandboxLogicalPath,
) -> Result<(SandboxLogicalPath, SandboxEntryName), DriveServiceError> {
    path.split_entry()
        .map_err(|error| DriveServiceError::Validation(error.to_string()))
}

fn ensure_entry_identity(
    entry: &SandboxDirectoryEntry,
    expected_entry_id: &str,
) -> Result<(), DriveServiceError> {
    if entry.id == expected_entry_id {
        Ok(())
    } else {
        Err(DriveServiceError::Conflict(
            "sandbox entry id does not match logical path".to_string(),
        ))
    }
}

fn mutation_fingerprint(fields: &[&str]) -> String {
    sha256_hash(fields.join("\0").as_bytes())
}

fn new_lease_token() -> String {
    Uuid::new_v4().simple().to_string()
}

fn previous_conflict<T>() -> Result<T, DriveServiceError> {
    Err(DriveServiceError::Conflict(
        "sandbox mutation previously failed due to a state conflict".to_string(),
    ))
}

fn invalid_completed_result<T>() -> Result<T, DriveServiceError> {
    Err(DriveServiceError::Internal(
        "completed sandbox mutation has an invalid result".to_string(),
    ))
}
