use sdkwork_drive_workspace_service::application::space_service::{
    CreateSpaceCommand, DeleteSpaceCommand, DriveSpaceService, GetSpaceCommand, ListSpacesCommand,
    SqlDriveSpaceService, UpdateSpaceCommand,
};
use sdkwork_drive_workspace_service::domain::space::DriveSpaceType;
use sdkwork_drive_workspace_service::infrastructure::sql::space_store::SqlSpaceStore;
use sdkwork_drive_workspace_service::DriveServiceError;

#[test]
fn drive_space_type_maps_rtc_space_type() {
    let parsed =
        DriveSpaceType::try_from_str("rtc").expect("rtc should be a first-class Drive space type");

    assert_eq!(parsed.as_str(), "rtc");
}

#[tokio::test]
async fn create_space_supports_knowledge_ai_git_repository_deployment_and_upload_types() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let store = SqlSpaceStore::new(pool.clone());
    let service = DriveSpaceService::new(store);

    for (index, space_type) in [
        DriveSpaceType::KnowledgeBase,
        DriveSpaceType::AiGenerated,
        DriveSpaceType::GitRepository,
        DriveSpaceType::Deployment,
        DriveSpaceType::AppUpload,
        DriveSpaceType::Im,
        DriveSpaceType::Notary,
    ]
    .into_iter()
    .enumerate()
    {
        let created = service
            .create_space(CreateSpaceCommand {
                id: format!("space-{index}"),
                tenant_id: "tenant-001".to_string(),
                owner_subject_type: "user".to_string(),
                owner_subject_id: "user-001".to_string(),
                display_name: format!("space-{index}"),
                space_type: space_type.clone(),
                presentation_icon: None,
                presentation_color: None,
                description: None,
                operator_id: "user-001".to_string(),
            })
            .await
            .expect("space creation should succeed");
        assert_eq!(created.space_type, space_type);
    }
}

#[tokio::test]
async fn create_rtc_space_is_user_owned_and_unique_per_user() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveSpaceService::new(SqlSpaceStore::new(pool));
    let created = service
        .create_space(CreateSpaceCommand {
            id: "space-rtc-user-001".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-001".to_string(),
            display_name: "RTC Records".to_string(),
            space_type: DriveSpaceType::Rtc,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "system".to_string(),
        })
        .await
        .expect("user RTC space should be created");

    assert_eq!(created.space_type, DriveSpaceType::Rtc);
    assert_eq!(created.owner_subject_type, "user");
    assert_eq!(created.owner_subject_id, "user-001");

    let duplicate = service
        .create_space(CreateSpaceCommand {
            id: "space-rtc-user-duplicate".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-001".to_string(),
            display_name: "RTC Records Duplicate".to_string(),
            space_type: DriveSpaceType::Rtc,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "system".to_string(),
        })
        .await;

    assert!(matches!(
        duplicate,
        Err(DriveServiceError::Conflict(message))
            if message.contains("tenant/owner/type")
    ));
}

#[tokio::test]
async fn create_rtc_space_rejects_non_user_owner() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveSpaceService::new(SqlSpaceStore::new(pool));

    let result = service
        .create_space(CreateSpaceCommand {
            id: "space-rtc-group-owner".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "group".to_string(),
            owner_subject_id: "group-001".to_string(),
            display_name: "RTC Records".to_string(),
            space_type: DriveSpaceType::Rtc,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "system".to_string(),
        })
        .await;

    assert!(
        matches!(result, Err(DriveServiceError::Validation(message)) if message.contains("rtc space") && message.contains("user"))
    );
}

#[tokio::test]
async fn create_git_repository_space_rejects_non_user_owner() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveSpaceService::new(SqlSpaceStore::new(pool));

    let result = service
        .create_space(CreateSpaceCommand {
            id: "space-git-repository-group-owner".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "group".to_string(),
            owner_subject_id: "group-001".to_string(),
            display_name: "Repositories".to_string(),
            space_type: DriveSpaceType::GitRepository,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-001".to_string(),
        })
        .await;

    assert!(
        matches!(result, Err(DriveServiceError::Validation(message)) if message.contains("git repository space") && message.contains("user"))
    );
}

#[tokio::test]
async fn create_deployment_space_allows_app_owner() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveSpaceService::new(SqlSpaceStore::new(pool));

    let created = service
        .create_space(CreateSpaceCommand {
            id: "space-deployment-app-owner".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "app".to_string(),
            owner_subject_id: "app-001".to_string(),
            display_name: "Deployments".to_string(),
            space_type: DriveSpaceType::Deployment,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("deployment space should accept app ownership");

    assert_eq!(created.space_type, DriveSpaceType::Deployment);
    assert_eq!(created.owner_subject_type, "app");
    assert_eq!(created.owner_subject_id, "app-001");
}

#[tokio::test]
async fn delete_space_rejects_user_git_repository_space() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveSpaceService::new(SqlSpaceStore::new(pool));
    service
        .create_space(CreateSpaceCommand {
            id: "space-git-repository-delete-guard".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-001".to_string(),
            display_name: "Git Repositories".to_string(),
            space_type: DriveSpaceType::GitRepository,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("git repository space should be created");

    let result = service
        .delete_space(DeleteSpaceCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: "space-git-repository-delete-guard".to_string(),
            operator_id: "user-001".to_string(),
        })
        .await;

    assert!(
        matches!(result, Err(DriveServiceError::Validation(message)) if message.contains("git repository space") && message.contains("cannot be deleted"))
    );
    let still_active = service
        .get_space(GetSpaceCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: "space-git-repository-delete-guard".to_string(),
        })
        .await
        .expect("git repository space should remain active");
    assert_eq!(still_active.lifecycle_status, "active");
}

#[tokio::test]
async fn list_spaces_supports_tenant_and_owner_filters() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let store = SqlSpaceStore::new(pool);
    let service = DriveSpaceService::new(store);

    service
        .create_space(CreateSpaceCommand {
            id: "space-001".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-001".to_string(),
            display_name: "main".to_string(),
            space_type: DriveSpaceType::Personal,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-001".to_string(),
        })
        .await
        .expect("first space should be created");
    service
        .create_space(CreateSpaceCommand {
            id: "space-002".to_string(),
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-002".to_string(),
            display_name: "kb".to_string(),
            space_type: DriveSpaceType::KnowledgeBase,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-002".to_string(),
        })
        .await
        .expect("second space should be created");
    service
        .create_space(CreateSpaceCommand {
            id: "space-003".to_string(),
            tenant_id: "tenant-002".to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-003".to_string(),
            display_name: "other".to_string(),
            space_type: DriveSpaceType::Team,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-003".to_string(),
        })
        .await
        .expect("third space should be created");

    let tenant_spaces = service
        .list_spaces(ListSpacesCommand {
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: None,
            owner_subject_id: None,
            space_type: None,
            offset: 0,
            limit: 201,
        })
        .await
        .expect("tenant space list should succeed");
    assert_eq!(tenant_spaces.len(), 2);

    let owner_spaces = service
        .list_spaces(ListSpacesCommand {
            tenant_id: "tenant-001".to_string(),
            owner_subject_type: Some("user".to_string()),
            owner_subject_id: Some("user-002".to_string()),
            space_type: None,
            offset: 0,
            limit: 201,
        })
        .await
        .expect("owner space list should succeed");
    assert_eq!(owner_spaces.len(), 1);
    assert_eq!(owner_spaces[0].id, "space-002");
}

#[tokio::test]
async fn space_service_get_update_and_delete_manage_space_lifecycle() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let store = SqlSpaceStore::new(pool.clone());
    let service = DriveSpaceService::new(store);

    service
        .create_space(CreateSpaceCommand {
            id: "space-lifecycle".to_string(),
            tenant_id: "tenant-lifecycle".to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-owner".to_string(),
            display_name: "Lifecycle".to_string(),
            space_type: DriveSpaceType::Team,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-owner".to_string(),
        })
        .await
        .expect("space should be created");

    let found = service
        .get_space(GetSpaceCommand {
            tenant_id: "tenant-lifecycle".to_string(),
            space_id: "space-lifecycle".to_string(),
        })
        .await
        .expect("space should be found");
    assert_eq!(found.display_name, "Lifecycle");
    assert_eq!(found.version, 1);

    let updated = service
        .update_space(UpdateSpaceCommand {
            tenant_id: "tenant-lifecycle".to_string(),
            space_id: "space-lifecycle".to_string(),
            display_name: Some("Lifecycle Updated".to_string()),
            presentation_icon: None,

            presentation_color: None,

            description: None,

            operator_id: "user-admin".to_string(),
        })
        .await
        .expect("space should be updated");
    assert_eq!(updated.display_name, "Lifecycle Updated");
    assert_eq!(updated.version, 2);

    let deleted = service
        .delete_space(DeleteSpaceCommand {
            tenant_id: "tenant-lifecycle".to_string(),
            space_id: "space-lifecycle".to_string(),
            operator_id: "user-admin".to_string(),
        })
        .await
        .expect("space should be deleted");
    assert_eq!(deleted.lifecycle_status, "deleted");
    assert_eq!(deleted.version, 3);

    let listed = service
        .list_spaces(ListSpacesCommand {
            tenant_id: "tenant-lifecycle".to_string(),
            owner_subject_type: None,
            owner_subject_id: None,
            space_type: None,
            offset: 0,
            limit: 201,
        })
        .await
        .expect("space list should succeed");
    assert!(
        listed.is_empty(),
        "deleted spaces should not be listed as active"
    );

    let missing = service
        .get_space(GetSpaceCommand {
            tenant_id: "tenant-lifecycle".to_string(),
            space_id: "space-lifecycle".to_string(),
        })
        .await
        .expect_err("deleted space should not be returned by get");
    assert_eq!(
        missing,
        DriveServiceError::NotFound("space not found".to_string())
    );
}

#[tokio::test]
async fn sql_drive_space_service_exposes_space_operations_without_callers_using_sql_store() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = SqlDriveSpaceService::new(pool);
    let created = service
        .create_space(CreateSpaceCommand {
            id: "kb-space-001".to_string(),
            tenant_id: "tenant-knowledge".to_string(),
            owner_subject_type: "app".to_string(),
            owner_subject_id: "sdkwork-knowledgebase:space-uuid-001".to_string(),
            display_name: "Knowledge Space".to_string(),
            space_type: DriveSpaceType::KnowledgeBase,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-123".to_string(),
        })
        .await
        .expect("knowledge space should be created through SQL workspace service");

    assert_eq!(created.id, "kb-space-001");
    assert_eq!(created.space_type, DriveSpaceType::KnowledgeBase);
    assert_eq!(
        created.owner_subject_id,
        "sdkwork-knowledgebase:space-uuid-001"
    );

    let listed = service
        .list_spaces(ListSpacesCommand {
            tenant_id: "tenant-knowledge".to_string(),
            owner_subject_type: Some("app".to_string()),
            owner_subject_id: Some("sdkwork-knowledgebase:space-uuid-001".to_string()),
            space_type: None,
            offset: 0,
            limit: 201,
        })
        .await
        .expect("knowledge spaces should be listed through SQL workspace service");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, "kb-space-001");

    let found = service
        .get_space(GetSpaceCommand {
            tenant_id: "tenant-knowledge".to_string(),
            space_id: "kb-space-001".to_string(),
        })
        .await
        .expect("knowledge space should be loaded through SQL workspace service");
    assert_eq!(found.display_name, "Knowledge Space");
}

#[tokio::test]
async fn space_service_rejects_invalid_owner_and_operator_before_store_write() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = SqlDriveSpaceService::new(pool);
    let invalid_owner = service
        .create_space(CreateSpaceCommand {
            id: "kb-space-invalid-owner".to_string(),
            tenant_id: "tenant-knowledge".to_string(),
            owner_subject_type: "system".to_string(),
            owner_subject_id: "sdkwork-knowledgebase:space-uuid-001".to_string(),
            display_name: "Knowledge Space".to_string(),
            space_type: DriveSpaceType::KnowledgeBase,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-123".to_string(),
        })
        .await
        .expect_err("invalid owner_subject_type should be rejected by service validation");
    assert_eq!(
        invalid_owner,
        DriveServiceError::Validation(
            "owner_subject_type must be app, user, group, or organization".to_string()
        )
    );

    let missing_operator = service
        .create_space(CreateSpaceCommand {
            id: "kb-space-missing-operator".to_string(),
            tenant_id: "tenant-knowledge".to_string(),
            owner_subject_type: "app".to_string(),
            owner_subject_id: "sdkwork-knowledgebase:space-uuid-001".to_string(),
            display_name: "Knowledge Space".to_string(),
            space_type: DriveSpaceType::KnowledgeBase,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: " ".to_string(),
        })
        .await
        .expect_err("operator_id should be rejected by service validation");
    assert_eq!(
        missing_operator,
        DriveServiceError::Validation("operator_id is required".to_string())
    );
}
