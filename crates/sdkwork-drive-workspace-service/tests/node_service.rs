use sdkwork_drive_workspace_service::application::node_service::{
    CreateNodeCommand, DriveNodeService,
};
use sdkwork_drive_workspace_service::application::space_service::{
    CreateSpaceCommand, DriveSpaceService,
};
use sdkwork_drive_workspace_service::domain::node::DriveNodeType;
use sdkwork_drive_workspace_service::domain::space::DriveSpaceType;
use sdkwork_drive_workspace_service::infrastructure::sql::node_store::SqlNodeStore;
use sdkwork_drive_workspace_service::infrastructure::sql::space_store::SqlSpaceStore;
use sdkwork_drive_workspace_service::DriveServiceError;

#[tokio::test]
async fn create_folder_enforces_live_name_uniqueness_per_parent() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let space_service = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()));
    let node_service = DriveNodeService::new(SqlNodeStore::new(pool.clone()));

    let space = create_space(&space_service, "space-main", "tenant-001", "user-001").await;

    node_service
        .create_node(CreateNodeCommand {
            id: "node-001".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: space.id.clone(),
            parent_node_id: None,
            node_type: DriveNodeType::Folder,
            node_name: "docs".to_string(),
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("first folder should be created");

    let duplicate = node_service
        .create_node(CreateNodeCommand {
            id: "node-002".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: space.id,
            parent_node_id: None,
            node_type: DriveNodeType::Folder,
            node_name: "docs".to_string(),
            operator_id: "user-001".to_string(),
        })
        .await;

    let error = duplicate.expect_err("duplicate folder name must be rejected");
    let message = format!("{error:?}");
    assert!(message.contains("already exists"));
}

#[tokio::test]
async fn create_folder_rejects_parent_from_another_space() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let space_service = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()));
    let node_service = DriveNodeService::new(SqlNodeStore::new(pool.clone()));
    let source_space = create_space(&space_service, "space-source", "tenant-001", "user-001").await;
    let target_space = create_space(&space_service, "space-target", "tenant-001", "user-002").await;

    let parent = node_service
        .create_node(CreateNodeCommand {
            id: "node-parent".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: source_space.id,
            parent_node_id: None,
            node_type: DriveNodeType::Folder,
            node_name: "apps".to_string(),
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("source parent folder should be created");

    let result = node_service
        .create_node(CreateNodeCommand {
            id: "node-cross-space-child".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: target_space.id,
            parent_node_id: Some(parent.id),
            node_type: DriveNodeType::Folder,
            node_name: "app-alpha".to_string(),
            operator_id: "user-002".to_string(),
        })
        .await;

    assert!(
        matches!(result, Err(DriveServiceError::NotFound(message)) if message.contains("parent"))
    );
}

#[tokio::test]
async fn create_folder_rejects_file_parent() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let space_service = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()));
    let node_service = DriveNodeService::new(SqlNodeStore::new(pool.clone()));
    let space = create_space(&space_service, "space-main", "tenant-001", "user-001").await;

    let parent = node_service
        .create_node(CreateNodeCommand {
            id: "node-file-parent".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: space.id.clone(),
            parent_node_id: None,
            node_type: DriveNodeType::File,
            node_name: "release.zip".to_string(),
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("file parent candidate should be created");

    let result = node_service
        .create_node(CreateNodeCommand {
            id: "node-under-file".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: space.id,
            parent_node_id: Some(parent.id),
            node_type: DriveNodeType::Folder,
            node_name: "invalid-child".to_string(),
            operator_id: "user-001".to_string(),
        })
        .await;

    assert!(
        matches!(result, Err(DriveServiceError::Validation(message)) if message.contains("active folder"))
    );
}

#[tokio::test]
async fn create_folder_rejects_trashed_parent() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let space_service = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()));
    let node_service = DriveNodeService::new(SqlNodeStore::new(pool.clone()));
    let space = create_space(&space_service, "space-main", "tenant-001", "user-001").await;

    let parent = node_service
        .create_node(CreateNodeCommand {
            id: "node-trashed-parent".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: space.id.clone(),
            parent_node_id: None,
            node_type: DriveNodeType::Folder,
            node_name: "old-apps".to_string(),
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("parent folder should be created");

    sqlx::query("UPDATE dr_drive_node SET lifecycle_status='trashed' WHERE id=$1")
        .bind(&parent.id)
        .execute(&pool)
        .await
        .expect("parent should be moved to trash");

    let result = node_service
        .create_node(CreateNodeCommand {
            id: "node-under-trashed".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: space.id,
            parent_node_id: Some(parent.id),
            node_type: DriveNodeType::Folder,
            node_name: "invalid-child".to_string(),
            operator_id: "user-001".to_string(),
        })
        .await;

    assert!(
        matches!(result, Err(DriveServiceError::NotFound(message)) if message.contains("parent"))
    );
}

#[tokio::test]
async fn create_folder_rejects_self_parent() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let space_service = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()));
    let node_service = DriveNodeService::new(SqlNodeStore::new(pool));
    let space = create_space(&space_service, "space-main", "tenant-001", "user-001").await;

    let result = node_service
        .create_node(CreateNodeCommand {
            id: "node-self-parent".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: space.id,
            parent_node_id: Some("node-self-parent".to_string()),
            node_type: DriveNodeType::Folder,
            node_name: "invalid".to_string(),
            operator_id: "user-001".to_string(),
        })
        .await;

    assert!(
        matches!(result, Err(DriveServiceError::Validation(message)) if message.contains("parent_node_id"))
    );
}

#[tokio::test]
async fn create_node_rejects_file_at_git_repository_space_root_but_allows_files_inside_repository_directory(
) {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let space_service = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()));
    let node_service = DriveNodeService::new(SqlNodeStore::new(pool));
    let git_repository_space = create_space_with_type(
        &space_service,
        "space-git-repository",
        "tenant-001",
        "user-001",
        DriveSpaceType::GitRepository,
    )
    .await;

    let root_file = node_service
        .create_node(CreateNodeCommand {
            id: "file-git-repository-root".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: git_repository_space.id.clone(),
            parent_node_id: None,
            node_type: DriveNodeType::File,
            node_name: "root-source.zip".to_string(),
            operator_id: "user-001".to_string(),
        })
        .await;
    assert!(
        matches!(root_file, Err(DriveServiceError::Validation(message)) if message.contains("git repository space root"))
    );

    let repository_directory = node_service
        .create_node(CreateNodeCommand {
            id: "folder-repository-alpha".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: git_repository_space.id.clone(),
            parent_node_id: None,
            node_type: DriveNodeType::Folder,
            node_name: "repository-alpha".to_string(),
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("repository directory should be created at git repository space root");

    let source_file = node_service
        .create_node(CreateNodeCommand {
            id: "file-repository-source".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: git_repository_space.id,
            parent_node_id: Some(repository_directory.id),
            node_type: DriveNodeType::File,
            node_name: "source.zip".to_string(),
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("file should be created inside a repository directory");
    assert_eq!(source_file.node_type, DriveNodeType::File);
}

#[tokio::test]
async fn create_node_allows_file_at_deployment_space_root() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let space_service = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()));
    let node_service = DriveNodeService::new(SqlNodeStore::new(pool));
    let deployment_space = create_space_with_type(
        &space_service,
        "space-deployment",
        "tenant-001",
        "app-001",
        DriveSpaceType::Deployment,
    )
    .await;

    let root_file = node_service
        .create_node(CreateNodeCommand {
            id: "file-deployment-root".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: deployment_space.id,
            parent_node_id: None,
            node_type: DriveNodeType::File,
            node_name: "index.html".to_string(),
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("deployment space should accept files at root");

    assert_eq!(root_file.node_type, DriveNodeType::File);
}

#[tokio::test]
async fn create_node_denormalizes_notary_space_type_from_parent_space() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let space_service = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()));
    let node_service = DriveNodeService::new(SqlNodeStore::new(pool.clone()));
    let notary_space = space_service
        .create_space(CreateSpaceCommand {
            id: "space-notary".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "organization".to_string(),
            owner_subject_id: "organization-001".to_string(),
            display_name: "Notary".to_string(),
            space_type: DriveSpaceType::Notary,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "member-001".to_string(),
        })
        .await
        .expect("organization notary space should be created");

    let case_folder = node_service
        .create_node(CreateNodeCommand {
            id: "folder-notary-case-001".to_string(),
            tenant_id: "tenant-001".to_string(),
            space_id: notary_space.id,
            parent_node_id: None,
            node_type: DriveNodeType::Folder,
            node_name: "notary-case-001".to_string(),
            operator_id: "member-001".to_string(),
        })
        .await
        .expect("notary case folder should be created");

    assert_eq!(case_folder.space_type, DriveSpaceType::Notary);

    let persisted_space_type: String =
        sqlx::query_scalar("SELECT space_type FROM dr_drive_node WHERE id=$1")
            .bind(&case_folder.id)
            .fetch_one(&pool)
            .await
            .expect("node space_type should be persisted on dr_drive_node");
    assert_eq!(persisted_space_type, "notary");
}

async fn create_space(
    service: &DriveSpaceService<SqlSpaceStore>,
    id: &str,
    tenant_id: &str,
    owner_subject_id: &str,
) -> sdkwork_drive_workspace_service::domain::space::DriveSpace {
    create_space_with_type(
        service,
        id,
        tenant_id,
        owner_subject_id,
        DriveSpaceType::Personal,
    )
    .await
}

async fn create_space_with_type(
    service: &DriveSpaceService<SqlSpaceStore>,
    id: &str,
    tenant_id: &str,
    owner_subject_id: &str,
    space_type: DriveSpaceType,
) -> sdkwork_drive_workspace_service::domain::space::DriveSpace {
    service
        .create_space(CreateSpaceCommand {
            id: id.to_string(),
            tenant_id: tenant_id.to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: owner_subject_id.to_string(),
            display_name: "Main".to_string(),
            space_type,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: owner_subject_id.to_string(),
        })
        .await
        .expect("space should be created")
}
