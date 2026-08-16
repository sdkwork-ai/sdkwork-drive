use sdkwork_drive_workspace_service::application::root_scope_subscription_service::{
    DriveRootScopeSubscriptionService, RegisterKnowledgebaseRawScopeCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::next_drive_runtime_id;
use sdkwork_drive_workspace_service::infrastructure::sql::root_scope_subscription_store::SqlRootScopeSubscriptionStore;
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::PgPool;

async fn insert_knowledgebase_raw_tree(
    pool: &PgPool,
    tenant_id: &str,
    owner_subject_id: &str,
    space_id: &str,
    root_node_id: &str,
    sources_node_id: &str,
    raw_node_id: &str,
) {
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, 'app', $3, 'knowledge_base',
                   'Knowledgebase files', 'active', 1, 'service-kb', 'service-kb')",
    )
    .bind(space_id)
    .bind(tenant_id)
    .bind(owner_subject_id)
    .execute(pool)
    .await
    .expect("knowledge_base Space should be inserted");
    for (node_id, parent_node_id, node_name) in [
        (root_node_id, None, "Knowledgebase files"),
        (sources_node_id, Some(root_node_id), "sources"),
        (raw_node_id, Some(sources_node_id), "raw"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, space_type, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
             ) VALUES ($1, $2, $3, 'knowledge_base', $4, 'folder', $5,
                       'ready', 'active', 1, 'service-kb', 'service-kb')",
        )
        .bind(node_id)
        .bind(tenant_id)
        .bind(space_id)
        .bind(parent_node_id)
        .bind(node_name)
        .execute(pool)
        .await
        .expect("knowledgebase folder should be inserted");
    }
}

fn register_command(
    tenant_id: &str,
    space_id: &str,
    knowledge_base_id: &str,
    raw_folder_node_id: &str,
) -> RegisterKnowledgebaseRawScopeCommand {
    RegisterKnowledgebaseRawScopeCommand {
        tenant_id: tenant_id.to_string(),
        space_id: space_id.to_string(),
        knowledge_base_id: knowledge_base_id.to_string(),
        raw_folder_node_id: raw_folder_node_id.to_string(),
        operator_id: "service-kb".to_string(),
    }
}

#[tokio::test]
async fn knowledgebase_raw_registration_is_idempotent_and_cannot_be_retargeted() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_knowledgebase_raw_tree(
        &pool,
        "tenant-kb",
        "knowledge-base-owner-one",
        "space-kb-one",
        "root-one",
        "sources-one",
        "raw-one",
    )
    .await;
    insert_knowledgebase_raw_tree(
        &pool,
        "tenant-kb",
        "knowledge-base-owner-two",
        "space-kb-two",
        "root-two",
        "sources-two",
        "raw-two",
    )
    .await;
    let service =
        DriveRootScopeSubscriptionService::new(SqlRootScopeSubscriptionStore::new(pool.clone()));

    let created = service
        .register_knowledgebase_raw(register_command(
            "tenant-kb",
            "space-kb-one",
            "knowledge-base-001",
            "raw-one",
        ))
        .await
        .expect("knowledgebase raw subscription should be created");
    assert!(created.created);
    assert_eq!(created.subscription.consumer_kind, "knowledgebase_raw");
    assert_eq!(created.subscription.root_node_id, "raw-one");

    let replay = service
        .register_knowledgebase_raw(register_command(
            "tenant-kb",
            "space-kb-one",
            "knowledge-base-001",
            "raw-one",
        ))
        .await
        .expect("same knowledgebase raw subscription should replay");
    assert!(!replay.created);
    assert_eq!(created.subscription.uuid, replay.subscription.uuid);

    let retarget = service
        .register_knowledgebase_raw(register_command(
            "tenant-kb",
            "space-kb-two",
            "knowledge-base-001",
            "raw-two",
        ))
        .await
        .expect_err("knowledgebase root scope retarget must be rejected");
    assert!(matches!(retarget, DriveServiceError::Conflict(_)));
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_root_scope_subscription
         WHERE tenant_id='tenant-kb' AND consumer_resource_id='knowledge-base-001'",
    )
    .fetch_one(&pool)
    .await
    .expect("subscription count should be queryable");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn knowledgebase_raw_registration_requires_exact_sources_raw_identity() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_knowledgebase_raw_tree(
        &pool,
        "tenant-invalid",
        "knowledge-base-invalid",
        "space-invalid",
        "root-invalid",
        "sources-invalid",
        "raw-invalid",
    )
    .await;
    let service =
        DriveRootScopeSubscriptionService::new(SqlRootScopeSubscriptionStore::new(pool.clone()));

    let wrong_level = service
        .register_knowledgebase_raw(register_command(
            "tenant-invalid",
            "space-invalid",
            "knowledge-base-wrong-level",
            "sources-invalid",
        ))
        .await
        .expect_err("sources folder must not be accepted as raw");
    assert!(matches!(wrong_level, DriveServiceError::Validation(_)));

    sqlx::query("UPDATE dr_drive_node SET node_name='Raw' WHERE id='raw-invalid'")
        .execute(&pool)
        .await
        .expect("raw folder should be renamed for validation test");
    let wrong_case = service
        .register_knowledgebase_raw(register_command(
            "tenant-invalid",
            "space-invalid",
            "knowledge-base-wrong-case",
            "raw-invalid",
        ))
        .await
        .expect_err("raw path identity must be exact and case-sensitive");
    assert!(matches!(wrong_case, DriveServiceError::Validation(_)));

    let personal_space_id = format!("space-personal-{}", next_drive_runtime_id("test").unwrap());
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, 'tenant-invalid', 'user', 'user-1', 'personal',
                   'Personal', 'active', 1, 'user-1', 'user-1')",
    )
    .bind(&personal_space_id)
    .execute(&pool)
    .await
    .expect("personal Space should be inserted");
    let non_knowledgebase = service
        .register_knowledgebase_raw(register_command(
            "tenant-invalid",
            &personal_space_id,
            "knowledge-base-personal",
            "raw-invalid",
        ))
        .await
        .expect_err("non-knowledge_base Space must be rejected");
    assert!(matches!(
        non_knowledgebase,
        DriveServiceError::Validation(_)
    ));
}
