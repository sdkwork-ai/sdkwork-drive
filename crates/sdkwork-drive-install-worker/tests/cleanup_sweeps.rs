use sdkwork_drive_install_worker::maintenance::expired_download_package_cleanup::cleanup_expired_download_packages;
use sdkwork_drive_install_worker::maintenance::expired_multipart_abort::abort_expired_multipart_uploads;
use sdkwork_drive_install_worker::maintenance::storage_object_purge::purge_deleted_storage_objects;
use sqlx::PgPool;

async fn seed_space_node(pool: &PgPool) {
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-cleanup-sweep', 'tenant-cleanup-sweep', 'user',
            'user-cleanup-sweep', 'app_upload', 'Cleanup', 'active', 1,
            'user-cleanup-sweep', 'user-cleanup-sweep'
        )",
    )
    .execute(pool)
    .await
    .expect("cleanup space should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, head_content_type, head_content_type_group,
            head_content_length, head_version_no, lifecycle_status, version,
            created_by, updated_by
        ) VALUES (
            'node-cleanup-sweep', 'tenant-cleanup-sweep', 'space-cleanup-sweep', NULL,
            'file', 'cleanup.txt', 'ready', 'text/plain', 'text', 4, 1,
            'deleted', 1, 'user-cleanup-sweep', 'user-cleanup-sweep'
        )",
    )
    .execute(pool)
    .await
    .expect("cleanup node should be inserted");
}

async fn seed_local_provider(pool: &PgPool, root: &str) -> String {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket,
            path_style, strict_tls, status, version, created_by, updated_by
        ) VALUES (
            'provider-cleanup-sweep', 'tenant-cleanup-sweep', 'local_filesystem',
            'Cleanup Local', $1, NULL, 'bucket-cleanup-sweep', TRUE, FALSE,
            'active', 1, 'test', 'test'
        )",
    )
    .bind(root)
    .execute(pool)
    .await
    .expect("cleanup provider should be inserted");
    "provider-cleanup-sweep".to_string()
}

#[tokio::test]
async fn deleted_storage_object_is_removed_from_provider_and_metadata() {
    let Some((pool, _guard)) = sdkwork_drive_test_support::postgres_test_database().await else {
        return;
    };
    let directory = tempfile::tempdir().expect("provider root should exist");
    let root = directory.path().join("bucket-cleanup-sweep");
    std::fs::create_dir_all(root.join("objects")).expect("object parent should exist");
    let object_path = root.join("objects/cleanup.txt");
    std::fs::write(&object_path, b"data").expect("object should be written");
    seed_space_node(&pool).await;
    seed_local_provider(&pool, &format!("file://{}", directory.path().display())).await;
    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES (
            'object-cleanup-sweep', 'tenant-cleanup-sweep', 'node-cleanup-sweep', 1,
            'provider-cleanup-sweep', 'bucket-cleanup-sweep', 'objects/cleanup.txt',
            'text/plain', 4,
            'sha256:1111111111111111111111111111111111111111111111111111111111111111',
            'deleted', 'test', 'test'
        )",
    )
    .execute(&pool)
    .await
    .expect("deleted storage object should be inserted");

    let result = purge_deleted_storage_objects(&pool, 100)
        .await
        .expect("purge should succeed");
    assert_eq!(result.purged_objects, 1);
    assert!(!object_path.exists(), "provider bytes must be deleted");
    let remaining: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dr_drive_storage_object WHERE id = 'object-cleanup-sweep'",
    )
    .fetch_one(&pool)
    .await
    .expect("storage object count should be readable");
    assert_eq!(remaining, 0);
}

#[tokio::test]
async fn expired_download_package_archive_is_deleted_and_package_expired() {
    let Some((pool, _guard)) = sdkwork_drive_test_support::postgres_test_database().await else {
        return;
    };
    let directory = tempfile::tempdir().expect("provider root should exist");
    let root = directory.path().join("bucket-cleanup-sweep");
    std::fs::create_dir_all(root.join("packages")).expect("archive parent should exist");
    let archive_path = root.join("packages/archive.zip");
    std::fs::write(&archive_path, b"zip").expect("archive should be written");
    seed_space_node(&pool).await;
    seed_local_provider(&pool, &format!("file://{}", directory.path().display())).await;
    sqlx::query(
        "INSERT INTO dr_drive_download_package (
            id, tenant_id, package_name, state, storage_provider_id, bucket,
            archive_object_key, content_type, file_count, total_bytes,
            archive_size_bytes, requested_node_ids_json, item_manifest_json,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'package-cleanup-sweep', 'tenant-cleanup-sweep', 'cleanup.zip', 'ready',
            'provider-cleanup-sweep', 'bucket-cleanup-sweep', 'packages/archive.zip',
            'application/zip', 1, 4, 3, '[]', '[]', 1, 1, 'test', 'test'
        )",
    )
    .execute(&pool)
    .await
    .expect("expired package should be inserted");

    let result = cleanup_expired_download_packages(&pool, 100)
        .await
        .expect("download package cleanup should succeed");
    assert_eq!(result.cleaned_packages, 1);
    assert!(!archive_path.exists(), "package archive must be deleted");
    let state: String = sqlx::query_scalar(
        "SELECT state FROM dr_drive_download_package WHERE id = 'package-cleanup-sweep'",
    )
    .fetch_one(&pool)
    .await
    .expect("package state should be readable");
    assert_eq!(state, "expired");
}

#[tokio::test]
async fn expired_multipart_session_is_aborted_even_when_provider_lacks_multipart() {
    let Some((pool, _guard)) = sdkwork_drive_test_support::postgres_test_database().await else {
        return;
    };
    let directory = tempfile::tempdir().expect("provider root should exist");
    seed_space_node(&pool).await;
    seed_local_provider(&pool, &format!("file://{}", directory.path().display())).await;
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key, idempotency_key,
            storage_provider_id, storage_upload_id, state, expires_at_epoch_ms,
            version, created_by, updated_by
        ) VALUES (
            'session-cleanup-sweep', 'tenant-cleanup-sweep', 'space-cleanup-sweep',
            'node-cleanup-sweep', 'bucket-cleanup-sweep', 'objects/upload.bin',
            'idem-cleanup-sweep', 'provider-cleanup-sweep', 'upload-id',
            'expired', 1, 1, 'test', 'test'
        )",
    )
    .execute(&pool)
    .await
    .expect("expired session should be inserted");

    let result = abort_expired_multipart_uploads(&pool, 100)
        .await
        .expect("multipart abort should succeed");
    assert_eq!(result.aborted_uploads, 1);
    let state: String = sqlx::query_scalar(
        "SELECT state FROM dr_drive_upload_session WHERE id = 'session-cleanup-sweep'",
    )
    .fetch_one(&pool)
    .await
    .expect("session state should be readable");
    assert_eq!(state, "aborted");
}
