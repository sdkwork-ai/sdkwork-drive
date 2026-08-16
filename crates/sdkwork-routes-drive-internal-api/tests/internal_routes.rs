use axum::body::{to_bytes, Body};
use axum::Router;
use http::{Method, Request, Response, StatusCode};
use serde_json::{json, Value};
use sqlx::PgPool;
use tempfile::TempDir;
use tower::ServiceExt;

const WEBSITE_ROOT_UUID: &str = "11111111-1111-4111-8111-111111111111";

async fn setup() -> Option<(
    PgPool,
    Router,
    TempDir,
    sdkwork_drive_test_support::test_database::PostgresTestDatabaseGuard,
)> {
    let (pool, database_guard) =
        sdkwork_drive_test_support::test_database::postgres_test_database().await?;
    let temp = tempfile::tempdir().expect("temp object root should be created");
    let app = sdkwork_routes_drive_internal_api::build_router_with_pool(pool.clone());
    Some((pool, app, temp, database_guard))
}

fn ingress_token(tenant_id: &str, user_id: &str, app_id: &str) -> String {
    format!("api_key_id=internal-test;tenant_id={tenant_id};user_id={user_id};app_id={app_id}")
}

fn request(
    method: Method,
    uri: impl AsRef<str>,
    tenant_id: Option<&str>,
    body: Body,
) -> Request<Body> {
    request_for_app(method, uri, tenant_id, "knowledgebase", body)
}

fn request_for_app(
    method: Method,
    uri: impl AsRef<str>,
    tenant_id: Option<&str>,
    app_id: &str,
    body: Body,
) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri.as_ref());
    if let Some(tenant_id) = tenant_id {
        builder = builder.header(
            "x-api-key",
            ingress_token(tenant_id, "service-publisher", app_id),
        );
    }
    builder.body(body).expect("request should be built")
}

fn json_request_for_app(
    method: Method,
    uri: impl AsRef<str>,
    tenant_id: Option<&str>,
    app_id: &str,
    payload: Value,
) -> Request<Body> {
    let mut request = request_for_app(
        method,
        uri,
        tenant_id,
        app_id,
        Body::from(payload.to_string()),
    );
    request.headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/json"),
    );
    request
}

fn json_request(
    method: Method,
    uri: impl AsRef<str>,
    tenant_id: Option<&str>,
    payload: Value,
) -> Request<Body> {
    let mut request = request(method, uri, tenant_id, Body::from(payload.to_string()));
    request.headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/json"),
    );
    request
}

async fn response_json(response: Response<Body>) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&body).expect("response body should be JSON")
}

async fn insert_knowledgebase_space(pool: &PgPool) {
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
         ) VALUES ('space-kb', 'tenant-kb', 'app', 'knowledge-base-1', 'knowledge_base',
                   'Knowledgebase', 'active', 1, 'service-kb', 'service-kb')",
    )
    .execute(pool)
    .await
    .expect("knowledgebase Space should be inserted");
}

async fn insert_website_resource(pool: &PgPool, temp: &TempDir) -> String {
    let content = b"hello world";
    let checksum = format!("sha256:{}", sdkwork_utils_rust::sha256_hash(content));
    let endpoint = url::Url::from_directory_path(temp.path())
        .expect("temp root should become file URL")
        .to_string();
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, status, version, created_by, updated_by
         ) VALUES ('provider-local', 'local_filesystem', 'Local', $1, NULL,
                   'website-bucket', 1, 0, NULL, 'active', 1, 'test', 'test')",
    )
    .bind(endpoint)
    .execute(pool)
    .await
    .expect("local provider should be inserted");
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
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, space_type, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
         ) VALUES ('root-web', 'tenant-web', 'space-web', 'website', NULL, 'folder',
                   'Website', 'ready', 'active', 1, 'owner-web', 'owner-web')",
    )
    .execute(pool)
    .await
    .expect("website root node should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, space_type, parent_node_id, node_type, node_name,
            content_state, head_content_type, head_content_type_group, head_content_length,
            head_version_no, head_checksum_sha256_hex, lifecycle_status, version,
            created_by, updated_by
         ) VALUES ('file-index', 'tenant-web', 'space-web', 'website', 'root-web', 'file',
                   'index.html', 'ready', 'text/html', 'document', 11, 1, $1,
                   'active', 1, 'owner-web', 'owner-web')",
    )
    .bind(&checksum)
    .execute(pool)
    .await
    .expect("website file should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
         ) VALUES ('object-index', 'tenant-web', 'file-index', 1, 'provider-local',
                   'website-bucket', 'site/index.html', 'text/html', 11, $1,
                   'active', 'owner-web', 'owner-web')",
    )
    .bind(&checksum)
    .execute(pool)
    .await
    .expect("website object should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_node_version (
            id, tenant_id, space_id, node_id, version_no, storage_object_id,
            content_type, content_length, checksum_sha256_hex, version_kind,
            change_source, lifecycle_status, created_by, updated_by
         ) VALUES ('version-index', 'tenant-web', 'space-web', 'file-index', 1,
                   'object-index', 'text/html', 11, $1, 'auto', 'uploader',
                   'active', 'owner-web', 'owner-web')",
    )
    .bind(&checksum)
    .execute(pool)
    .await
    .expect("website node version should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_website_root (
            id, uuid, tenant_id, space_id, root_key, display_name, source_root_mode,
            selected_folder_node_id, selector_key, content_mode, active_node_id,
            active_generation, root_status, last_switch_by, version, created_by, updated_by
         ) VALUES ('root-record', $1, 'tenant-web', 'space-web', 'default', 'Default',
                   'space_root', NULL, 'space_root', 'live_tree', 'root-web', 1,
                   'active', 'owner-web', 1, 'owner-web', 'owner-web')",
    )
    .bind(WEBSITE_ROOT_UUID)
    .execute(pool)
    .await
    .expect("WebsiteRoot should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_website_root_generation (
            id, tenant_id, website_root_id, generation_no, root_node_id,
            generation_status, activated_by
         ) VALUES ('generation-1', 'tenant-web', 'root-record', 1, 'root-web',
                   'current', 'owner-web')",
    )
    .execute(pool)
    .await
    .expect("WebsiteRoot generation should be inserted");

    let object_path = temp.path().join("website-bucket").join("site/index.html");
    std::fs::create_dir_all(object_path.parent().expect("object parent"))
        .expect("object directory should be created");
    std::fs::write(object_path, content).expect("object bytes should be written");
    checksum
}

#[tokio::test]
async fn subscription_routes_require_ingress_auth_replay_and_isolate_tenants() {
    let Some((pool, app, _temp, _database_guard)) = setup().await else {
        return;
    };
    insert_knowledgebase_space(&pool).await;
    let payload = json!({
        "spaceId": "space-kb",
        "knowledgeBaseId": "knowledge-base-1"
    });

    let unauthenticated = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/internal/v3/api/drive/root_scope_subscriptions",
            None,
            payload.clone(),
        ))
        .await
        .expect("request should be handled");
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let created = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/internal/v3/api/drive/root_scope_subscriptions",
            Some("tenant-kb"),
            payload.clone(),
        ))
        .await
        .expect("create request should be handled");
    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = response_json(created).await;
    let uuid = created_json["data"]["item"]["uuid"]
        .as_str()
        .expect("subscription uuid")
        .to_string();
    assert_eq!(
        created_json["data"]["item"]["consumerKind"],
        "KNOWLEDGEBASE_RAW"
    );
    let raw_folder_node_id = created_json["data"]["item"]["rootNodeId"]
        .as_str()
        .expect("raw folder node id");
    let canonical_tree_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node raw_node
         INNER JOIN dr_drive_node sources_node ON sources_node.id=raw_node.parent_node_id
         INNER JOIN dr_drive_node root_node ON root_node.id=sources_node.parent_node_id
         WHERE raw_node.tenant_id='tenant-kb' AND raw_node.space_id='space-kb'
           AND raw_node.id=$1 AND raw_node.node_name='raw' AND raw_node.node_type='folder'
           AND sources_node.node_name='sources' AND sources_node.node_type='folder'
           AND root_node.node_name='root' AND root_node.node_type='folder'
           AND root_node.parent_node_id IS NULL",
    )
    .bind(raw_folder_node_id)
    .fetch_one(&pool)
    .await
    .expect("canonical raw tree should be queryable");
    assert_eq!(canonical_tree_count, 1);

    let replay = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/internal/v3/api/drive/root_scope_subscriptions",
            Some("tenant-kb"),
            payload,
        ))
        .await
        .expect("replay request should be handled");
    assert_eq!(replay.status(), StatusCode::OK);

    let delivery_uri =
        format!("/internal/v3/api/drive/root_scope_subscriptions/{uuid}/event_delivery");
    let verification_token = "6Yw1nJ37GZ8E0l9INjzQXklNSL4HE6Xe7n9m6hYS3jk";
    let delivery_payload = json!({
        "address": "https://knowledgebase.example.com/internal/v3/api/knowledgebase/drive_events",
        "verificationToken": verification_token,
        "expirationEpochMs": "1999999999999"
    });
    let delivery = app
        .clone()
        .oneshot(json_request(
            Method::PUT,
            &delivery_uri,
            Some("tenant-kb"),
            delivery_payload.clone(),
        ))
        .await
        .expect("event delivery request should be handled");
    assert_eq!(delivery.status(), StatusCode::CREATED);
    let delivery_json = response_json(delivery).await;
    assert_eq!(delivery_json["data"]["item"]["subscriptionUuid"], uuid);
    assert_eq!(delivery_json["data"]["item"]["lifecycleStatus"], "ACTIVE");
    assert!(!delivery_json.to_string().contains(verification_token));
    let (stored_resource_id, stored_signing_key): (Option<String>, String) = sqlx::query_as(
        "SELECT resource_id, token_hash
         FROM dr_drive_watch_channel WHERE tenant_id='tenant-kb' AND id=$1",
    )
    .bind(format!("kbraw:{uuid}"))
    .fetch_one(&pool)
    .await
    .expect("derived event delivery signing key should be stored");
    assert_eq!(stored_resource_id.as_deref(), Some(uuid.as_str()));
    assert_eq!(
        stored_signing_key,
        sdkwork_utils_rust::sha256_hash(verification_token.as_bytes())
    );

    let delivery_replay = app
        .clone()
        .oneshot(json_request(
            Method::PUT,
            &delivery_uri,
            Some("tenant-kb"),
            delivery_payload,
        ))
        .await
        .expect("event delivery replay should be handled");
    assert_eq!(delivery_replay.status(), StatusCode::OK);
    let delivery_replay_json = response_json(delivery_replay).await;
    assert_eq!(delivery_replay_json["data"]["item"]["version"], "1");

    let invalid_delivery = app
        .clone()
        .oneshot(json_request(
            Method::PUT,
            &delivery_uri,
            Some("tenant-kb"),
            json!({
                "address": "https://knowledgebase.example.com/internal/v3/api/knowledgebase/drive_events",
                "verificationToken": "too-short",
                "expirationEpochMs": "1999999999999"
            }),
        ))
        .await
        .expect("invalid event delivery should be handled");
    assert_eq!(invalid_delivery.status(), StatusCode::BAD_REQUEST);

    let cross_tenant = app
        .clone()
        .oneshot(request(
            Method::GET,
            format!("/internal/v3/api/drive/root_scope_subscriptions/{uuid}"),
            Some("tenant-other"),
            Body::empty(),
        ))
        .await
        .expect("cross-tenant request should be handled");
    assert_eq!(cross_tenant.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn website_root_event_delivery_is_root_scoped_write_only_and_idempotent() {
    let Some((pool, app, temp, _database_guard)) = setup().await else {
        return;
    };
    insert_website_resource(&pool, &temp).await;
    let uri = format!(
        "/internal/v3/api/drive/website_roots/{WEBSITE_ROOT_UUID}/event_deliveries/web-node-1"
    );
    let verification_token = "K3p4Jc9zYv2Nw8Tx5Qg7Ld1Rs6Hm0BaUEfIiVnXoS_4";
    let payload = json!({
        "address": "https://web-events.example.com/provider-events/drive-node-1",
        "verificationToken": verification_token,
        "expirationEpochMs": "1999999999999"
    });

    let wrong_caller = app
        .clone()
        .oneshot(json_request(
            Method::PUT,
            &uri,
            Some("tenant-web"),
            payload.clone(),
        ))
        .await
        .expect("wrong caller request should be handled");
    assert_eq!(wrong_caller.status(), StatusCode::FORBIDDEN);

    let created = app
        .clone()
        .oneshot(json_request_for_app(
            Method::PUT,
            &uri,
            Some("tenant-web"),
            "sdkwork-deployments",
            payload.clone(),
        ))
        .await
        .expect("website event delivery request should be handled");
    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = response_json(created).await;
    assert_eq!(created_json["data"]["item"]["channelId"], "web-node-1");
    assert_eq!(
        created_json["data"]["item"]["websiteRootUuid"],
        WEBSITE_ROOT_UUID
    );
    assert!(!created_json.to_string().contains(verification_token));
    let stored: (String, String, String) = sqlx::query_as(
        "SELECT space_id, resource_id, token_hash
         FROM dr_drive_watch_channel
         WHERE tenant_id='tenant-web' AND id='web-node-1'",
    )
    .fetch_one(&pool)
    .await
    .expect("website event channel should be stored");
    assert_eq!(stored.0, "space-web");
    assert_eq!(stored.1, WEBSITE_ROOT_UUID);
    assert_eq!(
        stored.2,
        sdkwork_utils_rust::sha256_hash(verification_token.as_bytes())
    );

    let replay = app
        .clone()
        .oneshot(json_request_for_app(
            Method::PUT,
            &uri,
            Some("tenant-web"),
            "sdkwork-deployments",
            payload,
        ))
        .await
        .expect("website event delivery replay should be handled");
    assert_eq!(replay.status(), StatusCode::OK);
    assert_eq!(response_json(replay).await["data"]["item"]["version"], "1");

    let cross_tenant = app
        .oneshot(json_request_for_app(
            Method::PUT,
            &uri,
            Some("tenant-other"),
            "sdkwork-deployments",
            json!({
                "address": "https://web-events.example.com/provider-events/drive-node-1",
                "verificationToken": verification_token,
                "expirationEpochMs": "1999999999999"
            }),
        ))
        .await
        .expect("cross-tenant request should be handled");
    assert_eq!(cross_tenant.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn resolution_and_content_routes_hide_locators_and_implement_http_semantics() {
    let Some((pool, app, temp, _database_guard)) = setup().await else {
        return;
    };
    let checksum = insert_website_resource(&pool, &temp).await;
    let resolution = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/internal/v3/api/drive/resource_resolutions",
            Some("tenant-web"),
            json!({
                "scopeType": "WEBSITE_ROOT",
                "scopeUuid": WEBSITE_ROOT_UUID,
                "relativePath": "index.html"
            }),
        ))
        .await
        .expect("resolution request should be handled");
    assert_eq!(resolution.status(), StatusCode::OK);
    let resolution_json = response_json(resolution).await;
    let item = &resolution_json["data"]["item"];
    assert_eq!(item["logicalNodeVersionId"], "version-index");
    assert_eq!(item["checksumSha256Hex"], checksum);
    let serialized = resolution_json.to_string();
    assert!(!serialized.contains("website-bucket"));
    assert!(!serialized.contains("site/index.html"));
    assert!(!serialized.contains("storageProvider"));

    let content_uri = format!(
        "/internal/v3/api/drive/node_versions/version-index/content?scopeType=WEBSITE_ROOT&scopeUuid={WEBSITE_ROOT_UUID}&relativePath=index.html"
    );
    let full = app
        .clone()
        .oneshot(request(
            Method::GET,
            &content_uri,
            Some("tenant-web"),
            Body::empty(),
        ))
        .await
        .expect("content request should be handled");
    assert_eq!(full.status(), StatusCode::OK);
    assert_eq!(full.headers()[http::header::ACCEPT_RANGES], "bytes");
    assert_eq!(full.headers()[http::header::CONTENT_TYPE], "text/html");
    let etag = full.headers()[http::header::ETAG]
        .to_str()
        .expect("etag header")
        .to_string();
    let full_body = to_bytes(full.into_body(), usize::MAX)
        .await
        .expect("full body should stream");
    assert_eq!(&full_body[..], b"hello world");

    let mut range_request = request(Method::GET, &content_uri, Some("tenant-web"), Body::empty());
    range_request.headers_mut().insert(
        http::header::RANGE,
        http::HeaderValue::from_static("bytes=1-4"),
    );
    let range = app
        .clone()
        .oneshot(range_request)
        .await
        .expect("range request should be handled");
    assert_eq!(range.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(range.headers()[http::header::CONTENT_RANGE], "bytes 1-4/11");
    let range_body = to_bytes(range.into_body(), usize::MAX)
        .await
        .expect("range body should stream");
    assert_eq!(&range_body[..], b"ello");

    let mut conditional = request(Method::GET, &content_uri, Some("tenant-web"), Body::empty());
    conditional.headers_mut().insert(
        http::header::IF_NONE_MATCH,
        http::HeaderValue::from_str(&etag).expect("etag should be a header"),
    );
    let not_modified = app
        .clone()
        .oneshot(conditional)
        .await
        .expect("conditional request should be handled");
    assert_eq!(not_modified.status(), StatusCode::NOT_MODIFIED);

    let wrong_scope_uri =
        content_uri.replace(WEBSITE_ROOT_UUID, "22222222-2222-4222-8222-222222222222");
    let wrong_scope = app
        .clone()
        .oneshot(request(
            Method::GET,
            wrong_scope_uri,
            Some("tenant-web"),
            Body::empty(),
        ))
        .await
        .expect("wrong-scope request should be handled");
    assert_eq!(wrong_scope.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn website_root_retrieve_is_tenant_scoped_and_hides_drive_structure() {
    let Some((pool, app, temp, _database_guard)) = setup().await else {
        return;
    };
    insert_website_resource(&pool, &temp).await;
    let uri = format!("/internal/v3/api/drive/website_roots/{WEBSITE_ROOT_UUID}");

    let unauthenticated = app
        .clone()
        .oneshot(request(Method::GET, &uri, None, Body::empty()))
        .await
        .expect("request should be handled");
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let retrieved = app
        .clone()
        .oneshot(request(
            Method::GET,
            &uri,
            Some("tenant-web"),
            Body::empty(),
        ))
        .await
        .expect("request should be handled");
    assert_eq!(retrieved.status(), StatusCode::OK);
    let response = response_json(retrieved).await;
    let item = &response["data"]["item"];
    assert_eq!(item["uuid"], WEBSITE_ROOT_UUID);
    assert_eq!(item["spaceId"], "space-web");
    assert_eq!(item["sourceRootMode"], "SPACE_ROOT");
    assert_eq!(item["contentMode"], "LIVE_TREE");
    assert_eq!(item["activeGeneration"], "1");
    assert_eq!(item["rootStatus"], "ACTIVE");
    assert_eq!(
        item["capabilities"],
        json!(["STATIC_CONTENT", "BYTE_RANGE", "CONDITIONAL_REQUESTS"])
    );
    let serialized = response.to_string();
    assert!(!serialized.contains("root-web"));
    assert!(!serialized.contains("selectedFolderNodeId"));
    assert!(!serialized.contains("activeNodeId"));
    assert!(!serialized.contains("objectKey"));
    assert!(!serialized.contains("bucket"));

    let cross_tenant = app
        .oneshot(request(
            Method::GET,
            &uri,
            Some("tenant-other"),
            Body::empty(),
        ))
        .await
        .expect("request should be handled");
    assert_eq!(cross_tenant.status(), StatusCode::NOT_FOUND);
}

#[test]
fn route_manifest_is_internal_and_ingress_token_only() {
    let manifest = sdkwork_routes_drive_internal_api::internal_route_manifest();
    assert_eq!(manifest.routes().len(), 7);
    assert!(manifest.routes().iter().any(|route| {
        route.operation_id == "rootScopeEventDeliveries.replace"
            && route.method == sdkwork_web_core::HttpMethod::Put
    }));
    assert!(manifest.routes().iter().any(|route| {
        route.operation_id == "websiteRoots.retrieve"
            && route.method == sdkwork_web_core::HttpMethod::Get
    }));
    assert!(manifest.routes().iter().any(|route| {
        route.operation_id == "websiteRootEventDeliveries.replace"
            && route.method == sdkwork_web_core::HttpMethod::Put
    }));
    for route in manifest.routes() {
        assert!(route.path.starts_with("/internal/v3/api/"));
        assert_eq!(route.auth, sdkwork_web_core::RouteAuth::IngressToken);
    }
}
