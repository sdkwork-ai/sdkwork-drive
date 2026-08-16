use sdkwork_drive_workspace_service::application::maintenance_service::{
    DriveMaintenanceService, ListMaintenanceJobsCommand, SweepAbandonedUploadTasksCommand,
    SweepExpiredUploadContentCommand, SweepObjectStoreCommand, SweepUploadSessionsCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::maintenance_store::SqlMaintenanceStore;

#[tokio::test]
async fn upload_session_sweep_marks_expired_sessions() {
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
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("space should be inserted");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, NULL, 'file', $4, 'ready', 'active', 1, $5, $6)",
    )
    .bind("node-001")
    .bind("tenant-001")
    .bind("space-001")
    .bind("node-001.bin")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("node should be inserted");

    seed_storage_provider(&pool, "provider-001", "bucket-001").await;

    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'provider-001', $1, 'created', $8, 1, $9, $10)",
    )
    .bind("session-001")
    .bind("tenant-001")
    .bind("space-001")
    .bind("node-001")
    .bind("bucket-001")
    .bind("objects/node-001.bin")
    .bind("idem-001")
    .bind(1_700_000_000_000_i64)
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("upload session should be inserted");

    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(pool.clone()));
    let result = service
        .sweep_upload_sessions(SweepUploadSessionsCommand {
            now_epoch_ms: 1_800_000_000_000_i64,
            dry_run: false,
            limit: Some(100),
            operator_id: "admin-ops".to_string(),
            request_id: None,
            trace_id: None,
        })
        .await
        .expect("upload session sweep should succeed");
    assert_eq!(result.scanned_count, 1);
    assert_eq!(result.affected_count, 1);

    let state: String =
        sqlx::query_scalar("SELECT state FROM dr_drive_upload_session WHERE id='session-001'")
            .fetch_one(&pool)
            .await
            .expect("session state should be queryable");
    assert_eq!(state, "expired");
}

#[tokio::test]
async fn upload_session_sweep_expires_stale_completing_sessions() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-completing-sweep', 'tenant-completing-sweep', 'user', 'user-001', 'personal', 'Main', 'active', 1, 'admin-001', 'admin-001')",
    )
    .execute(&pool)
    .await
    .expect("space should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-completing-sweep', 'tenant-completing-sweep', 'space-completing-sweep', NULL, 'file', 'node.bin', 'uploading', 'active', 1, 'admin-001', 'admin-001')",
    )
    .execute(&pool)
    .await
    .expect("node should be inserted");
    seed_storage_provider(&pool, "provider-001", "bucket-001").await;
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'session-completing-sweep', 'tenant-completing-sweep',
            'space-completing-sweep', 'node-completing-sweep',
            'bucket-001', 'objects/node.bin', 'idem-completing-sweep',
            'provider-001', 'storage-upload-completing-sweep',
            'completing', 1700000000000, 2, 'admin-001', 'admin-001'
        )",
    )
    .execute(&pool)
    .await
    .expect("completing upload session should be inserted");

    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(pool.clone()));
    let result = service
        .sweep_upload_sessions(SweepUploadSessionsCommand {
            now_epoch_ms: 1_800_000_000_000_i64,
            dry_run: false,
            limit: Some(100),
            operator_id: "admin-ops".to_string(),
            request_id: None,
            trace_id: None,
        })
        .await
        .expect("upload session sweep should succeed");
    assert_eq!(result.scanned_count, 1);
    assert_eq!(result.affected_count, 1);

    let state: String = sqlx::query_scalar(
        "SELECT state FROM dr_drive_upload_session WHERE id='session-completing-sweep'",
    )
    .fetch_one(&pool)
    .await
    .expect("session state should be queryable");
    assert_eq!(state, "expired");
}

#[tokio::test]
async fn abandoned_upload_task_sweep_marks_stuck_items_failed_when_session_expired() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-abandoned-sweep', 'tenant-abandoned-sweep', 'user', 'user-001', 'personal', 'Main', 'active', 1, 'admin-001', 'admin-001')",
    )
    .execute(&pool)
    .await
    .expect("space should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-abandoned-sweep', 'tenant-abandoned-sweep', 'space-abandoned-sweep', NULL, 'file', 'node.bin', 'uploading', 'active', 1, 'admin-001', 'admin-001')",
    )
    .execute(&pool)
    .await
    .expect("node should be inserted");
    seed_storage_provider(&pool, "provider-abandoned", "bucket-abandoned").await;
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'session-abandoned-sweep', 'tenant-abandoned-sweep',
            'space-abandoned-sweep', 'node-abandoned-sweep',
            'bucket-abandoned', 'objects/node.bin', 'idem-abandoned-sweep',
            'provider-abandoned', 'storage-upload-abandoned-sweep',
            'expired', 1_700_000_000_000, 2, 'admin-001', 'admin-001'
        )",
    )
    .execute(&pool)
    .await
    .expect("expired upload session should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_upload_item (
            id, task_id, tenant_id, organization_id, user_id, actor_type, actor_id,
            app_id, app_resource_type, app_resource_id, upload_profile_code,
            file_fingerprint, space_id, node_id, upload_session_id,
            storage_provider_id, storage_upload_id, original_file_name, file_extension,
            content_type, content_type_group, detected_content_type, content_length,
            checksum_sha256_hex, chunk_size_bytes, total_parts, uploaded_parts_count,
            uploaded_bytes, status, retention_mode, retention_expires_at_epoch_ms,
            cleanup_action, hard_delete_after_epoch_ms, cleanup_status,
            post_process_status, created_by, updated_by
        ) VALUES (
            'upload-item-abandoned', 'task-abandoned', 'tenant-abandoned-sweep',
            NULL, 'user-001', 'user', 'user-001',
            'drive-pc', 'desktop-file-browser', 'root', 'generic',
            'fp-abandoned', 'space-abandoned-sweep', 'node-abandoned-sweep',
            'session-abandoned-sweep',
            'provider-abandoned', 'storage-upload-abandoned-sweep', 'node.bin', 'bin',
            'application/octet-stream', 'binary', NULL, 1024,
            NULL, 5242880, 1, 0,
            0, 'uploading', 'long_term', NULL,
            NULL, NULL, 'active',
            'not_required', 'admin-001', 'admin-001'
        )",
    )
    .execute(&pool)
    .await
    .expect("upload item should be inserted");

    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(pool.clone()));
    let result = service
        .sweep_abandoned_upload_tasks(SweepAbandonedUploadTasksCommand {
            now_epoch_ms: 1_800_000_000_000_i64,
            dry_run: false,
            limit: Some(100),
            operator_id: "admin-ops".to_string(),
            request_id: None,
            trace_id: None,
        })
        .await
        .expect("abandoned upload task sweep should succeed");
    assert_eq!(result.scanned_count, 1);
    assert_eq!(result.affected_count, 1);

    let status: String = sqlx::query_scalar(
        "SELECT status FROM dr_drive_upload_item WHERE id='upload-item-abandoned'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload item status should be queryable");
    assert_eq!(status, "failed");
}

#[tokio::test]
async fn object_sweep_deletes_deleted_storage_objects() {
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
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("space should be inserted");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, NULL, 'file', $4, 'ready', 'active', 1, $5, $6)",
    )
    .bind("node-001")
    .bind("tenant-001")
    .bind("space-001")
    .bind("node-001.bin")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("node should be inserted");

    seed_storage_provider(&pool, "provider-001", "bucket-001").await;

    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES
            ($1, $2, $3, 1, $4, $5, $6, $7, $8, $9, 'deleted', $10, $11),
            ($12, $13, $14, 2, $15, $16, $17, $18, $19, $20, 'active', $21, $22)",
    )
    .bind("obj-001")
    .bind("tenant-001")
    .bind("node-001")
    .bind("provider-001")
    .bind("bucket-001")
    .bind("objects/node-001/a.bin")
    .bind("application/octet-stream")
    .bind(128_i64)
    .bind("sha256:1111111111111111111111111111111111111111111111111111111111111111")
    .bind("admin-001")
    .bind("admin-001")
    .bind("obj-002")
    .bind("tenant-001")
    .bind("node-001")
    .bind("provider-001")
    .bind("bucket-001")
    .bind("objects/node-001/b.bin")
    .bind("application/octet-stream")
    .bind(256_i64)
    .bind("sha256:2222222222222222222222222222222222222222222222222222222222222222")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("storage objects should be inserted");

    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(pool.clone()));
    let result = service
        .sweep_object_store(SweepObjectStoreCommand {
            dry_run: false,
            limit: Some(100),
            operator_id: "admin-ops".to_string(),
            request_id: None,
            trace_id: None,
        })
        .await
        .expect("object sweep should succeed");
    assert_eq!(result.scanned_count, 1);
    assert_eq!(result.affected_count, 1);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_storage_object")
        .fetch_one(&pool)
        .await
        .expect("object count should be queryable");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn expired_upload_content_sweep_soft_deletes_nodes_and_records_sensitive_operation() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-expired-upload', 'tenant-expired-upload', 'user',
            'user-expired-upload', 'app_upload', 'Upload', 'active', 1,
            'user-expired-upload', 'user-expired-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-expired-upload', 'tenant-expired-upload', 'space-expired-upload',
            NULL, 'file', 'expired.txt', 'ready', 'active', 1,
            'user-expired-upload', 'user-expired-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be inserted");
    seed_storage_provider(&pool, "provider-expired-upload", "bucket-expired-upload").await;
    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES (
            'object-expired-upload', 'tenant-expired-upload', 'node-expired-upload',
            1, 'provider-expired-upload', 'bucket-expired-upload',
            'objects/expired.txt', 'text/plain', 12,
            'sha256:1111111111111111111111111111111111111111111111111111111111111111',
            'active', 'user-expired-upload', 'user-expired-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage object should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_upload_item (
            id, task_id, tenant_id, organization_id, user_id, actor_type, actor_id,
            app_id, app_resource_type, app_resource_id, upload_profile_code,
            file_fingerprint, space_id, node_id, upload_session_id,
            storage_provider_id, storage_upload_id, original_file_name, file_extension,
            content_type, content_type_group, detected_content_type, content_length,
            checksum_sha256_hex, chunk_size_bytes, total_parts, uploaded_parts_count,
            uploaded_bytes, status, retention_mode, retention_expires_at_epoch_ms,
            cleanup_action, hard_delete_after_epoch_ms, cleanup_status,
            post_process_status, created_by, updated_by
        ) VALUES (
            'upload-item-expired', 'task-expired', 'tenant-expired-upload',
            'org-expired-upload', 'user-expired-upload', 'user', 'user-expired-upload',
            'drive-pc', 'desktop-file-browser', 'root', 'generic',
            'fp-expired', 'space-expired-upload', 'node-expired-upload', NULL,
            'provider-expired-upload', NULL, 'expired.txt', 'txt',
            'text/plain', 'text', 'text/plain', 12,
            'sha256:1111111111111111111111111111111111111111111111111111111111111111',
            8, 2, 2, 12, 'completed', 'temporary', 1700000000000,
            'soft_delete', NULL, 'active', 'not_required',
            'user-expired-upload', 'user-expired-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("upload item should be inserted");

    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(pool.clone()));
    let result = service
        .sweep_expired_upload_content(SweepExpiredUploadContentCommand {
            now_epoch_ms: 1_800_000_000_000,
            dry_run: false,
            limit: Some(100),
            operator_id: "admin-ops".to_string(),
            request_id: Some("req-expired-upload".to_string()),
            trace_id: Some("trace-expired-upload".to_string()),
        })
        .await
        .expect("expired upload content sweep should succeed");
    assert_eq!(result.scanned_count, 1);
    assert_eq!(result.affected_count, 1);

    let node_status: String = sqlx::query_scalar(
        "SELECT lifecycle_status FROM dr_drive_node WHERE id='node-expired-upload'",
    )
    .fetch_one(&pool)
    .await
    .expect("node status should be queryable");
    assert_eq!(node_status, "trashed");

    let cleanup_status: String = sqlx::query_scalar(
        "SELECT cleanup_status FROM dr_drive_upload_item WHERE id='upload-item-expired'",
    )
    .fetch_one(&pool)
    .await
    .expect("cleanup status should be queryable");
    assert_eq!(cleanup_status, "soft_deleted");

    let sensitive_operation: (String, String, String) = sqlx::query_as(
        "SELECT operation_type, operation_reason, object_delete_status
         FROM dr_drive_file_sensitive_operation
         WHERE upload_item_id='upload-item-expired'",
    )
    .fetch_one(&pool)
    .await
    .expect("sensitive operation should be recorded");
    assert_eq!(
        sensitive_operation,
        (
            "soft_delete".to_string(),
            "retention_expired".to_string(),
            "not_required".to_string()
        )
    );

    let change_event: String = sqlx::query_scalar(
        "SELECT event_type FROM dr_drive_change_log WHERE space_id='space-expired-upload'",
    )
    .fetch_one(&pool)
    .await
    .expect("content expired change should be recorded");
    assert_eq!(change_event, "drive.uploader.content_expired");
}

#[tokio::test]
async fn expired_upload_content_sweep_hard_deletes_objects_and_records_sensitive_operation() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-expired-hard-upload', 'tenant-expired-hard-upload', 'user',
            'user-expired-hard-upload', 'app_upload', 'Upload', 'active', 1,
            'user-expired-hard-upload', 'user-expired-hard-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-expired-hard-upload', 'tenant-expired-hard-upload',
            'space-expired-hard-upload', NULL, 'file', 'expired-hard.txt',
            'ready', 'active', 1, 'user-expired-hard-upload', 'user-expired-hard-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be inserted");
    seed_storage_provider(
        &pool,
        "provider-expired-hard-upload",
        "bucket-expired-hard-upload",
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES (
            'object-expired-hard-upload', 'tenant-expired-hard-upload',
            'node-expired-hard-upload', 1, 'provider-expired-hard-upload',
            'bucket-expired-hard-upload', 'objects/expired-hard.txt',
            'text/plain', 32,
            'sha256:2222222222222222222222222222222222222222222222222222222222222222',
            'active', 'user-expired-hard-upload', 'user-expired-hard-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage object should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_upload_item (
            id, task_id, tenant_id, organization_id, user_id, actor_type, actor_id,
            app_id, app_resource_type, app_resource_id, upload_profile_code,
            file_fingerprint, space_id, node_id, upload_session_id,
            storage_provider_id, storage_upload_id, original_file_name, file_extension,
            content_type, content_type_group, detected_content_type, content_length,
            checksum_sha256_hex, chunk_size_bytes, total_parts, uploaded_parts_count,
            uploaded_bytes, status, retention_mode, retention_expires_at_epoch_ms,
            cleanup_action, hard_delete_after_epoch_ms, cleanup_status,
            post_process_status, created_by, updated_by
        ) VALUES (
            'upload-item-expired-hard', 'task-expired-hard',
            'tenant-expired-hard-upload', 'org-expired-hard-upload',
            'user-expired-hard-upload', 'user', 'user-expired-hard-upload',
            'drive-pc', 'desktop-file-browser', 'root', 'generic',
            'fp-expired-hard', 'space-expired-hard-upload',
            'node-expired-hard-upload', NULL, 'provider-expired-hard-upload',
            NULL, 'expired-hard.txt', 'txt', 'text/plain', 'text',
            'text/plain', 32,
            'sha256:2222222222222222222222222222222222222222222222222222222222222222',
            8, 4, 4, 32, 'completed', 'temporary', 1700000000000,
            'hard_delete', NULL, 'active', 'not_required',
            'user-expired-hard-upload', 'user-expired-hard-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("upload item should be inserted");

    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(pool.clone()));
    let result = service
        .sweep_expired_upload_content(SweepExpiredUploadContentCommand {
            now_epoch_ms: 1_800_000_000_000,
            dry_run: false,
            limit: Some(100),
            operator_id: "admin-ops".to_string(),
            request_id: Some("req-expired-hard-upload".to_string()),
            trace_id: Some("trace-expired-hard-upload".to_string()),
        })
        .await
        .expect("expired upload content sweep should succeed");
    assert_eq!(result.scanned_count, 1);
    assert_eq!(result.affected_count, 1);

    let node_status: String = sqlx::query_scalar(
        "SELECT lifecycle_status FROM dr_drive_node WHERE id='node-expired-hard-upload'",
    )
    .fetch_one(&pool)
    .await
    .expect("node status should be queryable");
    assert_eq!(node_status, "deleted");

    let object_status: String = sqlx::query_scalar(
        "SELECT lifecycle_status FROM dr_drive_storage_object WHERE id='object-expired-hard-upload'",
    )
    .fetch_one(&pool)
    .await
    .expect("object status should be queryable");
    assert_eq!(object_status, "deleted");

    let cleanup_status: String = sqlx::query_scalar(
        "SELECT cleanup_status FROM dr_drive_upload_item WHERE id='upload-item-expired-hard'",
    )
    .fetch_one(&pool)
    .await
    .expect("cleanup status should be queryable");
    assert_eq!(cleanup_status, "hard_deleted");

    let sensitive_operation: (String, String, String, Option<String>) = sqlx::query_as(
        "SELECT operation_type, operation_reason, object_delete_status, object_key
         FROM dr_drive_file_sensitive_operation
         WHERE upload_item_id='upload-item-expired-hard'",
    )
    .fetch_one(&pool)
    .await
    .expect("sensitive operation should be recorded");
    assert_eq!(
        sensitive_operation,
        (
            "hard_delete".to_string(),
            "retention_expired".to_string(),
            "deleted".to_string(),
            Some("objects/expired-hard.txt".to_string())
        )
    );
}

#[tokio::test]
async fn maintenance_service_records_jobs_and_lists_with_filters() {
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
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("space should be inserted");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, NULL, 'file', $4, 'ready', 'active', 1, $5, $6)",
    )
    .bind("node-001")
    .bind("tenant-001")
    .bind("space-001")
    .bind("node-001.bin")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("node should be inserted");

    seed_storage_provider(&pool, "provider-001", "bucket-001").await;

    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES ($1, $2, $3, 1, $4, $5, $6, $7, $8, $9, 'deleted', $10, $11)",
    )
    .bind("obj-001")
    .bind("tenant-001")
    .bind("node-001")
    .bind("provider-001")
    .bind("bucket-001")
    .bind("objects/node-001/a.bin")
    .bind("application/octet-stream")
    .bind(128_i64)
    .bind("sha256:1111111111111111111111111111111111111111111111111111111111111111")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("deleted storage object should be inserted");

    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'provider-001', $1, 'created', $8, 1, $9, $10)",
    )
    .bind("session-001")
    .bind("tenant-001")
    .bind("space-001")
    .bind("node-001")
    .bind("bucket-001")
    .bind("objects/node-001.bin")
    .bind("idem-001")
    .bind(1_700_000_000_000_i64)
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("upload session should be inserted");

    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(pool.clone()));
    service
        .sweep_object_store(SweepObjectStoreCommand {
            dry_run: false,
            limit: Some(100),
            operator_id: "admin-ops".to_string(),
            request_id: Some("request-001".to_string()),
            trace_id: Some("trace-001".to_string()),
        })
        .await
        .expect("object sweep should succeed");
    service
        .sweep_upload_sessions(SweepUploadSessionsCommand {
            now_epoch_ms: 1_800_000_000_000_i64,
            dry_run: false,
            limit: Some(100),
            operator_id: "admin-ops".to_string(),
            request_id: Some("request-001".to_string()),
            trace_id: Some("trace-001".to_string()),
        })
        .await
        .expect("upload session sweep should succeed");

    let page = service
        .list_maintenance_jobs(ListMaintenanceJobsCommand {
            job_type: None,
            status: None,
            operator_id: Some("admin-ops".to_string()),
            page: Some(1),
            page_size: Some(10),
        })
        .await
        .expect("maintenance jobs list should succeed");
    assert_eq!(page.total, 2);
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].job_type, "upload_session_sweep");
    assert_eq!(page.items[1].job_type, "object_sweep");
    assert_eq!(page.items[0].status, "completed");
    assert_eq!(page.items[0].request_id.as_deref(), Some("request-001"));
    assert_eq!(page.items[0].trace_id.as_deref(), Some("trace-001"));
    for item in page.items {
        assert!(
            item.started_at.contains('T') && item.started_at.ends_with('Z'),
            "started_at should be RFC3339 UTC: {}",
            item.started_at
        );
        assert!(
            item.finished_at.contains('T') && item.finished_at.ends_with('Z'),
            "finished_at should be RFC3339 UTC: {}",
            item.finished_at
        );
        assert!(
            item.created_at.contains('T') && item.created_at.ends_with('Z'),
            "created_at should be RFC3339 UTC: {}",
            item.created_at
        );
    }
}

#[tokio::test]
async fn maintenance_service_records_failed_job_when_sweep_errors() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query("DROP TABLE dr_drive_storage_object")
        .execute(&pool)
        .await
        .expect("drop storage object table should succeed");

    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(pool.clone()));
    let error = service
        .sweep_object_store(SweepObjectStoreCommand {
            dry_run: false,
            limit: Some(100),
            operator_id: "admin-ops".to_string(),
            request_id: Some("request-failed-001".to_string()),
            trace_id: Some("trace-failed-001".to_string()),
        })
        .await
        .expect_err("object sweep should fail when table missing");
    let error_message = format!("{error:?}");
    assert!(
        error_message.contains("count deleted dr_drive_storage_object failed"),
        "unexpected error: {error_message}"
    );

    let failed_jobs: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_maintenance_job
         WHERE job_type='object_sweep'
           AND status='failed'
           AND operator_id='admin-ops'
           AND request_id='request-failed-001'
           AND trace_id='trace-failed-001'
           AND error_message IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("failed maintenance jobs should be queryable");
    assert_eq!(failed_jobs, 1);
}

#[tokio::test]
async fn maintenance_service_records_failed_upload_sweep_job_when_table_missing() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query("DROP TABLE dr_drive_upload_session")
        .execute(&pool)
        .await
        .expect("drop upload session table should succeed");

    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(pool.clone()));
    let error = service
        .sweep_upload_sessions(SweepUploadSessionsCommand {
            now_epoch_ms: 1_800_000_000_000_i64,
            dry_run: false,
            limit: Some(100),
            operator_id: "admin-upload-failed".to_string(),
            request_id: Some("request-upload-failed-001".to_string()),
            trace_id: Some("trace-upload-failed-001".to_string()),
        })
        .await
        .expect_err("upload sweep should fail when table missing");
    let error_message = format!("{error:?}");
    assert!(
        error_message.contains("count expired dr_drive_upload_session failed"),
        "unexpected error: {error_message}"
    );

    let failed_jobs: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_maintenance_job
         WHERE job_type='upload_session_sweep'
           AND status='failed'
           AND operator_id='admin-upload-failed'
           AND request_id='request-upload-failed-001'
           AND trace_id='trace-upload-failed-001'
           AND error_message IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("failed upload maintenance jobs should be queryable");
    assert_eq!(failed_jobs, 1);
}

#[tokio::test]
async fn maintenance_service_rejects_invalid_identifier_inputs() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveMaintenanceService::new(SqlMaintenanceStore::new(pool));
    let invalid_operator_error = service
        .sweep_object_store(SweepObjectStoreCommand {
            dry_run: true,
            limit: Some(1),
            operator_id: "admin ops".to_string(),
            request_id: None,
            trace_id: None,
        })
        .await
        .expect_err("operator_id with spaces should be rejected");
    let invalid_operator_message = format!("{invalid_operator_error:?}");
    assert!(
        invalid_operator_message.contains("operator_id contains invalid characters"),
        "unexpected error: {invalid_operator_message}"
    );

    let too_long_request_id = "a".repeat(65);
    let invalid_request_error = service
        .sweep_object_store(SweepObjectStoreCommand {
            dry_run: true,
            limit: Some(1),
            operator_id: "admin-ops".to_string(),
            request_id: Some(too_long_request_id),
            trace_id: None,
        })
        .await
        .expect_err("request_id longer than 64 should be rejected");
    let invalid_request_message = format!("{invalid_request_error:?}");
    assert!(
        invalid_request_message.contains("request_id length must be <= 64"),
        "unexpected error: {invalid_request_message}"
    );
}

async fn seed_storage_provider(pool: &sqlx::PgPool, provider_id: &str, bucket: &str) {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            $1, 's3_compatible', $1, 'https://s3.example.com', 'us-east-1',
            $2, 1, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-001', 'admin-001'
        )",
    )
    .bind(provider_id)
    .bind(bucket)
    .execute(pool)
    .await
    .expect("storage provider should be inserted");
}
