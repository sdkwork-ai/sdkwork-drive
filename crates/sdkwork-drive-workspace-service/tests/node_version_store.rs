use sdkwork_drive_workspace_service::domain::node_version::{
    CreateDriveNodeVersionCommand, DriveNodeVersionChangeSource, DriveNodeVersionKind,
};
use sdkwork_drive_workspace_service::infrastructure::sql::node_version_store::SqlDriveNodeVersionStore;
use sdkwork_drive_workspace_service::ports::node_version_store::DriveNodeVersionStore;

#[tokio::test]
async fn sql_node_version_store_creates_logical_version_from_storage_object() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_file_object(&pool).await;

    let store = SqlDriveNodeVersionStore::new(pool.clone());
    let created = store
        .create(CreateDriveNodeVersionCommand {
            id: "ver-node-001-v1".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: "space-001".to_string(),
            node_id: "node-001".to_string(),
            version_no: 1,
            storage_object_id: Some("obj-001".to_string()),
            content_type: "text/markdown".to_string(),
            content_length: 42,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
            version_kind: DriveNodeVersionKind::Auto,
            version_label: Some("Initial draft".to_string()),
            change_source: DriveNodeVersionChangeSource::Uploader,
            change_summary: Some("Created from upload completion".to_string()),
            restored_from_version_id: None,
            app_id: Some("sdkwork-notes".to_string()),
            app_resource_type: Some("page".to_string()),
            app_resource_id: Some("page-001".to_string()),
            scene: Some("notes_page".to_string()),
            source: Some("editor".to_string()),
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("logical node version should be created");

    assert_eq!(created.id, "ver-node-001-v1");
    assert_eq!(created.storage_object_id.as_deref(), Some("obj-001"));
    assert_eq!(created.version_kind, DriveNodeVersionKind::Auto);
    assert_eq!(
        created.change_source,
        DriveNodeVersionChangeSource::Uploader
    );
    assert_eq!(created.app_id.as_deref(), Some("sdkwork-notes"));

    let listed = store
        .list_by_node("tenant-001", "node-001", 10, 0)
        .await
        .expect("logical node versions should be listed");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, "ver-node-001-v1");
    assert_eq!(listed[0].version_no, 1);
}

async fn seed_file_object(pool: &sqlx::PgPool) {
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
    .bind("provider-001")
    .bind("bucket-001")
    .execute(pool)
    .await
    .expect("seed storage provider should succeed");

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, 'user', 'user-001', 'personal', 'Main', 'active', 1, 'user-001', 'user-001')",
    )
    .bind("space-001")
    .bind("tenant-001")
    .execute(pool)
    .await
    .expect("seed space should succeed");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, NULL, 'file', 'page.md', 'ready', 'active', 1, 'user-001', 'user-001')",
    )
    .bind("node-001")
    .bind("tenant-001")
    .bind("space-001")
    .execute(pool)
    .await
    .expect("seed node should succeed");

    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES ($1, $2, $3, 1, $4, $5, $6, $7, 42, $8, 'active', 'user-001', 'user-001')",
    )
    .bind("obj-001")
    .bind("tenant-001")
    .bind("node-001")
    .bind("provider-001")
    .bind("bucket-001")
    .bind("objects/node-001/v1.md")
    .bind("text/markdown")
    .bind("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    .execute(pool)
    .await
    .expect("seed storage object should succeed");
}
