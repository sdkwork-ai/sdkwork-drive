//! Contracts of `DriveUploadService::complete_stored_version`.
//!
//! Validation contracts run against a lazy pool (no database needed); the
//! happy-path/idempotency/scoping tests run against the PostgreSQL fixture and
//! are skipped when `SDKWORK_DATABASE_URL` is unset.

use sdkwork_drive_workspace_service::application::upload_service::{
    CompleteStoredVersionUploadCommand, DriveUploadService,
};
use sdkwork_drive_workspace_service::infrastructure::sql::upload_session_store::SqlUploadSessionStore;
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::PgPool;

fn command(tenant_id: &str, node_id: &str) -> CompleteStoredVersionUploadCommand {
    CompleteStoredVersionUploadCommand {
        tenant_id: tenant_id.to_string(),
        space_id: "space-1".to_string(),
        node_id: node_id.to_string(),
        storage_provider_id: "provider-1".to_string(),
        bucket: "bucket-1".to_string(),
        object_key: "sdkwork-drive/v1/t/ab/tenants/tenant-1/spaces/space-1/nodes/n/cd/node-1/versions/0000000001/o1/content".to_string(),
        content_type: "application/json".to_string(),
        content_length: 42,
        checksum_sha256_hex: format!("sha256:{}", "a".repeat(64)),
        idempotency_key: "idem-version-1".to_string(),
        operator_id: "user-1".to_string(),
    }
}

fn service_without_database() -> DriveUploadService<SqlUploadSessionStore> {
    DriveUploadService::new(SqlUploadSessionStore::new(
        sdkwork_drive_test_support::lazy_postgres_test_pool(),
    ))
}

#[tokio::test]
async fn complete_stored_version_rejects_unprefixed_checksum() {
    let mut command = command("tenant-1", "node-1");
    command.checksum_sha256_hex = "a".repeat(64);
    let error = service_without_database()
        .complete_stored_version(command)
        .await
        .expect_err("checksum must be sha256-prefixed");
    assert!(matches!(error, DriveServiceError::Validation(_)));
}

#[tokio::test]
async fn complete_stored_version_rejects_dot_segment_object_key() {
    let mut command = command("tenant-1", "node-1");
    command.object_key = "objects/../content".to_string();
    let error = service_without_database()
        .complete_stored_version(command)
        .await
        .expect_err("object key segments must be normal");
    assert!(
        matches!(error, DriveServiceError::Validation(message) if message.contains("object_key"))
    );
}

#[tokio::test]
async fn complete_stored_version_rejects_negative_content_length() {
    let mut command = command("tenant-1", "node-1");
    command.content_length = -1;
    let error = service_without_database()
        .complete_stored_version(command)
        .await
        .expect_err("content length must be non-negative");
    assert!(matches!(error, DriveServiceError::Validation(_)));
}

#[tokio::test]
async fn complete_stored_version_is_idempotent_and_scoped_to_one_tenant() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_space_node_and_provider(&pool).await;

    let service = DriveUploadService::new(SqlUploadSessionStore::new(pool.clone()));
    let first = service
        .complete_stored_version(command("tenant-1", "node-1"))
        .await
        .expect("first stored version completion");
    assert_eq!(first.version_no, 1);
    assert_eq!(first.content_type, "application/json");
    assert_eq!(first.content_length, 42);

    let replay = service
        .complete_stored_version(command("tenant-1", "node-1"))
        .await
        .expect("replayed stored version completion");
    assert_eq!(replay.id, first.id, "replay must return the same version");

    let version_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_node_version WHERE tenant_id='tenant-1' AND node_id='node-1'",
    )
    .fetch_one(&pool)
    .await
    .expect("count versions");
    assert_eq!(version_count, 1, "replay must not append a version");

    let head_version_no: Option<i64> =
        sqlx::query_scalar("SELECT head_version_no FROM dr_drive_node WHERE id='node-1'")
            .fetch_one(&pool)
            .await
            .expect("read node head");
    assert_eq!(head_version_no, Some(1));

    // Every read is tenant scoped: the same node id under another tenant does not
    // resolve, so nothing is written for it.
    let foreign = service
        .complete_stored_version(command("tenant-other", "node-1"))
        .await;
    assert!(matches!(foreign, Err(DriveServiceError::NotFound(_))));
    let foreign_versions: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_node_version WHERE tenant_id='tenant-other'",
    )
    .fetch_one(&pool)
    .await
    .expect("count foreign versions");
    assert_eq!(foreign_versions, 0);
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
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
         ) VALUES ('provider-1', 's3_compatible', 'provider-1', 'https://s3.example.com',
            'us-east-1', 'bucket-1', TRUE, TRUE, 'plain:test:test', NULL, NULL,
            'active', 1, 'user-1', 'user-1')",
    )
    .execute(pool)
    .await
    .expect("seed provider");
}
