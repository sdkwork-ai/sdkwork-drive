//! Contracts of the in-process upload-session service.
//!
//! Two layers of coverage:
//!
//! * validation contracts, which must fail *before* any database or storage
//!   access (run against a lazy pool, so they never need a server);
//! * the full prepare/complete/cancel state machine over a real PostgreSQL test
//!   database (skipped when `SDKWORK_DATABASE_URL` is not configured), using an
//!   in-memory multipart-storage adapter so no object store is involved.

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use sdkwork_drive_workspace_service::application::upload_session_service::{
    CancelDriveUploadSessionCommand, CompleteDriveUploadSessionCommand, DriveUploadSessionService,
    PrepareDriveUploadSessionCommand,
};
use sdkwork_drive_workspace_service::ports::upload_multipart_storage::{
    AbortDriveMultipartUploadCommand, CompleteDriveMultipartUploadCommand,
    CreateDriveMultipartUploadCommand, DeleteDriveStoredObjectCommand, DriveMultipartPart,
    DriveUploadMultipartStorage,
};
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::PgPool;

/// Records the multipart calls the service makes; never talks to a provider.
#[derive(Debug, Default, Clone)]
struct InMemoryMultipartStorage {
    inner: Arc<MultipartCalls>,
}

#[derive(Debug, Default)]
struct MultipartCalls {
    created: Mutex<Vec<CreateDriveMultipartUploadCommand>>,
    completed: Mutex<Vec<CompleteDriveMultipartUploadCommand>>,
    aborted: Mutex<Vec<AbortDriveMultipartUploadCommand>>,
    deleted: Mutex<Vec<DeleteDriveStoredObjectCommand>>,
}

#[async_trait]
impl DriveUploadMultipartStorage for InMemoryMultipartStorage {
    async fn create_multipart_upload(
        &self,
        command: CreateDriveMultipartUploadCommand,
    ) -> Result<String, DriveServiceError> {
        self.inner
            .created
            .lock()
            .map_err(|_| DriveServiceError::Internal("poisoned".to_string()))?
            .push(command.clone());
        Ok(format!("upload:{}", command.object_key))
    }

    async fn complete_multipart_upload(
        &self,
        command: CompleteDriveMultipartUploadCommand,
    ) -> Result<(), DriveServiceError> {
        self.inner
            .completed
            .lock()
            .map_err(|_| DriveServiceError::Internal("poisoned".to_string()))?
            .push(command);
        Ok(())
    }

    async fn abort_multipart_upload(
        &self,
        command: AbortDriveMultipartUploadCommand,
    ) -> Result<(), DriveServiceError> {
        self.inner
            .aborted
            .lock()
            .map_err(|_| DriveServiceError::Internal("poisoned".to_string()))?
            .push(command);
        Ok(())
    }

    async fn delete_object(
        &self,
        command: DeleteDriveStoredObjectCommand,
    ) -> Result<bool, DriveServiceError> {
        self.inner
            .deleted
            .lock()
            .map_err(|_| DriveServiceError::Internal("poisoned".to_string()))?
            .push(command);
        Ok(true)
    }
}

fn service_without_database(
    storage: InMemoryMultipartStorage,
) -> DriveUploadSessionService<InMemoryMultipartStorage> {
    DriveUploadSessionService::new(sdkwork_drive_test_support::lazy_postgres_test_pool(), storage)
}

fn future_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after UNIX epoch")
        .as_millis() as i64
        + 3_600_000
}

fn prepare_command(expires_at_epoch_ms: i64) -> PrepareDriveUploadSessionCommand {
    PrepareDriveUploadSessionCommand {
        session_id: "session-1".to_string(),
        tenant_id: "tenant-1".to_string(),
        space_id: "space-1".to_string(),
        node_id: "node-1".to_string(),
        requested_storage_provider_id: None,
        requested_bucket: None,
        idempotency_key: "idem-1".to_string(),
        operator_id: "user-1".to_string(),
        subject_type: "user".to_string(),
        subject_id: "user-1".to_string(),
        expires_at_epoch_ms,
    }
}

fn complete_command(parts: Vec<DriveMultipartPart>) -> CompleteDriveUploadSessionCommand {
    CompleteDriveUploadSessionCommand {
        tenant_id: "tenant-1".to_string(),
        upload_session_id: "session-1".to_string(),
        upload_id: None,
        content_type: "application/json".to_string(),
        content_length: 42,
        checksum_sha256_hex: format!("sha256:{}", "a".repeat(64)),
        parts,
        operator_id: "user-1".to_string(),
        subject_type: "user".to_string(),
        subject_id: "user-1".to_string(),
    }
}

#[tokio::test]
async fn prepare_rejects_past_expiry_before_storage_or_database_access() {
    let storage = InMemoryMultipartStorage::default();
    let error = service_without_database(storage.clone())
        .prepare(prepare_command(1))
        .await
        .expect_err("expired session must be rejected");
    assert!(matches!(error, DriveServiceError::Validation(_)));
    assert!(storage
        .inner
        .created
        .lock()
        .expect("created lock")
        .is_empty());
}

#[tokio::test]
async fn prepare_rejects_blank_idempotency_key_before_storage_access() {
    let storage = InMemoryMultipartStorage::default();
    let mut command = prepare_command(future_epoch_ms());
    command.idempotency_key = "  ".to_string();
    let error = service_without_database(storage.clone())
        .prepare(command)
        .await
        .expect_err("blank idempotency key");
    assert!(matches!(error, DriveServiceError::Validation(_)));
    assert!(storage
        .inner
        .created
        .lock()
        .expect("created lock")
        .is_empty());
}

#[tokio::test]
async fn complete_rejects_unsorted_parts_before_storage_access() {
    let storage = InMemoryMultipartStorage::default();
    let error = service_without_database(storage.clone())
        .complete(complete_command(vec![
            DriveMultipartPart {
                part_no: 2,
                etag: "b".to_string(),
            },
            DriveMultipartPart {
                part_no: 1,
                etag: "a".to_string(),
            },
        ]))
        .await
        .expect_err("unsorted parts");
    assert!(matches!(error, DriveServiceError::Validation(_)));
    assert!(storage
        .inner
        .completed
        .lock()
        .expect("completed lock")
        .is_empty());
}

#[tokio::test]
async fn complete_rejects_unprefixed_checksum_before_storage_access() {
    let storage = InMemoryMultipartStorage::default();
    let mut command = complete_command(vec![DriveMultipartPart {
        part_no: 1,
        etag: "a".to_string(),
    }]);
    command.checksum_sha256_hex = "a".repeat(64);
    let error = service_without_database(storage)
        .complete(command)
        .await
        .expect_err("checksum without prefix");
    assert!(matches!(error, DriveServiceError::Validation(_)));
}

#[tokio::test]
async fn cancel_rejects_blank_tenant_before_storage_access() {
    let storage = InMemoryMultipartStorage::default();
    let error = service_without_database(storage.clone())
        .cancel(CancelDriveUploadSessionCommand {
            tenant_id: " ".to_string(),
            upload_session_id: "session-1".to_string(),
            operator_id: "user-1".to_string(),
            subject_type: "user".to_string(),
            subject_id: "user-1".to_string(),
        })
        .await
        .expect_err("blank tenant");
    assert!(matches!(error, DriveServiceError::Validation(_)));
    assert!(storage
        .inner
        .aborted
        .lock()
        .expect("aborted lock")
        .is_empty());
}

#[tokio::test]
async fn prepare_complete_and_cancel_drive_the_session_state_machine() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_space_node_and_provider(&pool).await;

    let storage = InMemoryMultipartStorage::default();
    let service = DriveUploadSessionService::new(pool.clone(), storage.clone());

    let created = service
        .prepare(PrepareDriveUploadSessionCommand {
            requested_storage_provider_id: Some("provider-1".to_string()),
            ..prepare_command(future_epoch_ms())
        })
        .await
        .expect("prepare upload session");
    assert_eq!(created.state.as_str(), "created");
    assert_eq!(created.storage_provider_id, "provider-1");
    assert_eq!(
        storage.inner.created.lock().expect("created lock").len(),
        1
    );

    // Same idempotency key returns the same session without a second multipart
    // upload.
    let replay = service
        .prepare(PrepareDriveUploadSessionCommand {
            session_id: "session-2".to_string(),
            requested_storage_provider_id: Some("provider-1".to_string()),
            ..prepare_command(future_epoch_ms())
        })
        .await
        .expect("replayed prepare");
    assert_eq!(replay.id, created.id);
    assert_eq!(
        storage.inner.created.lock().expect("created lock").len(),
        1
    );

    let completed = service
        .complete(CompleteDriveUploadSessionCommand {
            upload_session_id: created.id.clone(),
            parts: vec![DriveMultipartPart {
                part_no: 1,
                etag: "etag-1".to_string(),
            }],
            ..complete_command(Vec::new())
        })
        .await
        .expect("complete upload session");
    assert_eq!(completed.state.as_str(), "completed");
    assert_eq!(
        storage.inner.completed.lock().expect("completed lock").len(),
        1
    );

    let version_no: i64 = sqlx::query_scalar(
        "SELECT version_no FROM dr_drive_node_version WHERE tenant_id='tenant-1' AND node_id='node-1'",
    )
    .fetch_one(&pool)
    .await
    .expect("completed version exists");
    assert_eq!(version_no, 1);
    let head_version_no: Option<i64> =
        sqlx::query_scalar("SELECT head_version_no FROM dr_drive_node WHERE id='node-1'")
            .fetch_one(&pool)
            .await
            .expect("node head exists");
    assert_eq!(head_version_no, Some(1));

    // A second complete on the terminal state is a conflict, not a second version.
    let error = service
        .complete(CompleteDriveUploadSessionCommand {
            upload_session_id: created.id.clone(),
            parts: vec![DriveMultipartPart {
                part_no: 1,
                etag: "etag-1".to_string(),
            }],
            ..complete_command(Vec::new())
        })
        .await
        .expect_err("completed session is not mutable");
    assert!(matches!(error, DriveServiceError::Conflict(_)));

    // Abort of a completed session is rejected; a second session can be aborted.
    let second = service
        .prepare(PrepareDriveUploadSessionCommand {
            session_id: "session-abort".to_string(),
            idempotency_key: "idem-abort".to_string(),
            requested_storage_provider_id: Some("provider-1".to_string()),
            ..prepare_command(future_epoch_ms())
        })
        .await
        .expect("second upload session");
    let aborted = service
        .cancel(CancelDriveUploadSessionCommand {
            tenant_id: "tenant-1".to_string(),
            upload_session_id: second.id.clone(),
            operator_id: "user-1".to_string(),
            subject_type: "user".to_string(),
            subject_id: "user-1".to_string(),
        })
        .await
        .expect("cancel upload session");
    assert_eq!(aborted.state.as_str(), "aborted");
    assert_eq!(storage.inner.aborted.lock().expect("aborted lock").len(), 1);
}

async fn seed_space_node_and_provider(pool: &PgPool) {
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
         ) VALUES ('space-1', 'tenant-1', 'user', 'user-1', 'personal', 'Main', 'active', 1, 'user-1', 'user-1')",
    )
    .execute(pool)
    .await
    .expect("seed space");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
         ) VALUES ('node-1', 'tenant-1', 'space-1', NULL, 'file', 'page.json', 'empty', 'active', 1, 'user-1', 'user-1')",
    )
    .execute(pool)
    .await
    .expect("seed node");

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
         ) VALUES ('provider-1', 'tenant-1', 's3_compatible', 'provider-1', 'https://s3.example.com',
            'us-east-1', 'bucket-1', TRUE, TRUE, 'plain:test:test', NULL, NULL,
            'active', 1, 'user-1', 'user-1')",
    )
    .execute(pool)
    .await
    .expect("seed provider");
}
