use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use sdkwork_drive_workspace_service::application::sandbox_directory_service::{
    CreateSandboxDirectoryCommand, DriveSandboxDirectoryService, ListSandboxDirectoryCommand,
    SANDBOX_DIRECTORY_CREATED_AUDIT_ACTION,
};
use sdkwork_drive_workspace_service::domain::sandbox::AuthorizedSandboxMount;
use sdkwork_drive_workspace_service::domain::sandbox_directory::{
    sandbox_idempotency_key_hash, SandboxDirectoryEntry, SandboxDirectoryPage,
    SandboxDirectoryPageRequest, SandboxEntryKind, SandboxEntryName, SandboxIdempotencyKey,
    SandboxLogicalPath,
};
use sdkwork_drive_workspace_service::infrastructure::sql::sandbox_mutation_operation_store::SqlSandboxMutationOperationStore;
use sdkwork_drive_workspace_service::infrastructure::sql::sandbox_store::SqlSandboxStore;
use sdkwork_drive_workspace_service::ports::sandbox_directory_provider::DriveSandboxDirectoryProvider;
use sdkwork_drive_workspace_service::ports::sandbox_mutation_operation_store::{
    BeginSandboxMutationOperation, CompleteSandboxMutationOperation,
    DriveSandboxMutationOperationStore, SandboxMutationOperationBeginResult, SandboxMutationResult,
};
use sdkwork_drive_workspace_service::ports::sandbox_principal_resolver::EffectiveSandboxPrincipal;
use sdkwork_drive_workspace_service::DriveServiceError;
use sdkwork_utils_rust::sha256_hash;
use sqlx::{PgPool, Row};

#[derive(Clone, Default)]
struct RecordingDirectoryProvider {
    list_calls: Arc<AtomicUsize>,
    create_calls: Arc<AtomicUsize>,
    get_calls: Arc<AtomicUsize>,
    existing_directory: Arc<Mutex<Option<String>>>,
}

#[async_trait]
impl DriveSandboxDirectoryProvider for RecordingDirectoryProvider {
    fn supports(&self, provider_kind: &str) -> bool {
        provider_kind == "local_filesystem"
    }

    async fn list_children(
        &self,
        mount: &AuthorizedSandboxMount,
        parent: &SandboxLogicalPath,
        _page: &SandboxDirectoryPageRequest,
    ) -> Result<SandboxDirectoryPage, DriveServiceError> {
        self.list_calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(mount.private_root_ref(), "private-root-ref");
        Ok(SandboxDirectoryPage {
            items: vec![directory_entry(mount, parent, "existing")],
            next_cursor: None,
            has_more: false,
        })
    }

    async fn create_directory(
        &self,
        mount: &AuthorizedSandboxMount,
        parent: &SandboxLogicalPath,
        name: &SandboxEntryName,
    ) -> Result<SandboxDirectoryEntry, DriveServiceError> {
        self.create_calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(mount.private_root_ref(), "private-root-ref");
        *self.existing_directory.lock().expect("existing directory") =
            Some(name.as_str().to_string());
        Ok(directory_entry(mount, parent, name.as_str()))
    }

    async fn get_directory(
        &self,
        mount: &AuthorizedSandboxMount,
        parent: &SandboxLogicalPath,
        name: &SandboxEntryName,
    ) -> Result<Option<SandboxDirectoryEntry>, DriveServiceError> {
        self.get_calls.fetch_add(1, Ordering::SeqCst);
        let existing = self.existing_directory.lock().expect("existing directory");
        Ok(existing
            .as_deref()
            .filter(|value| *value == name.as_str())
            .map(|_| directory_entry(mount, parent, name.as_str())))
    }
}

#[tokio::test]
async fn authorized_directory_read_reaches_provider_with_private_root_after_grant_lookup() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_sandbox_and_grant(&pool, "full").await;
    let provider = RecordingDirectoryProvider::default();
    let service = DriveSandboxDirectoryService::new(
        SqlSandboxStore::new(pool.clone()),
        provider.clone(),
        SqlSandboxMutationOperationStore::new(pool),
    );

    let page = service
        .list_children(ListSandboxDirectoryCommand {
            tenant_id: "tenant-a".to_string(),
            sandbox_id: "sandbox-a".to_string(),
            principals: user_principals(),
            parent_logical_path: "projects".to_string(),
            page_size: 20,
            cursor: None,
        })
        .await
        .expect("authorized directory should be listed");

    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].logical_path, "projects/existing");
    assert_eq!(provider.list_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn read_only_grant_rejects_create_before_provider_and_audit() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_sandbox_and_grant(&pool, "read_only").await;
    let provider = RecordingDirectoryProvider::default();
    let service = DriveSandboxDirectoryService::new(
        SqlSandboxStore::new(pool.clone()),
        provider.clone(),
        SqlSandboxMutationOperationStore::new(pool.clone()),
    );

    let error = service
        .create_directory(create_command("new-directory"))
        .await
        .expect_err("read-only grants must reject directory creation");

    assert!(matches!(
        error,
        DriveServiceError::PermissionDenied(message) if message == "sandbox is read only"
    ));
    assert_eq!(provider.create_calls.load(Ordering::SeqCst), 0);
    let audit_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_audit_event")
        .fetch_one(&pool)
        .await
        .expect("audit count");
    assert_eq!(audit_count, 0);
}

#[tokio::test]
async fn ungranted_directory_read_is_rejected_before_provider_access() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_sandbox_and_grant(&pool, "full").await;
    sqlx::query("DELETE FROM dr_drive_sandbox_grant")
        .execute(&pool)
        .await
        .expect("remove fixture grant");
    let provider = RecordingDirectoryProvider::default();
    let service = DriveSandboxDirectoryService::new(
        SqlSandboxStore::new(pool.clone()),
        provider.clone(),
        SqlSandboxMutationOperationStore::new(pool),
    );

    let error = service
        .list_children(ListSandboxDirectoryCommand {
            tenant_id: "tenant-a".to_string(),
            sandbox_id: "sandbox-a".to_string(),
            principals: user_principals(),
            parent_logical_path: String::new(),
            page_size: 20,
            cursor: None,
        })
        .await
        .expect_err("an ungranted sandbox must remain inaccessible");

    assert!(matches!(error, DriveServiceError::PermissionDenied(_)));
    assert_eq!(provider.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn successful_directory_create_records_correlated_sandbox_audit() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_sandbox_and_grant(&pool, "full").await;
    let provider = RecordingDirectoryProvider::default();
    let service = DriveSandboxDirectoryService::new(
        SqlSandboxStore::new(pool.clone()),
        provider.clone(),
        SqlSandboxMutationOperationStore::new(pool.clone()),
    );

    let entry = service
        .create_directory(create_command("new-directory"))
        .await
        .expect("full grant should create directory");

    assert_eq!(entry.logical_path, "projects/new-directory");
    assert_eq!(provider.create_calls.load(Ordering::SeqCst), 1);
    let audit = sqlx::query(
        "SELECT action, resource_type, resource_id, operator_id, request_id, trace_id
         FROM dr_drive_audit_event",
    )
    .fetch_one(&pool)
    .await
    .expect("sandbox audit");
    assert_eq!(
        audit.get::<String, _>("action"),
        SANDBOX_DIRECTORY_CREATED_AUDIT_ACTION
    );
    assert_eq!(audit.get::<String, _>("resource_type"), "sandbox");
    assert_eq!(audit.get::<String, _>("resource_id"), "sandbox-a");
    assert_eq!(audit.get::<String, _>("operator_id"), "user-a");
    assert_eq!(audit.get::<String, _>("request_id"), "request-a");
    assert_eq!(audit.get::<String, _>("trace_id"), "trace-a");
}

#[tokio::test]
async fn completed_idempotent_replay_returns_the_original_entry_without_duplicate_effects() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_sandbox_and_grant(&pool, "full").await;
    let provider = RecordingDirectoryProvider::default();
    let service = DriveSandboxDirectoryService::new(
        SqlSandboxStore::new(pool.clone()),
        provider.clone(),
        SqlSandboxMutationOperationStore::new(pool.clone()),
    );

    let first = service
        .create_directory(create_command("replayed"))
        .await
        .expect("first create");
    let replay = service
        .create_directory(create_command("replayed"))
        .await
        .expect("idempotent replay");

    assert_eq!(replay, first);
    assert_eq!(provider.create_calls.load(Ordering::SeqCst), 1);
    let audit_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_audit_event")
        .fetch_one(&pool)
        .await
        .expect("audit count");
    assert_eq!(audit_count, 1);
    let stored_key_hash: String =
        sqlx::query_scalar("SELECT idempotency_key_hash FROM dr_drive_sandbox_mutation_operation")
            .fetch_one(&pool)
            .await
            .expect("stored idempotency key hash");
    assert_eq!(stored_key_hash, create_directory_key_hash());
    assert_ne!(stored_key_hash, "request-key-a");
}

#[tokio::test]
async fn idempotency_key_reuse_with_a_different_request_conflicts_before_provider() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_sandbox_and_grant(&pool, "full").await;
    let provider = RecordingDirectoryProvider::default();
    let service = DriveSandboxDirectoryService::new(
        SqlSandboxStore::new(pool.clone()),
        provider.clone(),
        SqlSandboxMutationOperationStore::new(pool.clone()),
    );

    service
        .create_directory(create_command("first-name"))
        .await
        .expect("first create");
    let error = service
        .create_directory(create_command("different-name"))
        .await
        .expect_err("different request fingerprint must conflict");

    assert!(matches!(
        error,
        DriveServiceError::Conflict(message)
            if message.contains("idempotency key belongs to a different")
    ));
    assert_eq!(provider.create_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn pending_operation_recovers_a_directory_created_before_database_completion() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_sandbox_and_grant(&pool, "full").await;
    let provider = RecordingDirectoryProvider::default();
    *provider
        .existing_directory
        .lock()
        .expect("existing directory") = Some("recovered".to_string());
    let idempotency_key_hash = create_directory_key_hash();
    let fingerprint = sha256_hash(b"sandbox-directory-v1\0projects\0recovered");
    sqlx::query(
        "INSERT INTO dr_drive_sandbox_mutation_operation (
            id, tenant_id, sandbox_id, actor_id, idempotency_key_hash, request_fingerprint,
            mutation_kind, parent_logical_path, entry_name, operation_status, lease_token,
            lease_expires_at_ms
         ) VALUES (9001, 'tenant-a', 'sandbox-a', 'user-a', $1, $2,
                   'create_directory', 'projects', 'recovered', 'pending', 'crashed-lease', 0)",
    )
    .bind(idempotency_key_hash)
    .bind(fingerprint)
    .execute(&pool)
    .await
    .expect("pending operation fixture");
    let service = DriveSandboxDirectoryService::new(
        SqlSandboxStore::new(pool.clone()),
        provider.clone(),
        SqlSandboxMutationOperationStore::new(pool.clone()),
    );

    let entry = service
        .create_directory(create_command("recovered"))
        .await
        .expect("pending operation recovery");

    assert_eq!(entry.logical_path, "projects/recovered");
    assert_eq!(provider.create_calls.load(Ordering::SeqCst), 0);
    let status: String = sqlx::query_scalar(
        "SELECT operation_status FROM dr_drive_sandbox_mutation_operation WHERE id=9001",
    )
    .fetch_one(&pool)
    .await
    .expect("operation status");
    assert_eq!(status, "completed");
    let audit_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_audit_event")
        .fetch_one(&pool)
        .await
        .expect("audit count");
    assert_eq!(audit_count, 1);
}

#[tokio::test]
async fn expired_pending_lease_recovers_a_crash_before_provider_creation() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_sandbox_and_grant(&pool, "full").await;
    let provider = RecordingDirectoryProvider::default();
    let idempotency_key_hash = create_directory_key_hash();
    let fingerprint = sha256_hash(b"sandbox-directory-v1\0projects\0resumed");
    sqlx::query(
        "INSERT INTO dr_drive_sandbox_mutation_operation (
            id, tenant_id, sandbox_id, actor_id, idempotency_key_hash, request_fingerprint,
            mutation_kind, parent_logical_path, entry_name, operation_status, lease_token,
            lease_expires_at_ms
         ) VALUES (9002, 'tenant-a', 'sandbox-a', 'user-a', $1, $2,
                   'create_directory', 'projects', 'resumed', 'pending', 'expired-lease', 0)",
    )
    .bind(idempotency_key_hash)
    .bind(fingerprint)
    .execute(&pool)
    .await
    .expect("expired pending operation fixture");
    let service = DriveSandboxDirectoryService::new(
        SqlSandboxStore::new(pool.clone()),
        provider.clone(),
        SqlSandboxMutationOperationStore::new(pool.clone()),
    );

    let entry = service
        .create_directory(create_command("resumed"))
        .await
        .expect("expired pending operation should be claimed");

    assert_eq!(entry.logical_path, "projects/resumed");
    assert_eq!(provider.create_calls.load(Ordering::SeqCst), 1);
    let status: String = sqlx::query_scalar(
        "SELECT operation_status FROM dr_drive_sandbox_mutation_operation WHERE id=9002",
    )
    .fetch_one(&pool)
    .await
    .expect("operation status");
    assert_eq!(status, "completed");
}

#[tokio::test]
async fn active_pending_lease_does_not_treat_a_preexisting_target_as_recovery() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_sandbox_and_grant(&pool, "full").await;
    let provider = RecordingDirectoryProvider::default();
    *provider
        .existing_directory
        .lock()
        .expect("existing directory") = Some("preexisting".to_string());
    let idempotency_key_hash = create_directory_key_hash();
    let fingerprint = sha256_hash(b"sandbox-directory-v1\0projects\0preexisting");
    sqlx::query(
        "INSERT INTO dr_drive_sandbox_mutation_operation (
            id, tenant_id, sandbox_id, actor_id, idempotency_key_hash, request_fingerprint,
            mutation_kind, parent_logical_path, entry_name, operation_status, lease_token,
            lease_expires_at_ms
         ) VALUES (9003, 'tenant-a', 'sandbox-a', 'user-a', $1, $2,
                   'create_directory', 'projects', 'preexisting', 'pending', 'active-lease',
                   9223372036854775807)",
    )
    .bind(idempotency_key_hash)
    .bind(fingerprint)
    .execute(&pool)
    .await
    .expect("active pending operation fixture");
    let service = DriveSandboxDirectoryService::new(
        SqlSandboxStore::new(pool.clone()),
        provider.clone(),
        SqlSandboxMutationOperationStore::new(pool),
    );

    let error = service
        .create_directory(create_command("preexisting"))
        .await
        .expect_err("active operation must remain in progress");

    assert!(matches!(
        error,
        DriveServiceError::Conflict(message) if message.contains("in progress")
    ));
    assert_eq!(provider.get_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn result_completion_rolls_back_when_atomic_audit_persistence_fails() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_sandbox_and_grant(&pool, "full").await;
    let store = SqlSandboxMutationOperationStore::new(pool.clone());
    let begin = store
        .begin_or_load(&BeginSandboxMutationOperation {
            tenant_id: "tenant-a".to_string(),
            sandbox_id: "sandbox-a".to_string(),
            actor_id: "user-a".to_string(),
            idempotency_key_hash: sha256_hash(b"atomic-key-a"),
            request_fingerprint: "fingerprint-a".to_string(),
            mutation_kind: "create_directory".to_string(),
            parent_logical_path: "projects".to_string(),
            entry_name: "atomic".to_string(),
            lease_token: "lease-a".to_string(),
            lease_expires_at_ms: i64::MAX,
        })
        .await
        .expect("begin operation");
    let (operation_id, lease_token) = match begin {
        SandboxMutationOperationBeginResult::Started {
            operation_id,
            lease_token,
        } => (operation_id, lease_token),
        other => panic!("expected started operation, got {other:?}"),
    };
    sqlx::query("DROP TABLE dr_drive_audit_event")
        .execute(&pool)
        .await
        .expect("drop audit table for rollback injection");
    let entry = SandboxDirectoryEntry {
        id: "entry-atomic".to_string(),
        sandbox_id: "sandbox-a".to_string(),
        parent_id: "root-entry-a".to_string(),
        parent_logical_path: "projects".to_string(),
        name: "atomic".to_string(),
        kind: SandboxEntryKind::Directory,
        logical_path: "projects/atomic".to_string(),
        revision: "revision-atomic".to_string(),
    };

    store
        .complete_with_audit(&CompleteSandboxMutationOperation {
            operation_id,
            tenant_id: "tenant-a".to_string(),
            lease_token: Some(lease_token),
            result: SandboxMutationResult::Entry(entry),
            audit_action: SANDBOX_DIRECTORY_CREATED_AUDIT_ACTION.to_string(),
            audit_resource_type: "sandbox".to_string(),
            audit_resource_id: "sandbox-a".to_string(),
            operator_id: "user-a".to_string(),
            request_id: Some("request-a".to_string()),
            trace_id: Some("trace-a".to_string()),
        })
        .await
        .expect_err("audit failure must fail atomic completion");

    let row = sqlx::query(
        "SELECT operation_status, result_entry_id
         FROM dr_drive_sandbox_mutation_operation WHERE id=$1",
    )
    .bind(operation_id)
    .fetch_one(&pool)
    .await
    .expect("operation after rollback");
    assert_eq!(row.get::<String, _>("operation_status"), "pending");
    assert_eq!(row.get::<Option<String>, _>("result_entry_id"), None);
}

fn directory_entry(
    mount: &AuthorizedSandboxMount,
    parent: &SandboxLogicalPath,
    name: &str,
) -> SandboxDirectoryEntry {
    let logical_path = if parent.as_str().is_empty() {
        name.to_string()
    } else {
        format!("{}/{}", parent.as_str(), name)
    };
    SandboxDirectoryEntry {
        id: format!("entry-{name}"),
        sandbox_id: mount.sandbox_id().to_string(),
        parent_id: mount.root_entry_id().to_string(),
        parent_logical_path: parent.as_str().to_string(),
        name: name.to_string(),
        kind: SandboxEntryKind::Directory,
        logical_path,
        revision: "revision-1".to_string(),
    }
}

fn create_command(name: &str) -> CreateSandboxDirectoryCommand {
    CreateSandboxDirectoryCommand {
        tenant_id: "tenant-a".to_string(),
        sandbox_id: "sandbox-a".to_string(),
        principals: user_principals(),
        parent_logical_path: "projects".to_string(),
        name: name.to_string(),
        operator_id: "user-a".to_string(),
        request_id: Some("request-a".to_string()),
        trace_id: Some("trace-a".to_string()),
        idempotency_key: "request-key-a".to_string(),
    }
}

fn create_directory_key_hash() -> String {
    let key = SandboxIdempotencyKey::parse("request-key-a").expect("idempotency key");
    sandbox_idempotency_key_hash("create_directory", &key)
}

fn user_principals() -> Vec<EffectiveSandboxPrincipal> {
    vec![EffectiveSandboxPrincipal {
        subject_type: "user".to_string(),
        subject_id: "user-a".to_string(),
    }]
}

async fn insert_sandbox_and_grant(pool: &PgPool, access_level: &str) {
    sqlx::query(
        "INSERT INTO dr_drive_sandbox_volume (
            id, tenant_id, display_name, root_entry_id, provider_kind, provider_root_ref,
            lifecycle_status, default_access, version, created_by, updated_by
         ) VALUES ('sandbox-a', 'tenant-a', 'Sandbox A', 'root-entry-a',
                   'local_filesystem', 'private-root-ref', 'active', 'full', 1, 'test', 'test')",
    )
    .execute(pool)
    .await
    .expect("sandbox fixture");
    sqlx::query(
        "INSERT INTO dr_drive_sandbox_grant (
            id, sandbox_id, subject_type, subject_id, access_level, granted_by
         ) VALUES ('grant-a', 'sandbox-a', 'user', 'user-a', $1, 'test')",
    )
    .bind(access_level)
    .execute(pool)
    .await
    .expect("grant fixture");
}
