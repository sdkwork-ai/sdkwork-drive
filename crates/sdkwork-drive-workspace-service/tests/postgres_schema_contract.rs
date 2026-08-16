use sdkwork_drive_workspace_service::application::space_lifecycle_service::{
    DeleteSpaceWithContentsCommand, SqlDriveSpaceLifecycleService,
};
use sdkwork_drive_workspace_service::application::space_service::{
    CreateSpaceCommand, DriveSpaceService,
};
use sdkwork_drive_workspace_service::domain::space::DriveSpaceType;
use sdkwork_drive_workspace_service::infrastructure::sql::space_store::SqlSpaceStore;

#[tokio::test]
async fn postgres_installer_uses_drive_nodes_for_global_assets() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    for table_name in [
        "dr_asset_item",
        "dr_asset_resource_ref",
        "dr_asset_version",
        "dr_asset_relation",
        "dr_asset_collection",
        "dr_asset_collection_item",
        "dr_asset_event",
        "dr_asset_projection",
    ] {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema='public' AND table_name=$1
            )",
        )
        .bind(table_name)
        .fetch_one(&pool)
        .await
        .expect("postgres asset table lookup should succeed");
        assert!(
            !exists,
            "global assets must not duplicate Drive nodes with table: {table_name}"
        );
    }

    for index_name in [
        "ix_dr_drive_node_asset_list",
        "ix_dr_drive_node_asset_scene_source",
        "ix_dr_drive_storage_object_node_latest",
        "ix_dr_drive_node_version_node_latest",
    ] {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM pg_indexes
                WHERE schemaname='public' AND indexname=$1
            )",
        )
        .bind(index_name)
        .fetch_one(&pool)
        .await
        .expect("postgres asset index lookup should succeed");
        assert!(
            exists,
            "expected Drive-backed asset index exists: {index_name}"
        );
    }
}

#[tokio::test]
async fn postgres_installer_creates_special_space_profile_tables() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    for table_name in [
        "dr_drive_space_knowledge_profile",
        "dr_drive_space_ai_generation_profile",
        "dr_drive_space_app_upload_profile",
        "dr_drive_space_rtc_profile",
        "dr_drive_space_website_profile",
        "dr_drive_website_root",
        "dr_drive_website_root_generation",
        "dr_drive_website_sync",
        "dr_drive_root_scope_subscription",
        "dr_drive_node_permission",
        "dr_drive_node_share_link",
        "dr_drive_node_comment",
        "dr_drive_node_comment_reply",
        "dr_drive_node_favorite",
        "dr_drive_node_property",
        "dr_drive_label",
        "dr_drive_node_label",
        "dr_drive_watch_channel",
        "dr_drive_change_cursor",
        "dr_drive_change_log",
        "dr_drive_domain_outbox",
        "dr_drive_domain_outbox_channel_delivery",
        "dr_drive_maintenance_leader",
        "dr_drive_storage_provider",
        "dr_drive_storage_provider_binding",
        "dr_drive_audit_event",
        "dr_drive_maintenance_job",
        "dr_drive_node_version",
        "dr_drive_space_version_policy",
        "dr_drive_node_version_policy",
        "dr_drive_upload_item",
        "dr_drive_upload_part",
        "dr_drive_file_sensitive_operation",
    ] {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema='public' AND table_name=$1
            )",
        )
        .bind(table_name)
        .fetch_one(&pool)
        .await
        .expect("postgres table lookup should succeed");
        assert!(exists, "expected table exists: {table_name}");
    }

    for index_name in [
        "ux_dr_drive_space_rtc_profile_user",
        "ux_dr_drive_space_website_profile_project",
        "ux_dr_drive_website_root_uuid",
        "ux_dr_drive_website_root_space_key",
        "ux_dr_drive_website_root_selector_active",
        "ix_dr_drive_website_root_active_node",
        "ux_dr_drive_website_root_generation_no",
        "ux_dr_drive_website_root_generation_current",
        "ix_dr_drive_website_root_generation_retention",
        "ux_dr_drive_website_sync_idempotency",
        "ix_dr_drive_website_sync_worker",
        "ix_dr_drive_website_sync_root_status",
        "ux_dr_drive_root_scope_subscription_uuid",
        "ux_dr_drive_root_scope_subscription_consumer",
        "ix_dr_drive_root_scope_subscription_root",
        "ix_dr_drive_node_permission_resource",
        "ix_dr_drive_node_permission_subject",
        "ux_dr_drive_node_permission_node_subject_live",
        "ux_dr_drive_node_share_link_token_hash",
        "ix_dr_drive_node_share_link_resource",
        "ix_dr_drive_node_comment_node",
        "ix_dr_drive_node_comment_resolved",
        "ix_dr_drive_node_comment_reply_comment",
        "ix_dr_drive_node_comment_reply_node",
        "ux_dr_drive_node_favorite_subject_node",
        "ix_dr_drive_node_favorite_subject",
        "ux_dr_drive_node_property_key",
        "ix_dr_drive_node_property_node",
        "ix_dr_drive_node_property_key_reverse",
        "ux_dr_drive_label_key",
        "ix_dr_drive_label_tenant_status",
        "ux_dr_drive_node_label_node_label",
        "ix_dr_drive_node_label_node",
        "ix_dr_drive_node_label_label",
        "ix_dr_drive_watch_channel_tenant_status",
        "ix_dr_drive_watch_channel_resource",
        "ix_dr_drive_watch_channel_node",
        "ix_dr_drive_watch_channel_expires",
        "ux_dr_drive_node_root_name_live",
        "ux_dr_drive_node_child_name_live",
        "ix_dr_drive_node_shortcut_target",
        "ux_dr_drive_change_cursor_scope",
        "ux_dr_drive_change_log_space_sequence",
        "ix_dr_drive_change_log_tenant_space_created",
        "ix_dr_drive_domain_outbox_pending",
        "ix_dr_drive_domain_outbox_pending_dispatch",
        "ix_dr_drive_domain_outbox_channel_delivery_channel",
        "ix_dr_drive_audit_event_tenant_created",
        "ix_dr_drive_audit_event_resource",
        "ix_dr_drive_audit_event_action_created",
        "ix_dr_drive_audit_event_request_created",
        "ix_dr_drive_audit_event_trace_created",
        "ix_dr_drive_storage_provider_binding_lookup",
        "ix_dr_drive_storage_provider_binding_provider",
        "ux_dr_drive_storage_provider_binding_tenant_primary_active",
        "ux_dr_drive_storage_provider_binding_space_primary_active",
        "ux_dr_drive_storage_provider_binding_space_type_active",
        "ux_dr_drive_node_version_node_version",
        "ix_dr_drive_node_version_node_latest",
        "ix_dr_drive_node_version_storage_object",
        "ix_dr_drive_node_version_app_resource",
        "ux_dr_drive_space_version_policy_space",
        "ux_dr_drive_node_version_policy_node",
        "ux_dr_drive_upload_item_task",
        "ix_dr_drive_upload_item_fingerprint",
        "ix_dr_drive_upload_item_retention",
        "ux_dr_drive_upload_part_item_part",
        "ix_dr_drive_upload_part_session",
        "ix_dr_drive_file_sensitive_operation_upload_item",
        "ix_dr_drive_file_sensitive_operation_tenant_created",
    ] {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM pg_indexes
                WHERE schemaname='public' AND indexname=$1
            )",
        )
        .bind(index_name)
        .fetch_one(&pool)
        .await
        .expect("postgres index lookup should succeed");
        assert!(exists, "expected index exists: {index_name}");
    }

    let shortcut_target_column_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM information_schema.columns
            WHERE table_schema='public'
              AND table_name='dr_drive_node'
              AND column_name='shortcut_target_node_id'
        )",
    )
    .fetch_one(&pool)
    .await
    .expect("postgres dr_drive_node column lookup should succeed");
    assert!(
        shortcut_target_column_exists,
        "dr_drive_node should expose shortcut_target_node_id"
    );

    for column_name in ["scene", "source"] {
        let column_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema='public'
                  AND table_name='dr_drive_upload_item'
                  AND column_name=$1
            )",
        )
        .bind(column_name)
        .fetch_one(&pool)
        .await
        .expect("postgres dr_drive_upload_item column lookup should succeed");
        assert!(
            column_exists,
            "dr_drive_upload_item should expose {column_name}"
        );
    }

    for table_name in ["dr_drive_node", "dr_drive_storage_object"] {
        for column_name in ["scene", "source"] {
            let column_exists: bool = sqlx::query_scalar(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_schema='public'
                      AND table_name=$1
                      AND column_name=$2
                )",
            )
            .bind(table_name)
            .bind(column_name)
            .fetch_one(&pool)
            .await
            .expect("postgres usage context column lookup should succeed");
            assert!(column_exists, "{table_name} should expose {column_name}");
        }
    }

    for column_name in [
        "content_state",
        "file_extension",
        "head_content_type",
        "head_content_type_group",
        "head_content_length",
        "head_version_no",
        "head_checksum_sha256_hex",
    ] {
        let column_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema='public'
                  AND table_name='dr_drive_node'
                  AND column_name=$1
            )",
        )
        .bind(column_name)
        .fetch_one(&pool)
        .await
        .expect("postgres dr_drive_node head column lookup should succeed");
        assert!(
            column_exists,
            "dr_drive_node should expose head metadata column: {column_name}"
        );
    }
}

#[tokio::test]
async fn postgres_delete_space_with_contents_is_atomic() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let tenant_id = format!("tenant-pg-delete-{}", uuid::Uuid::new_v4());
    let space_id = format!("space-pg-delete-{}", uuid::Uuid::new_v4());

    let space_service = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()));
    space_service
        .create_space(CreateSpaceCommand {
            id: space_id.clone(),
            tenant_id: tenant_id.clone(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-pg-delete".to_string(),
            display_name: "PG Delete".to_string(),
            space_type: DriveSpaceType::Personal,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-pg-delete".to_string(),
        })
        .await
        .expect("postgres space should be created");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, node_type, node_name, content_state, lifecycle_status,
            version, created_by, updated_by
         ) VALUES ($1, $2, $3, 'folder', 'Docs', 'ready', 'active', 1, 'user-pg-delete', 'user-pg-delete')",
    )
    .bind(format!("node-{space_id}"))
    .bind(&tenant_id)
    .bind(&space_id)
    .execute(&pool)
    .await
    .expect("postgres node insert should succeed");

    let result = SqlDriveSpaceLifecycleService::new(pool.clone())
        .delete_space_with_contents(DeleteSpaceWithContentsCommand {
            tenant_id,
            space_id: space_id.clone(),
            operator_id: "user-pg-delete".to_string(),
        })
        .await
        .expect("postgres atomic delete should succeed");
    assert_eq!(result.deleted_node_count, 1);
    assert_eq!(result.space.lifecycle_status, "deleted");

    let active_nodes: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_node WHERE space_id=$1 AND lifecycle_status != 'deleted'",
    )
    .bind(&space_id)
    .fetch_one(&pool)
    .await
    .expect("active node count should be readable");
    assert_eq!(active_nodes, 0);
}
