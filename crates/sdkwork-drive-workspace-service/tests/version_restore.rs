//! Contracts of `SqlDriveNodeVersionStore::restore_version`.
//!
//! The blank-identifier contract runs against a lazy pool; the lifecycle flip
//! is exercised against the PostgreSQL fixture and skipped without
//! `SDKWORK_DATABASE_URL`.

use sdkwork_drive_workspace_service::domain::node_version::{
    CreateDriveNodeVersionCommand, DriveNodeVersionChangeSource, DriveNodeVersionKind,
};
use sdkwork_drive_workspace_service::infrastructure::sql::node_version_store::SqlDriveNodeVersionStore;
use sdkwork_drive_workspace_service::ports::node_version_store::DriveNodeVersionStore;
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::PgPool;

#[tokio::test]
async fn restore_version_rejects_blank_version_id() {
    let store = SqlDriveNodeVersionStore::new(sdkwork_drive_test_support::lazy_postgres_test_pool());
    let error = store
        .restore_version("tenant-1", "node-1", " ", "user-1")
        .await
        .expect_err("blank version id");
    assert!(matches!(error, DriveServiceError::Validation(_)));
}

#[tokio::test]
async fn restore_version_reactivates_the_version_and_its_storage_object() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_node_with_storage_object(&pool).await;

    let store = SqlDriveNodeVersionStore::new(pool.clone());
    store
        .create(CreateDriveNodeVersionCommand {
            id: "ver-1".to_string(),
            tenant_id: "tenant-1".to_string(),
            space_id: "space-1".to_string(),
            node_id: "node-1".to_string(),
            version_no: 1,
            storage_object_id: Some("obj-1".to_string()),
            content_type: "text/markdown".to_string(),
            content_length: 42,
            checksum_sha256_hex: format!("sha256:{}", "a".repeat(64)),
            version_kind: DriveNodeVersionKind::Auto,
            version_label: None,
            change_source: DriveNodeVersionChangeSource::Uploader,
            change_summary: None,
            restored_from_version_id: None,
            app_id: None,
            app_resource_type: None,
            app_resource_id: None,
            scene: None,
            source: None,
            operator_id: "user-1".to_string(),
        })
        .await
        .expect("create node version");

    // Simulate a delete: both rows are soft-deleted before the restore.
    sqlx::query("UPDATE dr_drive_node_version SET lifecycle_status='deleted' WHERE id='ver-1'")
        .execute(&pool)
        .await
        .expect("soft delete version");
    sqlx::query("UPDATE dr_drive_storage_object SET lifecycle_status='deleted' WHERE id='obj-1'")
        .execute(&pool)
        .await
        .expect("soft delete object");

    let restored = store
        .restore_version("tenant-1", "node-1", "ver-1", "user-2")
        .await
        .expect("restore version");
    assert_eq!(restored.id, "ver-1");
    assert_eq!(restored.lifecycle_status, "active");
    assert_eq!(restored.updated_by, "user-2");

    let object_status: String =
        sqlx::query_scalar("SELECT lifecycle_status FROM dr_drive_storage_object WHERE id='obj-1'")
            .fetch_one(&pool)
            .await
            .expect("read object status");
    assert_eq!(object_status, "active");

    let change_events: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_change_log
         WHERE tenant_id='tenant-1' AND node_id='node-1' AND event_type='drive.file_version.restored'",
    )
    .fetch_one(&pool)
    .await
    .expect("count restore change events");
    assert_eq!(change_events, 1, "restore must record one change event");

    // A version from another tenant never resolves.
    let foreign = store
        .restore_version("tenant-other", "node-1", "ver-1", "user-2")
        .await;
    assert!(matches!(foreign, Err(DriveServiceError::NotFound(_))));
}

async fn seed_node_with_storage_object(pool: &PgPool) {
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
            content_state, head_content_type, head_content_type_group,
            head_content_length, head_version_no, head_checksum_sha256_hex,
            lifecycle_status, version, created_by, updated_by
         ) VALUES ('node-1', 'tenant-1', 'space-1', NULL, 'file', 'doc.md', 'ready',
            'text/markdown', 'document', 42, 1, $1, 'active', 1, 'user-1', 'user-1')",
    )
    .bind(format!("sha256:{}", "a".repeat(64)))
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

    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
         ) VALUES ('obj-1', 'tenant-1', 'node-1', 1, 'provider-1', 'bucket-1',
            'objects/node-1/v1.md', 'text/markdown', 42, $1, 'active', 'user-1', 'user-1')",
    )
    .bind(format!("sha256:{}", "a".repeat(64)))
    .execute(pool)
    .await
    .expect("seed storage object");
}
