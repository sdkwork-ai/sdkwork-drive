use sdkwork_drive_storage_contract::{DriveObjectLocator, DriveObjectStore, HeadObjectRequest};
use sdkwork_drive_storage_local::LocalDriveObjectStore;
use sdkwork_drive_workspace_service::application::uploader_service::{
    CompleteStoredUploaderUploadCommand, DriveUploaderService, MarkUploaderPartUploadedCommand,
    PrepareUploaderUploadCommand, UploadBytesCommand, UploaderActor, UploaderRetention,
    UploaderTarget,
};
use sdkwork_drive_workspace_service::infrastructure::sql::uploader_store::SqlUploaderStore;
use sdkwork_drive_workspace_service::{drive_share_token_hash, DriveServiceError};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn prepare_upload_creates_logged_in_user_upload_space_and_task() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let service = DriveUploaderService::new(SqlUploaderStore::new(pool.clone()));

    let prepared = service
        .prepare_upload(PrepareUploaderUploadCommand {
            id: "upload-item-user".to_string(),
            task_id: "task-user".to_string(),
            tenant_id: "tenant-user".to_string(),
            organization_id: Some("org-user".to_string()),
            actor: UploaderActor::User {
                user_id: "user-001".to_string(),
            },
            app_id: "drive-pc".to_string(),
            app_resource_type: "desktop-file-browser".to_string(),
            app_resource_id: "root".to_string(),
            scene: Some("user_document_upload".to_string()),
            source: Some("pc_local_file".to_string()),
            upload_profile_code: "generic".to_string(),
            file_fingerprint: "fp-user".to_string(),
            original_file_name: "report.txt".to_string(),
            content_type: "text/plain".to_string(),
            content_length: 42,
            chunk_size_bytes: 8,
            target: UploaderTarget::AutoUploadSpace {
                parent_node_id: None,
            },
            retention: UploaderRetention::LongTerm,
            operator_id: "user-001".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect("user uploader task should be prepared");

    assert_eq!(prepared.upload_profile_code, "generic");
    assert_eq!(prepared.actor_type, "user");
    assert_eq!(prepared.user_id.as_deref(), Some("user-001"));
    assert_eq!(prepared.retention_mode, "long_term");
    assert_eq!(prepared.total_parts, 6);
    assert_eq!(prepared.object_bucket.as_deref(), Some("bucket-uploader"));
    assert!(prepared
        .object_key
        .as_deref()
        .expect("prepare should return upload object key")
        .contains("/scene/user_document_upload/source/pc_local_file/profile/generic/"));

    let owner: (String, String, String) = sqlx::query_as(
        "SELECT owner_subject_type, owner_subject_id, space_type
         FROM dr_drive_space
         WHERE id=$1",
    )
    .bind(&prepared.space_id)
    .fetch_one(&pool)
    .await
    .expect("upload space should be queryable");
    assert_eq!(
        owner,
        (
            "user".to_string(),
            "user-001".to_string(),
            "app_upload".to_string()
        )
    );

    let node_state: String =
        sqlx::query_scalar("SELECT content_state FROM dr_drive_node WHERE id=$1")
            .bind(&prepared.node_id)
            .fetch_one(&pool)
            .await
            .expect("upload node should be queryable");
    assert_eq!(node_state, "uploading");

    let usage_context: (String, String) =
        sqlx::query_as("SELECT scene, source FROM dr_drive_upload_item WHERE id=$1")
            .bind(&prepared.id)
            .fetch_one(&pool)
            .await
            .expect("upload item usage context should be queryable");
    assert_eq!(
        usage_context,
        (
            "user_document_upload".to_string(),
            "pc_local_file".to_string()
        )
    );

    let node_usage_context: (String, String) =
        sqlx::query_as("SELECT scene, source FROM dr_drive_node WHERE id=$1")
            .bind(&prepared.node_id)
            .fetch_one(&pool)
            .await
            .expect("upload node usage context should be queryable");
    assert_eq!(
        node_usage_context,
        (
            "user_document_upload".to_string(),
            "pc_local_file".to_string()
        )
    );
}

#[tokio::test]
async fn prepare_upload_allocates_unique_name_when_file_already_exists() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let service = DriveUploaderService::new(SqlUploaderStore::new(pool.clone()));
    let base_command = |id: &str, task_id: &str| PrepareUploaderUploadCommand {
        id: id.to_string(),
        task_id: task_id.to_string(),
        tenant_id: "tenant-user".to_string(),
        organization_id: Some("org-user".to_string()),
        actor: UploaderActor::User {
            user_id: "user-001".to_string(),
        },
        app_id: "drive-pc".to_string(),
        app_resource_type: "desktop-file-browser".to_string(),
        app_resource_id: "root".to_string(),
        scene: Some("drive_pc_file_upload".to_string()),
        source: Some("pc_local_file".to_string()),
        upload_profile_code: "generic".to_string(),
        file_fingerprint: format!("fp-{task_id}"),
        original_file_name: "report.txt".to_string(),
        content_type: "text/plain".to_string(),
        content_length: 42,
        chunk_size_bytes: 8,
        target: UploaderTarget::AutoUploadSpace {
            parent_node_id: None,
        },
        retention: UploaderRetention::LongTerm,
        operator_id: "user-001".to_string(),
        now_epoch_ms: 1_800_000_000_000,
    };

    let first = service
        .prepare_upload(base_command("upload-item-dup-1", "task-dup-1"))
        .await
        .expect("first upload should be prepared");
    assert_eq!(first.original_file_name, "report.txt");

    sqlx::query("UPDATE dr_drive_node SET content_state='ready', head_content_type='text/plain', head_content_type_group='text', head_content_length=42, head_version_no=1, file_extension='txt', head_checksum_sha256_hex='sha256:0000000000000000000000000000000000000000000000000000000000000000' WHERE id=$1")
        .bind(&first.node_id)
        .execute(&pool)
        .await
        .expect("first upload node should be marked ready");

    let second = service
        .prepare_upload(base_command("upload-item-dup-2", "task-dup-2"))
        .await
        .expect("duplicate upload should be prepared with a unique name");
    assert_eq!(second.original_file_name, "report (1).txt");

    let node_name: String = sqlx::query_scalar("SELECT node_name FROM dr_drive_node WHERE id=$1")
        .bind(&second.node_id)
        .fetch_one(&pool)
        .await
        .expect("second upload node name should be queryable");
    assert_eq!(node_name, "report (1).txt");
}

#[tokio::test]
async fn prepare_upload_allocates_unique_name_for_stale_uploading_node() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let service = DriveUploaderService::new(SqlUploaderStore::new(pool.clone()));

    let first = service
        .prepare_upload(PrepareUploaderUploadCommand {
            id: "upload-item-stale-1".to_string(),
            task_id: "task-stale-1".to_string(),
            tenant_id: "tenant-stale".to_string(),
            organization_id: None,
            actor: UploaderActor::User {
                user_id: "user-stale".to_string(),
            },
            app_id: "drive-pc".to_string(),
            app_resource_type: "desktop-file-browser".to_string(),
            app_resource_id: "root".to_string(),
            scene: Some("drive_pc_file_upload".to_string()),
            source: Some("pc_local_file".to_string()),
            upload_profile_code: "generic".to_string(),
            file_fingerprint: "fp-stale-1".to_string(),
            original_file_name: "draft.txt".to_string(),
            content_type: "text/plain".to_string(),
            content_length: 12,
            chunk_size_bytes: 8,
            target: UploaderTarget::AutoUploadSpace {
                parent_node_id: None,
            },
            retention: UploaderRetention::LongTerm,
            operator_id: "user-stale".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect("stale upload should be prepared");

    let second = service
        .prepare_upload(PrepareUploaderUploadCommand {
            id: "upload-item-stale-2".to_string(),
            task_id: "task-stale-2".to_string(),
            tenant_id: "tenant-stale".to_string(),
            organization_id: None,
            actor: UploaderActor::User {
                user_id: "user-stale".to_string(),
            },
            app_id: "drive-pc".to_string(),
            app_resource_type: "desktop-file-browser".to_string(),
            app_resource_id: "root".to_string(),
            scene: Some("drive_pc_file_upload".to_string()),
            source: Some("pc_local_file".to_string()),
            upload_profile_code: "generic".to_string(),
            file_fingerprint: "fp-stale-2".to_string(),
            original_file_name: "draft.txt".to_string(),
            content_type: "text/plain".to_string(),
            content_length: 12,
            chunk_size_bytes: 8,
            target: UploaderTarget::AutoUploadSpace {
                parent_node_id: None,
            },
            retention: UploaderRetention::LongTerm,
            operator_id: "user-stale".to_string(),
            now_epoch_ms: 1_800_000_000_001,
        })
        .await
        .expect("retry upload should not fail on stale uploading node name");
    assert_eq!(second.original_file_name, "draft (1).txt");
    assert_ne!(second.node_id, first.node_id);
}

#[tokio::test]
async fn prepare_anonymous_upload_uses_app_owned_upload_space() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let service = DriveUploaderService::new(SqlUploaderStore::new(pool.clone()));

    let prepared = service
        .prepare_upload(PrepareUploaderUploadCommand {
            id: "upload-item-anon".to_string(),
            task_id: "task-anon".to_string(),
            tenant_id: "tenant-anon".to_string(),
            organization_id: Some("org-anon".to_string()),
            actor: UploaderActor::Anonymous {
                anonymous_id: "anon-session-001".to_string(),
            },
            app_id: "drive-public".to_string(),
            app_resource_type: "public-form".to_string(),
            app_resource_id: "form-001".to_string(),
            scene: Some("anonymous_public_upload".to_string()),
            source: Some("public_browser".to_string()),
            upload_profile_code: "generic".to_string(),
            file_fingerprint: "fp-anon".to_string(),
            original_file_name: "anonymous.bin".to_string(),
            content_type: "application/octet-stream".to_string(),
            content_length: 1,
            chunk_size_bytes: 8,
            target: UploaderTarget::AutoUploadSpace {
                parent_node_id: None,
            },
            retention: UploaderRetention::Temporary {
                ttl_seconds: 60,
                cleanup_action: "soft_delete".to_string(),
                hard_delete_after_seconds: Some(120),
            },
            operator_id: "anon-session-001".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect("anonymous uploader task should be prepared");

    assert_eq!(prepared.actor_type, "anonymous");
    assert_eq!(prepared.user_id, None);
    assert_eq!(
        prepared.retention_expires_at_epoch_ms,
        Some(1_800_000_060_000)
    );

    let owner: (String, String, String) = sqlx::query_as(
        "SELECT owner_subject_type, owner_subject_id, space_type
         FROM dr_drive_space
         WHERE id=$1",
    )
    .bind(&prepared.space_id)
    .fetch_one(&pool)
    .await
    .expect("anonymous upload space should be queryable");
    assert_eq!(
        owner,
        (
            "app".to_string(),
            "app:drive-public:anonymous".to_string(),
            "app_upload".to_string()
        )
    );

    let object_key: String = sqlx::query_scalar(
        "SELECT object_key
         FROM dr_drive_upload_session
         WHERE id=$1",
    )
    .bind(
        prepared
            .upload_session_id
            .as_deref()
            .expect("anonymous prepare should create upload session id"),
    )
    .fetch_one(&pool)
    .await
    .expect("anonymous upload session object key should be queryable");
    assert!(
        object_key.contains("/tenants/tenant-anon/"),
        "anonymous upload key should include tenant locator: {object_key}"
    );
    assert!(
        object_key.contains("/spaces/"),
        "anonymous upload key should include space locator: {object_key}"
    );
    assert!(
        object_key.contains("/actors/anonymous/"),
        "anonymous upload key should include actor type locator: {object_key}"
    );
    assert!(
        object_key.contains("/scene/anonymous_public_upload/"),
        "anonymous upload key should include scene locator: {object_key}"
    );
    assert!(
        object_key.contains("/source/public_browser/"),
        "anonymous upload key should include source locator: {object_key}"
    );
    assert!(
        object_key.contains("/profile/generic/"),
        "anonymous upload key should include profile locator: {object_key}"
    );
    assert!(
        object_key.contains("/dt/2027-01-15/"),
        "anonymous upload key should include date locator: {object_key}"
    );
}

#[tokio::test]
async fn prepare_im_upload_uses_im_space_type_for_auto_upload_space() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let service = DriveUploaderService::new(SqlUploaderStore::new(pool.clone()));

    let prepared = service
        .prepare_upload(PrepareUploaderUploadCommand {
            id: "upload-item-im".to_string(),
            task_id: "task-im".to_string(),
            tenant_id: "tenant-im".to_string(),
            organization_id: Some("org-im".to_string()),
            actor: UploaderActor::User {
                user_id: "user-im".to_string(),
            },
            app_id: "chat".to_string(),
            app_resource_type: "im_conversation".to_string(),
            app_resource_id: "conversation-001".to_string(),
            scene: Some("im".to_string()),
            source: Some("chat_message".to_string()),
            upload_profile_code: "attachment".to_string(),
            file_fingerprint: "fp-im".to_string(),
            original_file_name: "image.png".to_string(),
            content_type: "image/png".to_string(),
            content_length: 42,
            chunk_size_bytes: 8,
            target: UploaderTarget::AutoUploadSpace {
                parent_node_id: None,
            },
            retention: UploaderRetention::LongTerm,
            operator_id: "user-im".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect("IM uploader task should be prepared");

    let owner: (String, String, String, String) = sqlx::query_as(
        "SELECT owner_subject_type, owner_subject_id, space_type, display_name
         FROM dr_drive_space
         WHERE id=$1",
    )
    .bind(&prepared.space_id)
    .fetch_one(&pool)
    .await
    .expect("IM upload space should be queryable");
    assert_eq!(
        owner,
        (
            "user".to_string(),
            "user-im".to_string(),
            "im".to_string(),
            "IM".to_string()
        )
    );

    let usage_context: (String, String, String) = sqlx::query_as(
        "SELECT scene, source, upload_profile_code
         FROM dr_drive_upload_item
         WHERE id=$1",
    )
    .bind(&prepared.id)
    .fetch_one(&pool)
    .await
    .expect("IM upload item usage context should be queryable");
    assert_eq!(
        usage_context,
        (
            "im".to_string(),
            "chat_message".to_string(),
            "attachment".to_string()
        )
    );
}

#[tokio::test]
async fn prepare_rtc_upload_uses_rtc_space_type_for_auto_upload_space() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let service = DriveUploaderService::new(SqlUploaderStore::new(pool.clone()));

    let prepared = service
        .prepare_upload(PrepareUploaderUploadCommand {
            id: "upload-item-rtc".to_string(),
            task_id: "task-rtc".to_string(),
            tenant_id: "tenant-rtc".to_string(),
            organization_id: Some("org-rtc".to_string()),
            actor: UploaderActor::User {
                user_id: "user-rtc".to_string(),
            },
            app_id: "sdkwork-rtc".to_string(),
            app_resource_type: "rtc_recording".to_string(),
            app_resource_id: "rtc-session-001".to_string(),
            scene: Some("rtc".to_string()),
            source: Some("provider_agora".to_string()),
            upload_profile_code: "video".to_string(),
            file_fingerprint: "fp-rtc".to_string(),
            original_file_name: "recording.mp4".to_string(),
            content_type: "video/mp4".to_string(),
            content_length: 42,
            chunk_size_bytes: 8,
            target: UploaderTarget::AutoUploadSpace {
                parent_node_id: None,
            },
            retention: UploaderRetention::LongTerm,
            operator_id: "user-rtc".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect("RTC uploader task should be prepared");

    let owner: (String, String, String, String) = sqlx::query_as(
        "SELECT owner_subject_type, owner_subject_id, space_type, display_name
         FROM dr_drive_space
         WHERE id=$1",
    )
    .bind(&prepared.space_id)
    .fetch_one(&pool)
    .await
    .expect("RTC upload space should be queryable");
    assert_eq!(
        owner,
        (
            "user".to_string(),
            "user-rtc".to_string(),
            "rtc".to_string(),
            "RTC Records".to_string()
        )
    );

    let usage_context: (String, String, String) = sqlx::query_as(
        "SELECT scene, source, upload_profile_code
         FROM dr_drive_upload_item
         WHERE id=$1",
    )
    .bind(&prepared.id)
    .fetch_one(&pool)
    .await
    .expect("RTC upload item usage context should be queryable");
    assert_eq!(
        usage_context,
        (
            "rtc".to_string(),
            "provider_agora".to_string(),
            "video".to_string()
        )
    );
}

#[tokio::test]
async fn prepare_upload_to_target_space_requires_owner_or_writer_permission() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_space(
        &pool,
        "space-owned-by-user",
        "tenant-target-permission",
        "user",
        "user-owner",
        "team",
    )
    .await;
    seed_folder(
        &pool,
        "folder-owned-by-user",
        "tenant-target-permission",
        "space-owned-by-user",
        None,
        "Target",
        "user-owner",
    )
    .await;
    let service = DriveUploaderService::new(SqlUploaderStore::new(pool.clone()));

    let denied = service
        .prepare_upload(PrepareUploaderUploadCommand {
            id: "upload-item-denied-space".to_string(),
            task_id: "task-denied-space".to_string(),
            tenant_id: "tenant-target-permission".to_string(),
            organization_id: None,
            actor: UploaderActor::User {
                user_id: "user-writer".to_string(),
            },
            app_id: "drive-pc".to_string(),
            app_resource_type: "desktop-file-browser".to_string(),
            app_resource_id: "team-space".to_string(),
            scene: Some("team_upload".to_string()),
            source: Some("pc_local_file".to_string()),
            upload_profile_code: "generic".to_string(),
            file_fingerprint: "fp-denied-space".to_string(),
            original_file_name: "denied.txt".to_string(),
            content_type: "text/plain".to_string(),
            content_length: 10,
            chunk_size_bytes: 10,
            target: UploaderTarget::Space {
                space_id: "space-owned-by-user".to_string(),
                parent_node_id: Some("folder-owned-by-user".to_string()),
                share_token: None,
            },
            retention: UploaderRetention::LongTerm,
            operator_id: "user-writer".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect_err("non-owner without writer permission must not upload into target space");

    assert!(
        matches!(denied, DriveServiceError::PermissionDenied(_)),
        "unexpected denial error: {denied:?}"
    );

    let leaked_nodes: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-target-permission'
           AND id='node-upload-item-denied-space'",
    )
    .fetch_one(&pool)
    .await
    .expect("node leak count should be queryable");
    assert_eq!(leaked_nodes, 0, "permission denial must not create nodes");

    seed_permission(
        &pool,
        "permission-writer-folder",
        "tenant-target-permission",
        "folder-owned-by-user",
        "user",
        "user-writer",
        "writer",
    )
    .await;

    let prepared = service
        .prepare_upload(PrepareUploaderUploadCommand {
            id: "upload-item-allowed-space".to_string(),
            task_id: "task-allowed-space".to_string(),
            tenant_id: "tenant-target-permission".to_string(),
            organization_id: None,
            actor: UploaderActor::User {
                user_id: "user-writer".to_string(),
            },
            app_id: "drive-pc".to_string(),
            app_resource_type: "desktop-file-browser".to_string(),
            app_resource_id: "team-space".to_string(),
            scene: Some("team_upload".to_string()),
            source: Some("pc_local_file".to_string()),
            upload_profile_code: "generic".to_string(),
            file_fingerprint: "fp-allowed-space".to_string(),
            original_file_name: "allowed.txt".to_string(),
            content_type: "text/plain".to_string(),
            content_length: 10,
            chunk_size_bytes: 10,
            target: UploaderTarget::Space {
                space_id: "space-owned-by-user".to_string(),
                parent_node_id: Some("folder-owned-by-user".to_string()),
                share_token: None,
            },
            retention: UploaderRetention::LongTerm,
            operator_id: "user-writer".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect("writer permission should allow upload into target folder");

    assert_eq!(prepared.space_id, "space-owned-by-user");
    assert_eq!(prepared.node_id, "node-upload-item-allowed-space");
}

#[tokio::test]
async fn prepare_anonymous_upload_to_explicit_space_requires_public_writer_share() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_space(
        &pool,
        "space-public-upload",
        "tenant-public-upload",
        "user",
        "user-owner",
        "team",
    )
    .await;
    seed_folder(
        &pool,
        "folder-public-upload",
        "tenant-public-upload",
        "space-public-upload",
        None,
        "Drop Box",
        "user-owner",
    )
    .await;
    let service = DriveUploaderService::new(SqlUploaderStore::new(pool.clone()));

    let denied = service
        .prepare_upload(PrepareUploaderUploadCommand {
            id: "upload-item-anon-denied-space".to_string(),
            task_id: "task-anon-denied-space".to_string(),
            tenant_id: "tenant-public-upload".to_string(),
            organization_id: None,
            actor: UploaderActor::Anonymous {
                anonymous_id: "anon-space".to_string(),
            },
            app_id: "drive-public".to_string(),
            app_resource_type: "public-form".to_string(),
            app_resource_id: "form-001".to_string(),
            scene: Some("anonymous_public_upload".to_string()),
            source: Some("public_browser".to_string()),
            upload_profile_code: "generic".to_string(),
            file_fingerprint: "fp-anon-denied-space".to_string(),
            original_file_name: "denied.bin".to_string(),
            content_type: "application/octet-stream".to_string(),
            content_length: 1,
            chunk_size_bytes: 1,
            target: UploaderTarget::Space {
                space_id: "space-public-upload".to_string(),
                parent_node_id: Some("folder-public-upload".to_string()),
                share_token: None,
            },
            retention: UploaderRetention::Temporary {
                ttl_seconds: 60,
                cleanup_action: "soft_delete".to_string(),
                hard_delete_after_seconds: None,
            },
            operator_id: "anon-space".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect_err("anonymous explicit-space upload without public writer grant must be denied");

    assert!(
        matches!(denied, DriveServiceError::PermissionDenied(_)),
        "unexpected anonymous denial error: {denied:?}"
    );

    seed_share_link(
        &pool,
        "share-public-writer",
        "tenant-public-upload",
        "folder-public-upload",
        "public-share-token",
        "writer",
    )
    .await;

    let prepared = service
        .prepare_upload(PrepareUploaderUploadCommand {
            id: "upload-item-anon-allowed-space".to_string(),
            task_id: "task-anon-allowed-space".to_string(),
            tenant_id: "tenant-public-upload".to_string(),
            organization_id: None,
            actor: UploaderActor::Anonymous {
                anonymous_id: "anon-space".to_string(),
            },
            app_id: "drive-public".to_string(),
            app_resource_type: "public-form".to_string(),
            app_resource_id: "form-001".to_string(),
            scene: Some("anonymous_public_upload".to_string()),
            source: Some("public_browser".to_string()),
            upload_profile_code: "generic".to_string(),
            file_fingerprint: "fp-anon-allowed-space".to_string(),
            original_file_name: "allowed.bin".to_string(),
            content_type: "application/octet-stream".to_string(),
            content_length: 1,
            chunk_size_bytes: 1,
            target: UploaderTarget::Space {
                space_id: "space-public-upload".to_string(),
                parent_node_id: Some("folder-public-upload".to_string()),
                share_token: Some("public-share-token".to_string()),
            },
            retention: UploaderRetention::Temporary {
                ttl_seconds: 60,
                cleanup_action: "soft_delete".to_string(),
                hard_delete_after_seconds: None,
            },
            operator_id: "anon-space".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect("public writer share should allow anonymous explicit-space upload");

    assert_eq!(prepared.space_id, "space-public-upload");
    assert_eq!(prepared.node_id, "node-upload-item-anon-allowed-space");
}

#[tokio::test]
async fn prepare_video_upload_selects_video_profile_and_temporary_retention() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let service = DriveUploaderService::new(SqlUploaderStore::new(pool));

    let prepared = service
        .prepare_video_upload(PrepareUploaderUploadCommand {
            id: "upload-item-video".to_string(),
            task_id: "task-video".to_string(),
            tenant_id: "tenant-video".to_string(),
            organization_id: None,
            actor: UploaderActor::User {
                user_id: "user-video".to_string(),
            },
            app_id: "studio".to_string(),
            app_resource_type: "render-job".to_string(),
            app_resource_id: "job-001".to_string(),
            scene: Some("ai_generated_video".to_string()),
            source: Some("server_render".to_string()),
            upload_profile_code: "generic".to_string(),
            file_fingerprint: "fp-video".to_string(),
            original_file_name: "render.mp4".to_string(),
            content_type: "video/mp4".to_string(),
            content_length: 17,
            chunk_size_bytes: 8,
            target: UploaderTarget::AutoUploadSpace {
                parent_node_id: None,
            },
            retention: UploaderRetention::Temporary {
                ttl_seconds: 7,
                cleanup_action: "hard_delete".to_string(),
                hard_delete_after_seconds: None,
            },
            operator_id: "user-video".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect("video uploader task should be prepared");

    assert_eq!(prepared.upload_profile_code, "video");
    assert_eq!(prepared.content_type_group, "video");
    assert_eq!(prepared.scene.as_deref(), Some("ai_generated_video"));
    assert_eq!(prepared.retention_mode, "temporary");
    assert_eq!(
        prepared.retention_expires_at_epoch_ms,
        Some(1_800_000_007_000)
    );
    assert_eq!(prepared.cleanup_action.as_deref(), Some("hard_delete"));
}

#[tokio::test]
async fn prepare_ai_generated_upload_targets_ai_generated_space_for_user_and_anonymous_actor() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let service = DriveUploaderService::new(SqlUploaderStore::new(pool.clone()));

    let user_prepared = service
        .prepare_image_upload(PrepareUploaderUploadCommand {
            id: "upload-item-ai-user".to_string(),
            task_id: "task-ai-user".to_string(),
            tenant_id: "tenant-ai".to_string(),
            organization_id: Some("org-ai".to_string()),
            actor: UploaderActor::User {
                user_id: "user-ai".to_string(),
            },
            app_id: "sdkwork-video".to_string(),
            app_resource_type: "video_generation_output".to_string(),
            app_resource_id: "generation-001-output-000".to_string(),
            scene: Some("ai_generated_image".to_string()),
            source: Some("provider_result".to_string()),
            upload_profile_code: "generic".to_string(),
            file_fingerprint: "fp-ai-user".to_string(),
            original_file_name: "generation-001-0.png".to_string(),
            content_type: "image/png".to_string(),
            content_length: 1024,
            chunk_size_bytes: 512,
            target: UploaderTarget::AiGeneratedSpace {
                parent_node_id: None,
            },
            retention: UploaderRetention::LongTerm,
            operator_id: "user-ai".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect("AI generated user upload should be prepared");

    let user_space: (String, String, String) = sqlx::query_as(
        "SELECT owner_subject_type, owner_subject_id, space_type
         FROM dr_drive_space
         WHERE id=$1",
    )
    .bind(&user_prepared.space_id)
    .fetch_one(&pool)
    .await
    .expect("AI generated user space should be queryable");
    assert_eq!(
        user_space,
        (
            "user".to_string(),
            "user-ai".to_string(),
            "ai_generated".to_string()
        )
    );

    let anonymous_prepared = service
        .prepare_video_upload(PrepareUploaderUploadCommand {
            id: "upload-item-ai-anon".to_string(),
            task_id: "task-ai-anon".to_string(),
            tenant_id: "tenant-ai".to_string(),
            organization_id: Some("org-ai".to_string()),
            actor: UploaderActor::Anonymous {
                anonymous_id: "anon-ai".to_string(),
            },
            app_id: "sdkwork-video".to_string(),
            app_resource_type: "video_generation_output".to_string(),
            app_resource_id: "generation-001-output-001".to_string(),
            scene: Some("ai_generated_video".to_string()),
            source: Some("provider_result".to_string()),
            upload_profile_code: "generic".to_string(),
            file_fingerprint: "fp-ai-anon".to_string(),
            original_file_name: "generation-001-1.mp4".to_string(),
            content_type: "video/mp4".to_string(),
            content_length: 2048,
            chunk_size_bytes: 1024,
            target: UploaderTarget::AiGeneratedSpace {
                parent_node_id: None,
            },
            retention: UploaderRetention::LongTerm,
            operator_id: "anon-ai".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect("AI generated anonymous upload should be prepared");

    let anonymous_space: (String, String, String) = sqlx::query_as(
        "SELECT owner_subject_type, owner_subject_id, space_type
         FROM dr_drive_space
         WHERE id=$1",
    )
    .bind(&anonymous_prepared.space_id)
    .fetch_one(&pool)
    .await
    .expect("AI generated anonymous space should be queryable");
    assert_eq!(
        anonymous_space,
        (
            "app".to_string(),
            "app:sdkwork-video:anonymous".to_string(),
            "ai_generated".to_string()
        )
    );
}

#[tokio::test]
async fn mark_part_uploaded_is_idempotent_and_updates_upload_item_counters() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let service = DriveUploaderService::new(SqlUploaderStore::new(pool.clone()));
    let prepared = service
        .prepare_upload(PrepareUploaderUploadCommand {
            id: "upload-item-part".to_string(),
            task_id: "task-part".to_string(),
            tenant_id: "tenant-part".to_string(),
            organization_id: None,
            actor: UploaderActor::User {
                user_id: "user-part".to_string(),
            },
            app_id: "drive-pc".to_string(),
            app_resource_type: "desktop-file-browser".to_string(),
            app_resource_id: "root".to_string(),
            scene: None,
            source: None,
            upload_profile_code: "generic".to_string(),
            file_fingerprint: "fp-part".to_string(),
            original_file_name: "part.bin".to_string(),
            content_type: "application/octet-stream".to_string(),
            content_length: 9,
            chunk_size_bytes: 4,
            target: UploaderTarget::AutoUploadSpace {
                parent_node_id: None,
            },
            retention: UploaderRetention::LongTerm,
            operator_id: "user-part".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect("task should be prepared");

    let command = MarkUploaderPartUploadedCommand {
        id: "upload-part-001".to_string(),
        tenant_id: "tenant-part".to_string(),
        upload_item_id: prepared.id.clone(),
        upload_session_id: prepared
            .upload_session_id
            .clone()
            .expect("prepare should create an upload session id"),
        part_no: 1,
        offset_bytes: 0,
        size_bytes: 4,
        etag: "etag-part-1".to_string(),
        checksum_sha256_hex: None,
        uploaded_at_epoch_ms: 1_800_000_001_000,
    };

    let first = service
        .mark_part_uploaded(command.clone())
        .await
        .expect("first uploaded part should be recorded");
    let second = service
        .mark_part_uploaded(command)
        .await
        .expect("duplicate uploaded part should be idempotent");

    assert_eq!(first.id, second.id);
    assert_eq!(first.etag, second.etag);

    let counters: (i64, i64) = sqlx::query_as(
        "SELECT uploaded_parts_count, uploaded_bytes
         FROM dr_drive_upload_item
         WHERE id=$1",
    )
    .bind(&prepared.id)
    .fetch_one(&pool)
    .await
    .expect("upload item counters should be queryable");
    assert_eq!(counters, (1, 4));
}

#[tokio::test]
async fn complete_stored_upload_marks_generated_object_ready_and_is_idempotent() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let service = DriveUploaderService::new(SqlUploaderStore::new(pool.clone()));
    let prepared = service
        .prepare_upload(PrepareUploaderUploadCommand {
            id: "upload-item-ai-complete".to_string(),
            task_id: "task-ai-complete".to_string(),
            tenant_id: "tenant-ai-complete".to_string(),
            organization_id: Some("org-ai-complete".to_string()),
            actor: UploaderActor::User {
                user_id: "user-ai-complete".to_string(),
            },
            app_id: "sdkwork-music".to_string(),
            app_resource_type: "music_ai_generation_variant".to_string(),
            app_resource_id: "generation-001-output-001".to_string(),
            scene: Some("music_ai_generation".to_string()),
            source: Some("ai_generated".to_string()),
            upload_profile_code: "audio".to_string(),
            file_fingerprint:
                "sha256:00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
                    .to_string(),
            original_file_name: "generation.mp3".to_string(),
            content_type: "audio/mpeg".to_string(),
            content_length: 13,
            chunk_size_bytes: 8,
            target: UploaderTarget::AiGeneratedSpace {
                parent_node_id: None,
            },
            retention: UploaderRetention::LongTerm,
            operator_id: "user-ai-complete".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect("AI generated upload should be prepared");

    let completed = service
        .complete_stored_upload(CompleteStoredUploaderUploadCommand {
            tenant_id: "tenant-ai-complete".to_string(),
            upload_item_id: prepared.id.clone(),
            upload_session_id: prepared
                .upload_session_id
                .clone()
                .expect("prepare should create an upload session"),
            content_type: "audio/mpeg".to_string(),
            content_length: 13,
            checksum_sha256_hex:
                "sha256:00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
                    .to_string(),
            uploaded_parts_count: 1,
            operator_id: "user-ai-complete".to_string(),
        })
        .await
        .expect("stored generated object should complete drive upload lifecycle");
    let replayed = service
        .complete_stored_upload(CompleteStoredUploaderUploadCommand {
            tenant_id: "tenant-ai-complete".to_string(),
            upload_item_id: prepared.id.clone(),
            upload_session_id: prepared
                .upload_session_id
                .clone()
                .expect("prepare should create an upload session"),
            content_type: "audio/mpeg".to_string(),
            content_length: 13,
            checksum_sha256_hex:
                "sha256:00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
                    .to_string(),
            uploaded_parts_count: 1,
            operator_id: "user-ai-complete".to_string(),
        })
        .await
        .expect("completion replay should return the existing completed upload item");

    assert_eq!(completed.id, replayed.id);
    assert_eq!(completed.status, "completed");
    assert_eq!(replayed.status, "completed");
    assert_eq!(
        completed.checksum_sha256_hex.as_deref(),
        Some("sha256:00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff")
    );
    assert_eq!(completed.uploaded_bytes, 13);
    assert_eq!(completed.uploaded_parts_count, 1);

    let node_state: String = sqlx::query_scalar(
        "SELECT content_state
         FROM dr_drive_node
         WHERE tenant_id=$1 AND id=$2",
    )
    .bind("tenant-ai-complete")
    .bind(&prepared.node_id)
    .fetch_one(&pool)
    .await
    .expect("completed node state should be queryable");
    assert_eq!(node_state, "ready");

    let session_state: String = sqlx::query_scalar(
        "SELECT state
         FROM dr_drive_upload_session
         WHERE tenant_id=$1 AND id=$2",
    )
    .bind("tenant-ai-complete")
    .bind(
        prepared
            .upload_session_id
            .as_deref()
            .expect("prepare should create an upload session"),
    )
    .fetch_one(&pool)
    .await
    .expect("completed upload session state should be queryable");
    assert_eq!(session_state, "completed");

    let storage_object: (String, i64, String, i64, String, String) = sqlx::query_as(
        "SELECT id, version_no, content_type, content_length, checksum_sha256_hex, lifecycle_status
         FROM dr_drive_storage_object
         WHERE tenant_id=$1 AND node_id=$2",
    )
    .bind("tenant-ai-complete")
    .bind(&prepared.node_id)
    .fetch_one(&pool)
    .await
    .expect("completed storage object should be queryable");
    assert_eq!(
        storage_object,
        (
            "session-upload-item-ai-complete-v1".to_string(),
            1,
            "audio/mpeg".to_string(),
            13,
            "sha256:00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff".to_string(),
            "active".to_string()
        )
    );

    let node_versions: Vec<(String, i64, String)> = sqlx::query_as(
        "SELECT id, version_no, storage_object_id
         FROM dr_drive_node_version
         WHERE tenant_id=$1 AND node_id=$2",
    )
    .bind("tenant-ai-complete")
    .bind(&prepared.node_id)
    .fetch_all(&pool)
    .await
    .expect("logical node versions should be queryable");
    assert_eq!(node_versions.len(), 1);
    assert_eq!(node_versions[0].1, 1);
    assert_eq!(node_versions[0].2, "session-upload-item-ai-complete-v1");

    let event_payload: String = sqlx::query_scalar(
        "SELECT payload_json
         FROM dr_drive_domain_outbox
         WHERE tenant_id=$1 AND node_id=$2 AND event_type=$3",
    )
    .bind("tenant-ai-complete")
    .bind(&prepared.node_id)
    .bind(sdkwork_drive_contract::drive::domain_events::node::VERSION_COMMITTED_V1)
    .fetch_one(&pool)
    .await
    .expect("version committed event should be queryable");
    let event: serde_json::Value =
        serde_json::from_str(&event_payload).expect("event payload should be valid JSON");
    assert_eq!(event["type"], "drive.node.version.committed.v1");
    assert_eq!(event["specversion"], "1.0");
    assert_eq!(event["data"]["driveVersionId"], node_versions[0].0);
    assert_eq!(event["data"]["versionNo"], "1");
    assert_eq!(event["data"]["contentLength"], "13");
    assert_eq!(event["data"]["rootScopes"], serde_json::json!([]));
    assert!(event["data"].get("objectKey").is_none());

    let sensitive_operations: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_file_sensitive_operation
         WHERE tenant_id=$1 AND upload_item_id=$2 AND operation_type='upload_completed'",
    )
    .bind("tenant-ai-complete")
    .bind(&prepared.id)
    .fetch_one(&pool)
    .await
    .expect("upload completion sensitive operation count should be queryable");
    assert_eq!(sensitive_operations, 1);
}

#[tokio::test]
async fn upload_bytes_writes_object_and_completes_ai_generated_upload() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let service = DriveUploaderService::new(SqlUploaderStore::new(pool.clone()));
    let object_store =
        LocalDriveObjectStore::new(unique_temp_storage_root("drive-uploader-upload-bytes"));
    let body = b"generated video bytes".to_vec();

    let completed = service
        .upload_bytes(
            &object_store,
            UploadBytesCommand {
                prepare: PrepareUploaderUploadCommand {
                    id: "upload-item-ai-bytes".to_string(),
                    task_id: "task-ai-bytes".to_string(),
                    tenant_id: "tenant-ai-bytes".to_string(),
                    organization_id: Some("org-ai-bytes".to_string()),
                    actor: UploaderActor::User {
                        user_id: "user-ai-bytes".to_string(),
                    },
                    app_id: "sdkwork-video".to_string(),
                    app_resource_type: "video_generation_output".to_string(),
                    app_resource_id: "generation-001-output-000-artifact-video".to_string(),
                    scene: Some("ai_generated_video".to_string()),
                    source: Some("provider_result".to_string()),
                    upload_profile_code: "video".to_string(),
                    file_fingerprint: "video-generation:generation-001:output:0:artifact:video"
                        .to_string(),
                    original_file_name: "generation-001-output-0-video.mp4".to_string(),
                    content_type: "video/mp4".to_string(),
                    content_length: body.len() as i64,
                    chunk_size_bytes: 8 * 1024 * 1024,
                    target: UploaderTarget::AiGeneratedSpace {
                        parent_node_id: None,
                    },
                    retention: UploaderRetention::LongTerm,
                    operator_id: "user-ai-bytes".to_string(),
                    now_epoch_ms: 1_800_000_000_000,
                },
                body,
                uploaded_at_epoch_ms: 1_800_000_001_000,
            },
        )
        .await
        .expect("byte upload should write the object and complete the uploader lifecycle");

    assert_eq!(completed.status, "completed");
    assert_eq!(completed.upload_profile_code, "video");
    assert_eq!(completed.uploaded_parts_count, 1);
    assert_eq!(
        completed.uploaded_bytes,
        "generated video bytes".len() as i64
    );
    assert_eq!(
        completed.checksum_sha256_hex.as_deref(),
        Some("sha256:69775371b43a16e47416f7c6003055cd69404150b9112967278c9251b1c6d32e")
    );

    let object_bucket = completed
        .object_bucket
        .clone()
        .expect("completed upload should expose bucket");
    let object_key = completed
        .object_key
        .clone()
        .expect("completed upload should expose object key");
    let head = object_store
        .head_object(HeadObjectRequest {
            locator: DriveObjectLocator {
                bucket: object_bucket,
                object_key,
            },
        })
        .await
        .expect("uploaded object should exist in the object store");
    assert_eq!(head.content_length, "generated video bytes".len() as u64);
    assert_eq!(head.content_type.as_deref(), Some("video/mp4"));
    assert_eq!(
        head.checksum_sha256_hex.as_deref(),
        Some("69775371b43a16e47416f7c6003055cd69404150b9112967278c9251b1c6d32e")
    );

    let space_type: String =
        sqlx::query_scalar("SELECT space_type FROM dr_drive_space WHERE id=$1")
            .bind(&completed.space_id)
            .fetch_one(&pool)
            .await
            .expect("AI generated upload space should be queryable");
    assert_eq!(space_type, "ai_generated");
}

#[tokio::test]
async fn complete_stored_upload_quarantine_trashes_node_records_sensitive_operation_and_aborts_session(
) {
    std::env::set_var("SDKWORK_DRIVE_CONTENT_SCAN_MODE", "quarantine");
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let service = DriveUploaderService::new(SqlUploaderStore::new(pool.clone()));
    let prepared = service
        .prepare_upload(PrepareUploaderUploadCommand {
            id: "upload-item-quarantine".to_string(),
            task_id: "task-quarantine".to_string(),
            tenant_id: "tenant-quarantine".to_string(),
            organization_id: Some("org-quarantine".to_string()),
            actor: UploaderActor::User {
                user_id: "user-quarantine".to_string(),
            },
            app_id: "drive-pc".to_string(),
            app_resource_type: "desktop-file-browser".to_string(),
            app_resource_id: "root".to_string(),
            scene: Some("user_document_upload".to_string()),
            source: Some("pc_local_file".to_string()),
            upload_profile_code: "generic".to_string(),
            file_fingerprint: "fp-quarantine".to_string(),
            original_file_name: "malware.exe".to_string(),
            content_type: "application/x-msdownload".to_string(),
            content_length: 128,
            chunk_size_bytes: 64,
            target: UploaderTarget::AutoUploadSpace {
                parent_node_id: None,
            },
            retention: UploaderRetention::LongTerm,
            operator_id: "user-quarantine".to_string(),
            now_epoch_ms: 1_800_000_000_000,
        })
        .await
        .expect("quarantine upload should be prepared");

    let upload_session_id = prepared
        .upload_session_id
        .clone()
        .expect("prepare should create upload session");

    sqlx::query("UPDATE dr_drive_space SET space_type='website' WHERE id=$1")
        .bind(&prepared.space_id)
        .execute(&pool)
        .await
        .expect("quarantine test Space should become website-capable");
    sqlx::query("UPDATE dr_drive_node SET space_type='website' WHERE id=$1")
        .bind(&prepared.node_id)
        .execute(&pool)
        .await
        .expect("quarantine test node should use website Space type");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, space_type, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
         ) VALUES (
            'quarantine-website-root', 'tenant-quarantine', $1, 'website', NULL,
            'folder', 'public', 'ready', 'active', 1,
            'user-quarantine', 'user-quarantine'
         )",
    )
    .bind(&prepared.space_id)
    .execute(&pool)
    .await
    .expect("quarantine WebsiteRoot folder should be inserted");
    sqlx::query(
        "UPDATE dr_drive_node
         SET parent_node_id='quarantine-website-root'
         WHERE tenant_id='tenant-quarantine' AND id=$1",
    )
    .bind(&prepared.node_id)
    .execute(&pool)
    .await
    .expect("quarantine upload should be placed in the active website generation");
    sqlx::query(
        "INSERT INTO dr_drive_website_root (
            id, uuid, tenant_id, space_id, root_key, display_name,
            source_root_mode, selected_folder_node_id, selector_key, content_mode,
            active_node_id, active_generation, root_status, last_switch_by,
            version, created_by, updated_by
         ) VALUES (
            'quarantine-root-record', 'quarantine-root-uuid', 'tenant-quarantine', $1,
            'default', 'Default', 'folder', 'quarantine-website-root',
            'folder:quarantine-website-root', 'atomic_generation',
            'quarantine-website-root', 1, 'active', 'user-quarantine', 1,
            'user-quarantine', 'user-quarantine'
         )",
    )
    .bind(&prepared.space_id)
    .execute(&pool)
    .await
    .expect("atomic quarantine WebsiteRoot should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_website_root_generation (
            id, tenant_id, website_root_id, generation_no, root_node_id,
            file_count, total_bytes, generation_status, activated_by
         ) VALUES (
            'quarantine-generation', 'tenant-quarantine', 'quarantine-root-record', 1,
            'quarantine-website-root', 1, 128, 'current', 'user-quarantine'
         )",
    )
    .execute(&pool)
    .await
    .expect("active quarantine website generation should be inserted");

    let error = service
        .complete_stored_upload(CompleteStoredUploaderUploadCommand {
            tenant_id: "tenant-quarantine".to_string(),
            upload_item_id: prepared.id.clone(),
            upload_session_id: upload_session_id.clone(),
            content_type: "application/x-msdownload".to_string(),
            content_length: 128,
            checksum_sha256_hex:
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
            uploaded_parts_count: 2,
            operator_id: "user-quarantine".to_string(),
        })
        .await
        .expect_err("blocked executable should be quarantined");
    assert!(
        matches!(error, DriveServiceError::Validation(message) if message.contains("quarantined"))
    );

    let node_lifecycle: String = sqlx::query_scalar(
        "SELECT lifecycle_status FROM dr_drive_node WHERE tenant_id=$1 AND id=$2",
    )
    .bind(&prepared.tenant_id)
    .bind(&prepared.node_id)
    .fetch_one(&pool)
    .await
    .expect("quarantined node should exist");
    assert_eq!(node_lifecycle, "trashed");

    let upload_item_status: (String, String) =
        sqlx::query_as("SELECT status, cleanup_status FROM dr_drive_upload_item WHERE id=$1")
            .bind(&prepared.id)
            .fetch_one(&pool)
            .await
            .expect("quarantined upload item should exist");
    assert_eq!(
        upload_item_status,
        ("failed".to_string(), "failed".to_string())
    );

    let session_state: String =
        sqlx::query_scalar("SELECT state FROM dr_drive_upload_session WHERE id=$1")
            .bind(&upload_session_id)
            .fetch_one(&pool)
            .await
            .expect("quarantined upload session should exist");
    assert_eq!(session_state, "aborted");

    let sensitive_operation: (String, String) = sqlx::query_as(
        "SELECT operation_type, operation_reason
         FROM dr_drive_file_sensitive_operation
         WHERE upload_item_id=$1",
    )
    .bind(&prepared.id)
    .fetch_one(&pool)
    .await
    .expect("quarantine sensitive operation should be recorded");
    assert_eq!(
        sensitive_operation,
        ("soft_delete".to_string(), "system".to_string())
    );

    let outbox_event: String = sqlx::query_scalar(
        "SELECT event_type FROM dr_drive_domain_outbox WHERE tenant_id=$1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&prepared.tenant_id)
    .fetch_one(&pool)
    .await
    .expect("quarantine outbox event should exist");
    assert_eq!(outbox_event, "drive.object.quarantined");

    let override_audit_action: String = sqlx::query_scalar(
        "SELECT action
         FROM dr_drive_audit_event
         WHERE tenant_id='tenant-quarantine' AND resource_id=$1",
    )
    .bind(&prepared.node_id)
    .fetch_one(&pool)
    .await
    .expect("quarantine system override audit should exist");
    assert_eq!(
        override_audit_action,
        "drive.website_tree.system_override.content_policy_quarantine"
    );

    std::env::remove_var("SDKWORK_DRIVE_CONTENT_SCAN_MODE");
}

fn unique_temp_storage_root(name: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{suffix}"))
}

async fn seed_space(
    pool: &sqlx::PgPool,
    space_id: &str,
    tenant_id: &str,
    owner_subject_type: &str,
    owner_subject_id: &str,
    space_type: &str,
) {
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $5, 'active', 1, $4, $4)",
    )
    .bind(space_id)
    .bind(tenant_id)
    .bind(owner_subject_type)
    .bind(owner_subject_id)
    .bind(space_type)
    .execute(pool)
    .await
    .expect("seed space should succeed");
}

async fn seed_folder(
    pool: &sqlx::PgPool,
    node_id: &str,
    tenant_id: &str,
    space_id: &str,
    parent_node_id: Option<&str>,
    node_name: &str,
    operator_id: &str,
) {
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type,
            node_name, content_state, lifecycle_status, version,
            created_by, updated_by
        ) VALUES ($1, $2, $3, $4, 'folder', $5, 'empty', 'active', 1, $6, $6)",
    )
    .bind(node_id)
    .bind(tenant_id)
    .bind(space_id)
    .bind(parent_node_id)
    .bind(node_name)
    .bind(operator_id)
    .execute(pool)
    .await
    .expect("seed folder should succeed");
}

async fn seed_permission(
    pool: &sqlx::PgPool,
    permission_id: &str,
    tenant_id: &str,
    node_id: &str,
    subject_type: &str,
    subject_id: &str,
    role: &str,
) {
    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 0, 'active', 1, $5, $5)",
    )
    .bind(permission_id)
    .bind(tenant_id)
    .bind(node_id)
    .bind(subject_type)
    .bind(subject_id)
    .bind(role)
    .execute(pool)
    .await
    .expect("seed permission should succeed");
}

async fn seed_share_link(
    pool: &sqlx::PgPool,
    share_link_id: &str,
    tenant_id: &str,
    node_id: &str,
    token: &str,
    role: &str,
) {
    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version,
            created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, NULL, NULL, 0, 'active', 1, 'user-owner', 'user-owner')",
    )
    .bind(share_link_id)
    .bind(tenant_id)
    .bind(node_id)
    .bind(drive_share_token_hash(token))
    .bind(role)
    .execute(pool)
    .await
    .expect("seed share link should succeed");
}
