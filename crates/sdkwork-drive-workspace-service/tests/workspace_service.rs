use sdkwork_drive_workspace_service::application::space_service::{
    CreateSpaceCommand, DriveSpaceService,
};
use sdkwork_drive_workspace_service::application::workspace_service::{
    DriveWorkspaceNodeKind, DriveWorkspaceObjectRef, DriveWorkspaceService,
    EnsureDriveWorkspaceNode, EnsureDriveWorkspaceNodesCommand, GetDriveWorkspaceNodeCommand,
    ListDriveWorkspaceChildrenCommand, ResolveDriveWorkspacePathCommand,
};
use sdkwork_drive_workspace_service::domain::space::DriveSpaceType;
use sdkwork_drive_workspace_service::infrastructure::sql::space_store::SqlSpaceStore;
use sdkwork_drive_workspace_service::infrastructure::sql::workspace_store::SqlDriveWorkspaceStore;
use std::sync::Arc;

#[tokio::test]
async fn workspace_service_ensures_nodes_and_lists_children_from_drive_schema() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_storage_provider(&pool, "provider-kb", "kb-bucket").await;

    let space = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()))
        .create_space(CreateSpaceCommand {
            id: "drv-kb-001".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "app".to_string(),
            owner_subject_id: "sdkwork-knowledgebase".to_string(),
            display_name: "Knowledge".to_string(),
            space_type: DriveSpaceType::KnowledgeBase,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "system".to_string(),
        })
        .await
        .expect("knowledge drive space should be created");
    let service = DriveWorkspaceService::new(SqlDriveWorkspaceStore::new(pool.clone()));
    let command = EnsureDriveWorkspaceNodesCommand {
        tenant_id: "tenant-001".to_string(),
        space_id: space.id.clone(),
        operator_id: "system".to_string(),
        nodes: vec![
            EnsureDriveWorkspaceNode::folder("wiki"),
            EnsureDriveWorkspaceNode::folder("wiki/schema"),
            EnsureDriveWorkspaceNode::file(
                "wiki/schema/AGENTS.md",
                DriveWorkspaceObjectRef {
                    storage_provider_id: "provider-kb".to_string(),
                    bucket: "kb-bucket".to_string(),
                    object_key: "knowledge/space/wiki/schema/AGENTS.md".to_string(),
                    content_type: "text/markdown; charset=utf-8".to_string(),
                    content_length: 64,
                    checksum_sha256_hex:
                        "9cb34ab8b2d953ad722c1d727df449e0a216e4fe12f5433a3a945db596d792fb"
                            .to_string(),
                },
            ),
        ],
    };

    service
        .ensure_nodes(command.clone())
        .await
        .expect("workspace nodes should be created");
    service
        .ensure_nodes(command)
        .await
        .expect("workspace node creation should be idempotent");

    let root = service
        .resolve_path(ResolveDriveWorkspacePathCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: space.id.clone(),
            logical_path: "wiki".to_string(),
        })
        .await
        .expect("root path should resolve")
        .expect("root path should exist");
    assert_eq!(root.name, "wiki");
    assert_eq!(root.kind, DriveWorkspaceNodeKind::Folder);

    let page = service
        .list_children(ListDriveWorkspaceChildrenCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: space.id.clone(),
            parent_node_id: Some(root.id),
            offset: 0,
            page_size: 200,
        })
        .await
        .expect("children should be listed");
    assert_eq!(page.nodes.len(), 1);
    assert_eq!(page.nodes[0].name, "schema");
    assert_eq!(page.nodes[0].path, "wiki/schema");
    assert_eq!(page.nodes[0].kind, DriveWorkspaceNodeKind::Folder);
    assert_eq!(page.next_offset, None);

    let schema_page = service
        .list_children(ListDriveWorkspaceChildrenCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: space.id,
            parent_node_id: Some(page.nodes[0].id.clone()),
            offset: 0,
            page_size: 200,
        })
        .await
        .expect("nested children should be listed");
    assert_eq!(schema_page.nodes.len(), 1);
    assert_eq!(schema_page.nodes[0].name, "AGENTS.md");
    assert_eq!(schema_page.nodes[0].path, "wiki/schema/AGENTS.md");
    assert_eq!(schema_page.nodes[0].kind, DriveWorkspaceNodeKind::File);
    assert_eq!(
        schema_page.nodes[0].content_type.as_deref(),
        Some("text/markdown")
    );
    assert_eq!(schema_page.nodes[0].content_length, Some(64));
    assert_eq!(schema_page.next_offset, None);

    let node_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_node")
        .fetch_one(&pool)
        .await
        .expect("node count should be readable");
    let object_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_storage_object")
        .fetch_one(&pool)
        .await
        .expect("storage object count should be readable");
    let file_version: i64 = sqlx::query_scalar(
        "SELECT version
         FROM dr_drive_node
         WHERE tenant_id='tenant-001'
           AND space_id='drv-kb-001'
           AND node_name='AGENTS.md'
           AND lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("file node version should be readable");
    assert_eq!(node_count, 3);
    assert_eq!(object_count, 1);
    assert_eq!(file_version, 2);
}

#[tokio::test]
async fn workspace_service_gets_node_by_id_with_resolved_path() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_storage_provider(&pool, "provider-kb", "kb-bucket").await;

    let space = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()))
        .create_space(CreateSpaceCommand {
            id: "drv-kb-001".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "app".to_string(),
            owner_subject_id: "sdkwork-knowledgebase".to_string(),
            display_name: "Knowledge".to_string(),
            space_type: DriveSpaceType::KnowledgeBase,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "system".to_string(),
        })
        .await
        .expect("knowledge drive space should be created");
    let service = DriveWorkspaceService::new(SqlDriveWorkspaceStore::new(pool));

    service
        .ensure_nodes(EnsureDriveWorkspaceNodesCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: space.id.clone(),
            operator_id: "system".to_string(),
            nodes: vec![
                EnsureDriveWorkspaceNode::folder("wiki"),
                EnsureDriveWorkspaceNode::folder("wiki/schema"),
                EnsureDriveWorkspaceNode::file(
                    "wiki/schema/AGENTS.md",
                    DriveWorkspaceObjectRef {
                        storage_provider_id: "provider-kb".to_string(),
                        bucket: "kb-bucket".to_string(),
                        object_key: "knowledge/space/wiki/schema/AGENTS.md".to_string(),
                        content_type: "text/markdown; charset=utf-8".to_string(),
                        content_length: 64,
                        checksum_sha256_hex:
                            "9cb34ab8b2d953ad722c1d727df449e0a216e4fe12f5433a3a945db596d792fb"
                                .to_string(),
                    },
                ),
            ],
        })
        .await
        .expect("workspace nodes should be created");

    let file = service
        .resolve_path(ResolveDriveWorkspacePathCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: space.id.clone(),
            logical_path: "wiki/schema/AGENTS.md".to_string(),
        })
        .await
        .expect("file path should resolve")
        .expect("file path should exist");

    let loaded = service
        .get_node(GetDriveWorkspaceNodeCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: space.id,
            node_id: file.id.clone(),
        })
        .await
        .expect("node should load")
        .expect("node should exist");

    assert_eq!(loaded.id, file.id);
    assert_eq!(loaded.path, "wiki/schema/AGENTS.md");
    assert_eq!(loaded.kind, DriveWorkspaceNodeKind::File);
    assert_eq!(loaded.content_length, Some(64));
}

#[tokio::test]
async fn workspace_service_versions_file_metadata_when_same_path_content_changes() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_storage_provider(&pool, "provider-kb", "kb-bucket").await;

    let space = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()))
        .create_space(CreateSpaceCommand {
            id: "drv-kb-001".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "app".to_string(),
            owner_subject_id: "sdkwork-knowledgebase".to_string(),
            display_name: "Knowledge".to_string(),
            space_type: DriveSpaceType::KnowledgeBase,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "system".to_string(),
        })
        .await
        .expect("knowledge drive space should be created");
    let service = DriveWorkspaceService::new(SqlDriveWorkspaceStore::new(pool.clone()));

    service
        .ensure_nodes(EnsureDriveWorkspaceNodesCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: space.id.clone(),
            operator_id: "system".to_string(),
            nodes: vec![EnsureDriveWorkspaceNode::file(
                "wiki/index.md",
                DriveWorkspaceObjectRef {
                    storage_provider_id: "provider-kb".to_string(),
                    bucket: "kb-bucket".to_string(),
                    object_key: "knowledge/space/wiki/index.md".to_string(),
                    content_type: "text/markdown; charset=utf-8".to_string(),
                    content_length: 64,
                    checksum_sha256_hex:
                        "9cb34ab8b2d953ad722c1d727df449e0a216e4fe12f5433a3a945db596d792fb"
                            .to_string(),
                },
            )],
        })
        .await
        .expect("first workspace file should be created");
    service
        .ensure_nodes(EnsureDriveWorkspaceNodesCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: space.id.clone(),
            operator_id: "system".to_string(),
            nodes: vec![EnsureDriveWorkspaceNode::file(
                "wiki/index.md",
                DriveWorkspaceObjectRef {
                    storage_provider_id: "provider-kb".to_string(),
                    bucket: "kb-bucket".to_string(),
                    object_key: "knowledge/space/wiki/index.md".to_string(),
                    content_type: "text/markdown; charset=utf-8".to_string(),
                    content_length: 96,
                    checksum_sha256_hex:
                        "cf14d3c0ac1a091f0d3719743af0e8ffa7a6a8fe6b555185766be273b995177f"
                            .to_string(),
                },
            )],
        })
        .await
        .expect("updated workspace file should be versioned");

    let file = service
        .resolve_path(ResolveDriveWorkspacePathCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: space.id,
            logical_path: "wiki/index.md".to_string(),
        })
        .await
        .expect("file path should resolve")
        .expect("file path should exist");
    assert_eq!(file.content_length, Some(96));

    let active_object_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-001'
           AND lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("active object count should be readable");
    let deleted_object_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-001'
           AND lifecycle_status='deleted'",
    )
    .fetch_one(&pool)
    .await
    .expect("deleted object count should be readable");
    let latest_version_no: i64 = sqlx::query_scalar(
        "SELECT version_no
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-001'
           AND lifecycle_status='active'
         ORDER BY version_no DESC
         LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("latest object version should be readable");
    assert_eq!(active_object_count, 1);
    assert_eq!(deleted_object_count, 1);
    assert_eq!(latest_version_no, 2);
}

#[tokio::test]
async fn workspace_service_concurrently_ensures_same_nodes_idempotently() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_storage_provider(&pool, "provider-kb", "kb-bucket").await;

    let space = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()))
        .create_space(CreateSpaceCommand {
            id: "drv-kb-concurrent".to_string(),
            tenant_id: "tenant-concurrent".to_string(),
            owner_subject_type: "app".to_string(),
            owner_subject_id: "sdkwork-knowledgebase".to_string(),
            display_name: "Concurrent Knowledge".to_string(),
            space_type: DriveSpaceType::KnowledgeBase,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "system".to_string(),
        })
        .await
        .expect("knowledge drive space should be created");

    let service = Arc::new(DriveWorkspaceService::new(SqlDriveWorkspaceStore::new(
        pool.clone(),
    )));
    let command = EnsureDriveWorkspaceNodesCommand {
        tenant_id: "tenant-concurrent".to_string(),
        space_id: space.id.clone(),
        operator_id: "system".to_string(),
        nodes: vec![
            EnsureDriveWorkspaceNode::folder("wiki"),
            EnsureDriveWorkspaceNode::folder("wiki/schema"),
            EnsureDriveWorkspaceNode::file(
                "wiki/schema/AGENTS.md",
                DriveWorkspaceObjectRef {
                    storage_provider_id: "provider-kb".to_string(),
                    bucket: "kb-bucket".to_string(),
                    object_key: "knowledge/space/wiki/schema/AGENTS.md".to_string(),
                    content_type: "text/markdown; charset=utf-8".to_string(),
                    content_length: 64,
                    checksum_sha256_hex:
                        "9cb34ab8b2d953ad722c1d727df449e0a216e4fe12f5433a3a945db596d792fb"
                            .to_string(),
                },
            ),
        ],
    };

    let mut handles = Vec::new();
    for _ in 0..16 {
        let service = Arc::clone(&service);
        let command = command.clone();
        handles.push(tokio::spawn(
            async move { service.ensure_nodes(command).await },
        ));
    }

    for handle in handles {
        handle
            .await
            .expect("concurrent ensure task should not panic")
            .expect("concurrent workspace ensure should be idempotent");
    }

    let node_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-concurrent'
           AND space_id='drv-kb-concurrent'
           AND lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("node count should be readable");
    let object_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-concurrent'
           AND lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("storage object count should be readable");
    let file_version: i64 = sqlx::query_scalar(
        "SELECT version
         FROM dr_drive_node
         WHERE tenant_id='tenant-concurrent'
           AND space_id='drv-kb-concurrent'
           AND node_name='AGENTS.md'
           AND lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("file node version should be readable");
    assert_eq!(node_count, 3);
    assert_eq!(object_count, 1);
    assert_eq!(file_version, 2);

    let root = service
        .resolve_path(ResolveDriveWorkspacePathCommand {
            tenant_id: "tenant-concurrent".to_string(),
            space_id: space.id.clone(),
            logical_path: "wiki".to_string(),
        })
        .await
        .expect("root path should resolve")
        .expect("root path should exist");
    let page = service
        .list_children(ListDriveWorkspaceChildrenCommand {
            tenant_id: "tenant-concurrent".to_string(),
            space_id: space.id,
            parent_node_id: Some(root.id),
            offset: 0,
            page_size: 200,
        })
        .await
        .expect("children should be listed");
    assert_eq!(page.nodes.len(), 1);
    assert_eq!(page.nodes[0].name, "schema");
}

async fn seed_storage_provider(pool: &sqlx::PgPool, provider_id: &str, bucket: &str) {
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
    .bind(provider_id)
    .bind(bucket)
    .execute(pool)
    .await
    .expect("seed storage provider should succeed");
}
