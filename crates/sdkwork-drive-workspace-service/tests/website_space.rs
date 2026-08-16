use sdkwork_drive_contract::drive::events::DriveNodeEligibility;
use sdkwork_drive_workspace_service::application::space_service::{
    CreateSpaceCommand, DriveSpaceService,
};
use sdkwork_drive_workspace_service::application::website_root_service::{
    CreateWebsiteRootCommand, DriveWebsiteRootService, ListWebsiteRootsCommand,
};
use sdkwork_drive_workspace_service::domain::space::DriveSpaceType;
use sdkwork_drive_workspace_service::domain::website_root::{
    DriveWebsiteContentMode, DriveWebsiteSourceRootMode,
};
use sdkwork_drive_workspace_service::infrastructure::change_recorder::{
    record_drive_node_deleted_on_connection, record_drive_node_eligibility_changed_on_connection,
    record_drive_node_path_changed_on_connection,
    record_drive_node_version_committed_on_connection,
    resolve_drive_node_location_snapshot_on_connection, RecordDriveNodeDeletedCommand,
    RecordDriveNodeEligibilityChangedCommand, RecordDriveNodePathChangedCommand,
    RecordDriveNodeVersionCommittedCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::space_store::SqlSpaceStore;
use sdkwork_drive_workspace_service::infrastructure::sql::website_root_store::SqlWebsiteRootStore;

#[tokio::test]
async fn website_spaces_are_multi_instance_and_provision_complete_default_roots_atomically() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()));
    for (id, display_name) in [
        ("website-project-pc", "Storefront PC"),
        ("website-project-mobile", "Storefront Mobile"),
    ] {
        let created = service
            .create_space(CreateSpaceCommand {
                id: id.to_string(),
                tenant_id: "tenant-website".to_string(),
                owner_subject_type: "organization".to_string(),
                owner_subject_id: "organization-website".to_string(),
                display_name: display_name.to_string(),
                space_type: DriveSpaceType::Website,
                presentation_icon: None,
                presentation_color: None,
                description: None,
                operator_id: "user-website-admin".to_string(),
            })
            .await
            .expect("website Space should be created");
        assert_eq!(created.space_type, DriveSpaceType::Website);
    }

    let spaces: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_space
         WHERE tenant_id='tenant-website'
           AND owner_subject_id='organization-website'
           AND space_type='website'
           AND lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("website Spaces should be counted");
    assert_eq!(spaces, 2);

    let complete_roots: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_space_website_profile profile
         INNER JOIN dr_drive_website_root root
            ON root.id=profile.default_root_id
           AND root.tenant_id=profile.tenant_id
           AND root.space_id=profile.space_id
         INNER JOIN dr_drive_website_root_generation generation
            ON generation.website_root_id=root.id
           AND generation.generation_no=1
           AND generation.generation_status='current'
         INNER JOIN dr_drive_node node
            ON node.id=root.active_node_id
           AND node.tenant_id=root.tenant_id
           AND node.space_id=root.space_id
           AND node.node_type='folder'
           AND node.lifecycle_status='active'
         WHERE profile.tenant_id='tenant-website'
           AND root.source_root_mode='space_root'
           AND root.selector_key='space_root'
           AND root.content_mode='live_tree'
           AND root.active_generation=1",
    )
    .fetch_one(&pool)
    .await
    .expect("complete default WebsiteRoots should be counted");
    assert_eq!(complete_roots, 2);

    let (root_node_id, root_uuid): (String, String) = sqlx::query_as(
        "SELECT active_node_id, uuid
         FROM dr_drive_website_root
         WHERE tenant_id='tenant-website'
           AND space_id='website-project-pc'
           AND root_key='default'",
    )
    .fetch_one(&pool)
    .await
    .expect("default WebsiteRoot should be queryable");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, space_type, parent_node_id, node_type, node_name,
            content_state, head_content_type, head_content_type_group, head_content_length,
            head_version_no, head_checksum_sha256_hex, lifecycle_status, version,
            created_by, updated_by
         ) VALUES (
            'website-index-node', 'tenant-website', 'website-project-pc', 'website', $1,
            'file', 'index.html', 'ready', 'text/html', 'text', 128, 1,
            'sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
            'active', 1, 'user-website-admin', 'user-website-admin'
         )",
    )
    .bind(&root_node_id)
    .execute(&pool)
    .await
    .expect("website index node should be inserted");

    let mut connection = pool.acquire().await.expect("connection should be acquired");
    record_drive_node_version_committed_on_connection(
        &mut connection,
        RecordDriveNodeVersionCommittedCommand {
            tenant_id: "tenant-website",
            organization_id: Some("organization-website"),
            space_id: "website-project-pc",
            node_id: "website-index-node",
            node_version_id: "website-index-version-1",
            version_no: 1,
            operation_id: "website-index-upload-1",
            content_type: "text/html",
            content_length: 128,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            actor_id: "user-website-admin",
        },
    )
    .await
    .expect("website version event should be recorded");
    drop(connection);

    let payload_json: String = sqlx::query_scalar(
        "SELECT payload_json
         FROM dr_drive_domain_outbox
         WHERE node_id='website-index-node'
           AND event_type='drive.node.version.committed.v1'",
    )
    .fetch_one(&pool)
    .await
    .expect("website event payload should be queryable");
    let payload: serde_json::Value =
        serde_json::from_str(&payload_json).expect("website event payload should be JSON");
    assert_eq!(payload["data"]["rootScopes"][0]["scopeId"], root_uuid);
    assert_eq!(
        payload["data"]["rootScopes"][0]["scopeKind"],
        "WEBSITE_ROOT"
    );
    assert_eq!(
        payload["data"]["rootScopes"][0]["relativePath"],
        "index.html"
    );
    assert_eq!(payload["data"]["rootScopes"][0]["rootGeneration"], "1");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, space_type, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
         ) VALUES (
            'website-docs-node', 'tenant-website', 'website-project-pc', 'website', $1,
            'folder', 'docs', 'ready', 'active', 1,
            'user-website-admin', 'user-website-admin'
         )",
    )
    .bind(&root_node_id)
    .execute(&pool)
    .await
    .expect("website docs folder should be inserted");

    let mut connection = pool.acquire().await.expect("connection should be acquired");
    let old_location = resolve_drive_node_location_snapshot_on_connection(
        &mut connection,
        "tenant-website",
        "website-project-pc",
        "website-index-node",
    )
    .await
    .expect("old node location should resolve");
    sqlx::query(
        "UPDATE dr_drive_node
         SET parent_node_id='website-docs-node', updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE id='website-index-node'",
    )
    .execute(&mut *connection)
    .await
    .expect("website index node should move");
    let new_location = resolve_drive_node_location_snapshot_on_connection(
        &mut connection,
        "tenant-website",
        "website-project-pc",
        "website-index-node",
    )
    .await
    .expect("new node location should resolve");

    record_drive_node_path_changed_on_connection(
        &mut connection,
        RecordDriveNodePathChangedCommand {
            tenant_id: "tenant-website",
            organization_id: Some("organization-website"),
            space_id: "website-project-pc",
            node_id: "website-index-node",
            operation_id: "request-move-1",
            actor_id: "user-website-admin",
            old_location: &old_location,
            new_location: &new_location,
        },
    )
    .await
    .expect("path event should be recorded");
    record_drive_node_eligibility_changed_on_connection(
        &mut connection,
        RecordDriveNodeEligibilityChangedCommand {
            tenant_id: "tenant-website",
            organization_id: Some("organization-website"),
            space_id: "website-project-pc",
            node_id: "website-index-node",
            operation_id: "request-trash-1",
            actor_id: "user-website-admin",
            old_eligibility: DriveNodeEligibility::Eligible,
            new_eligibility: DriveNodeEligibility::Ineligible,
            reason: "NODE_TRASHED",
            location: &new_location,
        },
    )
    .await
    .expect("eligibility event should be recorded");
    record_drive_node_deleted_on_connection(
        &mut connection,
        RecordDriveNodeDeletedCommand {
            tenant_id: "tenant-website",
            organization_id: Some("organization-website"),
            space_id: "website-project-pc",
            node_id: "website-index-node",
            operation_id: "request-delete-1",
            actor_id: "user-website-admin",
            deletion_reason: "PERMANENT_DELETE",
            last_location: &new_location,
        },
    )
    .await
    .expect("delete event should be recorded");
    drop(connection);

    let lifecycle_payloads: Vec<(String, String)> = sqlx::query_as(
        "SELECT event_type, payload_json
         FROM dr_drive_domain_outbox
         WHERE node_id='website-index-node'
           AND event_type IN (
             'drive.node.path.changed.v1',
             'drive.node.eligibility.changed.v1',
             'drive.node.deleted.v1'
           )
         ORDER BY sequence_no",
    )
    .fetch_all(&pool)
    .await
    .expect("lifecycle payloads should be queryable");
    assert_eq!(lifecycle_payloads.len(), 3);
    let path_payload: serde_json::Value =
        serde_json::from_str(&lifecycle_payloads[0].1).expect("path event payload should be JSON");
    assert_eq!(
        path_payload["data"]["oldRootScopes"][0]["relativePath"],
        "index.html"
    );
    assert_eq!(
        path_payload["data"]["newRootScopes"][0]["relativePath"],
        "docs/index.html"
    );
    let eligibility_payload: serde_json::Value = serde_json::from_str(&lifecycle_payloads[1].1)
        .expect("eligibility event payload should be JSON");
    assert_eq!(eligibility_payload["data"]["newEligibility"], "INELIGIBLE");
    assert_eq!(
        eligibility_payload["data"]["driveVersionId"],
        serde_json::Value::Null
    );
    let deleted_payload: serde_json::Value = serde_json::from_str(&lifecycle_payloads[2].1)
        .expect("delete event payload should be JSON");
    assert_eq!(
        deleted_payload["data"]["lastSpaceRelativePath"],
        "Storefront PC/docs/index.html"
    );
}

#[tokio::test]
async fn non_website_space_types_remain_singleton_per_owner() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveSpaceService::new(SqlSpaceStore::new(pool));
    for id in ["personal-one", "personal-two"] {
        let result = service
            .create_space(CreateSpaceCommand {
                id: id.to_string(),
                tenant_id: "tenant-singleton".to_string(),
                owner_subject_type: "user".to_string(),
                owner_subject_id: "user-singleton".to_string(),
                display_name: id.to_string(),
                space_type: DriveSpaceType::Personal,
                presentation_icon: None,
                presentation_color: None,
                description: None,
                operator_id: "user-singleton".to_string(),
            })
            .await;
        if id == "personal-one" {
            result.expect("first singleton Space should be created");
        } else {
            assert!(result.is_err(), "second singleton Space must conflict");
        }
    }
}

#[tokio::test]
async fn folder_website_root_is_idempotent_and_rejects_reserved_namespaces() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let space_service = DriveSpaceService::new(SqlSpaceStore::new(pool.clone()));
    space_service
        .create_space(CreateSpaceCommand {
            id: "website-folders".to_string(),
            tenant_id: "tenant-folders".to_string(),
            owner_subject_type: "user".to_string(),
            owner_subject_id: "user-folders".to_string(),
            display_name: "Multi App".to_string(),
            space_type: DriveSpaceType::Website,
            presentation_icon: None,
            presentation_color: None,
            description: None,
            operator_id: "user-folders".to_string(),
        })
        .await
        .expect("website Space should be created");
    let canonical_root_id: String = sqlx::query_scalar(
        "SELECT active_node_id
         FROM dr_drive_website_root
         WHERE space_id='website-folders' AND root_key='default'",
    )
    .fetch_one(&pool)
    .await
    .expect("canonical root should exist");
    for (id, parent_id, name) in [
        ("apps-folder", canonical_root_id.as_str(), "apps"),
        ("pc-folder", "apps-folder", "pc"),
        ("reserved-folder", canonical_root_id.as_str(), ".sdkwork"),
        ("reserved-child", "reserved-folder", "private"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, space_type, parent_node_id,
                node_type, node_name, content_state, lifecycle_status,
                version, created_by, updated_by
             ) VALUES ($1, 'tenant-folders', 'website-folders', 'website', $2,
                       'folder', $3, 'ready', 'active', 1, 'user-folders', 'user-folders')",
        )
        .bind(id)
        .bind(parent_id)
        .bind(name)
        .execute(&pool)
        .await
        .expect("folder should be inserted");
    }

    let service = DriveWebsiteRootService::new(SqlWebsiteRootStore::new(pool.clone()));
    let first = service
        .create_root(CreateWebsiteRootCommand {
            tenant_id: "tenant-folders".to_string(),
            space_id: "website-folders".to_string(),
            root_key: "pc".to_string(),
            display_name: "PC application".to_string(),
            source_root_mode: DriveWebsiteSourceRootMode::Folder,
            selected_folder_node_id: Some("pc-folder".to_string()),
            content_mode: DriveWebsiteContentMode::LiveTree,
            operator_id: "user-folders".to_string(),
        })
        .await
        .expect("folder WebsiteRoot should be created");
    let replay = service
        .create_root(CreateWebsiteRootCommand {
            tenant_id: "tenant-folders".to_string(),
            space_id: "website-folders".to_string(),
            root_key: "ignored-on-selector-replay".to_string(),
            display_name: "Replay".to_string(),
            source_root_mode: DriveWebsiteSourceRootMode::Folder,
            selected_folder_node_id: Some("pc-folder".to_string()),
            content_mode: DriveWebsiteContentMode::AtomicGeneration,
            operator_id: "user-folders".to_string(),
        })
        .await
        .expect("same folder selector should return existing WebsiteRoot");
    assert!(first.created);
    assert!(!replay.created);
    assert_eq!(first.root.uuid, replay.root.uuid);
    assert_eq!(replay.root.content_mode, DriveWebsiteContentMode::LiveTree);

    let reserved = service
        .create_root(CreateWebsiteRootCommand {
            tenant_id: "tenant-folders".to_string(),
            space_id: "website-folders".to_string(),
            root_key: "reserved".to_string(),
            display_name: "Reserved".to_string(),
            source_root_mode: DriveWebsiteSourceRootMode::Folder,
            selected_folder_node_id: Some("reserved-child".to_string()),
            content_mode: DriveWebsiteContentMode::LiveTree,
            operator_id: "user-folders".to_string(),
        })
        .await;
    assert!(reserved.is_err(), "reserved ancestry must fail closed");

    let roots = service
        .list_roots(ListWebsiteRootsCommand {
            tenant_id: "tenant-folders".to_string(),
            space_id: "website-folders".to_string(),
            offset: 0,
            limit: 10,
        })
        .await
        .expect("WebsiteRoots should be listed");
    assert_eq!(roots.len(), 2, "default and PC roots should exist");
}
