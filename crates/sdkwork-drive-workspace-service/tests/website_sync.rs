use chrono::{Duration, SecondsFormat, Utc};
use sdkwork_drive_workspace_service::application::maintenance_service::{
    DriveMaintenanceService, SweepExpiredUploadContentCommand,
};
use sdkwork_drive_workspace_service::application::space_service::{
    CreateSpaceCommand, DriveSpaceService,
};
use sdkwork_drive_workspace_service::application::uploader_service::{
    CompleteStoredUploaderUploadCommand, DriveUploaderService, PrepareUploaderUploadCommand,
    UploaderActor, UploaderRetention, UploaderTarget,
};
use sdkwork_drive_workspace_service::application::website_sync_service::{
    AbortWebsiteSyncCommand, ActivateWebsiteGenerationCommand, CreateWebsiteSyncCommand,
    DriveWebsiteSyncService, FinalizeWebsiteSyncCommand,
};
use sdkwork_drive_workspace_service::domain::node_version::{
    CreateDriveNodeVersionCommand, DriveNodeVersionChangeSource, DriveNodeVersionKind,
};
use sdkwork_drive_workspace_service::domain::space::DriveSpaceType;
use sdkwork_drive_workspace_service::domain::website_root::DriveWebsiteContentMode;
use sdkwork_drive_workspace_service::domain::website_sync::{
    validate_website_sync_tree, DriveWebsiteSyncStatus, DriveWebsiteSyncTreeEntry,
};
use sdkwork_drive_workspace_service::infrastructure::sql::begin_transaction_sql;
use sdkwork_drive_workspace_service::infrastructure::sql::maintenance_store::SqlMaintenanceStore;
use sdkwork_drive_workspace_service::infrastructure::sql::managed_website_tree_guard::ensure_managed_website_node_mutation_allowed;
use sdkwork_drive_workspace_service::infrastructure::sql::node_head_metadata::{
    apply_file_node_head_snapshot, FileNodeHeadSnapshot,
};
use sdkwork_drive_workspace_service::infrastructure::sql::node_store::SqlNodeStore;
use sdkwork_drive_workspace_service::infrastructure::sql::node_version_store::SqlDriveNodeVersionStore;
use sdkwork_drive_workspace_service::infrastructure::sql::space_store::SqlSpaceStore;
use sdkwork_drive_workspace_service::infrastructure::sql::uploader_store::SqlUploaderStore;
use sdkwork_drive_workspace_service::infrastructure::sql::website_publishing_maintenance_store::SqlWebsitePublishingMaintenanceStore;
use sdkwork_drive_workspace_service::infrastructure::sql::website_sync_store::SqlWebsiteSyncStore;
use sdkwork_drive_workspace_service::infrastructure::sql::workspace_store::SqlDriveWorkspaceStore;
use sdkwork_drive_workspace_service::ports::node_store::{DriveNodeStore, NewDriveNode};
use sdkwork_drive_workspace_service::ports::node_version_store::DriveNodeVersionStore;
use sdkwork_drive_workspace_service::ports::website_publishing_maintenance::{
    DriveWebsitePublishingMaintenanceStore, WebsiteTreeCleanupKind,
};
use sdkwork_drive_workspace_service::ports::website_sync_store::{
    ActivateValidatedWebsiteSync, DriveWebsiteSyncStore, ValidateDriveWebsiteSync,
};
use sdkwork_drive_workspace_service::ports::workspace_store::{
    DriveWorkspaceStore, NewDriveWorkspaceNodeRecord, NewDriveWorkspaceObjectRecord,
};
use sqlx::PgPool;

#[tokio::test]
async fn atomic_sync_is_idempotent_switches_once_and_rolls_back_as_a_new_generation() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    create_website_space(&pool).await;
    sqlx::query(
        "UPDATE dr_drive_space_website_profile
         SET retained_generation_count=1
         WHERE tenant_id='tenant-sync' AND space_id='space-sync'",
    )
    .execute(&pool)
    .await
    .expect("retained-generation policy should be configurable");
    let (root_uuid, original_root_node_id): (String, String) = sqlx::query_as(
        "SELECT uuid, active_node_id
         FROM dr_drive_website_root
         WHERE tenant_id='tenant-sync' AND space_id='space-sync' AND root_key='default'",
    )
    .fetch_one(&pool)
    .await
    .expect("default WebsiteRoot should exist");
    let service = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(pool.clone()));
    let entries = vec![
        file("index.html", 18, 'a'),
        folder("assets"),
        file("assets/app.js", 24, 'b'),
    ];
    let manifest = validate_website_sync_tree(&entries).expect("test manifest should be valid");
    let create = create_command(&root_uuid, "sync-key-1", &manifest);
    let created = service
        .create_sync(create.clone())
        .await
        .expect("WebsiteSync should be created");
    assert!(created.created);
    assert_eq!(created.sync.status, DriveWebsiteSyncStatus::Created);
    assert_ne!(created.sync.staging_node_id, original_root_node_id);
    assert_node_mutation_guard(&pool, &created.sync.staging_node_id, true).await;

    let replay = service
        .create_sync(create)
        .await
        .expect("identical idempotent request should replay");
    assert!(!replay.created);
    assert_eq!(replay.sync.id, created.sync.id);

    let mut conflict = create_command(&root_uuid, "sync-key-1", &manifest);
    conflict.manifest_total_bytes += 1;
    assert!(service.create_sync(conflict).await.is_err());

    insert_tree(&pool, &created.sync.staging_node_id, &entries).await;
    let activation = service
        .finalize_sync(FinalizeWebsiteSyncCommand {
            tenant_id: "tenant-sync".to_string(),
            website_root_uuid: root_uuid.clone(),
            sync_id: created.sync.id.clone(),
            expected_sync_version: 1,
            operator_id: "user-sync".to_string(),
        })
        .await
        .expect("valid WebsiteSync should activate");
    assert_eq!(activation.sync.status, DriveWebsiteSyncStatus::Completed);
    assert_eq!(activation.sync.uploaded_file_count, 2);
    assert_eq!(activation.sync.uploaded_total_bytes, 42);
    assert_eq!(activation.website_root.active_generation, 2);
    assert_eq!(
        activation.website_root.content_mode,
        DriveWebsiteContentMode::AtomicGeneration
    );
    assert_eq!(
        activation.website_root.active_node_id,
        created.sync.staging_node_id
    );
    assert_node_mutation_guard(&pool, &created.sync.staging_node_id, false).await;
    assert_node_mutation_guard(&pool, &original_root_node_id, false).await;

    let mut next_create = create_command(&root_uuid, "sync-key-2", &manifest);
    next_create.expected_root_version = 2;
    next_create.expected_generation = 2;
    let next_sync = service
        .create_sync(next_create)
        .await
        .expect("next WebsiteSync should be created below the retained source root");
    assert_node_mutation_guard(&pool, &next_sync.sync.staging_node_id, true).await;

    let generations: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT generation_no, root_node_id, generation_status
         FROM dr_drive_website_root_generation
         WHERE website_root_id=$1 ORDER BY generation_no",
    )
    .bind(&activation.website_root.id)
    .fetch_all(&pool)
    .await
    .expect("WebsiteRoot generations should be queryable");
    assert_eq!(
        generations,
        vec![
            (1, original_root_node_id.clone(), "retained".to_string()),
            (
                2,
                created.sync.staging_node_id.clone(),
                "current".to_string()
            ),
        ]
    );

    let event_payload: String = sqlx::query_scalar(
        "SELECT payload_json
         FROM dr_drive_domain_outbox
         WHERE tenant_id='tenant-sync'
           AND event_type='drive.website_root.generation.changed.v1'
         ORDER BY sequence_no DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("generation event should be committed with the switch");
    let event: serde_json::Value =
        serde_json::from_str(&event_payload).expect("generation event should be JSON");
    assert_eq!(event["data"]["websiteRootUuid"], root_uuid);
    assert_eq!(event["data"]["generation"], "2");
    assert_eq!(event["data"]["changeReason"], "SYNC_ACTIVATED");

    let replay_activation = service
        .finalize_sync(FinalizeWebsiteSyncCommand {
            tenant_id: "tenant-sync".to_string(),
            website_root_uuid: root_uuid.clone(),
            sync_id: created.sync.id.clone(),
            expected_sync_version: 1,
            operator_id: "user-sync".to_string(),
        })
        .await
        .expect("completed finalize should replay");
    assert_eq!(replay_activation.website_root.active_generation, 2);
    let generation_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_website_root_generation WHERE website_root_id=$1",
    )
    .bind(&activation.website_root.id)
    .fetch_one(&pool)
    .await
    .expect("generation count should be queryable");
    assert_eq!(generation_count, 2, "finalize replay must not switch twice");

    let rollback = service
        .activate_generation(ActivateWebsiteGenerationCommand {
            tenant_id: "tenant-sync".to_string(),
            website_root_uuid: root_uuid.clone(),
            target_generation: 1,
            expected_root_version: 2,
            expected_generation: 2,
            operator_id: "user-sync".to_string(),
        })
        .await
        .expect("retained generation should be activatable");
    assert_eq!(rollback.source_generation.generation_no, 1);
    assert_eq!(rollback.website_root.active_generation, 3);
    assert_eq!(rollback.website_root.active_node_id, original_root_node_id);
    let post_rollback_generations: Vec<(i64, String)> = sqlx::query_as(
        "SELECT generation_no, generation_status
         FROM dr_drive_website_root_generation
         WHERE website_root_id=$1 ORDER BY generation_no",
    )
    .bind(&activation.website_root.id)
    .fetch_all(&pool)
    .await
    .expect("post-rollback generation retention should be queryable");
    assert_eq!(
        post_rollback_generations,
        vec![
            (1, "expired".to_string()),
            (2, "retained".to_string()),
            (3, "current".to_string()),
        ]
    );
    sqlx::query(
        "UPDATE dr_drive_website_root_generation
         SET retention_until='2000-01-01T00:00:00.000Z'
         WHERE website_root_id=$1 AND generation_no=1",
    )
    .bind(&activation.website_root.id)
    .execute(&pool)
    .await
    .expect("expired generation should become cleanup-eligible");
    let maintenance = SqlWebsitePublishingMaintenanceStore::new(pool.clone());
    let candidate = maintenance
        .claim_next_cleanup_candidate("website-maintenance-test")
        .await
        .expect("expired generation cleanup should be claimable")
        .expect("expired generation cleanup candidate should exist");
    assert_eq!(candidate.kind, WebsiteTreeCleanupKind::ExpiredGeneration);
    assert!(
        !candidate.delete_tree,
        "generation metadata may retire, but a tree reused by the current generation must remain"
    );
    let deleted_nodes = maintenance
        .complete_cleanup_candidate(&candidate, "website-maintenance-test")
        .await
        .expect("shared generation metadata cleanup should complete");
    assert_eq!(deleted_nodes, 0);
    let cleaned_generation_status: String = sqlx::query_scalar(
        "SELECT generation_status FROM dr_drive_website_root_generation
         WHERE website_root_id=$1 AND generation_no=1",
    )
    .bind(&activation.website_root.id)
    .fetch_one(&pool)
    .await
    .expect("cleaned generation status should be queryable");
    assert_eq!(cleaned_generation_status, "deleted");
    let reused_root_status: String =
        sqlx::query_scalar("SELECT lifecycle_status FROM dr_drive_node WHERE id=$1")
            .bind(&original_root_node_id)
            .fetch_one(&pool)
            .await
            .expect("reused root status should be queryable");
    assert_eq!(reused_root_status, "active");

    let rollback_event: String = sqlx::query_scalar(
        "SELECT payload_json
         FROM dr_drive_domain_outbox
         WHERE tenant_id='tenant-sync'
           AND event_type='drive.website_root.generation.changed.v1'
         ORDER BY sequence_no DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("rollback event should exist");
    let rollback_event: serde_json::Value =
        serde_json::from_str(&rollback_event).expect("rollback event should be JSON");
    assert_eq!(rollback_event["data"]["generation"], "3");
    assert_eq!(rollback_event["data"]["changeReason"], "ROLLBACK_ACTIVATED");
}

#[tokio::test]
async fn invalid_manifest_fails_closed_and_abort_never_changes_the_active_root() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    create_website_space(&pool).await;
    let (root_uuid, root_node_id): (String, String) = sqlx::query_as(
        "SELECT uuid, active_node_id FROM dr_drive_website_root
         WHERE tenant_id='tenant-sync' AND space_id='space-sync' AND root_key='default'",
    )
    .fetch_one(&pool)
    .await
    .expect("default WebsiteRoot should exist");
    let service = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(pool.clone()));
    let declared = validate_website_sync_tree(&[file("index.html", 1, 'a')])
        .expect("declared manifest should be valid");
    let invalid = service
        .create_sync(create_command(&root_uuid, "invalid-sync", &declared))
        .await
        .expect("invalid-content sync reservation should succeed");
    insert_tree(
        &pool,
        &invalid.sync.staging_node_id,
        &[file("index.html", 2, 'b')],
    )
    .await;
    assert!(service
        .finalize_sync(FinalizeWebsiteSyncCommand {
            tenant_id: "tenant-sync".to_string(),
            website_root_uuid: root_uuid.clone(),
            sync_id: invalid.sync.id.clone(),
            expected_sync_version: 1,
            operator_id: "user-sync".to_string(),
        })
        .await
        .is_err());
    let failed_status: String =
        sqlx::query_scalar("SELECT sync_status FROM dr_drive_website_sync WHERE id=$1")
            .bind(&invalid.sync.id)
            .fetch_one(&pool)
            .await
            .expect("failed sync status should be queryable");
    assert_eq!(failed_status, "failed");

    let abortable = service
        .create_sync(create_command(&root_uuid, "abort-sync", &declared))
        .await
        .expect("abortable sync should be created");
    let aborted = service
        .abort_sync(AbortWebsiteSyncCommand {
            tenant_id: "tenant-sync".to_string(),
            website_root_uuid: root_uuid.clone(),
            sync_id: abortable.sync.id,
            expected_sync_version: 1,
            operator_id: "user-sync".to_string(),
        })
        .await
        .expect("unactivated sync should abort");
    assert_eq!(aborted.status, DriveWebsiteSyncStatus::Aborted);

    let active: (String, i64, i64) = sqlx::query_as(
        "SELECT active_node_id, active_generation, version
         FROM dr_drive_website_root WHERE uuid=$1",
    )
    .bind(&root_uuid)
    .fetch_one(&pool)
    .await
    .expect("active root should be queryable");
    assert_eq!(active, (root_node_id, 1, 1));
    let event_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_domain_outbox
         WHERE event_type='drive.website_root.generation.changed.v1'",
    )
    .fetch_one(&pool)
    .await
    .expect("generation events should be countable");
    assert_eq!(event_count, 0);
}

#[tokio::test]
async fn sync_quota_is_reserved_on_create_and_rechecked_before_switch() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    create_website_space(&pool).await;
    let root_uuid: String = sqlx::query_scalar(
        "SELECT uuid FROM dr_drive_website_root
         WHERE tenant_id='tenant-sync' AND space_id='space-sync' AND root_key='default'",
    )
    .fetch_one(&pool)
    .await
    .expect("quota WebsiteRoot should exist");
    let manifest = validate_website_sync_tree(&[file("index.html", 18, 'a')])
        .expect("quota manifest should be valid");
    sqlx::query(
        "INSERT INTO dr_drive_tenant_quota (tenant_id, max_bytes, updated_by)
         VALUES ('tenant-sync', 17, 'quota-test')",
    )
    .execute(&pool)
    .await
    .expect("quota policy should be inserted");
    let service = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(pool.clone()));
    let denied = service
        .create_sync(create_command(&root_uuid, "quota-denied", &manifest))
        .await;
    assert!(
        denied.is_err(),
        "declared staging reservation must honor quota"
    );
    let denied_syncs: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_website_sync WHERE tenant_id='tenant-sync'",
    )
    .fetch_one(&pool)
    .await
    .expect("denied syncs should be countable");
    assert_eq!(
        denied_syncs, 0,
        "quota rejection must not leave staging state"
    );

    sqlx::query(
        "UPDATE dr_drive_tenant_quota
         SET max_bytes=100, updated_by='quota-test', updated_at=CURRENT_TIMESTAMP
         WHERE tenant_id='tenant-sync'",
    )
    .execute(&pool)
    .await
    .expect("quota policy should allow a reservation");
    let created = service
        .create_sync(create_command(&root_uuid, "quota-race", &manifest))
        .await
        .expect("sync reservation should fit the initial quota");
    insert_tree(
        &pool,
        &created.sync.staging_node_id,
        &[file("index.html", 18, 'a')],
    )
    .await;
    sqlx::query(
        "UPDATE dr_drive_tenant_quota
         SET max_bytes=17, updated_by='quota-test', updated_at=CURRENT_TIMESTAMP
         WHERE tenant_id='tenant-sync'",
    )
    .execute(&pool)
    .await
    .expect("quota downgrade should be visible before activation");
    let activation = service
        .finalize_sync(FinalizeWebsiteSyncCommand {
            tenant_id: "tenant-sync".to_string(),
            website_root_uuid: root_uuid.clone(),
            sync_id: created.sync.id.clone(),
            expected_sync_version: 1,
            operator_id: "user-sync".to_string(),
        })
        .await;
    assert!(
        activation.is_err(),
        "quota race must fail closed before switch"
    );
    let active_generation: i64 =
        sqlx::query_scalar("SELECT active_generation FROM dr_drive_website_root WHERE uuid=$1")
            .bind(root_uuid)
            .fetch_one(&pool)
            .await
            .expect("quota-race WebsiteRoot should be queryable");
    assert_eq!(active_generation, 1);
}

#[tokio::test]
async fn disabled_storage_provider_blocks_generation_activation() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    create_website_space(&pool).await;
    let root_uuid: String = sqlx::query_scalar(
        "SELECT uuid FROM dr_drive_website_root
         WHERE tenant_id='tenant-sync' AND space_id='space-sync' AND root_key='default'",
    )
    .fetch_one(&pool)
    .await
    .expect("provider gate WebsiteRoot should exist");
    let entries = [file("index.html", 18, 'a')];
    let manifest =
        validate_website_sync_tree(&entries).expect("provider gate manifest should be valid");
    let service = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(pool.clone()));
    let created = service
        .create_sync(create_command(&root_uuid, "provider-disabled", &manifest))
        .await
        .expect("provider gate WebsiteSync should be created");
    insert_tree(&pool, &created.sync.staging_node_id, &entries).await;
    sqlx::query(
        "UPDATE dr_drive_storage_provider
         SET status='disabled', updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE id='provider-sync'",
    )
    .execute(&pool)
    .await
    .expect("provider should be disabled before activation");
    let activation = service
        .finalize_sync(FinalizeWebsiteSyncCommand {
            tenant_id: "tenant-sync".to_string(),
            website_root_uuid: root_uuid.clone(),
            sync_id: created.sync.id.clone(),
            expected_sync_version: 1,
            operator_id: "user-sync".to_string(),
        })
        .await;
    assert!(
        activation.is_err(),
        "disabled provider must block activation"
    );
    let state: (String, i64) = sqlx::query_as(
        "SELECT sync.sync_status, root.active_generation
         FROM dr_drive_website_sync sync
         INNER JOIN dr_drive_website_root root ON root.id=sync.website_root_id
         WHERE sync.id=$1",
    )
    .bind(&created.sync.id)
    .fetch_one(&pool)
    .await
    .expect("provider gate state should be queryable");
    assert_eq!(state, ("failed".to_string(), 1));
}

#[tokio::test]
async fn expired_validation_lease_is_recoverable_and_fences_the_stale_finalizer() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    create_website_space(&pool).await;
    let root_uuid: String = sqlx::query_scalar(
        "SELECT uuid FROM dr_drive_website_root
         WHERE tenant_id='tenant-sync' AND space_id='space-sync' AND root_key='default'",
    )
    .fetch_one(&pool)
    .await
    .expect("default WebsiteRoot should exist");
    let manifest = validate_website_sync_tree(&[file("index.html", 18, 'a')])
        .expect("lease test manifest should be valid");
    let store = SqlWebsiteSyncStore::new(pool.clone());
    let service = DriveWebsiteSyncService::new(store.clone());
    let created = service
        .create_sync(create_command(&root_uuid, "lease-recovery", &manifest))
        .await
        .expect("lease test WebsiteSync should be created");
    insert_tree(
        &pool,
        &created.sync.staging_node_id,
        &[file("index.html", 18, 'a')],
    )
    .await;

    let first = store
        .begin_validation(&ValidateDriveWebsiteSync {
            tenant_id: "tenant-sync".to_string(),
            website_root_uuid: root_uuid.clone(),
            sync_id: created.sync.id.clone(),
            expected_sync_version: 1,
            operator_id: "worker-first".to_string(),
        })
        .await
        .expect("first finalizer should acquire validation lease");
    let first_token = first
        .lease_token
        .expect("active validation should return a lease token");
    assert_eq!(first.sync.version, 2);
    let active_lease: (String, String) =
        sqlx::query_as("SELECT lease_owner, lease_token FROM dr_drive_website_sync WHERE id=$1")
            .bind(&created.sync.id)
            .fetch_one(&pool)
            .await
            .expect("validation lease should be queryable");
    assert_eq!(
        active_lease,
        ("worker-first".to_string(), first_token.clone())
    );

    assert!(
        store
            .begin_validation(&ValidateDriveWebsiteSync {
                tenant_id: "tenant-sync".to_string(),
                website_root_uuid: root_uuid.clone(),
                sync_id: created.sync.id.clone(),
                expected_sync_version: 2,
                operator_id: "worker-early".to_string(),
            })
            .await
            .is_err(),
        "an unexpired validation lease must not be stolen"
    );
    sqlx::query(
        "UPDATE dr_drive_website_sync
         SET lease_expires_at='2000-01-01T00:00:00.000Z'
         WHERE id=$1",
    )
    .bind(&created.sync.id)
    .execute(&pool)
    .await
    .expect("test lease should expire");

    let recovered = store
        .begin_validation(&ValidateDriveWebsiteSync {
            tenant_id: "tenant-sync".to_string(),
            website_root_uuid: root_uuid.clone(),
            sync_id: created.sync.id.clone(),
            expected_sync_version: 2,
            operator_id: "worker-recovered".to_string(),
        })
        .await
        .expect("expired validation lease should be recoverable");
    let recovered_token = recovered
        .lease_token
        .expect("recovered validation should return a lease token");
    assert_ne!(first_token, recovered_token);
    assert_eq!(recovered.sync.version, 3);

    let stale = store
        .activate_validated(&ActivateValidatedWebsiteSync {
            tenant_id: "tenant-sync".to_string(),
            website_root_uuid: root_uuid.clone(),
            sync_id: created.sync.id.clone(),
            expected_sync_version: 3,
            lease_token: Some(first_token),
            observed_manifest: manifest.clone(),
            operator_id: "worker-first".to_string(),
        })
        .await;
    assert!(
        stale.is_err(),
        "stale lease token must not activate a generation"
    );

    let activation = store
        .activate_validated(&ActivateValidatedWebsiteSync {
            tenant_id: "tenant-sync".to_string(),
            website_root_uuid: root_uuid,
            sync_id: created.sync.id.clone(),
            expected_sync_version: 3,
            lease_token: Some(recovered_token),
            observed_manifest: manifest,
            operator_id: "worker-recovered".to_string(),
        })
        .await
        .expect("current lease owner should activate");
    assert_eq!(activation.sync.status, DriveWebsiteSyncStatus::Completed);
    let remaining_lease_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_website_sync
         WHERE id=$1 AND (lease_owner IS NOT NULL OR lease_token IS NOT NULL OR lease_expires_at IS NOT NULL)",
    )
    .bind(&created.sync.id)
    .fetch_one(&pool)
    .await
    .expect("completed lease state should be queryable");
    assert_eq!(remaining_lease_count, 0);
}

#[tokio::test]
async fn uploader_writes_only_to_writable_website_sync_staging() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    create_website_space(&pool).await;
    let root_uuid: String = sqlx::query_scalar(
        "SELECT uuid FROM dr_drive_website_root
         WHERE tenant_id='tenant-sync' AND space_id='space-sync' AND root_key='default'",
    )
    .fetch_one(&pool)
    .await
    .expect("default WebsiteRoot should exist");
    let declared = validate_website_sync_tree(&[file("index.html", 5, 'a')])
        .expect("declared manifest should be valid");
    let sync_service = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(pool.clone()));
    let created = sync_service
        .create_sync(create_command(&root_uuid, "uploader-sync", &declared))
        .await
        .expect("WebsiteSync should be created");
    let uploader = DriveUploaderService::new(SqlUploaderStore::new(pool.clone()));
    let prepared = uploader
        .prepare_upload(uploader_command(
            "website-upload-1",
            "website-task-1",
            &created.sync.staging_node_id,
        ))
        .await
        .expect("uploader prepare should be allowed for created staging");

    sqlx::query("UPDATE dr_drive_website_sync SET sync_status='validating' WHERE id=$1")
        .bind(&created.sync.id)
        .execute(&pool)
        .await
        .expect("sync status should become validating");
    let denied_prepare = uploader
        .prepare_upload(uploader_command(
            "website-upload-2",
            "website-task-2",
            &created.sync.staging_node_id,
        ))
        .await;
    assert!(denied_prepare.is_err());
    let completion = CompleteStoredUploaderUploadCommand {
        tenant_id: "tenant-sync".to_string(),
        upload_item_id: prepared.id.clone(),
        upload_session_id: prepared
            .upload_session_id
            .clone()
            .expect("prepared upload should have a session"),
        content_type: "text/html".to_string(),
        content_length: 5,
        checksum_sha256_hex: format!("sha256:{}", "a".repeat(64)),
        uploaded_parts_count: 1,
        operator_id: "user-sync".to_string(),
    };
    assert!(uploader
        .complete_stored_upload(completion.clone())
        .await
        .is_err());

    sqlx::query("UPDATE dr_drive_website_sync SET sync_status='ready' WHERE id=$1")
        .bind(&created.sync.id)
        .execute(&pool)
        .await
        .expect("sync status should become ready");
    let completed = uploader
        .complete_stored_upload(completion.clone())
        .await
        .expect("uploader completion should be allowed for ready staging");
    assert_eq!(completed.status, "completed");

    sqlx::query("UPDATE dr_drive_website_sync SET sync_status='completed' WHERE id=$1")
        .bind(&created.sync.id)
        .execute(&pool)
        .await
        .expect("sync status should become completed");
    let replay = uploader
        .complete_stored_upload(completion)
        .await
        .expect("completed uploader replay must remain idempotent");
    assert_eq!(replay.status, "completed");
}

#[tokio::test]
async fn low_level_stores_cannot_mutate_activated_or_retained_atomic_generations() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    create_website_space(&pool).await;
    let (root_uuid, original_root_node_id): (String, String) = sqlx::query_as(
        "SELECT uuid, active_node_id
         FROM dr_drive_website_root
         WHERE tenant_id='tenant-sync' AND space_id='space-sync' AND root_key='default'",
    )
    .fetch_one(&pool)
    .await
    .expect("default WebsiteRoot should exist");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, space_type, parent_node_id,
            node_type, node_name, content_state, lifecycle_status, version,
            created_by, updated_by
         ) VALUES (
            'retained-file', 'tenant-sync', 'space-sync', 'website', $1,
            'file', 'legacy.html', 'empty', 'active', 1,
            'user-sync', 'user-sync'
         )",
    )
    .bind(&original_root_node_id)
    .execute(&pool)
    .await
    .expect("original generation file should be seeded before atomic activation");

    let entries = vec![file("index.html", 18, 'a')];
    let manifest = validate_website_sync_tree(&entries).expect("test manifest should be valid");
    let sync_service = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(pool.clone()));
    let sync = sync_service
        .create_sync(create_command(&root_uuid, "lower-store-guard", &manifest))
        .await
        .expect("WebsiteSync should be created");
    insert_tree(&pool, &sync.sync.staging_node_id, &entries).await;
    sync_service
        .finalize_sync(FinalizeWebsiteSyncCommand {
            tenant_id: "tenant-sync".to_string(),
            website_root_uuid: root_uuid,
            sync_id: sync.sync.id,
            expected_sync_version: 1,
            operator_id: "user-sync".to_string(),
        })
        .await
        .expect("WebsiteSync should activate");

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
            'retained-temporary-upload', 'retained-temporary-task', 'tenant-sync',
            NULL, 'user-sync', 'user', 'user-sync',
            'drive-pc', 'website-sync', 'website-root', 'generic',
            'retained-fingerprint', 'space-sync', 'retained-file', NULL,
            'provider-sync', NULL, 'legacy.html', 'html',
            'text/html', 'document', 'text/html', 18,
            'sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee',
            18, 1, 1, 18, 'completed', 'temporary', 1700000000000,
            'soft_delete', NULL, 'active', 'not_required',
            'user-sync', 'user-sync'
         )",
    )
    .execute(&pool)
    .await
    .expect("retained generation temporary upload fact should be seeded");
    let maintenance_result = DriveMaintenanceService::new(SqlMaintenanceStore::new(pool.clone()))
        .sweep_expired_upload_content(SweepExpiredUploadContentCommand {
            now_epoch_ms: 1_800_000_000_000,
            dry_run: false,
            limit: Some(100),
            operator_id: "maintenance".to_string(),
            request_id: Some("request-retained-cleanup".to_string()),
            trace_id: Some("trace-retained-cleanup".to_string()),
        })
        .await;
    assert!(
        maintenance_result.is_err(),
        "generic retention cleanup must not override retained website generation immutability"
    );
    let retained_cleanup_state: (String, String) = sqlx::query_as(
        "SELECT ui.cleanup_status, n.lifecycle_status
         FROM dr_drive_upload_item ui
         INNER JOIN dr_drive_node n ON n.id=ui.node_id
         WHERE ui.id='retained-temporary-upload'",
    )
    .fetch_one(&pool)
    .await
    .expect("retained cleanup state should remain readable");
    assert_eq!(
        retained_cleanup_state,
        ("active".to_string(), "active".to_string())
    );

    let node_store = SqlNodeStore::new(pool.clone());
    let node_insert = node_store
        .insert_node(&NewDriveNode {
            id: "forbidden-node-store-child".to_string(),
            tenant_id: "tenant-sync".to_string(),
            space_id: "space-sync".to_string(),
            space_type: "website".to_string(),
            parent_node_id: Some(original_root_node_id.clone()),
            shortcut_target_node_id: None,
            node_type: "folder".to_string(),
            node_name: "forbidden-node-store".to_string(),
            lifecycle_status: "active".to_string(),
            created_by: "user-sync".to_string(),
            updated_by: "user-sync".to_string(),
        })
        .await;
    assert!(
        node_insert.is_err(),
        "node store must reject retained-tree writes"
    );

    let workspace_store = SqlDriveWorkspaceStore::new(pool.clone());
    let workspace_node = workspace_store
        .ensure_node(NewDriveWorkspaceNodeRecord {
            id: "forbidden-workspace-child".to_string(),
            tenant_id: "tenant-sync".to_string(),
            space_id: "space-sync".to_string(),
            parent_node_id: Some(original_root_node_id),
            node_type: "folder".to_string(),
            node_name: "forbidden-workspace".to_string(),
            content_state: "empty".to_string(),
            operator_id: "user-sync".to_string(),
        })
        .await;
    assert!(
        workspace_node.is_err(),
        "workspace store must reject retained-tree node writes"
    );

    let head_snapshot = apply_file_node_head_snapshot(
        &pool,
        "tenant-sync",
        "retained-file",
        "user-sync",
        &FileNodeHeadSnapshot {
            file_extension: Some("html".to_string()),
            content_type: "text/html".to_string(),
            content_type_group: "document".to_string(),
            content_length: 18,
            version_no: 1,
            checksum_sha256_hex: format!("sha256:{}", "b".repeat(64)),
        },
    )
    .await;
    assert!(
        head_snapshot.is_err(),
        "node head metadata must reject retained-tree writes"
    );

    let object_ref = workspace_store
        .ensure_object_ref(NewDriveWorkspaceObjectRecord {
            id: "forbidden-object".to_string(),
            tenant_id: "tenant-sync".to_string(),
            node_id: "retained-file".to_string(),
            storage_provider_id: "provider-not-reached".to_string(),
            bucket: "bucket-not-reached".to_string(),
            object_key: "retained/legacy.html".to_string(),
            content_type: "text/html".to_string(),
            content_length: 18,
            checksum_sha256_hex: format!("sha256:{}", "c".repeat(64)),
            operator_id: "user-sync".to_string(),
        })
        .await;
    assert!(
        object_ref.is_err(),
        "workspace object store must reject retained-tree writes"
    );

    let node_version = SqlDriveNodeVersionStore::new(pool.clone())
        .create(CreateDriveNodeVersionCommand {
            id: "forbidden-version".to_string(),
            tenant_id: "tenant-sync".to_string(),
            space_id: "space-sync".to_string(),
            node_id: "retained-file".to_string(),
            version_no: 1,
            storage_object_id: None,
            content_type: "text/html".to_string(),
            content_length: 18,
            checksum_sha256_hex: format!("sha256:{}", "d".repeat(64)),
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
            operator_id: "user-sync".to_string(),
        })
        .await;
    assert!(
        node_version.is_err(),
        "logical node version store must reject retained-tree writes"
    );

    let forbidden_node_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE id IN ('forbidden-node-store-child', 'forbidden-workspace-child')",
    )
    .fetch_one(&pool)
    .await
    .expect("forbidden node writes should be countable");
    let forbidden_object_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_storage_object WHERE id='forbidden-object'",
    )
    .fetch_one(&pool)
    .await
    .expect("forbidden object writes should be countable");
    let forbidden_version_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_node_version WHERE id='forbidden-version'",
    )
    .fetch_one(&pool)
    .await
    .expect("forbidden logical version writes should be countable");
    assert_eq!(forbidden_node_count, 0);
    assert_eq!(forbidden_object_count, 0);
    assert_eq!(forbidden_version_count, 0);
}

async fn assert_node_mutation_guard(pool: &PgPool, node_id: &str, expected_allowed: bool) {
    let mut connection = pool
        .acquire()
        .await
        .expect("mutation guard connection should be acquired");
    sqlx::query(begin_transaction_sql())
        .execute(&mut *connection)
        .await
        .expect("mutation guard transaction should begin");
    let result =
        ensure_managed_website_node_mutation_allowed(&mut connection, "tenant-sync", node_id).await;
    sqlx::query("ROLLBACK")
        .execute(&mut *connection)
        .await
        .expect("mutation guard transaction should roll back");
    assert_eq!(
        result.is_ok(),
        expected_allowed,
        "unexpected managed website tree mutation decision for {node_id}: {result:?}"
    );
}

async fn create_website_space(pool: &PgPool) {
    DriveSpaceService::new(SqlSpaceStore::new(pool.clone()))
        .create_space(CreateSpaceCommand {
            id: "space-sync".to_string(),
            tenant_id: "tenant-sync".to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-sync".to_string(),
            display_name: "Atomic Website".to_string(),
            space_type: DriveSpaceType::Website,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-sync".to_string(),
        })
        .await
        .expect("website Space should be created");
    seed_storage_provider(pool).await;
}

fn uploader_command(id: &str, task_id: &str, parent_node_id: &str) -> PrepareUploaderUploadCommand {
    PrepareUploaderUploadCommand {
        id: id.to_string(),
        task_id: task_id.to_string(),
        tenant_id: "tenant-sync".to_string(),
        organization_id: None,
        actor: UploaderActor::User {
            user_id: "user-sync".to_string(),
        },
        app_id: "drive-pc".to_string(),
        app_resource_type: "website-sync".to_string(),
        app_resource_id: "website-root".to_string(),
        scene: Some("website_sync".to_string()),
        source: Some("test".to_string()),
        upload_profile_code: "generic".to_string(),
        file_fingerprint: format!("fingerprint-{id}"),
        original_file_name: "index.html".to_string(),
        content_type: "text/html".to_string(),
        content_length: 5,
        chunk_size_bytes: 5,
        target: UploaderTarget::Space {
            space_id: "space-sync".to_string(),
            parent_node_id: Some(parent_node_id.to_string()),
            share_token: None,
        },
        retention: UploaderRetention::LongTerm,
        operator_id: "user-sync".to_string(),
        now_epoch_ms: 1_800_000_000_000,
    }
}

async fn seed_storage_provider(pool: &PgPool) {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode,
            default_storage_class, status, version, created_by, updated_by
         ) VALUES (
            'provider-sync', 's3_compatible', 'Sync Provider',
            'https://s3.example.com', 'us-east-1', 'bucket-sync', 1,
            1, 'plain:test-access-key:test-secret-key', 'AES256',
            'STANDARD', 'active', 1, 'test', 'test'
         )",
    )
    .execute(pool)
    .await
    .expect("storage provider should be seeded");
}

fn create_command(
    root_uuid: &str,
    idempotency_key: &str,
    manifest: &sdkwork_drive_workspace_service::domain::website_sync::DriveWebsiteManifestSummary,
) -> CreateWebsiteSyncCommand {
    CreateWebsiteSyncCommand {
        tenant_id: "tenant-sync".to_string(),
        website_root_uuid: root_uuid.to_string(),
        idempotency_key: idempotency_key.to_string(),
        expected_root_version: 1,
        expected_generation: 1,
        manifest_sha256: manifest.sha256.clone(),
        manifest_file_count: manifest.file_count,
        manifest_total_bytes: manifest.total_bytes,
        expires_at: (Utc::now() + Duration::hours(1)).to_rfc3339_opts(SecondsFormat::Millis, true),
        operator_id: "user-sync".to_string(),
    }
}

fn file(path: &str, length: i64, digest_character: char) -> DriveWebsiteSyncTreeEntry {
    DriveWebsiteSyncTreeEntry {
        relative_path: path.to_string(),
        depth: path.split('/').count() as i64,
        node_type: "file".to_string(),
        content_state: "ready".to_string(),
        content_length: Some(length),
        checksum_sha256_hex: Some(format!(
            "sha256:{}",
            digest_character.to_string().repeat(64)
        )),
        shortcut_target_node_id: None,
    }
}

fn folder(path: &str) -> DriveWebsiteSyncTreeEntry {
    DriveWebsiteSyncTreeEntry {
        relative_path: path.to_string(),
        depth: path.split('/').count() as i64,
        node_type: "folder".to_string(),
        content_state: "ready".to_string(),
        content_length: None,
        checksum_sha256_hex: None,
        shortcut_target_node_id: None,
    }
}

async fn insert_tree(pool: &PgPool, staging_node_id: &str, entries: &[DriveWebsiteSyncTreeEntry]) {
    let mut parent_id = staging_node_id.to_string();
    for (index, entry) in entries.iter().enumerate() {
        let node_id = format!("sync-entry-{index}");
        let node_name = entry
            .relative_path
            .rsplit('/')
            .next()
            .expect("entry path should have a name");
        let parent = if entry.relative_path.contains('/') {
            parent_id.as_str()
        } else {
            staging_node_id
        };
        let is_file = entry.node_type == "file";
        let content_type = if is_file {
            Some(if node_name.ends_with(".html") {
                "text/html"
            } else if node_name.ends_with(".js") {
                "application/javascript"
            } else {
                "application/octet-stream"
            })
        } else {
            None
        };
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, space_type, parent_node_id,
                node_type, node_name, content_state, head_content_type,
                head_content_type_group, head_content_length, head_version_no,
                head_checksum_sha256_hex, lifecycle_status, version,
                created_by, updated_by
             ) VALUES ($1, 'tenant-sync', 'space-sync', 'website', $2,
                       $3, $4, $5, $6, $7, $8, $9, $10, 'active', 1,
                       'user-sync', 'user-sync')",
        )
        .bind(&node_id)
        .bind(parent)
        .bind(&entry.node_type)
        .bind(node_name)
        .bind(&entry.content_state)
        .bind(content_type)
        .bind(if is_file { Some("text") } else { None })
        .bind(entry.content_length)
        .bind(if is_file { Some(1_i64) } else { None })
        .bind(entry.checksum_sha256_hex.as_deref())
        .execute(pool)
        .await
        .expect("sync tree entry should insert");
        if is_file {
            sqlx::query(
                "INSERT INTO dr_drive_storage_object (
                   id, tenant_id, node_id, version_no, storage_provider_id,
                   bucket, object_key, content_type, content_length,
                   checksum_sha256_hex, lifecycle_status, created_by, updated_by
                 ) VALUES (
                   $1, 'tenant-sync', $2, 1, 'provider-sync',
                   'bucket-sync', $3, $4, $5, $6,
                   'active', 'user-sync', 'user-sync'
                 )",
            )
            .bind(format!("sync-object-{index}"))
            .bind(&node_id)
            .bind(format!("website-sync/{node_id}"))
            .bind(content_type.expect("file content type should exist"))
            .bind(entry.content_length.expect("file length should exist"))
            .bind(
                entry
                    .checksum_sha256_hex
                    .as_deref()
                    .expect("file checksum should exist"),
            )
            .execute(pool)
            .await
            .expect("sync tree storage object should insert");
        }
        if entry.node_type == "folder" {
            parent_id = node_id;
        }
    }
}
