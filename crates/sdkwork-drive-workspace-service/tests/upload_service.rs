use sdkwork_drive_workspace_service::application::upload_service::{
    CreateUploadSessionCommand, DriveUploadService,
};
use sdkwork_drive_workspace_service::infrastructure::sql::upload_session_store::SqlUploadSessionStore;
use sdkwork_drive_workspace_service::DriveServiceError;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn create_upload_session_is_idempotent_for_same_key() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $8)",
    )
    .bind("space-001")
    .bind("tenant-001")
    .bind("user")
    .bind("user-001")
    .bind("personal")
    .bind("Main")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed space should succeed");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, NULL, $4, $5, 'ready', 'active', 1, $6, $7)",
    )
    .bind("node-001")
    .bind("tenant-001")
    .bind("space-001")
    .bind("file")
    .bind("v1.bin")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed node should succeed");

    seed_storage_provider(&pool, "provider-001", "bucket-001", "user-001").await;

    let store = SqlUploadSessionStore::new(pool);
    let service = DriveUploadService::new(store);

    let first = service
        .create_upload_session(CreateUploadSessionCommand {
            session_id: "upload-session-001".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: "space-001".to_string(),
            node_id: "node-001".to_string(),
            bucket: "bucket-001".to_string(),
            object_key: "objects/node-001/v1.bin".to_string(),
            storage_provider_id: "provider-001".to_string(),
            storage_upload_id: None,
            idempotency_key: "idem-abc".to_string(),
            operator_id: "user-001".to_string(),
            expires_at_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect("first upload session should be created");

    let second = service
        .create_upload_session(CreateUploadSessionCommand {
            session_id: "upload-session-002".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: "space-001".to_string(),
            node_id: "node-001".to_string(),
            bucket: "bucket-001".to_string(),
            object_key: "objects/node-001/v1.bin".to_string(),
            storage_provider_id: "provider-002".to_string(),
            storage_upload_id: Some("ignored-s3-upload-id".to_string()),
            idempotency_key: "idem-abc".to_string(),
            operator_id: "user-001".to_string(),
            expires_at_epoch_ms: 1_800_000_000_500,
        })
        .await
        .expect("idempotent call should return existing session");

    assert_eq!(first.id, second.id);
    assert_eq!(first.idempotency_key, second.idempotency_key);
}

#[tokio::test]
async fn create_upload_session_rejects_past_expiration() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let store = SqlUploadSessionStore::new(pool);
    let service = DriveUploadService::new(store);
    let past_expiration_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after UNIX epoch")
        .as_millis() as i64
        - 1_000;

    let result = service
        .create_upload_session(CreateUploadSessionCommand {
            session_id: "upload-session-expired".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: "space-001".to_string(),
            node_id: "node-001".to_string(),
            bucket: "bucket-001".to_string(),
            object_key: "objects/node-001/v1.bin".to_string(),
            storage_provider_id: "provider-001".to_string(),
            storage_upload_id: None,
            idempotency_key: "idem-expired".to_string(),
            operator_id: "user-001".to_string(),
            expires_at_epoch_ms: past_expiration_epoch_ms,
        })
        .await;

    assert!(
        matches!(result, Err(DriveServiceError::Validation(detail)) if detail.contains("expires_at_epoch_ms"))
    );
}

#[tokio::test]
async fn create_upload_session_rejects_untrimmed_bucket_and_object_key_before_database_write() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_space_and_node(&pool, "space-upload-trim", "node-upload-trim").await;

    let service = DriveUploadService::new(SqlUploadSessionStore::new(pool.clone()));
    for (index, (bucket, object_key)) in [
        (" bucket-upload-trim ", "objects/upload-trim/content"),
        ("bucket-upload-trim", " objects/upload-trim/content "),
        ("bucket/upload-trim", "objects/upload-trim/content"),
        ("bucket-upload-trim", "objects/../content"),
    ]
    .into_iter()
    .enumerate()
    {
        let err = service
            .create_upload_session(CreateUploadSessionCommand {
                session_id: format!("upload-trim-{index}"),
                tenant_id: "tenant-upload".to_string(),
                space_id: "space-upload-trim".to_string(),
                node_id: "node-upload-trim".to_string(),
                bucket: bucket.to_string(),
                object_key: object_key.to_string(),
                storage_provider_id: "provider-upload-trim".to_string(),
                storage_upload_id: None,
                idempotency_key: format!("idem-upload-trim-{index}"),
                operator_id: "user-upload".to_string(),
                expires_at_epoch_ms: future_epoch_ms(),
            })
            .await
            .expect_err("invalid storage locator should be rejected");
        assert!(
            matches!(
                err,
                sdkwork_drive_workspace_service::DriveServiceError::Validation(message)
                    if message.contains("bucket") || message.contains("object_key")
            ),
            "invalid storage locator should return validation error"
        );
    }

    let stored_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_upload_session")
        .fetch_one(&pool)
        .await
        .expect("upload session count should be readable");
    assert_eq!(stored_count, 0);
}

async fn seed_space_and_node(pool: &sqlx::PgPool, space_id: &str, node_id: &str) {
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, 'tenant-upload', 'user', 'user-upload', 'personal',
            'Upload', 'active', 1, 'user-upload', 'user-upload')",
    )
    .bind(space_id)
    .execute(pool)
    .await
    .expect("seed space should succeed");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, 'tenant-upload', $2, NULL, 'file', 'upload.bin',
            'empty', 'active', 1, 'user-upload', 'user-upload')",
    )
    .bind(node_id)
    .bind(space_id)
    .execute(pool)
    .await
    .expect("seed node should succeed");
}

async fn seed_storage_provider(
    pool: &sqlx::PgPool,
    provider_id: &str,
    bucket: &str,
    actor_id: &str,
) {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            $1, 's3_compatible', $1, 'https://s3.example.com', 'us-east-1',
            $2, 1, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, $3, $3
        )",
    )
    .bind(provider_id)
    .bind(bucket)
    .bind(actor_id)
    .execute(pool)
    .await
    .expect("seed storage provider should succeed");
}

fn future_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after UNIX epoch")
        .as_millis() as i64
        + 3_600_000
}
