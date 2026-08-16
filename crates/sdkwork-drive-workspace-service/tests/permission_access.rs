use sdkwork_drive_workspace_service::application::permission_service::SqlDrivePermissionService;
use sdkwork_drive_workspace_service::ports::permission_store::{
    GrantDriveNodePermissionCommand, ResolveEffectiveNodeAccessCommand,
};

#[tokio::test]
async fn inherited_folder_writer_allows_child_upload_permission_check() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
         ) VALUES ('space-1', '100001', 'user', 'owner-1', 'personal', 'Personal', 'active', 1, 'owner-1', 'owner-1')",
    )
    .execute(&pool)
    .await
    .expect("insert space");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            lifecycle_status, content_state, version, created_by, updated_by
         ) VALUES
         ('folder-1', '100001', 'space-1', NULL, 'folder', 'Root', 'active', 'ready', 1, 'owner-1', 'owner-1'),
         ('file-1', '100001', 'space-1', 'folder-1', 'file', 'Doc', 'active', 'ready', 1, 'owner-1', 'owner-1')",
    )
    .execute(&pool)
    .await
    .expect("insert nodes");

    let service = SqlDrivePermissionService::new(pool.clone());
    service
        .grant_node_permission(GrantDriveNodePermissionCommand {
            tenant_id: "100001".to_string(),
            node_id: "folder-1".to_string(),
            subject_type: "user".to_string(),
            subject_id: "writer-1".to_string(),
            role: "writer".to_string(),
            operator_id: "owner-1".to_string(),
        })
        .await
        .expect("grant writer on folder");

    let child_access = service
        .resolve_effective_node_access(ResolveEffectiveNodeAccessCommand {
            tenant_id: "100001".to_string(),
            space_id: "space-1".to_string(),
            node_id: "file-1".to_string(),
            subject_type: "user".to_string(),
            subject_id: "writer-1".to_string(),
        })
        .await
        .expect("resolve child access");

    assert!(child_access.allows_role("writer"));
    assert!(child_access.inherited);
    assert_eq!(
        child_access.inherited_from_node_id.as_deref(),
        Some("folder-1")
    );
}

#[tokio::test]
async fn unrelated_subject_has_no_effective_access() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
         ) VALUES ('space-1', '100001', 'user', 'owner-1', 'personal', 'Personal', 'active', 1, 'owner-1', 'owner-1')",
    )
    .execute(&pool)
    .await
    .expect("insert space");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            lifecycle_status, content_state, version, created_by, updated_by
         ) VALUES ('file-1', '100001', 'space-1', NULL, 'file', 'Doc', 'active', 'ready', 1, 'owner-1', 'owner-1')",
    )
    .execute(&pool)
    .await
    .expect("insert node");

    let service = SqlDrivePermissionService::new(pool);
    let access = service
        .resolve_effective_node_access(ResolveEffectiveNodeAccessCommand {
            tenant_id: "100001".to_string(),
            space_id: "space-1".to_string(),
            node_id: "file-1".to_string(),
            subject_type: "user".to_string(),
            subject_id: "stranger-1".to_string(),
        })
        .await
        .expect("resolve access");

    assert!(!access.allows_role("reader"));
    assert_eq!(access.role, "none");
}

#[tokio::test]
async fn commenter_role_does_not_grant_writer_access() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
         ) VALUES ('space-1', '100001', 'user', 'owner-1', 'personal', 'Personal', 'active', 1, 'owner-1', 'owner-1')",
    )
    .execute(&pool)
    .await
    .expect("insert space");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            lifecycle_status, content_state, version, created_by, updated_by
         ) VALUES ('file-1', '100001', 'space-1', NULL, 'file', 'Doc', 'active', 'ready', 1, 'owner-1', 'owner-1')",
    )
    .execute(&pool)
    .await
    .expect("insert node");

    let service = SqlDrivePermissionService::new(pool.clone());
    service
        .grant_node_permission(GrantDriveNodePermissionCommand {
            tenant_id: "100001".to_string(),
            node_id: "file-1".to_string(),
            subject_type: "user".to_string(),
            subject_id: "commenter-1".to_string(),
            role: "commenter".to_string(),
            operator_id: "owner-1".to_string(),
        })
        .await
        .expect("grant commenter");

    let access = service
        .resolve_effective_node_access(ResolveEffectiveNodeAccessCommand {
            tenant_id: "100001".to_string(),
            space_id: "space-1".to_string(),
            node_id: "file-1".to_string(),
            subject_type: "user".to_string(),
            subject_id: "commenter-1".to_string(),
        })
        .await
        .expect("resolve access");

    assert!(access.allows_role("commenter"));
    assert!(access.allows_role("reader"));
    assert!(!access.allows_role("writer"));
}

#[tokio::test]
async fn trashed_node_direct_reader_permission_resolves_for_acl_checks() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
         ) VALUES ('space-1', '100001', 'user', 'owner-1', 'personal', 'Personal', 'active', 1, 'owner-1', 'owner-1')",
    )
    .execute(&pool)
    .await
    .expect("insert space");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            lifecycle_status, content_state, version, created_by, updated_by
         ) VALUES ('file-trashed', '100001', 'space-1', NULL, 'file', 'Trashed', 'trashed', 'ready', 1, 'owner-1', 'owner-1')",
    )
    .execute(&pool)
    .await
    .expect("insert trashed node");

    let service = SqlDrivePermissionService::new(pool.clone());
    service
        .grant_node_permission(GrantDriveNodePermissionCommand {
            tenant_id: "100001".to_string(),
            node_id: "file-trashed".to_string(),
            subject_type: "user".to_string(),
            subject_id: "reviewer-1".to_string(),
            role: "reader".to_string(),
            operator_id: "owner-1".to_string(),
        })
        .await
        .expect("grant reader on trashed node");

    let access = service
        .resolve_effective_node_access(ResolveEffectiveNodeAccessCommand {
            tenant_id: "100001".to_string(),
            space_id: "space-1".to_string(),
            node_id: "file-trashed".to_string(),
            subject_type: "user".to_string(),
            subject_id: "reviewer-1".to_string(),
        })
        .await
        .expect("resolve trashed node access");

    assert!(access.allows_role("reader"));
    assert_eq!(access.role, "reader");
}
