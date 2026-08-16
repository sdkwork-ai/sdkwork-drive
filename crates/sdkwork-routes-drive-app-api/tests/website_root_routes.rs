use axum::body::{to_bytes, Body};
use axum::Router;
use http::{Response, StatusCode};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::util::ServiceExt;

mod common;

async fn setup() -> Option<(
    PgPool,
    Router,
    sdkwork_drive_test_support::test_database::PostgresTestDatabaseGuard,
)> {
    let (pool, database_guard) =
        sdkwork_drive_test_support::test_database::postgres_test_database().await?;
    let app = common::test_router_with_pool(pool.clone());
    Some((pool, app, database_guard))
}

async fn response_json(response: Response<Body>) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&body).expect("response body should be JSON")
}

async fn create_space(
    app: &Router,
    tenant_id: &str,
    owner_id: &str,
    space_id: &str,
    space_type: &str,
) {
    let response = app
        .clone()
        .oneshot(common::authed_post_json(
            "/app/v3/api/drive/spaces",
            tenant_id,
            owner_id,
            "appbase",
            Body::from(
                json!({
                    "id": space_id,
                    "ownerSubjectType": "user",
                    "ownerSubjectId": owner_id,
                    "displayName": format!("{space_id} display name"),
                    "spaceType": space_type,
                })
                .to_string(),
            ),
        ))
        .await
        .expect("create space request should be handled");
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn default_root(pool: &PgPool, tenant_id: &str, space_id: &str) -> (String, String) {
    sqlx::query_as(
        "SELECT active_node_id, uuid
         FROM dr_drive_website_root
         WHERE tenant_id=$1 AND space_id=$2 AND selector_key='space_root'",
    )
    .bind(tenant_id)
    .bind(space_id)
    .fetch_one(pool)
    .await
    .expect("default WebsiteRoot should exist")
}

async fn insert_folder(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    parent_node_id: &str,
    folder_node_id: &str,
    folder_name: &str,
    operator_id: &str,
) {
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, space_type, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, 'website', $4, 'folder', $5, 'ready', 'active', 1, $6, $6)",
    )
    .bind(folder_node_id)
    .bind(tenant_id)
    .bind(space_id)
    .bind(parent_node_id)
    .bind(folder_name)
    .bind(operator_id)
    .execute(pool)
    .await
    .expect("website folder should be inserted");
}

async fn create_folder_root(
    app: &Router,
    tenant_id: &str,
    user_id: &str,
    space_id: &str,
    folder_node_id: &str,
    root_key: &str,
) -> Response<Body> {
    app.clone()
        .oneshot(common::authed_post_json(
            format!("/app/v3/api/drive/spaces/{space_id}/website_roots"),
            tenant_id,
            user_id,
            "appbase",
            Body::from(
                json!({
                    "rootKey": root_key,
                    "displayName": format!("{root_key} site"),
                    "sourceRoot": {
                        "mode": "FOLDER",
                        "folderNodeId": folder_node_id,
                    },
                    "contentMode": "LIVE_TREE",
                })
                .to_string(),
            ),
        ))
        .await
        .expect("create WebsiteRoot request should be handled")
}

#[tokio::test]
async fn website_root_routes_create_list_retrieve_and_replay_folder_selector() {
    let Some((pool, app, _database_guard)) = setup().await else {
        return;
    };
    create_space(
        &app,
        "tenant-website",
        "user-owner",
        "space-website",
        "website",
    )
    .await;
    let (root_node_id, _) = default_root(&pool, "tenant-website", "space-website").await;
    insert_folder(
        &pool,
        "tenant-website",
        "space-website",
        &root_node_id,
        "folder-pc",
        "pc",
        "user-owner",
    )
    .await;

    let created = create_folder_root(
        &app,
        "tenant-website",
        "user-owner",
        "space-website",
        "folder-pc",
        "pc",
    )
    .await;
    assert_eq!(created.status(), StatusCode::CREATED);
    let created_payload = response_json(created).await;
    let created_item = common::envelope_item(&created_payload);
    assert_eq!(created_item["selectedFolderNodeId"], "folder-pc");
    assert_eq!(created_item["sourceRootMode"], "FOLDER");
    assert_eq!(created_item["contentMode"], "LIVE_TREE");
    assert_eq!(created_item["activeGeneration"], "1");
    let created_uuid = created_item["uuid"]
        .as_str()
        .expect("created WebsiteRoot uuid should be a string")
        .to_string();

    let replayed = create_folder_root(
        &app,
        "tenant-website",
        "user-owner",
        "space-website",
        "folder-pc",
        "ignored-on-replay",
    )
    .await;
    assert_eq!(replayed.status(), StatusCode::OK);
    let replayed_payload = response_json(replayed).await;
    assert_eq!(
        common::envelope_item(&replayed_payload)["uuid"],
        created_uuid
    );
    let persisted_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_website_root
         WHERE tenant_id=$1 AND space_id=$2",
    )
    .bind("tenant-website")
    .bind("space-website")
    .fetch_one(&pool)
    .await
    .expect("WebsiteRoot count should be queryable");
    assert_eq!(persisted_count, 2);

    let first_page = app
        .clone()
        .oneshot(common::authed_get(
            "/app/v3/api/drive/spaces/space-website/website_roots?page_size=1",
            "tenant-website",
            "user-owner",
            "appbase",
        ))
        .await
        .expect("first WebsiteRoot page should be handled");
    assert_eq!(first_page.status(), StatusCode::OK);
    let first_payload = response_json(first_page).await;
    assert_eq!(
        common::envelope_items(&first_payload)
            .as_array()
            .expect("WebsiteRoot items should be an array")
            .len(),
        1
    );
    let next_cursor = common::envelope_next_page_token(&first_payload)
        .expect("first WebsiteRoot page should have a cursor");
    assert_eq!(
        common::envelope_page_info(&first_payload).and_then(|page_info| page_info.get("hasMore")),
        Some(&Value::Bool(true))
    );

    let second_page = app
        .clone()
        .oneshot(common::authed_get(
            format!(
                "/app/v3/api/drive/spaces/space-website/website_roots?page_size=1&cursor={next_cursor}"
            ),
            "tenant-website",
            "user-owner",
            "appbase",
        ))
        .await
        .expect("second WebsiteRoot page should be handled");
    assert_eq!(second_page.status(), StatusCode::OK);
    let second_payload = response_json(second_page).await;
    assert_eq!(
        common::envelope_items(&second_payload)
            .as_array()
            .expect("WebsiteRoot items should be an array")
            .len(),
        1
    );
    assert!(common::envelope_next_page_token(&second_payload).is_none());

    let retrieved = app
        .clone()
        .oneshot(common::authed_get(
            format!("/app/v3/api/drive/website_roots/{created_uuid}"),
            "tenant-website",
            "user-owner",
            "appbase",
        ))
        .await
        .expect("WebsiteRoot retrieve should be handled");
    assert_eq!(retrieved.status(), StatusCode::OK);
    let retrieved_payload = response_json(retrieved).await;
    assert_eq!(
        common::envelope_item(&retrieved_payload)["uuid"],
        created_uuid
    );
}

#[tokio::test]
async fn website_root_routes_enforce_owner_writes_reader_access_and_tenant_isolation() {
    let Some((pool, app, _database_guard)) = setup().await else {
        return;
    };
    create_space(&app, "tenant-acl", "user-owner", "space-acl", "website").await;
    let (root_node_id, root_uuid) = default_root(&pool, "tenant-acl", "space-acl").await;
    insert_folder(
        &pool,
        "tenant-acl",
        "space-acl",
        &root_node_id,
        "folder-mobile",
        "mobile",
        "user-owner",
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
         ) VALUES (
            'permission-website-reader', 'tenant-acl', $1, 'user', 'user-reader', 'reader',
            1, 'active', 1, 'user-owner', 'user-owner'
         )",
    )
    .bind(&root_node_id)
    .execute(&pool)
    .await
    .expect("WebsiteRoot reader permission should be inserted");

    let reader_list = app
        .clone()
        .oneshot(common::authed_get(
            "/app/v3/api/drive/spaces/space-acl/website_roots",
            "tenant-acl",
            "user-reader",
            "appbase",
        ))
        .await
        .expect("reader WebsiteRoot list should be handled");
    assert_eq!(reader_list.status(), StatusCode::OK);

    let reader_retrieve = app
        .clone()
        .oneshot(common::authed_get(
            format!("/app/v3/api/drive/website_roots/{root_uuid}"),
            "tenant-acl",
            "user-reader",
            "appbase",
        ))
        .await
        .expect("reader WebsiteRoot retrieve should be handled");
    assert_eq!(reader_retrieve.status(), StatusCode::OK);

    let reader_create = create_folder_root(
        &app,
        "tenant-acl",
        "user-reader",
        "space-acl",
        "folder-mobile",
        "mobile",
    )
    .await;
    assert_eq!(reader_create.status(), StatusCode::FORBIDDEN);

    let cross_tenant_retrieve = app
        .clone()
        .oneshot(common::authed_get(
            format!("/app/v3/api/drive/website_roots/{root_uuid}"),
            "tenant-other",
            "user-owner",
            "appbase",
        ))
        .await
        .expect("cross-tenant WebsiteRoot retrieve should be handled");
    assert_eq!(cross_tenant_retrieve.status(), StatusCode::NOT_FOUND);

    let cross_tenant_list = app
        .clone()
        .oneshot(common::authed_get(
            "/app/v3/api/drive/spaces/space-acl/website_roots",
            "tenant-other",
            "user-owner",
            "appbase",
        ))
        .await
        .expect("cross-tenant WebsiteRoot list should be handled");
    assert_eq!(cross_tenant_list.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn website_root_routes_reject_non_website_spaces_and_reserved_folders() {
    let Some((pool, app, _database_guard)) = setup().await else {
        return;
    };
    create_space(
        &app,
        "tenant-validation",
        "user-owner",
        "space-personal",
        "personal",
    )
    .await;
    let non_website = app
        .clone()
        .oneshot(common::authed_post_json(
            "/app/v3/api/drive/spaces/space-personal/website_roots",
            "tenant-validation",
            "user-owner",
            "appbase",
            Body::from(
                json!({
                    "rootKey": "invalid",
                    "displayName": "Invalid root",
                    "sourceRoot": { "mode": "SPACE_ROOT" },
                    "contentMode": "LIVE_TREE",
                })
                .to_string(),
            ),
        ))
        .await
        .expect("non-website WebsiteRoot request should be handled");
    assert_eq!(non_website.status(), StatusCode::BAD_REQUEST);
    let non_website_problem = response_json(non_website).await;
    assert!(non_website_problem["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("website Space")));

    create_space(
        &app,
        "tenant-validation",
        "user-owner",
        "space-reserved",
        "website",
    )
    .await;
    let (root_node_id, _) = default_root(&pool, "tenant-validation", "space-reserved").await;
    insert_folder(
        &pool,
        "tenant-validation",
        "space-reserved",
        &root_node_id,
        "folder-reserved",
        ".sdkwork",
        "user-owner",
    )
    .await;
    let reserved = create_folder_root(
        &app,
        "tenant-validation",
        "user-owner",
        "space-reserved",
        "folder-reserved",
        "reserved",
    )
    .await;
    assert_eq!(reserved.status(), StatusCode::BAD_REQUEST);
    let reserved_problem = response_json(reserved).await;
    assert!(reserved_problem["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("reserved Drive namespace")));
}
