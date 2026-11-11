use async_trait::async_trait;

use crate::domain::node_version::DriveNodeVersion;
use crate::DriveServiceError;

/// Register an object that is already stored as the next version of a node.
///
/// This is the in-process counterpart of the app API's upload-session
/// completion for callers that wrote the bytes themselves (a document renderer
/// writing a page body, for example). The object must already exist at
/// `storage_provider_id`/`bucket`/`object_key`; this call publishes the
/// `dr_drive_storage_object` row, the `dr_drive_node_version` row and the node
/// head snapshot in one transaction.
///
/// Field contract:
///
/// * `tenant_id`/`space_id`/`node_id` must describe one active file node in one
///   active space of the caller's tenant.
/// * `storage_provider_id`/`bucket`/`object_key` is the exact locator the bytes
///   were written to. `object_key` must follow the Drive storage-key rules (no
///   leading/trailing slash, no empty or `.`/`..` segments, at most 1024 bytes).
/// * `content_type` and `checksum_sha256_hex` (`sha256:<64 lowercase hex>`) are
///   validated, not trusted.
/// * `idempotency_key` identifies the completion attempt. Replaying a completion
///   for the same locator returns the version registered the first time; it never
///   appends a second version.
/// * `operator_id` is the actor recorded on the version, the node head and the
///   change/outbox event.
///
/// The caller is responsible for authorization: this store does not resolve a
/// subject. It is called from the service layer after the caller has enforced at
/// least `writer` on the node through Drive's permission service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteStoredVersionUploadCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: String,
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
    pub idempotency_key: String,
    pub operator_id: String,
}

/// Storage-side half of stored-version completion.
///
/// Implemented by the SQL repository and reached through
/// [`crate::application::upload_service::DriveUploadService::complete_stored_version`],
/// so callers keep using the upload service they already construct.
#[async_trait]
pub trait DriveStoredVersionStore: Send + Sync {
    /// Publish an already-stored object as the next node version.
    ///
    /// # Errors
    ///
    /// - `Validation` for a blank identifier, invalid content type, negative
    ///   length, malformed checksum, or an invalid object key.
    /// - `NotFound` when the node/space is absent or inactive in this tenant.
    /// - `Conflict` when the same locator was already completed with different
    ///   content metadata, when the version number raced another writer, or when
    ///   the managed-website guard rejects the mutation.
    /// - `Internal` when a query or commit fails.
    async fn complete_stored_version(
        &self,
        command: &CompleteStoredVersionUploadCommand,
    ) -> Result<DriveNodeVersion, DriveServiceError>;
}
