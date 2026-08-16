use async_trait::async_trait;

use crate::domain::sandbox_directory::SandboxDirectoryEntry;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct BeginSandboxMutationOperation {
    pub tenant_id: String,
    pub sandbox_id: String,
    pub actor_id: String,
    pub idempotency_key_hash: String,
    pub request_fingerprint: String,
    pub mutation_kind: String,
    pub parent_logical_path: String,
    pub entry_name: String,
    pub lease_token: String,
    pub lease_expires_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxMutationOperationBeginResult {
    Started {
        operation_id: i64,
        lease_token: String,
    },
    Pending {
        operation_id: i64,
        lease_expires_at_ms: i64,
    },
    Completed(SandboxMutationResult),
    FailedConflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxMutationResult {
    Entry(SandboxDirectoryEntry),
    Deleted,
}

#[derive(Debug, Clone)]
pub struct CompleteSandboxMutationOperation {
    pub operation_id: i64,
    pub tenant_id: String,
    pub lease_token: Option<String>,
    pub result: SandboxMutationResult,
    pub audit_action: String,
    pub audit_resource_type: String,
    pub audit_resource_id: String,
    pub operator_id: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
}

#[async_trait]
pub trait DriveSandboxMutationOperationStore: Send + Sync {
    async fn begin_or_load(
        &self,
        operation: &BeginSandboxMutationOperation,
    ) -> Result<SandboxMutationOperationBeginResult, DriveServiceError>;

    async fn try_claim_pending(
        &self,
        operation_id: i64,
        tenant_id: &str,
        now_ms: i64,
        lease_token: &str,
        lease_expires_at_ms: i64,
    ) -> Result<bool, DriveServiceError>;

    /// Persists the successful result and its audit event in one database transaction.
    /// A concurrent/replayed completion returns the originally stored entry without a second audit.
    async fn complete_with_audit(
        &self,
        operation: &CompleteSandboxMutationOperation,
    ) -> Result<SandboxMutationResult, DriveServiceError>;

    /// Marks a stable target conflict. If another caller already completed the operation, its
    /// original result is returned so all same-key callers converge on the same response.
    async fn mark_conflict(
        &self,
        operation_id: i64,
        tenant_id: &str,
        lease_token: &str,
    ) -> Result<Option<SandboxMutationResult>, DriveServiceError>;
}
