use sdkwork_drive_sandbox_local::LocalSandboxDirectoryProvider;
use sdkwork_drive_workspace_service::application::sandbox_file_system_service::{
    CreateSandboxFileCommand, DeleteSandboxEntryCommand, DriveSandboxFileSystemService,
    MoveSandboxEntryCommand, ReadSandboxFileCommand, SandboxMutationContext,
    UpdateSandboxFileCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::sandbox_mutation_operation_store::SqlSandboxMutationOperationStore;
use sdkwork_drive_workspace_service::infrastructure::sql::sandbox_store::SqlSandboxStore;
use sdkwork_drive_workspace_service::ports::sandbox_principal_resolver::EffectiveSandboxPrincipal;
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::PgPool;

#[tokio::test]
async fn service_completes_file_lifecycle_idempotently_with_audit_and_concurrency() {
    let root = tempfile::tempdir().expect("sandbox root");
    std::fs::create_dir(root.path().join("src")).expect("src");
    std::fs::create_dir(root.path().join("archive")).expect("archive");
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_sandbox(&pool, root.path().to_string_lossy().as_ref(), "full").await;
    let service = service(&pool);

    let create_command = CreateSandboxFileCommand {
        tenant_id: "tenant-a".to_string(),
        sandbox_id: "sandbox-a".to_string(),
        principals: principals(),
        parent_logical_path: "src".to_string(),
        name: "main.rs".to_string(),
        bytes: b"first".to_vec(),
        mutation: mutation("create-file-001"),
    };
    let created = service
        .create_file(create_command.clone())
        .await
        .expect("create file");
    let replayed = service
        .create_file(create_command)
        .await
        .expect("create replay");
    assert_eq!(replayed, created);
    assert_eq!(
        std::fs::read(root.path().join("src/main.rs")).unwrap(),
        b"first"
    );

    let content = service
        .read_file(ReadSandboxFileCommand {
            tenant_id: "tenant-a".to_string(),
            sandbox_id: "sandbox-a".to_string(),
            principals: principals(),
            entry_id: created.id.clone(),
            logical_path: created.logical_path.clone(),
        })
        .await
        .expect("read file");
    assert_eq!(content.bytes, b"first");

    let stale = service
        .update_file(UpdateSandboxFileCommand {
            tenant_id: "tenant-a".to_string(),
            sandbox_id: "sandbox-a".to_string(),
            principals: principals(),
            entry_id: created.id.clone(),
            logical_path: created.logical_path.clone(),
            expected_revision: "stale".to_string(),
            bytes: b"wrong".to_vec(),
            mutation: mutation("update-file-stale-001"),
        })
        .await
        .expect_err("stale revision");
    assert!(matches!(stale, DriveServiceError::Conflict(_)));
    assert_eq!(
        std::fs::read(root.path().join("src/main.rs")).unwrap(),
        b"first"
    );

    let updated = service
        .update_file(UpdateSandboxFileCommand {
            tenant_id: "tenant-a".to_string(),
            sandbox_id: "sandbox-a".to_string(),
            principals: principals(),
            entry_id: created.id.clone(),
            logical_path: created.logical_path.clone(),
            expected_revision: created.revision,
            bytes: b"second".to_vec(),
            mutation: mutation("update-file-001"),
        })
        .await
        .expect("update file");
    let moved = service
        .move_entry(MoveSandboxEntryCommand {
            tenant_id: "tenant-a".to_string(),
            sandbox_id: "sandbox-a".to_string(),
            principals: principals(),
            entry_id: updated.id.clone(),
            logical_path: updated.logical_path.clone(),
            destination_parent_logical_path: "archive".to_string(),
            destination_name: "renamed.rs".to_string(),
            expected_revision: updated.revision,
            mutation: mutation("move-file-001"),
        })
        .await
        .expect("move file");
    assert_eq!(moved.logical_path, "archive/renamed.rs");
    assert_ne!(moved.id, updated.id);
    service
        .delete_entry(DeleteSandboxEntryCommand {
            tenant_id: "tenant-a".to_string(),
            sandbox_id: "sandbox-a".to_string(),
            principals: principals(),
            entry_id: moved.id,
            logical_path: moved.logical_path,
            expected_revision: moved.revision,
            recursive: false,
            mutation: mutation("delete-file-001"),
        })
        .await
        .expect("delete file");
    assert!(!root.path().join("archive/renamed.rs").exists());

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dr_drive_audit_event
         WHERE action IN (
             'drive.sandbox.file.created',
             'drive.sandbox.file.updated',
             'drive.sandbox.entry.moved',
             'drive.sandbox.entry.deleted'
         )",
    )
    .fetch_one(&pool)
    .await
    .expect("audit count");
    assert_eq!(audit_count, 4);
}

#[tokio::test]
async fn service_rejects_read_only_mutation_before_provider_side_effect() {
    let root = tempfile::tempdir().expect("sandbox root");
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_sandbox(&pool, root.path().to_string_lossy().as_ref(), "read_only").await;
    let error = service(&pool)
        .create_file(CreateSandboxFileCommand {
            tenant_id: "tenant-a".to_string(),
            sandbox_id: "sandbox-a".to_string(),
            principals: principals(),
            parent_logical_path: String::new(),
            name: "blocked.txt".to_string(),
            bytes: b"blocked".to_vec(),
            mutation: mutation("create-blocked-001"),
        })
        .await
        .expect_err("read-only mutation");
    assert!(matches!(error, DriveServiceError::PermissionDenied(_)));
    assert!(!root.path().join("blocked.txt").exists());
    let operation_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM dr_drive_sandbox_mutation_operation")
            .fetch_one(&pool)
            .await
            .expect("operation count");
    assert_eq!(operation_count, 0);
}

fn service(
    pool: &PgPool,
) -> DriveSandboxFileSystemService<
    SqlSandboxStore,
    LocalSandboxDirectoryProvider,
    SqlSandboxMutationOperationStore,
> {
    DriveSandboxFileSystemService::new(
        SqlSandboxStore::new(pool.clone()),
        LocalSandboxDirectoryProvider,
        SqlSandboxMutationOperationStore::new(pool.clone()),
    )
}

fn principals() -> Vec<EffectiveSandboxPrincipal> {
    vec![EffectiveSandboxPrincipal {
        subject_type: "user".to_string(),
        subject_id: "user-a".to_string(),
    }]
}

fn mutation(idempotency_key: &str) -> SandboxMutationContext {
    SandboxMutationContext {
        operator_id: "user-a".to_string(),
        request_id: Some("request-a".to_string()),
        trace_id: Some("trace-a".to_string()),
        idempotency_key: idempotency_key.to_string(),
    }
}

async fn seed_sandbox(pool: &PgPool, provider_root_ref: &str, access_level: &str) {
    sqlx::query(
        "INSERT INTO dr_drive_sandbox_volume (
            id, tenant_id, display_name, root_entry_id, provider_kind, provider_root_ref,
            lifecycle_status, default_access, version, created_by, updated_by
         ) VALUES ('sandbox-a', 'tenant-a', 'Sandbox A', 'root-entry-a',
                   'local_filesystem', ?, 'active', 'full', 1, 'test', 'test')",
    )
    .bind(provider_root_ref)
    .execute(pool)
    .await
    .expect("sandbox");
    sqlx::query(
        "INSERT INTO dr_drive_sandbox_grant (
            id, sandbox_id, subject_type, subject_id, access_level, granted_by
         ) VALUES ('grant-a', 'sandbox-a', 'user', 'user-a', ?, 'test')",
    )
    .bind(access_level)
    .execute(pool)
    .await
    .expect("grant");
}
