use std::time::Duration as StdDuration;

use chrono::{Duration, SecondsFormat, Utc};
use sdkwork_drive_workspace_service::application::space_service::{
    CreateSpaceCommand, DriveSpaceService,
};
use sdkwork_drive_workspace_service::application::website_sync_service::{
    CreateWebsiteSyncCommand, DriveWebsiteSyncService, FinalizeWebsiteSyncCommand,
};
use sdkwork_drive_workspace_service::domain::space::DriveSpaceType;
use sdkwork_drive_workspace_service::domain::website_sync::{
    validate_website_sync_tree, DriveWebsiteSyncTreeEntry,
};
use sdkwork_drive_workspace_service::infrastructure::sql::begin_transaction_sql;
use sdkwork_drive_workspace_service::infrastructure::sql::managed_website_tree_guard::ensure_managed_website_parent_mutation_allowed;
use sdkwork_drive_workspace_service::infrastructure::sql::node_store::SqlNodeStore;
use sdkwork_drive_workspace_service::infrastructure::sql::space_store::SqlSpaceStore;
use sdkwork_drive_workspace_service::infrastructure::sql::website_sync_store::SqlWebsiteSyncStore;
use sdkwork_drive_workspace_service::ports::node_store::{DriveNodeStore, NewDriveNode};
use sqlx::PgPool;
use tokio::sync::oneshot;
use tokio::time::timeout;

#[tokio::test]
async fn postgres_mutation_first_blocks_finalize_and_finalize_observes_committed_tree() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let fixture = create_fixture(&pool, "mutation-first").await;
    let entry = file("index.html", 18, 'a');
    let manifest = validate_website_sync_tree(std::slice::from_ref(&entry))
        .expect("test manifest should be valid");
    let sync_service = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(pool.clone()));
    let sync = sync_service
        .create_sync(create_sync_command(&fixture, &manifest, "mutation-first"))
        .await
        .expect("WebsiteSync should be created");

    let mutation_pool = pool.clone();
    let mutation_tenant_id = fixture.tenant_id.clone();
    let mutation_space_id = fixture.space_id.clone();
    let mutation_provider_id = fixture.provider_id.clone();
    let staging_node_id = sync.sync.staging_node_id.clone();
    let (locked_tx, locked_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let mutation = tokio::spawn(async move {
        let mut connection = mutation_pool
            .acquire()
            .await
            .expect("mutation connection should be acquired");
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .expect("mutation transaction should begin");
        ensure_managed_website_parent_mutation_allowed(
            &mut connection,
            &mutation_tenant_id,
            &mutation_space_id,
            Some(&staging_node_id),
        )
        .await
        .expect("created staging should be writable");
        let file_node_id = format!("node-{}", uuid::Uuid::new_v4());
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, space_type, parent_node_id,
                node_type, node_name, content_state, head_content_type,
                head_content_type_group, head_content_length, head_version_no,
                head_checksum_sha256_hex, lifecycle_status, version,
                created_by, updated_by
             ) VALUES (
                $1, $2, $3, 'website', $4,
                'file', 'index.html', 'ready', 'text/html',
                'text', 18, 1,
                $5, 'active', 1, 'postgres-test', 'postgres-test'
             )",
        )
        .bind(&file_node_id)
        .bind(&mutation_tenant_id)
        .bind(&mutation_space_id)
        .bind(&staging_node_id)
        .bind(format!("sha256:{}", "a".repeat(64)))
        .execute(&mut *connection)
        .await
        .expect("staging file should insert");
        sqlx::query(
            "INSERT INTO dr_drive_storage_object (
               id, tenant_id, node_id, version_no, storage_provider_id,
               bucket, object_key, content_type, content_length,
               checksum_sha256_hex, lifecycle_status, created_by, updated_by
             ) VALUES (
               $1, $2, $3, 1, $4, 'bucket-pg-website-sync', $5,
               'text/html', 18, $6, 'active', 'postgres-test', 'postgres-test'
             )",
        )
        .bind(format!("object-{}", uuid::Uuid::new_v4()))
        .bind(&mutation_tenant_id)
        .bind(&file_node_id)
        .bind(&mutation_provider_id)
        .bind(format!("website-sync/{file_node_id}"))
        .bind(format!("sha256:{}", "a".repeat(64)))
        .execute(&mut *connection)
        .await
        .expect("staging storage object should insert");
        locked_tx
            .send(())
            .expect("mutation lock signal should be delivered");
        release_rx
            .await
            .expect("mutation release signal should be delivered");
        sqlx::query("COMMIT")
            .execute(&mut *connection)
            .await
            .expect("mutation transaction should commit");
    });
    locked_rx
        .await
        .expect("mutation should hold the WebsiteSync lock");

    let finalize_service = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(pool.clone()));
    let finalize_command = FinalizeWebsiteSyncCommand {
        tenant_id: fixture.tenant_id.clone(),
        website_root_uuid: fixture.website_root_uuid.clone(),
        sync_id: sync.sync.id.clone(),
        expected_sync_version: 1,
        operator_id: "postgres-test".to_string(),
    };
    let mut finalize =
        tokio::spawn(async move { finalize_service.finalize_sync(finalize_command).await });
    assert!(
        timeout(StdDuration::from_millis(250), &mut finalize)
            .await
            .is_err(),
        "finalize must wait while mutation owns the WebsiteSync row lock"
    );
    release_tx
        .send(())
        .expect("mutation release should be delivered");
    mutation.await.expect("mutation task should complete");
    let activation = timeout(StdDuration::from_secs(10), finalize)
        .await
        .expect("finalize should finish after mutation commit")
        .expect("finalize task should not panic")
        .expect("finalize should observe the committed staging tree");
    assert_eq!(activation.sync.uploaded_file_count, 1);
    assert_eq!(activation.sync.uploaded_total_bytes, 18);
}

#[tokio::test]
async fn postgres_validation_first_blocks_mutation_then_rejects_it_without_partial_write() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let fixture = create_fixture(&pool, "validation-first").await;
    let manifest = validate_website_sync_tree(&[file("index.html", 18, 'b')])
        .expect("test manifest should be valid");
    let sync_service = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(pool.clone()));
    let sync = sync_service
        .create_sync(create_sync_command(&fixture, &manifest, "validation-first"))
        .await
        .expect("WebsiteSync should be created");

    let mut validation_connection = pool
        .acquire()
        .await
        .expect("validation connection should be acquired");
    sqlx::query(begin_transaction_sql())
        .execute(&mut *validation_connection)
        .await
        .expect("validation transaction should begin");
    sqlx::query(
        "UPDATE dr_drive_website_sync
         SET sync_status='validating', version=version + 1
         WHERE tenant_id=$1 AND id=$2",
    )
    .bind(&fixture.tenant_id)
    .bind(&sync.sync.id)
    .execute(&mut *validation_connection)
    .await
    .expect("validation transaction should lock WebsiteSync");

    let store = SqlNodeStore::new(pool.clone());
    let node_id = format!("node-{}", uuid::Uuid::new_v4());
    let new_node = NewDriveNode {
        id: node_id.clone(),
        tenant_id: fixture.tenant_id.clone(),
        space_id: fixture.space_id,
        space_type: "website".to_string(),
        parent_node_id: Some(sync.sync.staging_node_id),
        shortcut_target_node_id: None,
        node_type: "folder".to_string(),
        node_name: "late-write".to_string(),
        lifecycle_status: "active".to_string(),
        created_by: "postgres-test".to_string(),
        updated_by: "postgres-test".to_string(),
    };
    let mut mutation = tokio::spawn(async move { store.insert_node(&new_node).await });
    assert!(
        timeout(StdDuration::from_millis(250), &mut mutation)
            .await
            .is_err(),
        "mutation must wait while validation owns the WebsiteSync row lock"
    );
    sqlx::query("COMMIT")
        .execute(&mut *validation_connection)
        .await
        .expect("validation transaction should commit");

    let mutation_result = timeout(StdDuration::from_secs(10), mutation)
        .await
        .expect("mutation should finish after validation commit")
        .expect("mutation task should not panic");
    assert!(
        mutation_result.is_err(),
        "mutation must reject validating WebsiteSync staging"
    );
    let inserted_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_node WHERE id=$1")
        .bind(node_id)
        .fetch_one(&pool)
        .await
        .expect("late mutation node count should be readable");
    assert_eq!(inserted_count, 0);
}

struct WebsiteFixture {
    tenant_id: String,
    space_id: String,
    website_root_uuid: String,
    provider_id: String,
}

async fn create_fixture(pool: &PgPool, label: &str) -> WebsiteFixture {
    let suffix = uuid::Uuid::new_v4();
    let tenant_id = format!("tenant-pg-{label}-{suffix}");
    let space_id = format!("space-pg-{label}-{suffix}");
    let provider_id = format!("provider-pg-{suffix}");
    DriveSpaceService::new(SqlSpaceStore::new(pool.clone()))
        .create_space(CreateSpaceCommand {
            id: space_id.clone(),
            tenant_id: tenant_id.clone(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "postgres-test".to_string(),
            display_name: format!("PostgreSQL {label}"),
            space_type: DriveSpaceType::Website,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "postgres-test".to_string(),
        })
        .await
        .expect("PostgreSQL website Space should be created");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
           id, provider_kind, name, endpoint_url, region, bucket, path_style,
           strict_tls, credential_ref, status, version, created_by, updated_by
         ) VALUES (
           $1, 's3_compatible', $2, 'https://s3.example.test', 'test-region',
           'bucket-pg-website-sync', TRUE, TRUE,
           'plain:test-access-key:test-secret-key', 'active', 1,
           'postgres-test', 'postgres-test'
         )",
    )
    .bind(&provider_id)
    .bind(format!("PostgreSQL {label} Provider"))
    .execute(pool)
    .await
    .expect("PostgreSQL storage provider should be created");
    let website_root_uuid = sqlx::query_scalar(
        "SELECT uuid
         FROM dr_drive_website_root
         WHERE tenant_id=$1 AND space_id=$2 AND root_key='default'",
    )
    .bind(&tenant_id)
    .bind(&space_id)
    .fetch_one(pool)
    .await
    .expect("default PostgreSQL WebsiteRoot should exist");
    WebsiteFixture {
        tenant_id,
        space_id,
        website_root_uuid,
        provider_id,
    }
}

fn create_sync_command(
    fixture: &WebsiteFixture,
    manifest: &sdkwork_drive_workspace_service::domain::website_sync::DriveWebsiteManifestSummary,
    label: &str,
) -> CreateWebsiteSyncCommand {
    CreateWebsiteSyncCommand {
        tenant_id: fixture.tenant_id.clone(),
        website_root_uuid: fixture.website_root_uuid.clone(),
        idempotency_key: format!("{label}-{}", uuid::Uuid::new_v4()),
        expected_root_version: 1,
        expected_generation: 1,
        manifest_sha256: manifest.sha256.clone(),
        manifest_file_count: manifest.file_count,
        manifest_total_bytes: manifest.total_bytes,
        expires_at: (Utc::now() + Duration::hours(1)).to_rfc3339_opts(SecondsFormat::Millis, true),
        operator_id: "postgres-test".to_string(),
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
