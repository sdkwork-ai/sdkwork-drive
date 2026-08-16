use sdkwork_drive_workspace_service::application::resource_resolution_service::{
    DriveResourceResolutionService, ResolveDriveResourceCommand,
};
use sdkwork_drive_workspace_service::domain::resource_resolution::DriveResourceScopeKind;
use sdkwork_drive_workspace_service::infrastructure::sql::resource_resolution_store::SqlResourceResolutionStore;
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::PgPool;

async fn insert_provider(pool: &PgPool) {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, status, version, created_by, updated_by
         ) VALUES (
            'provider-local', 'local_filesystem', 'Local', 'file:///tmp/sdkwork-drive',
            NULL, 'website-bucket', 1, 0, NULL, 'active', 1, 'test', 'test'
         )",
    )
    .execute(pool)
    .await
    .expect("local provider should be inserted");
}

async fn insert_file_version(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    node_id: &str,
    version_id: &str,
    version_no: i64,
    checksum_char: char,
) {
    let object_id = format!("object-{version_id}");
    let checksum = format!("sha256:{}", checksum_char.to_string().repeat(64));
    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
         ) VALUES ($1, $2, $3, $4, 'provider-local', 'website-bucket', $5,
                   'text/html', 12, $6, 'active', 'test', 'test')",
    )
    .bind(&object_id)
    .bind(tenant_id)
    .bind(node_id)
    .bind(version_no)
    .bind(format!("{tenant_id}/{node_id}/{version_no}.html"))
    .bind(&checksum)
    .execute(pool)
    .await
    .expect("storage object should be inserted");

    sqlx::query(
        "INSERT INTO dr_drive_node_version (
            id, tenant_id, space_id, node_id, version_no, storage_object_id,
            content_type, content_length, checksum_sha256_hex, version_kind,
            change_source, lifecycle_status, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, $5, $6,
                   'text/html', 12, $7, 'auto', 'uploader', 'active', 'test', 'test')",
    )
    .bind(version_id)
    .bind(tenant_id)
    .bind(space_id)
    .bind(node_id)
    .bind(version_no)
    .bind(&object_id)
    .bind(&checksum)
    .execute(pool)
    .await
    .expect("node version should be inserted");
}

async fn insert_website_fixture(pool: &PgPool) {
    insert_provider(pool).await;
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
         ) VALUES ('space-web', 'tenant-web', 'user', 'owner-web', 'website',
                   'Website', 'active', 1, 'owner-web', 'owner-web')",
    )
    .execute(pool)
    .await
    .expect("website Space should be inserted");

    for (id, parent_id, node_type, node_name, content_state, head_version_no) in [
        ("root-web", None, "folder", "Website", "ready", None),
        (
            "folder-assets",
            Some("root-web"),
            "folder",
            "assets",
            "ready",
            None,
        ),
        (
            "file-index",
            Some("folder-assets"),
            "file",
            "index.html",
            "ready",
            Some(2_i64),
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, space_type, parent_node_id, node_type, node_name,
                content_state, head_content_type, head_content_type_group, head_content_length,
                head_version_no, head_checksum_sha256_hex, lifecycle_status, version,
                created_by, updated_by
             ) VALUES ($1, 'tenant-web', 'space-web', 'website', $2, $3, $4,
                       $5, CASE WHEN $3='file' THEN 'text/html' ELSE NULL END,
                       CASE WHEN $3='file' THEN 'document' ELSE NULL END,
                       CASE WHEN $3='file' THEN 12 ELSE NULL END,
                       $6, CASE WHEN $3='file' THEN $7 ELSE NULL END,
                       'active', 1, 'owner-web', 'owner-web')",
        )
        .bind(id)
        .bind(parent_id)
        .bind(node_type)
        .bind(node_name)
        .bind(content_state)
        .bind(head_version_no)
        .bind(format!("sha256:{}", "b".repeat(64)))
        .execute(pool)
        .await
        .expect("website node should be inserted");
    }

    insert_file_version(
        pool,
        "tenant-web",
        "space-web",
        "file-index",
        "version-index-1",
        1,
        'a',
    )
    .await;
    insert_file_version(
        pool,
        "tenant-web",
        "space-web",
        "file-index",
        "version-index-2",
        2,
        'b',
    )
    .await;

    sqlx::query(
        "INSERT INTO dr_drive_website_root (
            id, uuid, tenant_id, space_id, root_key, display_name, source_root_mode,
            selected_folder_node_id, selector_key, content_mode, active_node_id,
            active_generation, root_status, last_switch_by, version, created_by, updated_by
         ) VALUES (
            'website-root-id', 'website-root-uuid', 'tenant-web', 'space-web',
            'default', 'Default', 'space_root', NULL, 'space_root', 'live_tree',
            'root-web', 1, 'active', 'owner-web', 1, 'owner-web', 'owner-web'
         )",
    )
    .execute(pool)
    .await
    .expect("WebsiteRoot should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_website_root_generation (
            id, tenant_id, website_root_id, generation_no, root_node_id,
            generation_status, activated_by
         ) VALUES (
            'website-generation-1', 'tenant-web', 'website-root-id', 1,
            'root-web', 'current', 'owner-web'
         )",
    )
    .execute(pool)
    .await
    .expect("WebsiteRoot generation should be inserted");
}

async fn insert_knowledgebase_fixture(pool: &PgPool) {
    insert_provider(pool).await;
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
         ) VALUES ('space-kb', 'tenant-kb', 'app', 'knowledge-base-1', 'knowledge_base',
                   'Knowledgebase', 'active', 1, 'service-kb', 'service-kb')",
    )
    .execute(pool)
    .await
    .expect("knowledge_base Space should be inserted");

    for (id, parent_id, node_type, node_name, head_version_no) in [
        ("root-kb", None, "folder", "Knowledgebase", None),
        ("sources-kb", Some("root-kb"), "folder", "sources", None),
        ("raw-kb", Some("sources-kb"), "folder", "raw", None),
        ("guide-kb", Some("raw-kb"), "file", "guide.md", Some(1_i64)),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, space_type, parent_node_id, node_type, node_name,
                content_state, head_content_type, head_content_type_group, head_content_length,
                head_version_no, head_checksum_sha256_hex, lifecycle_status, version,
                created_by, updated_by
             ) VALUES ($1, 'tenant-kb', 'space-kb', 'knowledge_base', $2, $3, $4,
                       'ready', CASE WHEN $3='file' THEN 'text/html' ELSE NULL END,
                       CASE WHEN $3='file' THEN 'document' ELSE NULL END,
                       CASE WHEN $3='file' THEN 12 ELSE NULL END,
                       $5, CASE WHEN $3='file' THEN $6 ELSE NULL END,
                       'active', 1, 'service-kb', 'service-kb')",
        )
        .bind(id)
        .bind(parent_id)
        .bind(node_type)
        .bind(node_name)
        .bind(head_version_no)
        .bind(format!("sha256:{}", "c".repeat(64)))
        .execute(pool)
        .await
        .expect("knowledgebase node should be inserted");
    }
    insert_file_version(
        pool,
        "tenant-kb",
        "space-kb",
        "guide-kb",
        "version-guide-1",
        1,
        'c',
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_root_scope_subscription (
            id, uuid, tenant_id, space_id, consumer_kind, consumer_resource_id,
            root_node_id, scope_status, version, created_by, updated_by
         ) VALUES (
            'subscription-kb', 'subscription-kb-uuid', 'tenant-kb', 'space-kb',
            'knowledgebase_raw', 'knowledge-base-1', 'raw-kb', 'active', 1,
            'service-kb', 'service-kb'
         )",
    )
    .execute(pool)
    .await
    .expect("knowledgebase raw subscription should be inserted");
}

fn resolve_command(
    tenant_id: &str,
    scope_kind: DriveResourceScopeKind,
    scope_uuid: &str,
    relative_path: &str,
) -> ResolveDriveResourceCommand {
    ResolveDriveResourceCommand {
        tenant_id: tenant_id.to_string(),
        scope_kind,
        scope_uuid: scope_uuid.to_string(),
        relative_path: relative_path.to_string(),
        pinned_generation: None,
        pinned_node_version_id: None,
    }
}

#[tokio::test]
async fn website_root_resolution_returns_current_and_pinned_logical_versions() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_website_fixture(&pool).await;
    let service =
        DriveResourceResolutionService::new(SqlResourceResolutionStore::new(pool.clone()));

    let current = service
        .resolve(resolve_command(
            "tenant-web",
            DriveResourceScopeKind::WebsiteRoot,
            "website-root-uuid",
            "assets/index.html",
        ))
        .await
        .expect("current website resource should resolve");
    assert_eq!(current.node_id, "file-index");
    assert_eq!(current.node_version_id, "version-index-2");
    assert_eq!(current.scope_generation, 1);
    assert_eq!(current.eligibility, "ELIGIBLE");
    assert_eq!(
        current.content_locator.storage_provider_id,
        "provider-local"
    );
    assert_eq!(current.content_locator.storage_provider_version, 1);

    let mut pinned = resolve_command(
        "tenant-web",
        DriveResourceScopeKind::WebsiteRoot,
        "website-root-uuid",
        "assets/index.html",
    );
    pinned.pinned_generation = Some(1);
    pinned.pinned_node_version_id = Some("version-index-1".to_string());
    let pinned = service
        .resolve(pinned)
        .await
        .expect("pinned website resource should resolve");
    assert_eq!(pinned.node_version_id, "version-index-1");
    assert_eq!(
        pinned.checksum_sha256_hex,
        format!("sha256:{}", "a".repeat(64))
    );
}

#[tokio::test]
async fn resolution_rejects_cross_tenant_escape_reserved_and_wrong_version_paths() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_website_fixture(&pool).await;
    let service =
        DriveResourceResolutionService::new(SqlResourceResolutionStore::new(pool.clone()));

    let cross_tenant = service
        .resolve(resolve_command(
            "tenant-other",
            DriveResourceScopeKind::WebsiteRoot,
            "website-root-uuid",
            "assets/index.html",
        ))
        .await
        .expect_err("cross-tenant scope must not resolve");
    assert!(matches!(cross_tenant, DriveServiceError::NotFound(_)));

    for invalid in ["../assets/index.html", ".sdkwork/manifest.json"] {
        let error = service
            .resolve(resolve_command(
                "tenant-web",
                DriveResourceScopeKind::WebsiteRoot,
                "website-root-uuid",
                invalid,
            ))
            .await
            .expect_err("invalid path must fail closed");
        assert!(matches!(
            error,
            DriveServiceError::Validation(_) | DriveServiceError::PermissionDenied(_)
        ));
    }

    let mut wrong_version = resolve_command(
        "tenant-web",
        DriveResourceScopeKind::WebsiteRoot,
        "website-root-uuid",
        "assets/index.html",
    );
    wrong_version.pinned_node_version_id = Some("version-from-another-node".to_string());
    let error = service
        .resolve(wrong_version)
        .await
        .expect_err("unbound version must not resolve");
    assert!(matches!(error, DriveServiceError::NotFound(_)));
}

#[tokio::test]
async fn knowledgebase_raw_subscription_resolves_only_paths_below_raw_root() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    insert_knowledgebase_fixture(&pool).await;
    let service =
        DriveResourceResolutionService::new(SqlResourceResolutionStore::new(pool.clone()));

    let resource = service
        .resolve(resolve_command(
            "tenant-kb",
            DriveResourceScopeKind::RootScopeSubscription,
            "subscription-kb-uuid",
            "guide.md",
        ))
        .await
        .expect("knowledgebase raw resource should resolve");
    assert_eq!(resource.node_id, "guide-kb");
    assert_eq!(resource.relative_path, "guide.md");
    assert_eq!(resource.scope_generation, 1);

    let outside = service
        .resolve(resolve_command(
            "tenant-kb",
            DriveResourceScopeKind::RootScopeSubscription,
            "subscription-kb-uuid",
            "sources/raw/guide.md",
        ))
        .await
        .expect_err("path must be relative to raw root");
    assert!(matches!(outside, DriveServiceError::NotFound(_)));
}
