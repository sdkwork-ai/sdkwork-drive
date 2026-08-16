use sdkwork_drive_contract::drive::domain_events as drive_events;
use sdkwork_drive_workspace_service::application::change_feed_service::{
    ListChangesCommand, QueryStartPageTokenCommand, SqlDriveChangeFeedService,
};
use sdkwork_drive_workspace_service::application::space_lifecycle_service::{
    BootstrapTeamSpaceCreatorAccessCommand, DeleteSpaceWithContentsCommand,
    RetireSpaceContentsCommand, SqlDriveSpaceLifecycleService,
};
use sdkwork_drive_workspace_service::application::space_service::{
    CreateSpaceCommand, DriveSpaceService,
};
use sdkwork_drive_workspace_service::domain::space::DriveSpaceType;
use sdkwork_drive_workspace_service::infrastructure::change_recorder::{
    record_drive_change, RecordDriveChangeCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::space_store::SqlSpaceStore;

#[tokio::test]
async fn space_lifecycle_service_bootstraps_team_space_root_and_owner_permission() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let space_service = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()));
    let space = space_service
        .create_space(CreateSpaceCommand {
            id: "team-space-1".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "group".to_string(),
            owner_subject_id: "org-1:team-a".to_string(),
            display_name: "Engineering".to_string(),
            space_type: DriveSpaceType::Team,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-creator".to_string(),
        })
        .await
        .expect("team space should be created");

    SqlDriveSpaceLifecycleService::new(pool.clone())
        .bootstrap_team_space_creator_access(BootstrapTeamSpaceCreatorAccessCommand {
            tenant_id: space.tenant_id.clone(),
            space_id: space.id.clone(),
            creator_user_id: "user-creator".to_string(),
            display_name: space.display_name.clone(),
            root_folder_id: "folder_root_1".to_string(),
        })
        .await
        .expect("team space bootstrap should succeed");

    let permission_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node_permission
         WHERE tenant_id='tenant-001'
           AND subject_type='user'
           AND subject_id='user-creator'
           AND role='owner'
           AND lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("permission count should be readable");
    assert_eq!(permission_count, 1);
}

#[tokio::test]
async fn space_lifecycle_service_retires_space_contents_before_space_delete_side_effects() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let space_service = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()));
    let space = space_service
        .create_space(CreateSpaceCommand {
            id: "space-retire".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-001".to_string(),
            display_name: "Personal".to_string(),
            space_type: DriveSpaceType::Personal,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("space should be created");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, node_type, node_name, content_state, lifecycle_status,
            version, created_by, updated_by
         ) VALUES ('node-1', 'tenant-001', 'space-retire', 'folder', 'Docs', 'ready', 'active', 1, 'user-001', 'user-001')",
    )
    .execute(&pool)
    .await
    .expect("node insert should succeed");

    let deleted_count = SqlDriveSpaceLifecycleService::new(pool.clone())
        .retire_space_contents(RetireSpaceContentsCommand {
            tenant_id: space.tenant_id,
            space_id: space.id,
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("space contents should be retired");
    assert_eq!(deleted_count, 1);

    let active_nodes: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_node WHERE space_id='space-retire' AND lifecycle_status != 'deleted'",
    )
    .fetch_one(&pool)
    .await
    .expect("active node count should be readable");
    assert_eq!(active_nodes, 0);
}

#[tokio::test]
async fn delete_space_with_contents_is_atomic_and_records_change_feed() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let space_service = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()));
    let space = space_service
        .create_space(CreateSpaceCommand {
            id: "space-delete-atomic".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-001".to_string(),
            display_name: "Delete Me".to_string(),
            space_type: DriveSpaceType::Personal,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("space should be created");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, node_type, node_name, content_state, lifecycle_status,
            version, created_by, updated_by
         ) VALUES ('node-delete-1', 'tenant-001', 'space-delete-atomic', 'folder', 'Docs', 'ready', 'active', 1, 'user-001', 'user-001')",
    )
    .execute(&pool)
    .await
    .expect("node insert should succeed");

    let result = SqlDriveSpaceLifecycleService::new(pool.clone())
        .delete_space_with_contents(DeleteSpaceWithContentsCommand {
            tenant_id: space.tenant_id,
            space_id: space.id,
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("atomic delete should succeed");
    assert_eq!(result.deleted_node_count, 1);
    assert_eq!(result.space.lifecycle_status, "deleted");

    let active_nodes: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_node WHERE space_id='space-delete-atomic' AND lifecycle_status != 'deleted'",
    )
    .fetch_one(&pool)
    .await
    .expect("active node count should be readable");
    assert_eq!(active_nodes, 0);

    let change_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_change_log WHERE space_id='space-delete-atomic' AND event_type=$1",
    )
    .bind(drive_events::space::DELETED)
    .fetch_one(&pool)
    .await
    .expect("change log count should be readable");
    assert_eq!(change_count, 1);
}

#[tokio::test]
async fn delete_atomic_website_space_uses_typed_override_and_records_audit() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let space = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()))
        .create_space(CreateSpaceCommand {
            id: "space-delete-website".to_string(),
            tenant_id: "tenant-website".to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-website".to_string(),
            display_name: "Website".to_string(),
            space_type: DriveSpaceType::Website,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-website".to_string(),
        })
        .await
        .expect("website Space should be created");
    sqlx::query(
        "UPDATE dr_drive_website_root
         SET content_mode='atomic_generation'
         WHERE tenant_id='tenant-website' AND space_id='space-delete-website'",
    )
    .execute(&pool)
    .await
    .expect("WebsiteRoot should become atomic for the override test");

    let result = SqlDriveSpaceLifecycleService::new(pool.clone())
        .delete_space_with_contents(DeleteSpaceWithContentsCommand {
            tenant_id: space.tenant_id,
            space_id: space.id,
            operator_id: "user-website".to_string(),
        })
        .await
        .expect("tenant-authorized atomic website Space delete should succeed");
    assert_eq!(result.space.lifecycle_status, "deleted");
    assert!(result.deleted_node_count >= 1);

    let audit: (String, String, String) = sqlx::query_as(
        "SELECT action, resource_type, operator_id
         FROM dr_drive_audit_event
         WHERE tenant_id='tenant-website' AND resource_id='space-delete-website'",
    )
    .fetch_one(&pool)
    .await
    .expect("website Space retirement override audit should exist");
    assert_eq!(
        audit,
        (
            "drive.website_tree.system_override.tenant_authorized_space_retirement".to_string(),
            "drive_space".to_string(),
            "user-website".to_string(),
        )
    );
}

#[tokio::test]
async fn change_feed_service_lists_changes_and_start_page_token() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    record_drive_change(
        &pool,
        RecordDriveChangeCommand {
            tenant_id: "tenant-001",
            space_id: "space-1",
            node_id: Some("node-1"),
            event_type: drive_events::node::CREATED,
            actor_id: "user-001",
        },
    )
    .await
    .expect("first change should be recorded");
    record_drive_change(
        &pool,
        RecordDriveChangeCommand {
            tenant_id: "tenant-001",
            space_id: "space-1",
            node_id: Some("node-2"),
            event_type: drive_events::node::UPDATED,
            actor_id: "user-001",
        },
    )
    .await
    .expect("second change should be recorded");

    let service = SqlDriveChangeFeedService::new(pool.clone());
    let start_token = service
        .query_start_page_token(QueryStartPageTokenCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: Some("space-1".to_string()),
        })
        .await
        .expect("start page token should be computed");
    assert_eq!(start_token, 2);

    let changes = service
        .list_changes(ListChangesCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: "space-1".to_string(),
            after_sequence: 0,
            limit: 10,
            subject_type: None,
            subject_id: None,
            is_space_owner: true,
        })
        .await
        .expect("changes should be listed");
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0].sequence_no, 1);
    assert_eq!(changes[1].event_type, drive_events::node::UPDATED);
}
