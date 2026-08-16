use async_trait::async_trait;
use sdkwork_drive_workspace_service::application::download_service::{
    CreateDownloadUrlCommand, DriveDownloadService, ResolveDownloadTokenCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::storage_object_store::SqlStorageObjectStore;
use sdkwork_drive_workspace_service::ports::storage_object_store::{
    DownloadSignCommand, DriveDownloadSigner, SignedDownloadPayload,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct FakeDownloadSigner;

#[async_trait]
impl DriveDownloadSigner for FakeDownloadSigner {
    async fn sign_download(
        &self,
        command: DownloadSignCommand,
    ) -> Result<SignedDownloadPayload, sdkwork_drive_workspace_service::DriveServiceError> {
        Ok(SignedDownloadPayload {
            method: "GET".to_string(),
            raw_url: format!(
                "https://s3.example.com/{}/{}/{}?X-Amz-Signature=fake",
                command.storage_provider_id, command.bucket, command.object_key
            ),
            headers: Default::default(),
            expires_at_epoch_ms: command.expires_at_epoch_ms,
        })
    }
}

async fn seed_storage_provider(pool: &sqlx::PgPool, provider_id: &str, bucket: &str) {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            $1, 's3_compatible', $1, 'https://s3.example.com', 'us-east-1',
            $2, 1, 1, 'plain:test-access:test-secret', NULL, NULL,
            'active', 1, 'test', 'test'
        )",
    )
    .bind(provider_id)
    .bind(bucket)
    .execute(pool)
    .await
    .expect("seed storage provider should succeed");
}

#[tokio::test]
async fn download_url_is_short_lived_and_hides_object_key() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_storage_provider(&pool, "provider-001", "bucket-001").await;

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

    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'active', $11, $12)",
    )
    .bind("obj-001")
    .bind("tenant-001")
    .bind("node-001")
    .bind(1_i64)
    .bind("provider-001")
    .bind("bucket-001")
    .bind("objects/node-001/v1.bin")
    .bind("application/octet-stream")
    .bind(128_i64)
    .bind("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed storage object should succeed");

    let store = SqlStorageObjectStore::new(pool);
    let signer = FakeDownloadSigner;
    let service = DriveDownloadService::new(store, signer);

    let now_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_millis() as i64;

    let response = service
        .create_download_url(CreateDownloadUrlCommand {
            tenant_id: "tenant-001".to_string(),
            node_id: "node-001".to_string(),
            requested_ttl_seconds: 300,
            request_base_url: "https://drive.example.com".to_string(),
        })
        .await
        .expect("download URL should be generated");

    assert!(
        response.download_url.contains("/download_tokens/"),
        "download url should be opaque"
    );
    assert!(
        !response.download_url.contains("objects/node-001/v1.bin"),
        "download url should not leak object key"
    );
    assert!(
        response.expires_at_epoch_ms <= now_epoch_ms + 300_000 + 5_000,
        "ttl must be short lived"
    );
}

#[tokio::test]
async fn create_download_url_rejects_ttl_outside_contract() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_storage_provider(&pool, "provider-trashed", "bucket-trashed").await;

    let store = SqlStorageObjectStore::new(pool);
    let signer = FakeDownloadSigner;
    let service = DriveDownloadService::new(store, signer);

    let error = service
        .create_download_url(CreateDownloadUrlCommand {
            tenant_id: "tenant-001".to_string(),
            node_id: "node-001".to_string(),
            requested_ttl_seconds: 0,
            request_base_url: "https://drive.example.com".to_string(),
        })
        .await
        .expect_err("invalid TTL should be rejected");

    assert!(
        matches!(
            error,
            sdkwork_drive_workspace_service::DriveServiceError::Validation(ref message)
                if message.contains("requested_ttl_seconds")
        ),
        "invalid TTL should return a validation error, got {error:?}"
    );
}

#[tokio::test]
async fn create_download_url_rejects_trashed_node_even_when_storage_object_is_active() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_storage_provider(&pool, "provider-trashed", "bucket-trashed").await;

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $8)",
    )
    .bind("space-trashed")
    .bind("tenant-trashed")
    .bind("user")
    .bind("user-trashed")
    .bind("personal")
    .bind("Trashed")
    .bind("user-trashed")
    .bind("user-trashed")
    .execute(&pool)
    .await
    .expect("seed space should succeed");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, NULL, $4, $5, 'ready', 'trashed', 1, $6, $7)",
    )
    .bind("node-trashed")
    .bind("tenant-trashed")
    .bind("space-trashed")
    .bind("file")
    .bind("trashed.bin")
    .bind("user-trashed")
    .bind("user-trashed")
    .execute(&pool)
    .await
    .expect("seed trashed node should succeed");

    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'active', $11, $12)",
    )
    .bind("obj-trashed")
    .bind("tenant-trashed")
    .bind("node-trashed")
    .bind(1_i64)
    .bind("provider-trashed")
    .bind("bucket-trashed")
    .bind("objects/node-trashed/v1.bin")
    .bind("application/octet-stream")
    .bind(128_i64)
    .bind("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    .bind("user-trashed")
    .bind("user-trashed")
    .execute(&pool)
    .await
    .expect("seed storage object should succeed");

    let store = SqlStorageObjectStore::new(pool);
    let signer = FakeDownloadSigner;
    let service = DriveDownloadService::new(store, signer);

    let error = service
        .create_download_url(CreateDownloadUrlCommand {
            tenant_id: "tenant-trashed".to_string(),
            node_id: "node-trashed".to_string(),
            requested_ttl_seconds: 120,
            request_base_url: "https://drive.example.com".to_string(),
        })
        .await
        .expect_err("trashed node should not get a download URL");

    assert!(
        matches!(
            error,
            sdkwork_drive_workspace_service::DriveServiceError::NotFound(ref message)
                if message.contains("storage object")
        ),
        "inactive node should be hidden from download URL creation, got {error:?}"
    );
}

#[tokio::test]
async fn resolve_download_token_restores_node_and_signs_source_url() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_storage_provider(&pool, "provider-001", "bucket-001").await;

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

    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'active', $11, $12)",
    )
    .bind("obj-001")
    .bind("tenant-001")
    .bind("node-001")
    .bind(1_i64)
    .bind("provider-001")
    .bind("bucket-001")
    .bind("objects/node-001/v1.bin")
    .bind("application/octet-stream")
    .bind(128_i64)
    .bind("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed storage object should succeed");

    let store = SqlStorageObjectStore::new(pool);
    let signer = FakeDownloadSigner;
    let service = DriveDownloadService::new(store, signer);

    let created = service
        .create_download_url(CreateDownloadUrlCommand {
            tenant_id: "tenant-001".to_string(),
            node_id: "node-001".to_string(),
            requested_ttl_seconds: 120,
            request_base_url: "https://drive.example.com/app/v3/api/drive".to_string(),
        })
        .await
        .expect("download URL should be generated");

    let token = created
        .download_url
        .rsplit('/')
        .next()
        .expect("token should be present in path")
        .to_string();

    let resolved = service
        .resolve_download_token(ResolveDownloadTokenCommand {
            tenant_id: "tenant-001".to_string(),
            token,
        })
        .await
        .expect("token should be resolved");

    assert_eq!(resolved.node_id, "node-001");
    assert!(resolved
        .signed_source_url
        .contains("objects/node-001/v1.bin"));
}
