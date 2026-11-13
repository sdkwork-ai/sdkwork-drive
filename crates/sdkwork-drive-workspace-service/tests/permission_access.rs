use sdkwork_drive_workspace_service::application::permission_service::{
    SqlDrivePermissionService, MAX_NODE_ACCESS_BATCH,
};
use sdkwork_drive_workspace_service::ports::permission_store::{
    GrantDriveNodePermissionCommand, ResolveEffectiveNodeAccessCommand,
};
use sdkwork_drive_workspace_service::DriveServiceError;

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
            lifecycle_status, content_state, head_content_type, head_content_type_group, head_content_length, head_version_no, version, created_by, updated_by
         ) VALUES
         ('folder-1', '100001', 'space-1', NULL, 'folder', 'Root', 'active', 'ready', NULL, NULL, NULL, NULL, 1, 'owner-1', 'owner-1'),('file-1', '100001', 'space-1', 'folder-1', 'file', 'Doc', 'active', 'ready', 'application/octet-stream', 'binary', 0, 1, 1, 'owner-1', 'owner-1')",
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
            lifecycle_status, content_state, head_content_type, head_content_type_group, head_content_length, head_version_no, version, created_by, updated_by
         ) VALUES ('file-1', '100001', 'space-1', NULL, 'file', 'Doc', 'active', 'ready', 'application/octet-stream', 'binary', 0, 1, 1, 'owner-1', 'owner-1')",
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
            lifecycle_status, content_state, head_content_type, head_content_type_group, head_content_length, head_version_no, version, created_by, updated_by
         ) VALUES ('file-1', '100001', 'space-1', NULL, 'file', 'Doc', 'active', 'ready', 'application/octet-stream', 'binary', 0, 1, 1, 'owner-1', 'owner-1')",
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
            lifecycle_status, content_state, head_content_type, head_content_type_group, head_content_length, head_version_no, version, created_by, updated_by
         ) VALUES ('file-trashed', '100001', 'space-1', NULL, 'file', 'Trashed', 'trashed', 'ready', 'application/octet-stream', 'binary', 0, 1, 1, 'owner-1', 'owner-1')",
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

#[tokio::test]
async fn batch_access_rejects_an_oversized_node_list() {
    let service =
        SqlDrivePermissionService::new(sdkwork_drive_test_support::lazy_postgres_test_pool());
    let node_ids = (0..=MAX_NODE_ACCESS_BATCH)
        .map(|index| format!("node-{index}"))
        .collect::<Vec<_>>();
    let error = service
        .resolve_node_access_batch("100001", &node_ids, "user", "user-1")
        .await
        .expect_err("batch must stay bounded");
    assert!(matches!(error, DriveServiceError::Validation(_)));
}

#[tokio::test]
async fn batch_access_is_empty_for_an_empty_node_list() {
    let service =
        SqlDrivePermissionService::new(sdkwork_drive_test_support::lazy_postgres_test_pool());
    let access = service
        .resolve_node_access_batch("100001", &[], "user", "user-1")
        .await
        .expect("empty batch needs no database");
    assert!(access.is_empty());
}

#[tokio::test]
async fn batch_access_matches_single_node_resolution_for_every_node() {
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
            lifecycle_status, content_state, head_content_type, head_content_type_group, head_content_length, head_version_no, version, created_by, updated_by
         ) VALUES
         ('folder-1', '100001', 'space-1', NULL, 'folder', 'Root', 'active', 'ready', NULL, NULL, NULL, NULL, 1, 'owner-1', 'owner-1'),
         ('folder-2', '100001', 'space-1', 'folder-1', 'folder', 'Shared', 'active', 'ready', NULL, NULL, NULL, NULL, 1, 'owner-1', 'owner-1'),
         ('file-1', '100001', 'space-1', 'folder-1', 'file', 'Doc', 'active', 'ready', 'application/octet-stream', 'binary', 0, 1, 1, 'owner-1', 'owner-1'),
         ('file-2', '100001', 'space-1', 'folder-2', 'file', 'Shared doc', 'active', 'ready', 'application/octet-stream', 'binary', 0, 1, 1, 'owner-1', 'owner-1')",
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
        .expect("grant inherited writer");
    service
        .grant_node_permission(GrantDriveNodePermissionCommand {
            tenant_id: "100001".to_string(),
            node_id: "folder-2".to_string(),
            subject_type: "user".to_string(),
            subject_id: "writer-1".to_string(),
            role: "reader".to_string(),
            operator_id: "owner-1".to_string(),
        })
        .await
        .expect("grant nearer reader");

    let node_ids = ["file-1", "file-2", "folder-1", "missing-node"]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    let batch = service
        .resolve_node_access_batch("100001", &node_ids, "user", "writer-1")
        .await
        .expect("resolve batch access");

    for node_id in ["file-1", "file-2", "folder-1"] {
        let single = service
            .resolve_effective_node_access(ResolveEffectiveNodeAccessCommand {
                tenant_id: "100001".to_string(),
                space_id: "space-1".to_string(),
                node_id: node_id.to_string(),
                subject_type: "user".to_string(),
                subject_id: "writer-1".to_string(),
            })
            .await
            .expect("resolve single access");
        let batched = batch
            .get(node_id)
            .unwrap_or_else(|| panic!("batch must include {node_id}"));
        assert_eq!(
            batched, &single,
            "batch and single must agree for {node_id}"
        );
    }
    assert!(
        !batch.contains_key("missing-node"),
        "unknown nodes are absent from the batch result"
    );

    // The nearest grant wins: folder-2's reader overrides folder-1's writer.
    let nearest = batch.get("file-2").expect("file-2 access");
    assert_eq!(nearest.role, "reader");
    assert!(nearest.inherited);
    assert_eq!(nearest.inherited_from_node_id.as_deref(), Some("folder-2"));

    // A space owner resolves as owner without any grant.
    let owner_batch = service
        .resolve_node_access_batch(
            "100001",
            &["file-1".to_string(), "file-2".to_string()],
            "user",
            "owner-1",
        )
        .await
        .expect("resolve owner batch");
    for access in owner_batch.values() {
        assert_eq!(access.role, "owner");
        assert_eq!(access.source, "space_owner");
        assert!(!access.inherited);
    }
}
