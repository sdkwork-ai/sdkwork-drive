use chrono::{Duration, SecondsFormat, Utc};
use sdkwork_drive_install_worker::maintenance::website_publishing_cleanup::cleanup_website_publishing;
use sdkwork_drive_workspace_service::application::space_service::{
    CreateSpaceCommand, DriveSpaceService,
};
use sdkwork_drive_workspace_service::application::website_sync_service::{
    CreateWebsiteSyncCommand, DriveWebsiteSyncService,
};
use sdkwork_drive_workspace_service::domain::space::DriveSpaceType;
use sdkwork_drive_workspace_service::domain::website_sync::{
    validate_website_sync_tree, DriveWebsiteSyncTreeEntry,
};
use sdkwork_drive_workspace_service::infrastructure::sql::space_store::SqlSpaceStore;
use sdkwork_drive_workspace_service::infrastructure::sql::website_sync_store::SqlWebsiteSyncStore;

#[tokio::test]
async fn cleanup_expires_sync_deletes_provider_object_and_retires_staging_tree() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    DriveSpaceService::new(SqlSpaceStore::new(pool.clone()))
        .create_space(CreateSpaceCommand {
            id: "space-cleanup".to_string(),
            tenant_id: "tenant-cleanup".to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-cleanup".to_string(),
            display_name: "Cleanup Website".to_string(),
            space_type: DriveSpaceType::Website,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-cleanup".to_string(),
        })
        .await
        .expect("cleanup Website Space should be created");
    let root_uuid: String = sqlx::query_scalar(
        "SELECT uuid FROM dr_drive_website_root
         WHERE tenant_id='tenant-cleanup' AND space_id='space-cleanup' AND root_key='default'",
    )
    .fetch_one(&pool)
    .await
    .expect("cleanup WebsiteRoot should exist");
    let entry = DriveWebsiteSyncTreeEntry {
        relative_path: "index.html".to_string(),
        depth: 1,
        node_type: "file".to_string(),
        content_state: "ready".to_string(),
        content_length: Some(18),
        checksum_sha256_hex: Some(format!("sha256:{}", "a".repeat(64))),
        shortcut_target_node_id: None,
    };
    let manifest = validate_website_sync_tree(std::slice::from_ref(&entry))
        .expect("cleanup manifest should be valid");
    let sync = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(pool.clone()))
        .create_sync(CreateWebsiteSyncCommand {
            tenant_id: "tenant-cleanup".to_string(),
            website_root_uuid: root_uuid,
            idempotency_key: "cleanup-sync".to_string(),
            expected_root_version: 1,
            expected_generation: 1,
            manifest_sha256: manifest.sha256,
            manifest_file_count: manifest.file_count,
            manifest_total_bytes: manifest.total_bytes,
            expires_at: (Utc::now() + Duration::hours(1))
                .to_rfc3339_opts(SecondsFormat::Millis, true),
            operator_id: "user-cleanup".to_string(),
        })
        .await
        .expect("cleanup WebsiteSync should be created");
    let file_node_id = "node-cleanup-index";
    sqlx::query(
        "INSERT INTO dr_drive_node (
           id, tenant_id, space_id, space_type, parent_node_id,
           node_type, node_name, content_state, head_content_type,
           head_content_type_group, head_content_length, head_version_no,
           head_checksum_sha256_hex, lifecycle_status, version, created_by, updated_by
         ) VALUES (
           $1, 'tenant-cleanup', 'space-cleanup', 'website', $2,
           'file', 'index.html', 'ready', 'text/html', 'text', 18, 1,
           $3, 'active', 1, 'user-cleanup', 'user-cleanup'
         )",
    )
    .bind(file_node_id)
    .bind(&sync.sync.staging_node_id)
    .bind(format!("sha256:{}", "a".repeat(64)))
    .execute(&pool)
    .await
    .expect("cleanup file node should be inserted");

    let temporary_directory = tempfile::tempdir().expect("cleanup object root should exist");
    let endpoint_url = format!(
        "file:///{}",
        temporary_directory
            .path()
            .to_string_lossy()
            .replace('\\', "/")
    );
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
           id, provider_kind, name, endpoint_url, bucket, path_style, strict_tls,
           status, version, created_by, updated_by
         ) VALUES (
           'provider-cleanup', 'local_filesystem', 'Cleanup Provider', $1,
           'bucket-cleanup', 1, 1, 'active', 1, 'test', 'test'
         )",
    )
    .bind(endpoint_url)
    .execute(&pool)
    .await
    .expect("cleanup storage provider should be inserted");
    let object_path = temporary_directory
        .path()
        .join("bucket-cleanup")
        .join("objects")
        .join("index.html");
    std::fs::create_dir_all(object_path.parent().expect("object parent should exist"))
        .expect("cleanup object parent should be created");
    std::fs::write(&object_path, b"cleanup-index-data")
        .expect("cleanup provider object should be written");
    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
           id, tenant_id, node_id, version_no, storage_provider_id,
           bucket, object_key, content_type, content_length,
           checksum_sha256_hex, lifecycle_status, created_by, updated_by
         ) VALUES (
           'object-cleanup', 'tenant-cleanup', $1, 1, 'provider-cleanup',
           'bucket-cleanup', 'objects/index.html', 'text/html', 18,
           $2, 'active', 'user-cleanup', 'user-cleanup'
         )",
    )
    .bind(file_node_id)
    .bind(format!("sha256:{}", "a".repeat(64)))
    .execute(&pool)
    .await
    .expect("cleanup storage object should be inserted");
    sqlx::query(
        "UPDATE dr_drive_website_sync
         SET expires_at='2000-01-01T00:00:00.000Z'
         WHERE id=$1",
    )
    .bind(&sync.sync.id)
    .execute(&pool)
    .await
    .expect("cleanup WebsiteSync should be expired by the fixture");

    let result = cleanup_website_publishing(&pool, 100)
        .await
        .expect("website publishing cleanup should complete");
    assert_eq!(result.expired_syncs, 1);
    assert_eq!(result.completed_candidates, 1);
    assert_eq!(result.deleted_objects, 1);
    assert_eq!(result.deleted_nodes, 2);
    assert!(
        !object_path.exists(),
        "provider object must be physically deleted"
    );

    let sync_status: String =
        sqlx::query_scalar("SELECT sync_status FROM dr_drive_website_sync WHERE id=$1")
            .bind(&sync.sync.id)
            .fetch_one(&pool)
            .await
            .expect("expired WebsiteSync should be queryable");
    assert_eq!(sync_status, "expired");
    let object_status: String = sqlx::query_scalar(
        "SELECT lifecycle_status FROM dr_drive_storage_object WHERE id='object-cleanup'",
    )
    .fetch_one(&pool)
    .await
    .expect("deleted storage object should be queryable");
    assert_eq!(object_status, "deleted");
    let active_nodes: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_node
         WHERE id IN ($1, $2) AND lifecycle_status != 'deleted'",
    )
    .bind(&sync.sync.staging_node_id)
    .bind(file_node_id)
    .fetch_one(&pool)
    .await
    .expect("cleanup nodes should be queryable");
    assert_eq!(active_nodes, 0);
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_audit_event
         WHERE tenant_id='tenant-cleanup'
           AND action IN (
             'drive.website_sync.expired',
             'drive.website_tree.system_override.expired_publishing_cleanup'
           )",
    )
    .fetch_one(&pool)
    .await
    .expect("cleanup audits should be queryable");
    assert_eq!(audit_count, 2);
}
