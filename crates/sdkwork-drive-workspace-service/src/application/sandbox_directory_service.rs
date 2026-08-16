use crate::application::sandbox_service::DriveSandboxService;
use crate::domain::sandbox_directory::{
    sandbox_idempotency_key_hash, SandboxDirectoryAccess, SandboxDirectoryEntry,
    SandboxDirectoryPage, SandboxDirectoryPageRequest, SandboxEntryName, SandboxIdempotencyKey,
    SandboxLogicalPath,
};
use crate::ports::sandbox_directory_provider::DriveSandboxDirectoryProvider;
use crate::ports::sandbox_mutation_operation_store::{
    BeginSandboxMutationOperation, CompleteSandboxMutationOperation,
    DriveSandboxMutationOperationStore, SandboxMutationOperationBeginResult, SandboxMutationResult,
};
use crate::ports::sandbox_principal_resolver::EffectiveSandboxPrincipal;
use crate::ports::sandbox_store::DriveSandboxStore;
use crate::DriveServiceError;
use sdkwork_utils_rust::sha256_hash;
use uuid::Uuid;

pub const SANDBOX_DIRECTORY_CREATED_AUDIT_ACTION: &str = "drive.sandbox.directory.created";
pub const SANDBOX_AUDIT_RESOURCE_TYPE: &str = "sandbox";
const SANDBOX_DIRECTORY_OPERATION_LEASE_MS: i64 = 30_000;

#[derive(Debug, Clone)]
pub struct ListSandboxDirectoryCommand {
    pub tenant_id: String,
    pub sandbox_id: String,
    pub principals: Vec<EffectiveSandboxPrincipal>,
    pub parent_logical_path: String,
    pub page_size: usize,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateSandboxDirectoryCommand {
    pub tenant_id: String,
    pub sandbox_id: String,
    pub principals: Vec<EffectiveSandboxPrincipal>,
    pub parent_logical_path: String,
    pub name: String,
    pub operator_id: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub idempotency_key: String,
}

pub struct DriveSandboxDirectoryService<S, P, O>
where
    S: DriveSandboxStore,
    P: DriveSandboxDirectoryProvider,
    O: DriveSandboxMutationOperationStore,
{
    authorization: DriveSandboxService<S>,
    provider: P,
    operations: O,
}

impl<S, P, O> DriveSandboxDirectoryService<S, P, O>
where
    S: DriveSandboxStore,
    P: DriveSandboxDirectoryProvider,
    O: DriveSandboxMutationOperationStore,
{
    pub fn new(store: S, provider: P, operation_store: O) -> Self {
        Self {
            authorization: DriveSandboxService::new(store),
            provider,
            operations: operation_store,
        }
    }

    pub async fn list_children(
        &self,
        command: ListSandboxDirectoryCommand,
    ) -> Result<SandboxDirectoryPage, DriveServiceError> {
        let parent = SandboxLogicalPath::parse(&command.parent_logical_path)
            .map_err(|error| DriveServiceError::Validation(error.to_string()))?;
        let page = SandboxDirectoryPageRequest::new(command.page_size, command.cursor)
            .map_err(|error| DriveServiceError::Validation(error.to_string()))?;
        let mount = self
            .authorization
            .authorize_mount_for_principals(
                &command.tenant_id,
                &command.sandbox_id,
                &command.principals,
                SandboxDirectoryAccess::Read,
            )
            .await?;
        self.require_supported_provider(mount.provider_kind())?;
        self.provider.list_children(&mount, &parent, &page).await
    }

    pub async fn create_directory(
        &self,
        command: CreateSandboxDirectoryCommand,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        if command.operator_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "sandbox directory operator is required".to_string(),
            ));
        }
        let parent = SandboxLogicalPath::parse(&command.parent_logical_path)
            .map_err(|error| DriveServiceError::Validation(error.to_string()))?;
        let name = SandboxEntryName::parse(&command.name)
            .map_err(|error| DriveServiceError::Validation(error.to_string()))?;
        let idempotency_key = SandboxIdempotencyKey::parse(&command.idempotency_key)
            .map_err(|error| DriveServiceError::Validation(error.to_string()))?;
        let idempotency_key_hash =
            sandbox_idempotency_key_hash("create_directory", &idempotency_key);
        let mount = self
            .authorization
            .authorize_mount_for_principals(
                &command.tenant_id,
                &command.sandbox_id,
                &command.principals,
                SandboxDirectoryAccess::Full,
            )
            .await?;
        self.require_supported_provider(mount.provider_kind())?;
        let request_fingerprint = sha256_hash(
            format!(
                "sandbox-directory-v1\0{}\0{}",
                parent.as_str(),
                name.as_str()
            )
            .as_bytes(),
        );
        let now_ms = chrono::Utc::now().timestamp_millis();
        let lease_token = new_lease_token();
        let begin = self
            .operations
            .begin_or_load(&BeginSandboxMutationOperation {
                tenant_id: command.tenant_id.clone(),
                sandbox_id: command.sandbox_id.clone(),
                actor_id: command.operator_id.clone(),
                idempotency_key_hash,
                request_fingerprint,
                mutation_kind: "create_directory".to_string(),
                parent_logical_path: parent.as_str().to_string(),
                entry_name: name.as_str().to_string(),
                lease_token: lease_token.clone(),
                lease_expires_at_ms: now_ms + SANDBOX_DIRECTORY_OPERATION_LEASE_MS,
            })
            .await?;

        match begin {
            SandboxMutationOperationBeginResult::Completed(SandboxMutationResult::Entry(entry)) => {
                Ok(entry)
            }
            SandboxMutationOperationBeginResult::Completed(SandboxMutationResult::Deleted) => {
                Err(DriveServiceError::Internal(
                    "completed sandbox directory operation has invalid result".to_string(),
                ))
            }
            SandboxMutationOperationBeginResult::FailedConflict => Err(
                DriveServiceError::Conflict("sandbox directory already exists".to_string()),
            ),
            SandboxMutationOperationBeginResult::Started {
                operation_id,
                lease_token,
            } => {
                self.execute_create(operation_id, lease_token, &mount, &parent, &name, &command)
                    .await
            }
            SandboxMutationOperationBeginResult::Pending {
                operation_id,
                lease_expires_at_ms,
            } => {
                if lease_expires_at_ms > now_ms {
                    return Err(DriveServiceError::Conflict(
                        "sandbox directory operation is in progress".to_string(),
                    ));
                }
                if let Some(entry) = self.provider.get_directory(&mount, &parent, &name).await? {
                    return self
                        .complete_operation(operation_id, None, entry, &command)
                        .await;
                }
                let claimed_lease_token = new_lease_token();
                let claimed = self
                    .operations
                    .try_claim_pending(
                        operation_id,
                        &command.tenant_id,
                        now_ms,
                        &claimed_lease_token,
                        now_ms + SANDBOX_DIRECTORY_OPERATION_LEASE_MS,
                    )
                    .await?;
                if !claimed {
                    return Err(DriveServiceError::Conflict(
                        "sandbox directory operation is in progress".to_string(),
                    ));
                }
                self.execute_create(
                    operation_id,
                    claimed_lease_token,
                    &mount,
                    &parent,
                    &name,
                    &command,
                )
                .await
            }
        }
    }

    fn require_supported_provider(&self, provider_kind: &str) -> Result<(), DriveServiceError> {
        if self.provider.supports(provider_kind) {
            Ok(())
        } else {
            Err(DriveServiceError::Internal(
                "sandbox directory provider is unavailable".to_string(),
            ))
        }
    }

    async fn execute_create(
        &self,
        operation_id: i64,
        lease_token: String,
        mount: &crate::domain::sandbox::AuthorizedSandboxMount,
        parent: &SandboxLogicalPath,
        name: &SandboxEntryName,
        command: &CreateSandboxDirectoryCommand,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        match self.provider.create_directory(mount, parent, name).await {
            Ok(entry) => {
                self.complete_operation(operation_id, Some(lease_token), entry, command)
                    .await
            }
            Err(error @ DriveServiceError::Conflict(_)) => {
                if let Some(result) = self
                    .operations
                    .mark_conflict(operation_id, &command.tenant_id, &lease_token)
                    .await?
                {
                    match result {
                        SandboxMutationResult::Entry(entry) => Ok(entry),
                        SandboxMutationResult::Deleted => Err(DriveServiceError::Internal(
                            "completed sandbox directory operation has invalid result".to_string(),
                        )),
                    }
                } else {
                    Err(error)
                }
            }
            Err(error) => Err(error),
        }
    }

    async fn complete_operation(
        &self,
        operation_id: i64,
        lease_token: Option<String>,
        entry: SandboxDirectoryEntry,
        command: &CreateSandboxDirectoryCommand,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        self.operations
            .complete_with_audit(&CompleteSandboxMutationOperation {
                operation_id,
                tenant_id: command.tenant_id.clone(),
                lease_token,
                result: SandboxMutationResult::Entry(entry),
                audit_action: SANDBOX_DIRECTORY_CREATED_AUDIT_ACTION.to_string(),
                audit_resource_type: SANDBOX_AUDIT_RESOURCE_TYPE.to_string(),
                audit_resource_id: command.sandbox_id.clone(),
                operator_id: command.operator_id.clone(),
                request_id: command.request_id.clone(),
                trace_id: command.trace_id.clone(),
            })
            .await?
            .into_entry()
    }
}

trait SandboxDirectoryMutationResultExt {
    fn into_entry(self) -> Result<SandboxDirectoryEntry, DriveServiceError>;
}

impl SandboxDirectoryMutationResultExt for SandboxMutationResult {
    fn into_entry(self) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        match self {
            SandboxMutationResult::Entry(entry) => Ok(entry),
            SandboxMutationResult::Deleted => Err(DriveServiceError::Internal(
                "completed sandbox directory operation has invalid result".to_string(),
            )),
        }
    }
}

fn new_lease_token() -> String {
    Uuid::new_v4().simple().to_string()
}
