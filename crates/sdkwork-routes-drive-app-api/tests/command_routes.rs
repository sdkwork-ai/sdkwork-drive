use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use axum::Router;
use http::{Method, Request, StatusCode};
use sdkwork_drive_workspace_service::application::storage_key_service::{
    BuildStorageObjectKeyCommand, DriveStorageKeyService,
};
use sdkwork_drive_workspace_service::drive_share_token_hash;
use std::io::{Cursor, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

mod common;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapturedS3Request {
    method: String,
    path: String,
    query: String,
    body: String,
    body_bytes: Vec<u8>,
}

type CapturedS3Requests = Arc<Mutex<Vec<CapturedS3Request>>>;
type NodeVersionUsageContextRow = (
    String,
    i64,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);
type SensitiveOperationAuditRow = (
    String,
    String,
    String,
    String,
    String,
    i64,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
);

#[derive(Clone)]
struct S3MockState {
    requests: CapturedS3Requests,
    complete_delay: Duration,
}

fn assert_standard_storage_object_key(key: &str, tenant_id: &str, space_id: &str, node_id: &str) {
    assert!(
        key.starts_with("sdkwork-drive/v1/t/"),
        "object key should use sdkwork-drive v1 layout: {key}"
    );
    assert!(
        key.contains(&format!("/tenants/{tenant_id}/spaces/{space_id}/")),
        "object key should include tenant and space ids: {key}"
    );
    assert!(
        key.contains("/nodes/n/") && key.contains(&format!("/{node_id}/versions/")),
        "object key should include sharded node id and versions segment: {key}"
    );
    assert!(
        key.ends_with("/content"),
        "object key should end with content leaf: {key}"
    );
}

fn assert_rooted_standard_storage_object_key(
    key: &str,
    storage_root_prefix: &str,
    tenant_id: &str,
    space_id: &str,
    node_id: &str,
) {
    let expected_prefix = format!("{storage_root_prefix}/");
    assert!(
        key.starts_with(&expected_prefix),
        "object key should start with storage root prefix {storage_root_prefix}: {key}"
    );
    assert_standard_storage_object_key(
        key.strip_prefix(&expected_prefix)
            .expect("storage root prefix should be stripped"),
        tenant_id,
        space_id,
        node_id,
    );
}

fn assert_contains_standard_storage_object_key(
    key: &str,
    tenant_id: &str,
    space_id: &str,
    node_id: &str,
) {
    let Some(index) = key
        .rmatch_indices("sdkwork-drive/v1/t/")
        .next()
        .map(|(index, _)| index)
    else {
        panic!("object key should contain sdkwork-drive v1 layout: {key}");
    };
    assert_standard_storage_object_key(&key[index..], tenant_id, space_id, node_id);
}

fn standard_storage_object_key(
    tenant_id: &str,
    space_id: &str,
    node_id: &str,
    version_no: i64,
    object_id: &str,
) -> String {
    DriveStorageKeyService::build_object_key(BuildStorageObjectKeyCommand {
        tenant_id,
        space_id,
        node_id,
        version_no,
        object_id,
    })
    .expect("standard storage object key should be generated")
}

async fn start_s3_mock_server() -> (String, CapturedS3Requests) {
    start_s3_mock_server_with_complete_delay(Duration::ZERO).await
}

async fn start_s3_mock_server_with_complete_delay(
    complete_delay: Duration,
) -> (String, CapturedS3Requests) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let state = S3MockState {
        requests: requests.clone(),
        complete_delay,
    };
    let router = Router::new().fallback(mock_s3_endpoint).with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("mock s3 listener should bind");
    let address = listener
        .local_addr()
        .expect("mock s3 listener address should be available");
    tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("mock s3 server should run");
    });
    (format!("http://{address}"), requests)
}

async fn mock_s3_endpoint(
    State(state): State<S3MockState>,
    method: Method,
    uri: Uri,
    body: Body,
) -> Response {
    let query = uri.query().unwrap_or_default().to_string();
    let body = to_bytes(body, usize::MAX)
        .await
        .expect("mock s3 request body should be readable");
    let body_bytes = body.to_vec();
    let body = String::from_utf8_lossy(&body).to_string();
    state
        .requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .push(CapturedS3Request {
            method: method.as_str().to_string(),
            path: uri.path().to_string(),
            query: query.clone(),
            body: body.clone(),
            body_bytes: body_bytes.clone(),
        });

    if method == Method::GET {
        let path = uri.path();
        if path.ends_with("/objects/bulk/file-a.txt") {
            return (
                StatusCode::OK,
                [("content-type", "text/plain")],
                "alpha file",
            )
                .into_response();
        }
        if path.ends_with("/objects/bulk/file-b.txt") {
            return (
                StatusCode::OK,
                [("content-type", "text/plain")],
                "beta file",
            )
                .into_response();
        }
        if path.ends_with("/objects/bulk/folder-child.txt") {
            return (
                StatusCode::OK,
                [("content-type", "text/plain")],
                "folder child",
            )
                .into_response();
        }
        if path.ends_with("/objects/archive/report.zip") {
            return (
                StatusCode::OK,
                [("content-type", "application/zip")],
                build_archive_fixture_zip(),
            )
                .into_response();
        }
    }

    if method == Method::POST && query.contains("uploads") {
        let object_key = path_object_key(uri.path(), "bucket-s3");
        return (
            StatusCode::OK,
            [("content-type", "application/xml")],
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<InitiateMultipartUploadResult>
  <Bucket>bucket-s3</Bucket>
  <Key>{object_key}</Key>
  <UploadId>mock-s3-upload-id</UploadId>
</InitiateMultipartUploadResult>"#
            ),
        )
            .into_response();
    }

    if method == Method::POST && query.contains("uploadId=mock-s3-upload-id") {
        if !state.complete_delay.is_zero() {
            tokio::time::sleep(state.complete_delay).await;
        }
        let object_key = path_object_key(uri.path(), "bucket-s3");
        return (
            StatusCode::OK,
            [("content-type", "application/xml")],
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<CompleteMultipartUploadResult>
  <Location>http://127.0.0.1/bucket-s3/{object_key}</Location>
  <Bucket>bucket-s3</Bucket>
  <Key>{object_key}</Key>
  <ETag>"mock-complete-etag"</ETag>
</CompleteMultipartUploadResult>"#
            ),
        )
            .into_response();
    }

    if method == Method::DELETE && query.contains("uploadId=mock-s3-upload-id") {
        return StatusCode::NO_CONTENT.into_response();
    }

    StatusCode::OK.into_response()
}

fn path_object_key(path: &str, bucket: &str) -> String {
    let prefix = format!("/{bucket}/");
    path.strip_prefix(&prefix).unwrap_or(path).to_string()
}

fn build_archive_fixture_zip() -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o644);
    let directory_options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);
    writer
        .add_directory("docs/", directory_options)
        .expect("archive fixture directory should be added");
    writer
        .start_file("docs/readme.txt", options)
        .expect("archive fixture readme should start");
    writer
        .write_all(b"hello from archive")
        .expect("archive fixture readme should be written");
    writer
        .start_file("images/logo.png", options)
        .expect("archive fixture logo should start");
    writer
        .write_all(b"PNGDATA")
        .expect("archive fixture logo should be written");
    writer
        .finish()
        .expect("archive fixture should finish")
        .into_inner()
}

#[tokio::test]
async fn create_space_route_persists_space_with_special_type() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = common::test_router_with_pool(pool.clone());
    let request_body = r#"{
        "id":"space-kb-001",
                "ownerSubjectType":"user",
        "ownerSubjectId":"user-001",
        "displayName":"Knowledge Space",
        "spaceType":"knowledge_base"
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/spaces")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .expect("request should be built"),
        )
        .await
        .expect("create space request should be handled");

    assert_eq!(response.status(), StatusCode::CREATED);
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_space WHERE id=$1 AND space_type='knowledge_base'",
    )
    .bind("space-kb-001")
    .fetch_one(&pool)
    .await
    .expect("space should be persisted");
    assert_eq!(count, 1);
    let change_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_change_log WHERE event_type='drive.space.created' AND space_id='space-kb-001'",
    )
    .fetch_one(&pool)
    .await
    .expect("space created change should be queryable");
    assert_eq!(change_count, 1);
}

#[tokio::test]
async fn create_space_route_persists_user_git_repository_space() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = common::test_router_with_pool(pool.clone());
    let request_body = r#"{
        "id":"space-git-repository-001",
                "ownerSubjectType":"user",
        "ownerSubjectId":"user-001",
        "displayName":"Git Repositories",
        "spaceType":"git_repository"
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/spaces")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .expect("request should be built"),
        )
        .await
        .expect("create git repository space request should be handled");

    assert_eq!(response.status(), StatusCode::CREATED);
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_space
         WHERE id=$1 AND owner_subject_type='user' AND owner_subject_id='user-001' AND space_type='git_repository'",
    )
    .bind("space-git-repository-001")
    .fetch_one(&pool)
    .await
    .expect("git repository space should be persisted");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn delete_space_route_rejects_user_git_repository_space() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-git-repository-delete-guard', 'tenant-git-repository-delete-guard', 'user', 'user-owner',
            'git_repository', 'Git Repositories', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("git repository space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'git-repository-folder-delete-guard', 'tenant-git-repository-delete-guard', 'space-git-repository-delete-guard',
            NULL, 'folder', 'inventory-service', 'ready', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("repository directory should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-git-repository-delete-guard",
                            "user-owner",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token(
                        "tenant-git-repository-delete-guard",
                        "user-owner",
                        "appbase",
                    ),
                )
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/spaces/space-git-repository-delete-guard")
                .body(Body::empty())
                .expect("delete git repository space request should be built"),
        )
        .await
        .expect("delete git repository space request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("delete git repository space response should be read"),
    )
    .expect("delete git repository space response should be valid json");
    assert_eq!(
        payload["detail"].as_str(),
        Some("git repository space cannot be deleted")
    );

    let space_status: String = sqlx::query_scalar(
        "SELECT lifecycle_status FROM dr_drive_space WHERE id='space-git-repository-delete-guard'",
    )
    .fetch_one(&pool)
    .await
    .expect("space status should be readable");
    assert_eq!(space_status, "active");
    let node_status: String = sqlx::query_scalar(
        "SELECT lifecycle_status FROM dr_drive_node WHERE id='git-repository-folder-delete-guard'",
    )
    .fetch_one(&pool)
    .await
    .expect("node status should be readable");
    assert_eq!(node_status, "active");
}

#[tokio::test]
async fn list_nodes_route_validates_active_space_and_folder_parent() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-list-guard', 'tenant-list-guard', 'user', 'user-owner', 'personal', 'List Guard', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("active space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-list-deleted', 'tenant-list-guard', 'user', 'user-deleted-owner', 'team', 'Deleted List Guard', 'deleted', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("deleted space should be seeded");
    for (id, parent_node_id, node_type, node_name) in [
        (
            "folder-list-parent",
            Option::<&str>::None,
            "folder",
            "Parent",
        ),
        ("file-list-parent", None, "file", "not-a-folder.txt"),
        (
            "file-list-child",
            Some("folder-list-parent"),
            "file",
            "child.txt",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-list-guard', 'space-list-guard', $2, $3, $4, 'ready', 'active', 1, 'user-owner', 'user-owner')",
        )
        .bind(id)
        .bind(parent_node_id)
        .bind(node_type)
        .bind(node_name)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'file-list-trashed-child', 'tenant-list-guard', 'space-list-guard',
            'folder-list-parent', 'file', 'trashed-child.txt',
            'ready', 'trashed', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("trashed child node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'file-list-uploading-child', 'tenant-list-guard', 'space-list-guard',
            'folder-list-parent', 'file', 'uploading-child.txt',
            'uploading', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("uploading child node should be seeded");

    let app = common::test_router_with_pool(pool);

    for uri in [
        "/app/v3/api/drive/spaces/space-list-missing/nodes",
        "/app/v3/api/drive/spaces/space-list-deleted/nodes",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-list-guard", "user-owner", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-list-guard", "user-owner", "appbase"),
                    )
                    .method(Method::GET)
                    .uri(uri)
                    .body(Body::empty())
                    .expect("list nodes invalid space request should be built"),
            )
            .await
            .expect("list nodes invalid space request should be handled");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("list nodes invalid space response should be read"),
        )
        .expect("list nodes invalid space response should be valid json");
        assert_eq!(payload["detail"].as_str(), Some("space not found"));
    }

    let missing_parent_response = app
        .clone()
        .oneshot(
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-list-guard", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-list-guard", "user-owner", "appbase"))
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces/space-list-guard/nodes?parentNodeId=folder-list-missing")
                .body(Body::empty())
                .expect("list nodes missing parent request should be built"),
        )
        .await
        .expect("list nodes missing parent request should be handled");
    assert_eq!(missing_parent_response.status(), StatusCode::NOT_FOUND);
    let missing_parent_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(missing_parent_response.into_body(), usize::MAX)
            .await
            .expect("list nodes missing parent response should be read"),
    )
    .expect("list nodes missing parent response should be valid json");
    assert_eq!(
        missing_parent_payload["detail"].as_str(),
        Some("target parent node not found")
    );

    let file_parent_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-list-guard", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-list-guard", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri(
                    "/app/v3/api/drive/spaces/space-list-guard/nodes?parentNodeId=file-list-parent",
                )
                .body(Body::empty())
                .expect("list nodes file parent request should be built"),
        )
        .await
        .expect("list nodes file parent request should be handled");
    assert_eq!(file_parent_response.status(), StatusCode::BAD_REQUEST);
    let file_parent_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(file_parent_response.into_body(), usize::MAX)
            .await
            .expect("list nodes file parent response should be read"),
    )
    .expect("list nodes file parent response should be valid json");
    assert_eq!(
        file_parent_payload["detail"].as_str(),
        Some("targetParentNodeId must reference an active folder")
    );

    let valid_response = app
        .oneshot(
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-list-guard", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-list-guard", "user-owner", "appbase"))
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces/space-list-guard/nodes?parentNodeId=folder-list-parent")
                .body(Body::empty())
                .expect("list nodes valid parent request should be built"),
        )
        .await
        .expect("list nodes valid parent request should be handled");
    assert_eq!(valid_response.status(), StatusCode::OK);
    let valid_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(valid_response.into_body(), usize::MAX)
            .await
            .expect("list nodes valid parent response should be read"),
    )
    .expect("list nodes valid parent response should be valid json");
    let ids = common::envelope_items(&valid_payload)
        .as_array()
        .expect("list node items should be an array")
        .iter()
        .map(|item| item["id"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["file-list-child".to_string()]);
    assert!(
        !ids.iter().any(|id| id == "file-list-uploading-child"),
        "nodes.list should not expose uploading nodes before completion",
    );
}

#[tokio::test]
async fn list_nodes_rejects_invalid_sort_by() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-sort-guard', 'tenant-sort-guard', 'user', 'user-owner', 'personal', 'Sort Guard', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-sort-guard", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-sort-guard", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces/space-sort-guard/nodes?sortBy=not-a-field")
                .body(Body::empty())
                .expect("invalid sort request should be built"),
        )
        .await
        .expect("invalid sort request should be handled");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("invalid sort response should be read"),
    )
    .expect("invalid sort response should be valid json");
    assert_eq!(payload["detail"].as_str(), Some("sortBy is invalid"));
}

#[tokio::test]
async fn list_nodes_sort_by_name_desc_orders_active_children() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-sort-name', 'tenant-sort-name', 'user', 'user-owner', 'personal', 'Sort Name', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'folder-sort-parent', 'tenant-sort-name', 'space-sort-name',
            NULL, 'folder', 'Parent', 'ready', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("parent folder should be seeded");
    for (id, node_name) in [
        ("file-sort-alpha", "alpha.txt"),
        ("file-sort-zeta", "zeta.txt"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-sort-name', 'space-sort-name', 'folder-sort-parent', 'file', $2, 'ready', 'active', 1, 'user-owner', 'user-owner')",
        )
        .bind(id)
        .bind(node_name)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-sort-name", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-sort-name", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces/space-sort-name/nodes?parentNodeId=folder-sort-parent&sortBy=name&sortOrder=desc")
                .body(Body::empty())
                .expect("sorted list request should be built"),
        )
        .await
        .expect("sorted list request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("sorted list response should be read"),
    )
    .expect("sorted list response should be valid json");
    let names = common::envelope_items(&payload)
        .as_array()
        .expect("sorted list items should be an array")
        .iter()
        .filter(|item| item["nodeType"].as_str() == Some("file"))
        .map(|item| item["nodeName"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["zeta.txt".to_string(), "alpha.txt".to_string()]);
}

#[tokio::test]
async fn list_nodes_includes_ui_folder_color_from_node_properties() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-folder-color', 'tenant-folder-color', 'user', 'user-owner', 'personal', 'Colors', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'folder-colored', 'tenant-folder-color', 'space-folder-color', NULL, 'folder', 'Emerald',
            'ready', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("folder node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_property (
            id, tenant_id, node_id, property_key, property_value, visibility,
            lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'prop-folder-color', 'tenant-folder-color', 'folder-colored', 'ui.folderColor', 'emerald',
            'private', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("folder color property should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-folder-color", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-folder-color", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces/space-folder-color/nodes")
                .body(Body::empty())
                .expect("list nodes request should be built"),
        )
        .await
        .expect("list nodes request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("list nodes response body should be read"),
    )
    .expect("list nodes response json should be valid");
    assert_eq!(
        common::envelope_items(&payload)[0]["folderColor"].as_str(),
        Some("emerald")
    );
}

#[tokio::test]
async fn recent_search_and_favorite_routes_hide_uploading_nodes_until_ready() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-visibility', 'tenant-visibility', 'user', 'user-owner', 'personal', 'Visibility', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    for (id, name, state) in [
        ("node-ready-visible", "ready-report.txt", "ready"),
        ("node-uploading-hidden", "uploading-report.txt", "uploading"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-visibility', 'space-visibility', NULL, 'file', $2, $3, 'active', 1, 'user-owner', 'user-owner')",
        )
        .bind(id)
        .bind(name)
        .bind(state)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }

    sqlx::query(
        "INSERT INTO dr_drive_node_favorite (
            tenant_id, node_id, subject_type, subject_id, lifecycle_status, updated_at, created_by, updated_by
        ) VALUES
            ('tenant-visibility', 'node-ready-visible', 'user', 'user-owner', 'active', CURRENT_TIMESTAMP, 'user-owner', 'user-owner'),
            ('tenant-visibility', 'node-uploading-hidden', 'user', 'user-owner', 'active', CURRENT_TIMESTAMP, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("favorites should be seeded");

    let app = common::test_router_with_pool(pool);
    for (uri, label) in [
        (
            "/app/v3/api/drive/recent?spaceId=space-visibility",
            "recent",
        ),
        (
            "/app/v3/api/drive/search?q=report&spaceId=space-visibility",
            "search",
        ),
        (
            "/app/v3/api/drive/favorites?spaceId=space-visibility",
            "favorites",
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-visibility", "user-owner", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-visibility", "user-owner", "appbase"),
                    )
                    .method(Method::GET)
                    .uri(uri)
                    .body(Body::empty())
                    .expect("visibility request should be built"),
            )
            .await
            .expect("visibility request should be handled");
        assert_eq!(response.status(), StatusCode::OK, "{label}");
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("visibility response should be read"),
        )
        .expect("visibility response should be valid json");
        let ids = common::envelope_items(&payload)
            .as_array()
            .expect("items should be an array")
            .iter()
            .map(|item| item["id"].as_str().unwrap_or_default().to_string())
            .collect::<Vec<_>>();
        assert!(
            ids.iter().any(|id| id == "node-ready-visible"),
            "{label} should include ready nodes"
        );
        assert!(
            !ids.iter().any(|id| id == "node-uploading-hidden"),
            "{label} should hide uploading nodes"
        );
    }
}

#[tokio::test]
async fn quota_summary_route_counts_active_storage_objects() {
    let previous_quota = std::env::var("SDKWORK_DRIVE_TENANT_QUOTA_MAX_BYTES").ok();
    std::env::set_var("SDKWORK_DRIVE_TENANT_QUOTA_MAX_BYTES", "10485760");
    let _quota_guard = RestoreEnvVar {
        key: "SDKWORK_DRIVE_TENANT_QUOTA_MAX_BYTES",
        previous: previous_quota,
    };

    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, 'user', 'user-001', 'personal', 'Main', 'active', 1, 'user-001', 'user-001')",
    )
    .bind("space-quota")
    .bind("tenant-quota")
    .execute(&pool)
    .await
    .expect("space should be inserted");

    for node_id in ["node-quota-active", "node-quota-deleted"] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-quota', 'space-quota', NULL, 'file', $2, 'ready', 'active', 1, 'user-001', 'user-001')",
        )
        .bind(node_id)
        .bind(format!("{node_id}.bin"))
        .execute(&pool)
        .await
        .expect("node should be inserted");
    }
    seed_storage_metadata_provider_fixture(&pool, "provider-quota", "bucket-quota", "user-001")
        .await;

    for (object_id, node_id, content_length, lifecycle_status, checksum) in [
        (
            "object-quota-active",
            "node-quota-active",
            4096_i64,
            "active",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ),
        (
            "object-quota-deleted",
            "node-quota-deleted",
            8192_i64,
            "deleted",
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ),
    ] {
        seed_storage_object_fixture(
            &pool,
            StorageObjectFixture {
                object_id,
                tenant_id: "tenant-quota",
                node_id,
                version_no: 1,
                provider_id: "provider-quota",
                bucket: "bucket-quota",
                object_key: &format!("objects/{node_id}.bin"),
                content_type: "application/octet-stream",
                content_length,
                checksum_sha256_hex: checksum,
                lifecycle_status,
                actor_id: "user-001",
            },
        )
        .await
        .expect("storage object should be inserted");
    }

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-quota", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-quota", "user-001", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/quotas/summary")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("quota summary request should be handled");

    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    let quota_item = common::envelope_item(&payload);
    assert_eq!(quota_item["tenantId"], "tenant-quota");
    assert_eq!(quota_item["usedBytes"], 4096);
    assert_eq!(quota_item["objectCount"], 1);
    assert_eq!(quota_item["quotaBytes"], 10_485_760);
}

struct RestoreEnvVar {
    key: &'static str,
    previous: Option<String>,
}

impl Drop for RestoreEnvVar {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

#[tokio::test]
async fn create_upload_session_route_is_idempotent() {
    let (s3_endpoint, _captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $8)",
    )
    .bind("space-001")
    .bind("tenant-001")
    .bind("user")
    .bind("user-001")
    .bind("personal")
    .bind("Main")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed space should succeed");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, NULL, $4, $5, 'ready', 'active', 1, $6, $7)",
    )
    .bind("node-001")
    .bind("tenant-001")
    .bind("space-001")
    .bind("file")
    .bind("v1.bin")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed node should succeed");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-idempotent', 's3_compatible', 'Mock S3', $1, 'us-east-1',
            'bucket-001', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-001', 'admin-001'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let first_body = r#"{
        "sessionId":"upload-session-001",
                "spaceId":"space-001",
        "nodeId":"node-001",
        "bucket":"bucket-001",
        "objectKey":"objects/node-001/v1.bin",
        "idempotencyKey":"idem-abc",
        "expiresAtEpochMs":1800000000000
    }"#;
    let second_body = r#"{
        "sessionId":"upload-session-002",
                "spaceId":"space-001",
        "nodeId":"node-001",
        "bucket":"bucket-001",
        "objectKey":"objects/node-001/v1.bin",
        "idempotencyKey":"idem-abc",
        "expiresAtEpochMs":1800000005000
    }"#;

    let first_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(first_body))
                .expect("first request should be built"),
        )
        .await
        .expect("first upload session request should be handled");
    assert_eq!(first_response.status(), StatusCode::CREATED);
    let first_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(first_response.into_body(), usize::MAX)
            .await
            .expect("first response body should be read"),
    )
    .expect("first response json should be valid");

    let second_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(second_body))
                .expect("second request should be built"),
        )
        .await
        .expect("second upload session request should be handled");
    assert_eq!(second_response.status(), StatusCode::CREATED);
    let second_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(second_response.into_body(), usize::MAX)
            .await
            .expect("second response body should be read"),
    )
    .expect("second response json should be valid");

    assert_eq!(
        common::envelope_body(&first_payload)["id"],
        common::envelope_body(&second_payload)["id"]
    );
    let change_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_change_log WHERE event_type='drive.upload_session.created'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload session created change should be queryable");
    assert_eq!(change_count, 1);
}

#[tokio::test]
async fn app_api_rejects_storage_provider_administration_routes_without_s3_side_effects() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-app-admin-boundary', 's3_compatible', 'App Boundary S3', $1, 'us-east-1',
            'bucket-s3', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-app', 'admin-app'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = common::test_router_with_pool(pool);
    for (method, uri, body) in [
        (
            Method::POST,
            "/app/v3/api/drive/storage_providers",
            r#"{"id":"provider-new"}"#,
        ),
        (
            Method::GET,
            "/app/v3/api/drive/storage_providers/provider-app-admin-boundary/objects/objects/file-a.bin",
            "",
        ),
        (
            Method::DELETE,
            "/app/v3/api/drive/storage_providers/provider-app-admin-boundary/objects/objects/file-a.bin",
            "",
        ),
        (
            Method::POST,
            "/app/v3/api/drive/storage_providers/provider-app-admin-boundary/test",
            r#"{"operatorId":"admin-app"}"#,
        ),
        (
            Method::PUT,
            "/app/v3/api/drive/storage_provider_bindings/default",
            r#"{"providerId":"provider-app-admin-boundary","operatorId":"admin-app"}"#,
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                .header(
                    "authorization",
                    format!("Bearer {}", common::auth_token("tenant-001", "admin-app", "appbase")),
                )
                .header("access-token", common::access_token("tenant-001", "admin-app", "appbase"))
                    .method(method)
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .expect("storage admin boundary request should be built"),
            )
            .await
            .expect("storage admin boundary request should be handled");
        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "{uri} must not be exposed by the app api"
        );
    }

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(requests.is_empty());
}

#[tokio::test]
async fn create_upload_session_rejects_existing_session_id_before_storage_side_effects() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-upload-id-conflict', 'tenant-upload-id-conflict', 'user',
            'user-upload', 'personal', 'Upload Conflict', 'active', 1,
            'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-upload-id-conflict', 'tenant-upload-id-conflict',
            'space-upload-id-conflict', NULL, 'file', 'upload.bin',
            'ready', 'active', 1, 'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-upload-id-conflict",
        "Upload Conflict S3",
        &s3_endpoint,
        "bucket-upload-id-conflict",
        "active",
        "user-upload",
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'default:space:tenant-upload-id-conflict:space-upload-id-conflict',
            'tenant-upload-id-conflict', 'space-upload-id-conflict',
            'provider-upload-id-conflict', 'space', 'primary',
            'sdkwork-drive/v1/tenants/tenant-upload-id-conflict/spaces/space-upload-id-conflict',
            'active', 1, 'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage provider binding should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'upload-id-conflict', 'tenant-upload-id-conflict',
            'space-upload-id-conflict', 'node-upload-id-conflict',
            'bucket-upload-id-conflict', 'objects/existing.bin',
            'idem-existing-upload', 'provider-upload-id-conflict',
            'existing-storage-upload', 'created', 1800000000000, 1,
            'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("existing upload session should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-upload-id-conflict", "user-upload", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-upload-id-conflict", "user-upload", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sessionId":"upload-id-conflict",
                        "spaceId":"space-upload-id-conflict",
                        "nodeId":"node-upload-id-conflict",
                        "idempotencyKey":"idem-new-upload",
                        "expiresAtEpochMs":1800000000000
                    }"#,
                ))
                .expect("conflicting upload session request should be built"),
        )
        .await
        .expect("conflicting upload session request should be handled");

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("conflict response should be read"),
    )
    .expect("conflict response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40901));

    let upload_session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_upload_session
         WHERE tenant_id='tenant-upload-id-conflict'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload session count should be readable");
    assert_eq!(upload_session_count, 1);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(requests.is_empty());
}

#[tokio::test]
async fn create_upload_session_resolves_default_bucket_and_generates_standard_key_when_client_omits_storage_target(
) {
    let (s3_endpoint, _captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-keygen', 'tenant-keygen', 'user', 'user-keygen', 'personal', 'KeyGen', 'active', 1, 'user-keygen', 'user-keygen')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-keygen', 'tenant-keygen', 'space-keygen', NULL, 'file', 'report.txt', 'uploading', 'active', 1, 'user-keygen', 'user-keygen')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-keygen', 's3_compatible', 'Mock S3', $1, 'us-east-1',
            'bucket-keygen', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-keygen', 'admin-keygen'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'binding-keygen', 'tenant-keygen', 'space-keygen', 'provider-keygen',
            'space', 'primary', 'drive-custom-root/tenant-keygen/space-keygen',
            'active', 1, 'admin-keygen', 'admin-keygen'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage provider binding should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-keygen", "user-keygen", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-keygen", "user-keygen", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sessionId":"upload-keygen",
                        "spaceId":"space-keygen",
                        "nodeId":"node-keygen",
                        "idempotencyKey":"idem-keygen",
                        "expiresAtEpochMs":1800000000000
                    }"#,
                ))
                .expect("create upload session request should be built"),
        )
        .await
        .expect("create upload session should be handled");
    assert_eq!(response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(
        common::envelope_body(&payload)["bucket"].as_str(),
        Some("bucket-keygen")
    );
    let object_key = common::envelope_body(&payload)["objectKey"]
        .as_str()
        .expect("response objectKey should be present");
    assert!(object_key.starts_with("drive-custom-root/tenant-keygen/space-keygen/"));
    assert_standard_storage_object_key(
        object_key
            .strip_prefix("drive-custom-root/tenant-keygen/space-keygen/")
            .expect("custom root prefix should wrap the standard storage key"),
        "tenant-keygen",
        "space-keygen",
        "node-keygen",
    );
    assert!(object_key.ends_with("/upload-keygen/content"));

    let stored_object_key: String = sqlx::query_scalar(
        "SELECT object_key FROM dr_drive_upload_session WHERE id='upload-keygen'",
    )
    .fetch_one(&pool)
    .await
    .expect("object key should be stored");
    assert_eq!(stored_object_key, object_key);
}

#[tokio::test]
async fn create_upload_session_rejects_trashed_node_before_storage_side_effects() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-upload-create-trash', 'tenant-upload-create-trash',
            'user', 'user-upload', 'personal', 'Upload Create Trash',
            'active', 1, 'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-upload-create-trash', 'tenant-upload-create-trash',
            'space-upload-create-trash', NULL, 'file', 'trashed-upload.bin',
            'uploading', 'trashed', 1, 'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("trashed node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-upload-create-trash",
        "Upload Create Trash S3",
        &s3_endpoint,
        "bucket-upload-create-trash",
        "active",
        "user-upload",
    )
    .await;

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-upload-create-trash", "user-upload", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-upload-create-trash", "user-upload", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sessionId":"upload-create-trash",
                        "spaceId":"space-upload-create-trash",
                        "nodeId":"node-upload-create-trash",
                        "bucket":"bucket-upload-create-trash",
                        "objectKey":"objects/upload-create-trash.bin",
                        "idempotencyKey":"idem-upload-create-trash",
                        "expiresAtEpochMs":1800000000000
                    }"#,
                ))
                .expect("create upload session request should be built"),
        )
        .await
        .expect("create upload session request should be handled");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("create upload session error response should be read"),
    )
    .expect("create upload session error response should be valid json");
    assert_eq!(payload["detail"].as_str(), Some("node not found"));

    let upload_session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_upload_session
         WHERE tenant_id='tenant-upload-create-trash'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload session count should be readable");
    assert_eq!(upload_session_count, 0);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(requests.is_empty());
}

#[tokio::test]
async fn create_upload_session_rejects_past_expiration_before_storage_side_effects() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-upload-create-expired', 'tenant-upload-create-expired',
            'user', 'user-upload', 'personal', 'Upload Create Expired',
            'active', 1, 'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-upload-create-expired', 'tenant-upload-create-expired',
            'space-upload-create-expired', NULL, 'file', 'expired-upload.bin',
            'uploading', 'active', 1, 'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("active node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-upload-create-expired",
        "Upload Create Expired S3",
        &s3_endpoint,
        "bucket-upload-create-expired",
        "active",
        "user-upload",
    )
    .await;

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-upload-create-expired",
                            "user-upload",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-upload-create-expired", "user-upload", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sessionId":"upload-create-expired",
                        "spaceId":"space-upload-create-expired",
                        "nodeId":"node-upload-create-expired",
                        "bucket":"bucket-upload-create-expired",
                        "objectKey":"objects/upload-create-expired.bin",
                        "idempotencyKey":"idem-upload-create-expired",
                        "expiresAtEpochMs":1
                    }"#,
                ))
                .expect("create upload session request should be built"),
        )
        .await
        .expect("create upload session request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("create upload session error response should be read"),
    )
    .expect("create upload session error response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40001));
    assert!(payload["detail"]
        .as_str()
        .unwrap_or_default()
        .contains("expiresAtEpochMs must be in the future"));

    let upload_session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_upload_session
         WHERE tenant_id='tenant-upload-create-expired'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload session count should be readable");
    assert_eq!(upload_session_count, 0);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(requests.is_empty());
}

#[tokio::test]
async fn create_upload_session_rejects_empty_idempotency_key_before_storage_side_effects() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-upload-empty-idem', 'tenant-upload-empty-idem',
            'user', 'user-upload', 'personal', 'Upload Empty Idem',
            'active', 1, 'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-upload-empty-idem', 'tenant-upload-empty-idem',
            'space-upload-empty-idem', NULL, 'file', 'empty-idem.bin',
            'uploading', 'active', 1, 'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-upload-empty-idem",
        "Upload Empty Idem S3",
        &s3_endpoint,
        "bucket-upload-empty-idem",
        "active",
        "user-upload",
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'default:space:tenant-upload-empty-idem:space-upload-empty-idem',
            'tenant-upload-empty-idem', 'space-upload-empty-idem',
            'provider-upload-empty-idem', 'space', 'primary',
            'sdkwork-drive/v1/tenants/tenant-upload-empty-idem/spaces/space-upload-empty-idem',
            'active', 1, 'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage provider binding should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-upload-empty-idem", "user-upload", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-upload-empty-idem", "user-upload", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sessionId":"upload-empty-idem",
                        "spaceId":"space-upload-empty-idem",
                        "nodeId":"node-upload-empty-idem",
                        "idempotencyKey":"   ",
                        "expiresAtEpochMs":1800000000000
                    }"#,
                ))
                .expect("empty idempotency upload session request should be built"),
        )
        .await
        .expect("empty idempotency upload session request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("validation response should be read"),
    )
    .expect("validation response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40001));

    let upload_session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_upload_session
         WHERE tenant_id='tenant-upload-empty-idem'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload session count should be readable");
    assert_eq!(upload_session_count, 0);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(requests.is_empty());

    assert!(payload["detail"]
        .as_str()
        .unwrap_or_default()
        .contains("idempotencyKey is required"));
}

#[tokio::test]
async fn create_upload_session_normalizes_identifiers_before_idempotency_and_storage() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-upload-normalized', 'tenant-upload-normalized',
            'user', 'user-upload', 'personal', 'Upload Normalized',
            'active', 1, 'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-upload-normalized', 'tenant-upload-normalized',
            'space-upload-normalized', NULL, 'file', 'normalized.bin',
            'uploading', 'active', 1, 'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-upload-normalized",
        "Upload Normalized S3",
        &s3_endpoint,
        "bucket-upload-normalized",
        "active",
        "user-upload",
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'default:space:tenant-upload-normalized:space-upload-normalized',
            'tenant-upload-normalized', 'space-upload-normalized',
            'provider-upload-normalized', 'space', 'primary',
            'sdkwork-drive/v1/tenants/tenant-upload-normalized/spaces/space-upload-normalized',
            'active', 1, 'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage provider binding should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let first_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-upload-normalized",
                            "  user-upload  ",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-upload-normalized", "  user-upload  ", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sessionId":"  upload-normalized  ",
                        "spaceId":"space-upload-normalized",
                        "nodeId":"node-upload-normalized",
                        "idempotencyKey":"  idem-upload-normalized  ",
                        "expiresAtEpochMs":1800000000000
                    }"#,
                ))
                .expect("first normalized upload session request should be built"),
        )
        .await
        .expect("first normalized upload session request should be handled");
    assert_eq!(first_response.status(), StatusCode::CREATED);
    let first_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(first_response.into_body(), usize::MAX)
            .await
            .expect("first response body should be read"),
    )
    .expect("first response json should be valid");
    assert_eq!(
        common::envelope_body(&first_payload)["id"].as_str(),
        Some("upload-normalized")
    );
    assert_eq!(
        common::envelope_body(&first_payload)["idempotencyKey"].as_str(),
        Some("idem-upload-normalized")
    );

    let second_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-upload-normalized",
                            "  user-upload  ",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-upload-normalized", "  user-upload  ", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sessionId":"upload-normalized-retry",
                        "spaceId":"space-upload-normalized",
                        "nodeId":"node-upload-normalized",
                        "idempotencyKey":"idem-upload-normalized",
                        "expiresAtEpochMs":1800000005000
                    }"#,
                ))
                .expect("second normalized upload session request should be built"),
        )
        .await
        .expect("second normalized upload session request should be handled");
    assert_eq!(second_response.status(), StatusCode::CREATED);
    let second_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(second_response.into_body(), usize::MAX)
            .await
            .expect("second response body should be read"),
    )
    .expect("second response json should be valid");

    assert_eq!(
        common::envelope_body(&first_payload)["id"],
        common::envelope_body(&second_payload)["id"]
    );
    assert_eq!(
        common::envelope_body(&first_payload)["storageUploadId"],
        common::envelope_body(&second_payload)["storageUploadId"]
    );

    let stored: (String, String, String) = sqlx::query_as(
        "SELECT id, idempotency_key, created_by
         FROM dr_drive_upload_session
         WHERE tenant_id='tenant-upload-normalized'",
    )
    .fetch_one(&pool)
    .await
    .expect("stored normalized upload session should be readable");
    assert_eq!(stored.0, "upload-normalized");
    assert_eq!(stored.1, "idem-upload-normalized");
    assert_eq!(stored.2, "user-upload");

    let upload_session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_upload_session
         WHERE tenant_id='tenant-upload-normalized'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload session count should be readable");
    assert_eq!(upload_session_count, 1);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    let multipart_create_count = requests
        .iter()
        .filter(|request| request.method == "POST" && request.query.contains("uploads"))
        .count();
    assert_eq!(multipart_create_count, 1);
}

#[tokio::test]
async fn create_upload_session_requires_active_object_store_provider() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-create-no-provider', 'tenant-create-no-provider', 'user', 'user-create-no-provider', 'personal', 'No Provider', 'active', 1, 'user-create-no-provider', 'user-create-no-provider')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-create-no-provider', 'tenant-create-no-provider', 'space-create-no-provider', NULL, 'file', 'upload.bin', 'uploading', 'active', 1, 'user-create-no-provider', 'user-create-no-provider')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-create-no-provider",
                            "user-create-no-provider",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token(
                        "tenant-create-no-provider",
                        "user-create-no-provider",
                        "appbase",
                    ),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sessionId":"upload-create-no-provider",
                        "spaceId":"space-create-no-provider",
                        "nodeId":"node-create-no-provider",
                        "bucket":"bucket-create-no-provider",
                        "idempotencyKey":"idem-create-no-provider",
                        "expiresAtEpochMs":1800000000000
                    }"#,
                ))
                .expect("create upload session request should be built"),
        )
        .await
        .expect("create upload session should be handled");

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(payload["code"], 40901);
    assert!(payload["detail"]
        .as_str()
        .expect("detail should be a string")
        .contains("active storage provider"));

    let stored_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_upload_session WHERE id='upload-create-no-provider'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload session count should be readable");
    assert_eq!(stored_count, 0);
}

#[tokio::test]
async fn s3_upload_session_uses_real_multipart_upload_id_for_presign_and_complete() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-s3', 'tenant-s3', 'user', 'user-s3', 'personal', 'S3', 'active', 1, 'user-s3', 'user-s3')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-s3', 'tenant-s3', 'space-s3', NULL, 'file', 'data.bin', 'uploading', 'active', 1, 'user-s3', 'user-s3')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-s3-upload', 's3_compatible', 'Mock S3', $1, 'us-east-1',
            'bucket-s3', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-s3', 'admin-s3'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-s3", "user-s3", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-s3", "user-s3", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sessionId":"upload-s3",
                        "spaceId":"space-s3",
                        "nodeId":"node-s3",
                        "bucket":"bucket-s3",
                        "objectKey":"objects/node-s3/data.bin",
                        "idempotencyKey":"idem-s3",
                        "expiresAtEpochMs":1800000000000
                    }"#,
                ))
                .expect("create upload session request should be built"),
        )
        .await
        .expect("create upload session should be handled");
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("create response body should be read"),
    )
    .expect("create response json should be valid");
    assert_eq!(
        common::envelope_body(&create_payload)["storageUploadId"],
        "mock-s3-upload-id"
    );
    let created_object_key = common::envelope_body(&create_payload)["objectKey"]
        .as_str()
        .expect("create upload response should include objectKey");
    assert_rooted_standard_storage_object_key(
        created_object_key,
        "sdkwork-drive/v1/tenants/tenant-s3/spaces/space-s3",
        "tenant-s3",
        "space-s3",
        "node-s3",
    );
    assert!(
        !created_object_key.contains("objects/node-s3/data.bin"),
        "client supplied objectKey must not control storage layout"
    );

    let stored_upload_id: String = sqlx::query_scalar(
        "SELECT storage_upload_id FROM dr_drive_upload_session WHERE id='upload-s3'",
    )
    .fetch_one(&pool)
    .await
    .expect("storage upload id should be persisted");
    assert_eq!(stored_upload_id, "mock-s3-upload-id");

    let part_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-s3", "user-s3", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-s3", "user-s3", "appbase"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/upload_sessions/upload-s3/parts/1")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "requestedTtlSeconds":120
                    }"#,
                ))
                .expect("presign upload part request should be built"),
        )
        .await
        .expect("presign upload part should be handled");
    assert_eq!(part_response.status(), StatusCode::OK);
    let part_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(part_response.into_body(), usize::MAX)
            .await
            .expect("part response body should be read"),
    )
    .expect("part response json should be valid");
    let part_data = common::envelope_data(&part_payload);
    assert_eq!(part_data["uploadId"], "mock-s3-upload-id");
    assert!(part_data["uploadUrl"]
        .as_str()
        .expect("upload url should be present")
        .contains("uploadId=mock-s3-upload-id"));

    let complete_response = app
        .clone()
        .oneshot(
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-s3", "user-s3", "appbase")),
            )
            .header("access-token", common::access_token("tenant-s3", "user-s3", "appbase"))
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions/upload-s3/complete")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "contentType":"application/octet-stream",
                        "contentLength":12,
                        "checksumSha256Hex":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        "parts":[{"partNo":1,"etag":"etag-s3-1"}]
                    }"#,
                ))
                .expect("complete upload request should be built"),
        )
        .await
        .expect("complete upload should be handled");
    assert_eq!(complete_response.status(), StatusCode::OK);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(
        requests.iter().any(|request| request.method == "POST"
            && request.path.starts_with(
                "/bucket-s3/sdkwork-drive/v1/tenants/tenant-s3/spaces/space-s3/sdkwork-drive/v1/t/"
            )
            && request.path.contains("/tenants/tenant-s3/spaces/space-s3/")
            && request.path.contains("/node-s3/versions/")
            && request.path.ends_with("/upload-s3/content")
            && request.query.contains("uploads")),
        "create upload session should call S3 CreateMultipartUpload"
    );
    assert!(
        requests.iter().any(|request| request.method == "POST"
            && request.path.starts_with(
                "/bucket-s3/sdkwork-drive/v1/tenants/tenant-s3/spaces/space-s3/sdkwork-drive/v1/t/"
            )
            && request.path.contains("/tenants/tenant-s3/spaces/space-s3/")
            && request.path.contains("/node-s3/versions/")
            && request.path.ends_with("/upload-s3/content")
            && request.query.contains("uploadId=mock-s3-upload-id")
            && request.body.contains("<PartNumber>1</PartNumber>")
            && request.body.contains("<ETag>etag-s3-1</ETag>")),
        "complete upload session should call S3 CompleteMultipartUpload"
    );
}

#[tokio::test]
async fn uploader_prepare_creates_upload_space_item_and_real_multipart_upload() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-uploader-s3', 's3_compatible', 'Uploader S3', $1, 'us-east-1',
            'bucket-s3', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-uploader', 'admin-uploader'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token_for_organization(
                            "tenant-uploader-app",
                            "user-uploader-app",
                            "org-uploader-app",
                            "drive-pc",
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token_for_organization(
                        "tenant-uploader-app",
                        "user-uploader-app",
                        "org-uploader-app",
                        "drive-pc",
                    ),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/uploader/uploads")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"upload-item-app-001",
                        "taskId":"task-app-001",
                        "appResourceType":"desktop-file-browser",
                        "appResourceId":"root",
                        "scene":"user_document_upload",
                        "source":"pc_local_file",
                        "uploadProfileCode":"video",
                        "fileFingerprint":"sha256:fingerprint-app-001",
                        "originalFileName":"clip.mp4",
                        "contentType":"video/mp4",
                        "contentLength":10485760,
                        "chunkSizeBytes":5242880,
                        "retention":{
                            "mode":"temporary",
                            "ttlSeconds":86400,
                            "cleanupAction":"soft_delete"
                        },
                        "nowEpochMs":1700000000000
                    }"#,
                ))
                .expect("uploader prepare request should be built"),
        )
        .await
        .expect("uploader prepare request should be handled");
    assert_eq!(response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("uploader prepare response should be read"),
    )
    .expect("uploader prepare response should be valid json");
    let prepare_data = common::envelope_data(&payload);
    assert_eq!(prepare_data["uploadItem"]["id"], "upload-item-app-001");
    assert_eq!(prepare_data["uploadItem"]["uploadProfileCode"], "video");
    assert_eq!(prepare_data["uploadItem"]["contentTypeGroup"], "video");
    assert_eq!(prepare_data["uploadItem"]["scene"], "user_document_upload");
    assert_eq!(prepare_data["uploadItem"]["source"], "pc_local_file");
    assert_eq!(
        prepare_data["uploadSession"]["storageUploadId"],
        "mock-s3-upload-id"
    );
    assert_eq!(prepare_data["uploadSession"]["bucket"], "bucket-s3");

    let space_row: (String, String, String) = sqlx::query_as(
        "SELECT space_type, owner_subject_type, owner_subject_id
         FROM dr_drive_space
         WHERE tenant_id='tenant-uploader-app'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload space should be persisted");
    assert_eq!(
        space_row,
        (
            "app_upload".to_string(),
            "user".to_string(),
            "user-uploader-app".to_string()
        )
    );

    let stored_upload: (String, String, String) = sqlx::query_as(
        "SELECT storage_upload_id, scene, source
         FROM dr_drive_upload_item
         WHERE id='upload-item-app-001'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload item should be persisted");
    assert_eq!(
        stored_upload,
        (
            "mock-s3-upload-id".to_string(),
            "user_document_upload".to_string(),
            "pc_local_file".to_string()
        )
    );

    assert!(
        captured_requests
            .lock()
            .expect("captured s3 requests should be readable")
            .iter()
            .any(|request| request.method == "POST" && request.query.contains("uploads")),
        "uploader prepare should initiate real multipart upload"
    );
}

#[tokio::test]
async fn uploader_prepare_to_target_space_enforces_writer_permission_and_preserves_locator_key() {
    let (s3_endpoint, _captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-uploader-permission', 's3_compatible', 'Uploader Permission S3',
            $1, 'us-east-1', 'bucket-s3', 1, 0,
            'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD',
            'active', 1, 'admin-uploader', 'admin-uploader'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-uploader-permission', 'tenant-uploader-permission',
            'user', 'space-owner', 'app_upload', 'Shared Uploads',
            'active', 1, 'space-owner', 'space-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("target space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'folder-uploader-permission', 'tenant-uploader-permission',
            'space-uploader-permission', NULL, 'folder', 'Incoming',
            'empty', 'active', 1, 'space-owner', 'space-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("target parent folder should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let denied_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-uploader-permission",
                            "user-no-writer",
                            "drive-pc"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token(
                        "tenant-uploader-permission",
                        "user-no-writer",
                        "drive-pc",
                    ),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/uploader/uploads")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"upload-item-target-denied",
                        "taskId":"task-target-denied",
                        "appResourceType":"desktop-file-browser",
                        "appResourceId":"shared-space",
                        "scene":"drive_pc_file_upload",
                        "source":"pc_local_file",
                        "uploadProfileCode":"generic",
                        "fileFingerprint":"fp-target-denied",
                        "originalFileName":"denied.bin",
                        "contentType":"application/octet-stream",
                        "contentLength":1,
                        "chunkSizeBytes":1,
                        "spaceId":"space-uploader-permission",
                        "parentNodeId":"folder-uploader-permission",
                        "nowEpochMs":1700000000000
                    }"#,
                ))
                .expect("denied target upload request should be built"),
        )
        .await
        .expect("denied target upload request should be handled");
    assert_eq!(denied_response.status(), StatusCode::FORBIDDEN);
    let leaked_nodes: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-uploader-permission'
           AND id='node-upload-item-target-denied'",
    )
    .fetch_one(&pool)
    .await
    .expect("node leak count should be readable");
    assert_eq!(leaked_nodes, 0);

    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-uploader-writer', 'tenant-uploader-permission',
            'folder-uploader-permission', 'user', 'user-writer',
            'writer', 0, 'active', 1, 'space-owner', 'space-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("writer permission should be seeded");

    let allowed_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token_for_organization(
                            "tenant-uploader-permission",
                            "user-writer",
                            "org-uploader-permission",
                            "drive-pc",
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token_for_organization(
                        "tenant-uploader-permission",
                        "user-writer",
                        "org-uploader-permission",
                        "drive-pc",
                    ),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/uploader/uploads")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"upload-item-target-allowed",
                        "taskId":"task-target-allowed",
                        "appResourceType":"desktop-file-browser",
                        "appResourceId":"shared-space",
                        "scene":"drive_pc_file_upload",
                        "source":"pc_local_file",
                        "uploadProfileCode":"generic",
                        "fileFingerprint":"fp-target-allowed",
                        "originalFileName":"allowed.bin",
                        "contentType":"application/octet-stream",
                        "contentLength":1,
                        "chunkSizeBytes":1,
                        "spaceId":"space-uploader-permission",
                        "parentNodeId":"folder-uploader-permission",
                        "nowEpochMs":1700000000000
                    }"#,
                ))
                .expect("allowed target upload request should be built"),
        )
        .await
        .expect("allowed target upload request should be handled");
    assert_eq!(allowed_response.status(), StatusCode::CREATED);
    let allowed_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(allowed_response.into_body(), usize::MAX)
            .await
            .expect("allowed target upload response should be read"),
    )
    .expect("allowed target upload response should be valid json");
    let object_key = common::envelope_data(&allowed_payload)["uploadSession"]["objectKey"]
        .as_str()
        .expect("allowed response should include final object key");
    assert!(
        object_key.contains("sdkwork-drive/uploader/"),
        "final uploader object key should preserve upload locator prefix: {object_key}"
    );
    assert!(
        object_key.contains("/tenants/tenant-uploader-permission/"),
        "final uploader object key should include tenant locator: {object_key}"
    );
    assert!(
        object_key.contains("/organizations/org-uploader-permission/"),
        "final uploader object key should include organization locator: {object_key}"
    );
    assert!(
        object_key.contains("/users/user-writer/"),
        "final uploader object key should include user locator: {object_key}"
    );
    assert!(
        object_key.contains("/actors/user/user-writer/"),
        "final uploader object key should include actor locator: {object_key}"
    );
    assert!(
        object_key.contains("/scene/drive_pc_file_upload/"),
        "final uploader object key should include scene locator: {object_key}"
    );
    assert!(
        object_key.contains("/source/pc_local_file/"),
        "final uploader object key should include source locator: {object_key}"
    );
    assert_contains_standard_storage_object_key(
        object_key,
        "tenant-uploader-permission",
        "space-uploader-permission",
        "node-upload-item-target-allowed",
    );
}

#[tokio::test]
async fn uploader_prepare_anonymous_target_space_requires_public_writer_share_token() {
    let (s3_endpoint, _captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-uploader-anon-share', 's3_compatible', 'Uploader Anonymous Share S3',
            $1, 'us-east-1', 'bucket-s3', 1, 0,
            'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD',
            'active', 1, 'admin-uploader', 'admin-uploader'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-uploader-anon-share', 'tenant-uploader-anon-share',
            'user', 'space-owner', 'app_upload', 'Public Uploads',
            'active', 1, 'space-owner', 'space-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("target space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'folder-uploader-anon-share', 'tenant-uploader-anon-share',
            'space-uploader-anon-share', NULL, 'folder', 'Public Incoming',
            'empty', 'active', 1, 'space-owner', 'space-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("target parent folder should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let denied_response = app
        .clone()
        .oneshot(common::anonymous_post_json(
            "/app/v3/api/drive/uploader/uploads",
            "tenant-uploader-anon-share",
            "anon-public-uploader",
            "drive-public",
            Body::from(
                r#"{
                        "id":"upload-item-anon-share-denied",
                        "taskId":"task-anon-share-denied",
                        "appResourceType":"public-form",
                        "appResourceId":"form-001",
                        "scene":"anonymous_public_upload",
                        "source":"public_browser",
                        "uploadProfileCode":"generic",
                        "fileFingerprint":"fp-anon-share-denied",
                        "originalFileName":"denied.bin",
                        "contentType":"application/octet-stream",
                        "contentLength":1,
                        "chunkSizeBytes":1,
                        "spaceId":"space-uploader-anon-share",
                        "parentNodeId":"folder-uploader-anon-share",
                        "nowEpochMs":1700000000000
                    }"#,
            ),
        ))
        .await
        .expect("denied anonymous target upload request should be handled");
    assert_eq!(denied_response.status(), StatusCode::FORBIDDEN);

    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version,
            created_by, updated_by
        ) VALUES (
            'share-uploader-anon-writer', 'tenant-uploader-anon-share',
            'folder-uploader-anon-share', $1, 'writer', 1800000000000,
            NULL, 0, 'active', 1, 'space-owner', 'space-owner'
        )",
    )
    .bind(drive_share_token_hash("public-share-token"))
    .execute(&pool)
    .await
    .expect("writer share link should be seeded");

    let allowed_response = app
        .oneshot(common::anonymous_post_json(
            "/app/v3/api/drive/uploader/uploads",
            "tenant-uploader-anon-share",
            "anon-public-uploader",
            "drive-public",
            Body::from(
                r#"{
                        "id":"upload-item-anon-share-allowed",
                        "taskId":"task-anon-share-allowed",
                        "appResourceType":"public-form",
                        "appResourceId":"form-001",
                        "scene":"anonymous_public_upload",
                        "source":"public_browser",
                        "uploadProfileCode":"generic",
                        "fileFingerprint":"fp-anon-share-allowed",
                        "originalFileName":"allowed.bin",
                        "contentType":"application/octet-stream",
                        "contentLength":1,
                        "chunkSizeBytes":1,
                        "spaceId":"space-uploader-anon-share",
                        "parentNodeId":"folder-uploader-anon-share",
                        "shareToken":"public-share-token",
                        "nowEpochMs":1700000000000
                    }"#,
            ),
        ))
        .await
        .expect("allowed anonymous target upload request should be handled");
    assert_eq!(allowed_response.status(), StatusCode::CREATED);
    let allowed_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(allowed_response.into_body(), usize::MAX)
            .await
            .expect("allowed anonymous target upload response should be read"),
    )
    .expect("allowed anonymous target upload response should be valid json");
    let allowed_data = common::envelope_data(&allowed_payload);
    assert_eq!(
        allowed_data["uploadItem"]["spaceId"],
        "space-uploader-anon-share"
    );
    assert_eq!(allowed_data["uploadItem"]["actorType"], "anonymous");
    assert_eq!(
        allowed_data["uploadItem"]["actorId"],
        "anon-public-uploader"
    );
    assert!(
        allowed_data["uploadItem"]["userId"].is_null(),
        "anonymous uploader facts must not synthesize a user identity"
    );
    let object_key = allowed_data["uploadSession"]["objectKey"]
        .as_str()
        .expect("allowed anonymous response should include final object key");
    assert!(
        object_key.contains("/users/anonymous/"),
        "anonymous final uploader object key should include anonymous user locator: {object_key}"
    );
    assert!(
        object_key.contains("/actors/anonymous/anon-public-uploader/"),
        "anonymous final uploader object key should include actor locator: {object_key}"
    );
    assert_contains_standard_storage_object_key(
        object_key,
        "tenant-uploader-anon-share",
        "space-uploader-anon-share",
        "node-upload-item-anon-share-allowed",
    );
}

#[tokio::test]
async fn uploader_mark_part_uploaded_records_resumable_part_progress() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-uploader-part', 'tenant-uploader-part', 'user',
            'user-uploader-part', 'app_upload', 'Upload', 'active', 1,
            'user-uploader-part', 'user-uploader-part'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-uploader-part', 'tenant-uploader-part', 'space-uploader-part',
            NULL, 'file', 'part.bin', 'uploading', 'active', 1,
            'user-uploader-part', 'user-uploader-part'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-uploader-part', 's3_compatible', 'Uploader S3',
            'https://s3.example.com', 'us-east-1', 'bucket-s3', 1, 1,
            'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD',
            'active', 1, 'admin-uploader', 'admin-uploader'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'session-uploader-part', 'tenant-uploader-part', 'space-uploader-part',
            'node-uploader-part', 'bucket-s3', 'objects/part.bin',
            'idem-uploader-part', 'provider-uploader-part', 'storage-upload-part',
            'uploading', 1800000000000, 1, 'user-uploader-part', 'user-uploader-part'
        )",
    )
    .execute(&pool)
    .await
    .expect("upload session should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_upload_item (
            id, task_id, tenant_id, organization_id, user_id, actor_type, actor_id,
            app_id, app_resource_type, app_resource_id, upload_profile_code,
            file_fingerprint, space_id, node_id, upload_session_id,
            storage_provider_id, storage_upload_id, original_file_name, file_extension,
            content_type, content_type_group, detected_content_type, content_length,
            checksum_sha256_hex, chunk_size_bytes, total_parts, uploaded_parts_count,
            uploaded_bytes, status, retention_mode, retention_expires_at_epoch_ms,
            cleanup_action, hard_delete_after_epoch_ms, cleanup_status,
            post_process_status, created_by, updated_by
        ) VALUES (
            'upload-item-uploader-part', 'task-uploader-part',
            'tenant-uploader-part', NULL, 'user-uploader-part', 'user',
            'user-uploader-part', 'drive-pc', 'desktop-file-browser',
            'root', 'generic', 'fp-uploader-part', 'space-uploader-part',
            'node-uploader-part', 'session-uploader-part',
            'provider-uploader-part', 'storage-upload-part', 'part.bin',
            'bin', 'application/octet-stream', 'binary',
            'application/octet-stream', 12, NULL, 8, 2, 0, 0,
            'uploading', 'long_term', NULL, NULL, NULL, 'active',
            'not_required', 'user-uploader-part', 'user-uploader-part'
        )",
    )
    .execute(&pool)
    .await
    .expect("upload item should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-uploader-part", "user-uploader-part", "appbase")),
            )
            .header("access-token", common::access_token("tenant-uploader-part", "user-uploader-part", "appbase"))
                .method(Method::PUT)
                .uri("/app/v3/api/drive/uploader/uploads/upload-item-uploader-part/parts/1")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "uploadSessionId":"session-uploader-part",
                        "offsetBytes":0,
                        "sizeBytes":8,
                        "etag":"etag-part-1",
                        "checksumSha256Hex":"sha256:3333333333333333333333333333333333333333333333333333333333333333",
                        "uploadedAtEpochMs":1700000001000
                    }"#,
                ))
                .expect("mark part request should be built"),
        )
        .await
        .expect("mark part request should be handled");
    assert_eq!(response.status(), StatusCode::OK);

    let progress: (i64, i64) = sqlx::query_as(
        "SELECT uploaded_parts_count, uploaded_bytes
         FROM dr_drive_upload_item
         WHERE id='upload-item-uploader-part'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload item progress should be queryable");
    assert_eq!(progress, (1, 8));
}

#[tokio::test]
async fn uploader_upload_session_complete_updates_upload_item_and_sensitive_operation_log() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-uploader-complete', 's3_compatible', 'Uploader Complete S3',
            $1, 'us-east-1', 'bucket-s3', 1, 0,
            'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD',
            'active', 1, 'admin-uploader', 'admin-uploader'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let prepare_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token_for_organization(
                            "tenant-uploader-complete",
                            "user-uploader-complete",
                            "org-uploader-complete",
                            "drive-pc",
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token_for_organization(
                        "tenant-uploader-complete",
                        "user-uploader-complete",
                        "org-uploader-complete",
                        "drive-pc",
                    ),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/uploader/uploads")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"upload-item-complete",
                        "taskId":"task-uploader-complete",
                        "appResourceType":"desktop-file-browser",
                        "appResourceId":"root",
                        "scene":"drive_pc_file_upload",
                        "source":"pc_local_file",
                        "uploadProfileCode":"document",
                        "fileFingerprint":"sha256:fingerprint-uploader-complete",
                        "originalFileName":"contract.pdf",
                        "contentType":"application/pdf",
                        "contentLength":12,
                        "chunkSizeBytes":12,
                        "nowEpochMs":1700000000000
                    }"#,
                ))
                .expect("uploader prepare request should be built"),
        )
        .await
        .expect("uploader prepare request should be handled");
    assert_eq!(prepare_response.status(), StatusCode::CREATED);
    let prepare_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(prepare_response.into_body(), usize::MAX)
            .await
            .expect("uploader prepare response should be read"),
    )
    .expect("uploader prepare response should be valid json");
    let upload_session_id = common::envelope_data(&prepare_payload)["uploadSession"]["id"]
        .as_str()
        .expect("prepare response should contain upload session id")
        .to_string();
    assert_eq!(upload_session_id, "session-upload-item-complete");

    let complete_response = app
        .oneshot(
            Request::builder()
            .header(
                "authorization",
                format!(
                    "Bearer {}",
                    common::auth_token_for_organization(
                        "tenant-uploader-complete",
                        "user-uploader-complete",
                        "org-uploader-complete",
                        "drive-pc",
                    )
                ),
            )
            .header(
                "access-token",
                common::access_token_for_organization(
                    "tenant-uploader-complete",
                    "user-uploader-complete",
                    "org-uploader-complete",
                    "drive-pc",
                ),
            )
                .method(Method::POST)
                .uri(format!(
                    "/app/v3/api/drive/upload_sessions/{upload_session_id}/complete"
                ))
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "uploadId":"mock-s3-upload-id",
                        "contentType":"application/pdf",
                        "contentLength":12,
                        "checksumSha256Hex":"sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                        "parts":[{"partNo":1,"etag":"etag-uploader-complete-1"}]
                    }"#,
                ))
                .expect("complete uploader upload request should be built"),
        )
        .await
        .expect("complete uploader upload request should be handled");
    let complete_status = complete_response.status();
    let complete_body_bytes = to_bytes(complete_response.into_body(), usize::MAX)
        .await
        .expect("complete uploader upload response should be read");
    assert_eq!(
        complete_status,
        StatusCode::OK,
        "complete uploader upload response body: {}",
        String::from_utf8_lossy(&complete_body_bytes)
    );

    let item: (String, String, i64, i64) = sqlx::query_as(
        "SELECT status, checksum_sha256_hex, uploaded_bytes, uploaded_parts_count
         FROM dr_drive_upload_item
         WHERE tenant_id='tenant-uploader-complete'
           AND id='upload-item-complete'",
    )
    .fetch_one(&pool)
    .await
    .expect("completed upload item should be readable");
    assert_eq!(
        item,
        (
            "completed".to_string(),
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string(),
            12,
            1
        )
    );

    let storage_object: (String, String, String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT id, bucket, object_key, scene, source
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-uploader-complete'
           AND node_id='node-upload-item-complete'",
    )
    .fetch_one(&pool)
    .await
    .expect("completed storage object should be readable");
    assert_eq!(storage_object.3.as_deref(), Some("drive_pc_file_upload"));
    assert_eq!(storage_object.4.as_deref(), Some("pc_local_file"));

    let node_version: NodeVersionUsageContextRow = sqlx::query_as(
        "SELECT storage_object_id, version_no, app_id, app_resource_type, app_resource_id, scene, source
         FROM dr_drive_node_version
         WHERE tenant_id='tenant-uploader-complete'
           AND node_id='node-upload-item-complete'",
    )
    .fetch_one(&pool)
    .await
    .expect("completed logical node version should be readable");
    assert_eq!(node_version.0, storage_object.0);
    assert_eq!(node_version.1, 1);
    assert_eq!(node_version.2.as_deref(), Some("drive-pc"));
    assert_eq!(node_version.3.as_deref(), Some("desktop-file-browser"));
    assert_eq!(node_version.4.as_deref(), Some("root"));
    assert_eq!(node_version.5.as_deref(), Some("drive_pc_file_upload"));
    assert_eq!(node_version.6.as_deref(), Some("pc_local_file"));

    let node_usage_context: (Option<String>, Option<String>) = sqlx::query_as(
        "SELECT scene, source
         FROM dr_drive_node
         WHERE tenant_id='tenant-uploader-complete'
           AND id='node-upload-item-complete'",
    )
    .fetch_one(&pool)
    .await
    .expect("completed uploader node usage context should be readable");
    assert_eq!(
        node_usage_context.0.as_deref(),
        Some("drive_pc_file_upload")
    );
    assert_eq!(node_usage_context.1.as_deref(), Some("pc_local_file"));

    let operation: SensitiveOperationAuditRow = sqlx::query_as(
        "SELECT operation_type, operation_reason, object_delete_status,
                storage_object_id, upload_item_id, content_length,
                checksum_sha256_hex, object_bucket, object_key, operator_id,
                before_lifecycle_status, after_lifecycle_status
         FROM dr_drive_file_sensitive_operation
         WHERE tenant_id='tenant-uploader-complete'
           AND upload_item_id='upload-item-complete'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload completion sensitive operation should be recorded");
    assert_eq!(
        operation,
        (
            "upload_completed".to_string(),
            "user_request".to_string(),
            "not_required".to_string(),
            storage_object.0,
            "upload-item-complete".to_string(),
            12,
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string(),
            storage_object.1,
            storage_object.2,
            "user-uploader-complete".to_string(),
            Some("uploading".to_string()),
            Some("active".to_string())
        )
    );

    assert!(
        captured_requests
            .lock()
            .expect("captured s3 requests should be readable")
            .iter()
            .any(|request| request.method == "POST"
                && request.query.contains("uploadId=mock-s3-upload-id")),
        "complete should call S3 CompleteMultipartUpload"
    );
}

#[tokio::test]
async fn uploader_routes_accept_generated_sdk_int64_strings() {
    let (s3_endpoint, _captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-uploader-string-int', 's3_compatible', 'Uploader String Int S3',
            $1, 'us-east-1', 'bucket-s3', 1, 0,
            'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD',
            'active', 1, 'admin-uploader', 'admin-uploader'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let prepare_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-uploader-string-int", "drive-pc")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-uploader-string-int", "drive-pc"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/uploader/uploads")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"upload-item-string-int",
                        "taskId":"task-uploader-string-int",
                        "appResourceType":"desktop-file-browser",
                        "appResourceId":"root",
                        "scene":"drive_pc_file_upload",
                        "source":"pc_local_file",
                        "uploadProfileCode":"generic",
                        "fileFingerprint":"sha256:fingerprint-uploader-string-int",
                        "originalFileName":"string-int.txt",
                        "contentType":"text/plain",
                        "contentLength":"5",
                        "chunkSizeBytes":"5",
                        "retention":{
                            "mode":"temporary",
                            "ttlSeconds":"86400",
                            "cleanupAction":"soft_delete",
                            "hardDeleteAfterSeconds":"172800"
                        },
                        "nowEpochMs":"1700000000000"
                    }"#,
                ))
                .expect("uploader prepare request should be built"),
        )
        .await
        .expect("uploader prepare request should be handled");
    assert_eq!(prepare_response.status(), StatusCode::CREATED);

    let mark_part_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-uploader-string-int", "drive-pc")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-uploader-string-int", "drive-pc"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/uploader/uploads/upload-item-string-int/parts/1")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "uploadSessionId":"session-upload-item-string-int",
                        "offsetBytes":"0",
                        "sizeBytes":"5",
                        "etag":"etag-string-int-1",
                        "uploadedAtEpochMs":"1700000001000"
                    }"#,
                ))
                .expect("mark uploader part request should be built"),
        )
        .await
        .expect("mark uploader part request should be handled");
    assert_eq!(mark_part_response.status(), StatusCode::OK);

    let complete_response = app
        .oneshot(
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-001", "user-uploader-string-int", "drive-pc")),
            )
            .header("access-token", common::access_token("tenant-001", "user-uploader-string-int", "drive-pc"))
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions/session-upload-item-string-int/complete")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "uploadId":"mock-s3-upload-id",
                        "contentType":"text/plain",
                        "contentLength":"5",
                        "checksumSha256Hex":"sha256:1111111111111111111111111111111111111111111111111111111111111111",
                        "parts":[{"partNo":1,"etag":"etag-string-int-1"}]
                    }"#,
                ))
                .expect("complete uploader request should be built"),
        )
        .await
        .expect("complete uploader request should be handled");
    assert_eq!(complete_response.status(), StatusCode::OK);

    let item: (i64, i64, String) = sqlx::query_as(
        "SELECT content_length, chunk_size_bytes, status
         FROM dr_drive_upload_item
         WHERE id='upload-item-string-int'",
    )
    .fetch_one(&pool)
    .await
    .expect("string int upload item should be readable");
    assert_eq!(item, (5, 5, "completed".to_string()));
}

#[tokio::test]
async fn create_upload_session_for_existing_file_uses_next_storage_version_in_object_key() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-version-upload', 'tenant-version-upload', 'user',
            'user-version-upload', 'personal', 'Version Upload', 'active', 1,
            'user-version-upload', 'user-version-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-version-upload', 'tenant-version-upload', 'space-version-upload',
            NULL, 'file', 'versioned.bin', 'ready', 'active', 1,
            'user-version-upload', 'user-version-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-version-upload",
        "Version Upload S3",
        &s3_endpoint,
        "bucket-version-upload",
        "active",
        "user-version-upload",
    )
    .await;
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "obj-version-upload-v1",
            tenant_id: "tenant-version-upload",
            node_id: "node-version-upload",
            version_no: 1,
            provider_id: "provider-version-upload",
            bucket: "bucket-version-upload",
            object_key: "sdkwork-drive/v1/t/aa/tenants/tenant-version-upload/spaces/space-version-upload/nodes/n/bb/node-version-upload/versions/0000000001/obj-version-upload-v1/content",
            content_type: "application/octet-stream",
            content_length: 8,
            checksum_sha256_hex: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-version-upload",
        },
    )
    .await
    .expect("existing version should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-version-upload",
                            "user-version-upload",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-version-upload", "user-version-upload", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sessionId":"upload-version-v2",
                        "spaceId":"space-version-upload",
                        "nodeId":"node-version-upload",
                        "bucket":"bucket-version-upload",
                        "idempotencyKey":"idem-version-v2",
                        "expiresAtEpochMs":4102444800000
                    }"#,
                ))
                .expect("create v2 upload session request should be built"),
        )
        .await
        .expect("create v2 upload session request should be handled");
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("create v2 response should be read"),
    )
    .expect("create v2 response should be valid json");
    let created_object_key = common::envelope_body(&create_payload)["objectKey"]
        .as_str()
        .expect("create v2 response should include objectKey");
    assert_rooted_standard_storage_object_key(
        created_object_key,
        "sdkwork-drive/v1/tenants/tenant-version-upload/spaces/space-version-upload",
        "tenant-version-upload",
        "space-version-upload",
        "node-version-upload",
    );
    assert!(
        created_object_key.contains("/versions/0000000002/"),
        "second upload session object key must target the next Drive version: {created_object_key}"
    );

    let complete_response = app
        .oneshot(
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-version-upload", "user-version-upload", "appbase")),
            )
            .header("access-token", common::access_token("tenant-version-upload", "user-version-upload", "appbase"))
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions/upload-version-v2/complete")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "uploadId":"mock-s3-upload-id",
                        "contentType":"application/octet-stream",
                        "contentLength":16,
                        "checksumSha256Hex":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                        "parts":[{"partNo":1,"etag":"etag-version-v2"}]
                    }"#,
                ))
                .expect("complete v2 upload request should be built"),
        )
        .await
        .expect("complete v2 upload request should be handled");
    assert_eq!(complete_response.status(), StatusCode::OK);

    let stored_object_key: String = sqlx::query_scalar(
        "SELECT object_key
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-version-upload'
           AND node_id='node-version-upload'
           AND version_no=2",
    )
    .fetch_one(&pool)
    .await
    .expect("second storage version should be readable");
    assert_eq!(stored_object_key, created_object_key);
    assert!(stored_object_key.contains("/versions/0000000002/"));

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(requests.iter().any(|request| {
        request.method == "POST"
            && request
                .path
                .contains("/node-version-upload/versions/0000000002/")
            && request.path.ends_with("/upload-version-v2/content")
            && request.query.contains("uploads")
    }));
    assert!(requests.iter().any(|request| {
        request.method == "POST"
            && request
                .path
                .contains("/node-version-upload/versions/0000000002/")
            && request.path.ends_with("/upload-version-v2/content")
            && request.query.contains("uploadId=mock-s3-upload-id")
    }));
}

#[tokio::test]
async fn presign_upload_part_rejects_ttl_outside_contract_before_object_store_call() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-upload-ttl', 'tenant-upload-ttl', 'user', 'user-upload-ttl', 'personal', 'Upload TTL', 'active', 1, 'user-upload-ttl', 'user-upload-ttl')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-upload-ttl', 'tenant-upload-ttl', 'space-upload-ttl', NULL, 'file', 'upload.bin', 'uploading', 'active', 1, 'user-upload-ttl', 'user-upload-ttl')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-upload-ttl', 's3_compatible', 'TTL S3', $1, 'us-east-1',
            'bucket-upload-ttl', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-upload-ttl', 'admin-upload-ttl'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'upload-ttl', 'tenant-upload-ttl', 'space-upload-ttl', 'node-upload-ttl',
            'bucket-upload-ttl',
            'sdkwork-drive/v1/t/aa/tenants/tenant-upload-ttl/spaces/space-upload-ttl/nodes/n/bb/node-upload-ttl/versions/0000000001/upload-ttl/content',
            'idem-upload-ttl', 'provider-upload-ttl', 'upload-id-ttl', 'created',
            1800000000000, 1, 'user-upload-ttl', 'user-upload-ttl'
        )",
    )
    .execute(&pool)
    .await
    .expect("upload session should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-upload-ttl", "user-upload-ttl", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-upload-ttl", "user-upload-ttl", "appbase"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/upload_sessions/upload-ttl/parts/1")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "requestedTtlSeconds":0
                    }"#,
                ))
                .expect("presign request should be built"),
        )
        .await
        .expect("presign request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("error body should be readable"),
    )
    .expect("error body should be valid json");
    assert_eq!(payload["code"], 40001);
    assert!(payload["detail"]
        .as_str()
        .expect("detail should be present")
        .contains("requestedTtlSeconds"));
    assert!(
        captured_requests
            .lock()
            .expect("captured s3 requests mutex should not be poisoned")
            .is_empty(),
        "invalid TTL should fail before calling object storage"
    );
}

#[tokio::test]
async fn upload_session_presign_rejects_when_persisted_provider_is_disabled() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-provider-stickiness', 'tenant-provider-stickiness', 'user', 'user-provider-stickiness', 'personal', 'Provider Stickiness', 'active', 1, 'user-provider-stickiness', 'user-provider-stickiness')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-provider-stickiness', 'tenant-provider-stickiness', 'space-provider-stickiness', NULL, 'file', 'data.bin', 'uploading', 'active', 1, 'user-provider-stickiness', 'user-provider-stickiness')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-original', 's3_compatible', 'Original S3', $1, 'us-east-1',
            'bucket-s3', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-s3', 'admin-s3'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("original storage provider should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-provider-stickiness",
                            "user-provider-stickiness",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token(
                        "tenant-provider-stickiness",
                        "user-provider-stickiness",
                        "appbase",
                    ),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sessionId":"upload-provider-stickiness",
                        "spaceId":"space-provider-stickiness",
                        "nodeId":"node-provider-stickiness",
                        "bucket":"bucket-s3",
                        "idempotencyKey":"idem-provider-stickiness",
                        "expiresAtEpochMs":1800000000000
                    }"#,
                ))
                .expect("create upload session request should be built"),
        )
        .await
        .expect("create upload session should be handled");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    sqlx::query(
        "UPDATE dr_drive_storage_provider
         SET status='disabled', updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE id='provider-original'",
    )
    .execute(&pool)
    .await
    .expect("original provider should be disabled");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-replacement', 'local_filesystem', 'Replacement Local',
            'file:///tmp/sdkwork-drive', NULL, 'bucket-s3', 1, NULL, NULL, NULL,
            'active', 1, 'admin-s3', 'admin-s3'
        )",
    )
    .execute(&pool)
    .await
    .expect("replacement provider should be seeded");

    let part_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-provider-stickiness",
                            "user-provider-stickiness",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token(
                        "tenant-provider-stickiness",
                        "user-provider-stickiness",
                        "appbase",
                    ),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/upload_sessions/upload-provider-stickiness/parts/1")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "requestedTtlSeconds":120
                    }"#,
                ))
                .expect("presign upload part request should be built"),
        )
        .await
        .expect("presign upload part should be handled");
    assert_eq!(part_response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(part_response.into_body(), usize::MAX)
            .await
            .expect("presign error body should be readable"),
    )
    .expect("presign error body should be valid json");
    assert_eq!(payload["code"], 40901);
    assert!(payload["detail"]
        .as_str()
        .expect("detail should be present")
        .contains("active storage provider"));

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(
        requests
            .iter()
            .any(|request| request.method == "POST" && request.query.contains("uploads")),
        "create upload should use original provider"
    );
    assert!(
        !requests.iter().any(|request| request.method == "PUT"),
        "presign should not use a replacement provider after the bound provider is disabled"
    );
}

#[tokio::test]
async fn complete_upload_session_allows_only_one_in_flight_completion() {
    let (s3_endpoint, captured_requests) =
        start_s3_mock_server_with_complete_delay(Duration::from_millis(100)).await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-complete-race', 'tenant-complete-race', 'user', 'user-race', 'personal', 'Complete Race', 'active', 1, 'user-race', 'user-race')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-complete-race', 'tenant-complete-race', 'space-complete-race', NULL, 'file', 'race.bin', 'uploading', 'active', 1, 'user-race', 'user-race')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-complete-race', 's3_compatible', 'Mock S3', $1, 'us-east-1',
            'bucket-s3', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-race', 'admin-race'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");
    let upload_object_key = standard_storage_object_key(
        "tenant-complete-race",
        "space-complete-race",
        "node-complete-race",
        1,
        "upload-complete-race",
    );
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'upload-complete-race', 'tenant-complete-race', 'space-complete-race',
            'node-complete-race', 'bucket-s3', $1,
            'idem-complete-race', 'provider-complete-race', 'mock-s3-upload-id',
            'uploading', 4102444800000, 1, 'user-race', 'user-race'
        )",
    )
    .bind(&upload_object_key)
    .execute(&pool)
    .await
    .expect("upload session should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let request_body = r#"{
                "uploadId":"mock-s3-upload-id",
        "contentType":"application/octet-stream",
        "contentLength":4,
        "checksumSha256Hex":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "parts":[{"partNo":1,"etag":"etag-race-1"}]
    }"#;
    let first = app.clone().oneshot(
        Request::builder()
            .header(
                "authorization",
                format!(
                    "Bearer {}",
                    common::auth_token("tenant-complete-race", "user-race", "appbase")
                ),
            )
            .header(
                "access-token",
                common::access_token("tenant-complete-race", "user-race", "appbase"),
            )
            .method(Method::POST)
            .uri("/app/v3/api/drive/upload_sessions/upload-complete-race/complete")
            .header("content-type", "application/json")
            .body(Body::from(request_body))
            .expect("first complete upload request should be built"),
    );
    let second = app.oneshot(
        Request::builder()
            .header(
                "authorization",
                format!(
                    "Bearer {}",
                    common::auth_token("tenant-complete-race", "user-race", "appbase")
                ),
            )
            .header(
                "access-token",
                common::access_token("tenant-complete-race", "user-race", "appbase"),
            )
            .method(Method::POST)
            .uri("/app/v3/api/drive/upload_sessions/upload-complete-race/complete")
            .header("content-type", "application/json")
            .body(Body::from(request_body))
            .expect("second complete upload request should be built"),
    );

    let (first_response, second_response) = tokio::join!(first, second);
    let statuses = [
        first_response
            .expect("first complete request should be handled")
            .status(),
        second_response
            .expect("second complete request should be handled")
            .status(),
    ];
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == StatusCode::OK)
            .count(),
        1,
        "exactly one concurrent completion should succeed"
    );
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == StatusCode::CONFLICT)
            .count(),
        1,
        "the losing concurrent completion should fail before object store completion"
    );

    let complete_calls = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .iter()
        .filter(|request| {
            request.method == "POST" && request.query.contains("uploadId=mock-s3-upload-id")
        })
        .count();
    assert_eq!(
        complete_calls, 1,
        "only the winning request may call S3 CompleteMultipartUpload"
    );

    let object_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-complete-race'
           AND node_id='node-complete-race'",
    )
    .fetch_one(&pool)
    .await
    .expect("completed storage object count should be queryable");
    assert_eq!(object_count, 1);
}

#[tokio::test]
async fn complete_upload_session_rejects_metadata_conflict_before_object_store_complete() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-complete-metadata-conflict', 'tenant-complete-metadata-conflict',
            'user', 'user-complete', 'personal', 'Complete Metadata Conflict',
            'active', 1, 'user-complete', 'user-complete'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    for (id, name) in [
        ("node-complete-metadata-conflict", "target.bin"),
        ("node-complete-metadata-existing", "existing.bin"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES (
                $1, 'tenant-complete-metadata-conflict',
                'space-complete-metadata-conflict', NULL, 'file', $2,
                'uploading', 'active', 1, 'user-complete', 'user-complete'
            )",
        )
        .bind(id)
        .bind(name)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }
    seed_s3_provider_fixture(
        &pool,
        "provider-complete-metadata-conflict",
        "Complete Metadata Conflict S3",
        &s3_endpoint,
        "bucket-complete-metadata-conflict",
        "active",
        "user-complete",
    )
    .await;
    let upload_object_key = standard_storage_object_key(
        "tenant-complete-metadata-conflict",
        "space-complete-metadata-conflict",
        "node-complete-metadata-conflict",
        1,
        "upload-metadata-conflict",
    );
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'upload-metadata-conflict', 'tenant-complete-metadata-conflict',
            'space-complete-metadata-conflict', 'node-complete-metadata-conflict',
            'bucket-complete-metadata-conflict', $1,
            'idem-upload-metadata-conflict', 'provider-complete-metadata-conflict',
            'mock-s3-upload-id', 'uploading', 4102444800000, 1,
            'user-complete', 'user-complete'
        )",
    )
    .bind(&upload_object_key)
    .execute(&pool)
    .await
    .expect("upload session should be seeded");
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "upload-metadata-conflict-v1",
            tenant_id: "tenant-complete-metadata-conflict",
            node_id: "node-complete-metadata-existing",
            version_no: 1,
            provider_id: "provider-complete-metadata-conflict",
            bucket: "bucket-complete-metadata-conflict",
            object_key: "objects/existing-metadata-conflict.bin",
            content_type: "application/octet-stream",
            content_length: 1,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-complete",
        },
    )
    .await
    .expect("conflicting storage object id should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-complete-metadata-conflict", "user-complete", "appbase")),
            )
            .header("access-token", common::access_token("tenant-complete-metadata-conflict", "user-complete", "appbase"))
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions/upload-metadata-conflict/complete")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "uploadId":"mock-s3-upload-id",
                        "contentType":"application/octet-stream",
                        "contentLength":4,
                        "checksumSha256Hex":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                        "parts":[{"partNo":1,"etag":"etag-complete-1"}]
                    }"#,
                ))
                .expect("complete upload request should be built"),
        )
        .await
        .expect("complete upload request should be handled");

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("conflict response body should be read"),
    )
    .expect("conflict response json should be valid");
    assert_eq!(payload["code"].as_i64(), Some(40901));

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(!requests.iter().any(|request| {
        request.method == "POST" && request.query.contains("uploadId=mock-s3-upload-id")
    }));

    let session_state: String = sqlx::query_scalar(
        "SELECT state
         FROM dr_drive_upload_session
         WHERE tenant_id='tenant-complete-metadata-conflict'
           AND id='upload-metadata-conflict'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload session state should be readable");
    assert_eq!(session_state, "uploading");

    let completed_object_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-complete-metadata-conflict'
           AND node_id='node-complete-metadata-conflict'",
    )
    .fetch_one(&pool)
    .await
    .expect("completed object count should be readable");
    assert_eq!(completed_object_count, 0);
}

#[tokio::test]
async fn complete_upload_session_rejects_storage_version_drift_before_object_store_complete() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-complete-version-drift', 'tenant-complete-version-drift',
            'user', 'user-complete-version-drift', 'personal',
            'Complete Version Drift', 'active', 1,
            'user-complete-version-drift', 'user-complete-version-drift'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-complete-version-drift', 'tenant-complete-version-drift',
            'space-complete-version-drift', NULL, 'file', 'version-drift.bin',
            'uploading', 'active', 1,
            'user-complete-version-drift', 'user-complete-version-drift'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-complete-version-drift",
        "Complete Version Drift S3",
        &s3_endpoint,
        "bucket-complete-version-drift",
        "active",
        "user-complete-version-drift",
    )
    .await;
    let existing_v1_object_key = standard_storage_object_key(
        "tenant-complete-version-drift",
        "space-complete-version-drift",
        "node-complete-version-drift",
        1,
        "obj-complete-version-drift-v1",
    );
    let existing_v2_object_key = standard_storage_object_key(
        "tenant-complete-version-drift",
        "space-complete-version-drift",
        "node-complete-version-drift",
        2,
        "obj-complete-version-drift-v2",
    );
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "obj-complete-version-drift-v1",
            tenant_id: "tenant-complete-version-drift",
            node_id: "node-complete-version-drift",
            version_no: 1,
            provider_id: "provider-complete-version-drift",
            bucket: "bucket-complete-version-drift",
            object_key: &existing_v1_object_key,
            content_type: "application/octet-stream",
            content_length: 1,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-complete-version-drift",
        },
    )
    .await
    .expect("existing v1 should be seeded");
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "obj-complete-version-drift-v2",
            tenant_id: "tenant-complete-version-drift",
            node_id: "node-complete-version-drift",
            version_no: 2,
            provider_id: "provider-complete-version-drift",
            bucket: "bucket-complete-version-drift",
            object_key: &existing_v2_object_key,
            content_type: "application/octet-stream",
            content_length: 2,
            checksum_sha256_hex:
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            lifecycle_status: "active",
            actor_id: "user-complete-version-drift",
        },
    )
    .await
    .expect("existing v2 should be seeded");
    let upload_object_key = standard_storage_object_key(
        "tenant-complete-version-drift",
        "space-complete-version-drift",
        "node-complete-version-drift",
        2,
        "upload-complete-version-drift",
    );
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'upload-complete-version-drift', 'tenant-complete-version-drift',
            'space-complete-version-drift', 'node-complete-version-drift',
            'bucket-complete-version-drift', $1,
            'idem-complete-version-drift', 'provider-complete-version-drift',
            'mock-s3-upload-id', 'uploading', 4102444800000, 1,
            'user-complete-version-drift', 'user-complete-version-drift'
        )",
    )
    .bind(&upload_object_key)
    .execute(&pool)
    .await
    .expect("upload session should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-complete-version-drift", "user-complete-version-drift", "appbase")),
            )
            .header("access-token", common::access_token("tenant-complete-version-drift", "user-complete-version-drift", "appbase"))
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions/upload-complete-version-drift/complete")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "uploadId":"mock-s3-upload-id",
                        "contentType":"application/octet-stream",
                        "contentLength":3,
                        "checksumSha256Hex":"sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                        "parts":[{"partNo":1,"etag":"etag-complete-version-drift"}]
                    }"#,
                ))
                .expect("complete upload request should be built"),
        )
        .await
        .expect("complete upload request should be handled");

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("conflict response body should be read"),
    )
    .expect("conflict response json should be valid");
    assert_eq!(payload["code"].as_i64(), Some(40901));

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(
        !requests.iter().any(|request| {
            request.method == "POST" && request.query.contains("uploadId=mock-s3-upload-id")
        }),
        "version drift must be rejected before S3 CompleteMultipartUpload"
    );

    let session_state: String = sqlx::query_scalar(
        "SELECT state
         FROM dr_drive_upload_session
         WHERE tenant_id='tenant-complete-version-drift'
           AND id='upload-complete-version-drift'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload session state should be readable");
    assert_eq!(session_state, "uploading");

    let version_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-complete-version-drift'
           AND node_id='node-complete-version-drift'",
    )
    .fetch_one(&pool)
    .await
    .expect("storage object count should be readable");
    assert_eq!(version_count, 2);
}

#[tokio::test]
async fn s3_upload_session_abort_calls_object_store_abort() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-s3-abort', 'tenant-s3-abort', 'user', 'user-s3', 'personal', 'S3', 'active', 1, 'user-s3', 'user-s3')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-s3-abort', 'tenant-s3-abort', 'space-s3-abort', NULL, 'file', 'data.bin', 'uploading', 'active', 1, 'user-s3', 'user-s3')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-s3-abort', 's3_compatible', 'Mock S3', $1, 'us-east-1',
            'bucket-s3', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-s3', 'admin-s3'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-s3-abort", "user-s3", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-s3-abort", "user-s3", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sessionId":"upload-s3-abort",
                        "spaceId":"space-s3-abort",
                        "nodeId":"node-s3-abort",
                        "bucket":"bucket-s3",
                        "objectKey":"objects/node-s3/data.bin",
                        "idempotencyKey":"idem-s3-abort",
                        "expiresAtEpochMs":1800000000000
                    }"#,
                ))
                .expect("create upload session request should be built"),
        )
        .await
        .expect("create upload session should be handled");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let abort_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-s3-abort", "user-s3", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-s3-abort", "user-s3", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions/upload-s3-abort/abort")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("abort upload request should be built"),
        )
        .await
        .expect("abort upload should be handled");
    assert_eq!(abort_response.status(), StatusCode::OK);
    let abort_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(abort_response.into_body(), usize::MAX)
            .await
            .expect("abort response body should be read"),
    )
    .expect("abort response json should be valid");
    assert_eq!(common::envelope_body(&abort_payload)["state"], "aborted");

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(
        requests.iter().any(|request| request.method == "DELETE"
            && request
                .path
                .starts_with("/bucket-s3/sdkwork-drive/v1/tenants/tenant-s3-abort/spaces/space-s3-abort/sdkwork-drive/v1/t/")
            && request
                .path
                .contains("/tenants/tenant-s3-abort/spaces/space-s3-abort/")
            && request.path.contains("/node-s3-abort/versions/")
            && request.path.ends_with("/upload-s3-abort/content")
            && request.query.contains("uploadId=mock-s3-upload-id")),
        "abort upload session should call S3 AbortMultipartUpload"
    );
}

#[tokio::test]
async fn app_drive_file_lifecycle_routes_get_move_copy_upload_complete_download_and_delete() {
    let (s3_endpoint, _captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-life', 'tenant-life', 'user', 'user-life', 'personal', 'Lifecycle', 'active', 1, 'user-life', 'user-life')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    for (id, parent_id, node_type, node_name, content_state) in [
        (
            "folder-destination",
            Option::<&str>::None,
            "folder",
            "Destination",
            "empty",
        ),
        (
            "node-file",
            Option::<&str>::None,
            "file",
            "report.txt",
            "uploading",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-life', 'space-life', $2, $3, $4, $5, 'active', 1, 'user-life', 'user-life')",
        )
        .bind(id)
        .bind(parent_id)
        .bind(node_type)
        .bind(node_name)
        .bind(content_state)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-life', 's3_compatible', 'Mock S3', $1, 'us-east-1',
            'bucket-life', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-life', 'admin-life'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let create_session_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-life", "user-life", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-life", "user-life", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sessionId":"upload-life",
                        "spaceId":"space-life",
                        "nodeId":"node-file",
                        "bucket":"bucket-life",
                        "objectKey":"objects/node-file/report.txt",
                        "idempotencyKey":"idem-life",
                        "expiresAtEpochMs":1800000000000
                    }"#,
                ))
                .expect("create upload session request should be built"),
        )
        .await
        .expect("create upload session should be handled");
    assert_eq!(create_session_response.status(), StatusCode::CREATED);

    let upload_part_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-life", "user-life", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-life", "user-life", "appbase"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/upload_sessions/upload-life/parts/1")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "requestedTtlSeconds":120
                    }"#,
                ))
                .expect("upload part request should be built"),
        )
        .await
        .expect("upload part request should be handled");
    assert_eq!(upload_part_response.status(), StatusCode::OK);
    let upload_part_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(upload_part_response.into_body(), usize::MAX)
            .await
            .expect("upload part response body should be read"),
    )
    .expect("upload part response json should be valid");
    assert_eq!(common::envelope_data(&upload_part_payload)["method"], "PUT");
    assert_eq!(common::envelope_data(&upload_part_payload)["partNo"], 1);
    assert!(common::envelope_data(&upload_part_payload)["uploadUrl"]
        .as_str()
        .expect("uploadUrl should be present")
        .contains("sdkwork-drive/v1/t/"));

    let complete_response = app
        .clone()
        .oneshot(
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-life", "user-life", "appbase")),
            )
            .header("access-token", common::access_token("tenant-life", "user-life", "appbase"))
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions/upload-life/complete")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "contentType":"text/plain",
                        "contentLength":12,
                        "checksumSha256Hex":"sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                        "parts":[{"partNo":1,"etag":"etag-life-1"}]
                    }"#,
                ))
                .expect("complete upload request should be built"),
        )
        .await
        .expect("complete upload request should be handled");
    assert_eq!(complete_response.status(), StatusCode::OK);
    let complete_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(complete_response.into_body(), usize::MAX)
            .await
            .expect("complete response body should be read"),
    )
    .expect("complete response json should be valid");
    assert_eq!(
        common::envelope_body(&complete_payload)["state"],
        "completed"
    );

    let stored_object_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-life'
           AND node_id='node-file'
           AND bucket='bucket-life'
           AND object_key LIKE 'sdkwork-drive/v1/tenants/tenant-life/spaces/space-life/sdkwork-drive/v1/t/%/tenants/tenant-life/spaces/space-life/nodes/n/%/node-file/versions/0000000001/upload-life/content'
           AND content_type='text/plain'
           AND content_length=12
           AND checksum_sha256_hex='sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc'
           AND lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("completed upload should create storage object metadata");
    assert_eq!(stored_object_count, 1);

    let detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-life", "user-life", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-life", "user-life", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-file")
                .body(Body::empty())
                .expect("get node request should be built"),
        )
        .await
        .expect("get node request should be handled");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(detail_response.into_body(), usize::MAX)
            .await
            .expect("detail response body should be read"),
    )
    .expect("detail response json should be valid");
    assert_eq!(common::envelope_item(&detail_payload)["id"], "node-file");
    assert_eq!(common::envelope_item(&detail_payload)["nodeType"], "file");

    let node_download_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-life", "user-life", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-life", "user-life", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-file/download_url?requestedTtlSeconds=120")
                .body(Body::empty())
                .expect("node download url request should be built"),
        )
        .await
        .expect("node download url request should be handled");
    assert_eq!(node_download_response.status(), StatusCode::OK);
    let node_download_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(node_download_response.into_body(), usize::MAX)
            .await
            .expect("node download response body should be read"),
    )
    .expect("node download response json should be valid");
    assert!(common::envelope_data(&node_download_payload)["downloadUrl"]
        .as_str()
        .expect("downloadUrl should be present")
        .contains("/download_tokens/"));

    let move_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-life", "user-life", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-life", "user-life", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-file/move")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "targetParentNodeId":"folder-destination"
                    }"#,
                ))
                .expect("move node request should be built"),
        )
        .await
        .expect("move node request should be handled");
    assert_eq!(move_response.status(), StatusCode::OK);
    let move_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(move_response.into_body(), usize::MAX)
            .await
            .expect("move response body should be read"),
    )
    .expect("move response json should be valid");
    assert_eq!(
        common::envelope_item(&move_payload)["parentNodeId"],
        "folder-destination"
    );

    let copy_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-life", "user-life", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-life", "user-life", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-file/copy")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"node-file-copy",
                        "nodeName":"report-copy.txt"
                    }"#,
                ))
                .expect("copy node request should be built"),
        )
        .await
        .expect("copy node request should be handled");
    assert_eq!(copy_response.status(), StatusCode::CREATED);
    let copy_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(copy_response.into_body(), usize::MAX)
            .await
            .expect("copy response body should be read"),
    )
    .expect("copy response json should be valid");
    assert_eq!(common::envelope_item(&copy_payload)["id"], "node-file-copy");
    assert_eq!(common::envelope_item(&copy_payload)["nodeType"], "file");

    let copied_object_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-life'
           AND node_id='node-file-copy'
           AND bucket='bucket-life'
           AND object_key LIKE 'sdkwork-drive/v1/tenants/tenant-life/spaces/space-life/sdkwork-drive/v1/t/%/tenants/tenant-life/spaces/space-life/nodes/n/%/node-file/versions/0000000001/upload-life/content'
           AND lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("copy should duplicate active storage object metadata");
    assert_eq!(copied_object_count, 1);

    let copied_version: (String, i64, String, String) = sqlx::query_as(
        "SELECT v.storage_object_id, v.version_no, v.version_kind, v.change_source
         FROM dr_drive_node_version v
         INNER JOIN dr_drive_storage_object o ON o.id=v.storage_object_id
         WHERE v.tenant_id='tenant-life'
           AND v.node_id='node-file-copy'
           AND o.lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("copy should create logical node version metadata");
    assert!(copied_version.0.starts_with("node-file-copy-copy-"));
    assert_eq!(copied_version.1, 1);
    assert_eq!(copied_version.2, "auto");
    assert_eq!(copied_version.3, "app_api");

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-life", "user-life", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-life", "user-life", "appbase"),
                )
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/nodes/node-file")
                .body(Body::empty())
                .expect("delete node request should be built"),
        )
        .await
        .expect("delete node request should be handled");
    common::assert_no_content_response(delete_response).await;

    let deleted_detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-life", "user-life", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-life", "user-life", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-file")
                .body(Body::empty())
                .expect("get deleted node request should be built"),
        )
        .await
        .expect("get deleted node request should be handled");
    assert_eq!(deleted_detail_response.status(), StatusCode::NOT_FOUND);

    let changes_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-life", "user-life", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-life", "user-life", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/changes?spaceId=space-life")
                .body(Body::empty())
                .expect("changes request should be built"),
        )
        .await
        .expect("changes request should be handled");
    assert_eq!(changes_response.status(), StatusCode::OK);
    let changes_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(changes_response.into_body(), usize::MAX)
            .await
            .expect("changes response body should be read"),
    )
    .expect("changes response json should be valid");
    let events = common::envelope_items(&changes_payload)
        .as_array()
        .expect("changes items should be an array")
        .iter()
        .map(|item| item["eventType"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    for expected_event in [
        "drive.node.version.committed.v1",
        "drive.node.path.changed.v1",
        "drive.node.copied",
        "drive.node.deleted.v1",
    ] {
        assert!(
            events.contains(&expected_event.to_string()),
            "changes should include {expected_event}"
        );
    }
}

#[tokio::test]
async fn app_drive_delete_folder_recursively_deletes_descendants_and_storage_metadata() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-tree-delete', 'tenant-tree-delete', 'user', 'user-tree-delete', 'personal', 'Tree Delete', 'active', 1, 'user-tree-delete', 'user-tree-delete')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    for (id, parent_id, node_type, node_name, content_state) in [
        (
            "folder-tree-root",
            Option::<&str>::None,
            "folder",
            "Root Folder",
            "empty",
        ),
        (
            "folder-tree-child",
            Some("folder-tree-root"),
            "folder",
            "Child Folder",
            "empty",
        ),
        (
            "file-tree-child",
            Some("folder-tree-root"),
            "file",
            "child.txt",
            "ready",
        ),
        (
            "file-tree-grandchild",
            Some("folder-tree-child"),
            "file",
            "grandchild.txt",
            "ready",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-tree-delete', 'space-tree-delete', $2, $3, $4, $5, 'active', 1, 'user-tree-delete', 'user-tree-delete')",
        )
        .bind(id)
        .bind(parent_id)
        .bind(node_type)
        .bind(node_name)
        .bind(content_state)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }
    seed_storage_metadata_provider_fixture(
        &pool,
        "provider-tree-delete",
        "bucket-tree-delete",
        "user-tree-delete",
    )
    .await;
    for (object_id, node_id, object_key) in [
        (
            "obj-tree-child",
            "file-tree-child",
            "objects/tree/child.txt",
        ),
        (
            "obj-tree-grandchild",
            "file-tree-grandchild",
            "objects/tree/grandchild.txt",
        ),
    ] {
        seed_storage_object_fixture(
            &pool,
            StorageObjectFixture {
                object_id,
                tenant_id: "tenant-tree-delete",
                node_id,
                version_no: 1,
                provider_id: "provider-tree-delete",
                bucket: "bucket-tree-delete",
                object_key,
                content_type: "text/plain",
                content_length: 12,
                checksum_sha256_hex:
                    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                lifecycle_status: "active",
                actor_id: "user-tree-delete",
            },
        )
        .await
        .expect("storage object should be seeded");
    }

    let app = common::test_router_with_pool(pool.clone());
    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-tree-delete", "user-tree-delete", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-tree-delete", "user-tree-delete", "appbase"),
                )
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/nodes/folder-tree-root")
                .body(Body::empty())
                .expect("delete folder request should be built"),
        )
        .await
        .expect("delete folder request should be handled");
    common::assert_no_content_response(delete_response).await;

    let deleted_node_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-tree-delete'
           AND id IN (
               'folder-tree-root',
               'folder-tree-child',
               'file-tree-child',
               'file-tree-grandchild'
           )
           AND lifecycle_status='deleted'",
    )
    .fetch_one(&pool)
    .await
    .expect("deleted node count should be readable");
    assert_eq!(deleted_node_count, 4);

    let deleted_object_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-tree-delete'
           AND node_id IN ('file-tree-child', 'file-tree-grandchild')
           AND lifecycle_status='deleted'",
    )
    .fetch_one(&pool)
    .await
    .expect("deleted storage object count should be readable");
    assert_eq!(deleted_object_count, 2);

    let child_detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-tree-delete", "user-tree-delete", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-tree-delete", "user-tree-delete", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/file-tree-child")
                .body(Body::empty())
                .expect("get deleted child request should be built"),
        )
        .await
        .expect("get deleted child request should be handled");
    assert_eq!(child_detail_response.status(), StatusCode::NOT_FOUND);

    let search_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-tree-delete", "user-tree-delete", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-tree-delete", "user-tree-delete", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/search?spaceId=space-tree-delete&q=child")
                .body(Body::empty())
                .expect("search deleted child request should be built"),
        )
        .await
        .expect("search deleted child request should be handled");
    assert_eq!(search_response.status(), StatusCode::OK);
    let search_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(search_response.into_body(), usize::MAX)
            .await
            .expect("search deleted child response should be read"),
    )
    .expect("search deleted child response should be valid json");
    assert_eq!(
        common::envelope_items(&search_payload)
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

#[tokio::test]
async fn app_drive_trash_folder_recursively_trashes_descendants() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-tree-trash', 'tenant-tree-trash', 'user', 'user-tree-trash', 'personal', 'Tree Trash', 'active', 1, 'user-tree-trash', 'user-tree-trash')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    for (id, parent_id, node_type, node_name, content_state) in [
        (
            "folder-trash-root",
            Option::<&str>::None,
            "folder",
            "Root Folder",
            "empty",
        ),
        (
            "folder-trash-child",
            Some("folder-trash-root"),
            "folder",
            "Child Folder",
            "empty",
        ),
        (
            "file-trash-child",
            Some("folder-trash-root"),
            "file",
            "child.txt",
            "ready",
        ),
        (
            "file-trash-grandchild",
            Some("folder-trash-child"),
            "file",
            "grandchild.txt",
            "ready",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-tree-trash', 'space-tree-trash', $2, $3, $4, $5, 'active', 1, 'user-tree-trash', 'user-tree-trash')",
        )
        .bind(id)
        .bind(parent_id)
        .bind(node_type)
        .bind(node_name)
        .bind(content_state)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }

    let app = common::test_router_with_pool(pool.clone());
    let trash_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-tree-trash", "user-tree-trash", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-tree-trash", "user-tree-trash", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/folder-trash-root/trash")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("trash folder request should be built"),
        )
        .await
        .expect("trash folder request should be handled");
    assert_eq!(trash_response.status(), StatusCode::CREATED);
    let trash_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(trash_response.into_body(), usize::MAX)
            .await
            .expect("trash folder response should be read"),
    )
    .expect("trash folder response should be valid json");
    assert_eq!(
        common::envelope_item(&trash_payload)["lifecycleStatus"].as_str(),
        Some("trashed")
    );

    let trashed_node_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-tree-trash'
           AND id IN (
               'folder-trash-root',
               'folder-trash-child',
               'file-trash-child',
               'file-trash-grandchild'
           )
           AND lifecycle_status='trashed'",
    )
    .fetch_one(&pool)
    .await
    .expect("trashed node count should be readable");
    assert_eq!(trashed_node_count, 4);

    let child_detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-tree-trash", "user-tree-trash", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-tree-trash", "user-tree-trash", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/file-trash-child")
                .body(Body::empty())
                .expect("get trashed child request should be built"),
        )
        .await
        .expect("get trashed child request should be handled");
    assert_eq!(child_detail_response.status(), StatusCode::OK);
    let child_detail_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(child_detail_response.into_body(), usize::MAX)
            .await
            .expect("get trashed child response should be read"),
    )
    .expect("get trashed child response should be valid json");
    assert_eq!(
        common::envelope_item(&child_detail_payload)["lifecycleStatus"].as_str(),
        Some("trashed")
    );

    let search_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-tree-trash", "user-tree-trash", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-tree-trash", "user-tree-trash", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/search?spaceId=space-tree-trash&q=child")
                .body(Body::empty())
                .expect("search trashed child request should be built"),
        )
        .await
        .expect("search trashed child request should be handled");
    assert_eq!(search_response.status(), StatusCode::OK);
    let search_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(search_response.into_body(), usize::MAX)
            .await
            .expect("search trashed child response should be read"),
    )
    .expect("search trashed child response should be valid json");
    assert_eq!(
        common::envelope_items(&search_payload)
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

#[tokio::test]
async fn app_drive_restore_folder_recursively_restores_descendants_and_requires_active_parent() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-tree-restore', 'tenant-tree-restore', 'user', 'user-tree-restore', 'personal', 'Tree Restore', 'active', 1, 'user-tree-restore', 'user-tree-restore')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    for (id, parent_id, node_type, node_name) in [
        (
            "folder-restore-root",
            Option::<&str>::None,
            "folder",
            "Root Folder",
        ),
        (
            "folder-restore-child",
            Some("folder-restore-root"),
            "folder",
            "Child Folder",
        ),
        (
            "file-restore-child",
            Some("folder-restore-root"),
            "file",
            "child.txt",
        ),
        (
            "file-restore-grandchild",
            Some("folder-restore-child"),
            "file",
            "grandchild.txt",
        ),
        (
            "file-restore-duplicate-a",
            Some("folder-restore-root"),
            "file",
            "duplicate.txt",
        ),
        (
            "file-restore-duplicate-b",
            Some("folder-restore-root"),
            "file",
            "duplicate.txt",
        ),
        (
            "folder-restore-block-parent",
            None,
            "folder",
            "Blocked Parent",
        ),
        (
            "file-restore-block-child",
            Some("folder-restore-block-parent"),
            "file",
            "blocked-child.txt",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-tree-restore', 'space-tree-restore', $2, $3, $4, 'ready', 'trashed', 1, 'user-tree-restore', 'user-tree-restore')",
        )
        .bind(id)
        .bind(parent_id)
        .bind(node_type)
        .bind(node_name)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }

    let app = common::test_router_with_pool(pool.clone());
    let blocked_child_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-tree-restore", "user-tree-restore", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-tree-restore", "user-tree-restore", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/trash/file-restore-block-child/restore")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("restore child with trashed parent request should be built"),
        )
        .await
        .expect("restore child with trashed parent request should be handled");
    assert_eq!(blocked_child_response.status(), StatusCode::CONFLICT);
    let blocked_child_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(blocked_child_response.into_body(), usize::MAX)
            .await
            .expect("restore child with trashed parent response should be read"),
    )
    .expect("restore child with trashed parent response should be valid json");
    assert_eq!(
        blocked_child_payload["detail"].as_str(),
        Some("parent node must be active before restore")
    );

    let restore_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-tree-restore", "user-tree-restore", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-tree-restore", "user-tree-restore", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/trash/folder-restore-root/restore")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("restore folder request should be built"),
        )
        .await
        .expect("restore folder request should be handled");
    assert_eq!(restore_response.status(), StatusCode::OK);
    let restore_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(restore_response.into_body(), usize::MAX)
            .await
            .expect("restore folder response should be read"),
    )
    .expect("restore folder response should be valid json");
    assert_eq!(
        common::envelope_item(&restore_payload)["lifecycleStatus"].as_str(),
        Some("active")
    );

    let active_node_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-tree-restore'
           AND id IN (
               'folder-restore-root',
               'folder-restore-child',
               'file-restore-child',
               'file-restore-grandchild',
               'file-restore-duplicate-a',
               'file-restore-duplicate-b'
           )
           AND lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("active node count should be readable");
    assert_eq!(active_node_count, 6);

    let restored_duplicate_names: Vec<String> = sqlx::query_scalar(
        "SELECT node_name
         FROM dr_drive_node
         WHERE tenant_id='tenant-tree-restore'
           AND id IN ('file-restore-duplicate-a', 'file-restore-duplicate-b')
         ORDER BY node_name",
    )
    .fetch_all(&pool)
    .await
    .expect("restored duplicate names should be readable");
    assert_eq!(
        restored_duplicate_names,
        vec!["duplicate (1).txt".to_string(), "duplicate.txt".to_string()]
    );

    let still_trashed_blocked_child_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-tree-restore'
           AND id IN ('folder-restore-block-parent', 'file-restore-block-child')
           AND lifecycle_status='trashed'",
    )
    .fetch_one(&pool)
    .await
    .expect("blocked restore nodes should remain trashed");
    assert_eq!(still_trashed_blocked_child_count, 2);
}

#[tokio::test]
async fn app_dr_drive_upload_session_abort_marks_session_aborted() {
    let (s3_endpoint, _captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-abort', 'tenant-abort', 'user', 'user-abort', 'personal', 'Abort', 'active', 1, 'user-abort', 'user-abort')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-abort', 'tenant-abort', 'space-abort', NULL, 'file', 'abort.bin', 'uploading', 'active', 1, 'user-abort', 'user-abort')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-abort",
        "Mock S3",
        &s3_endpoint,
        "bucket-abort",
        "active",
        "admin-abort",
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'upload-abort', 'tenant-abort', 'space-abort', 'node-abort',
            'bucket-abort', 'objects/node-abort/abort.bin', 'idem-abort',
            'provider-abort', 'upload-abort', 'created', 1800000000000, 1,
            'user-abort', 'user-abort'
        )",
    )
    .execute(&pool)
    .await
    .expect("upload session should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-abort", "user-abort", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-abort", "user-abort", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions/upload-abort/abort")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("abort upload request should be built"),
        )
        .await
        .expect("abort upload request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("abort response body should be read"),
    )
    .expect("abort response json should be valid");
    assert_eq!(common::envelope_body(&payload)["state"], "aborted");
}

#[tokio::test]
async fn app_dr_drive_upload_session_abort_rejects_terminal_sessions_without_object_store_call() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-abort-terminal', 'tenant-abort-terminal', 'user', 'user-abort', 'personal', 'Abort Terminal', 'active', 1, 'user-abort', 'user-abort')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-abort-terminal', 'tenant-abort-terminal', 'space-abort-terminal', NULL, 'file', 'abort-terminal.bin', 'uploading', 'active', 1, 'user-abort', 'user-abort')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-abort-terminal', 's3_compatible', 'Mock S3', $1, 'us-east-1',
            'bucket-abort-terminal', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-abort', 'admin-abort'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'upload-abort-terminal', 'tenant-abort-terminal', 'space-abort-terminal',
            'node-abort-terminal', 'bucket-abort-terminal',
            'sdkwork-drive/v1/t/aa/tenants/tenant-abort-terminal/spaces/space-abort-terminal/nodes/n/bb/node-abort-terminal/versions/0000000001/upload-abort-terminal/content',
            'idem-abort-terminal', 'provider-abort-terminal', 'mock-s3-upload-id',
            'aborted', 1800000000000, 2, 'user-abort', 'user-abort'
        )",
    )
    .execute(&pool)
    .await
    .expect("terminal upload session should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-abort-terminal", "user-abort", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-abort-terminal", "user-abort", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions/upload-abort-terminal/abort")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("abort terminal upload request should be built"),
        )
        .await
        .expect("abort terminal upload request should be handled");

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("abort terminal response body should be read"),
    )
    .expect("abort terminal response json should be valid");
    assert_eq!(payload["code"], 40901);

    let delete_calls = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .iter()
        .filter(|request| request.method == "DELETE")
        .count();
    assert_eq!(delete_calls, 0);
}

#[tokio::test]
async fn presign_upload_part_requires_active_object_store_provider() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-no-provider', 'tenant-no-provider', 'user', 'user-no-provider', 'personal', 'No Provider', 'active', 1, 'user-no-provider', 'user-no-provider')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-no-provider', 'tenant-no-provider', 'space-no-provider', NULL, 'file', 'upload.bin', 'uploading', 'active', 1, 'user-no-provider', 'user-no-provider')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-no-provider",
        "Disabled S3",
        "https://s3.example.com",
        "bucket-no-provider",
        "disabled",
        "user-no-provider",
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES ('upload-no-provider', 'tenant-no-provider', 'space-no-provider', 'node-no-provider', 'bucket-no-provider', 'sdkwork-drive/v1/t/aa/tenants/tenant-no-provider/spaces/space-no-provider/nodes/n/bb/node-no-provider/versions/0000000001/upload-no-provider/content', 'idem-no-provider', 'provider-no-provider', 'upload-no-provider', 'created', 1800000000000, 1, 'user-no-provider', 'user-no-provider')",
    )
    .execute(&pool)
    .await
    .expect("upload session should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-no-provider", "user-no-provider", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-no-provider", "user-no-provider", "appbase"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/upload_sessions/upload-no-provider/parts/1")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "requestedTtlSeconds":120
                    }"#,
                ))
                .expect("presign request should be built"),
        )
        .await
        .expect("presign request should be handled");

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("error body should be readable"),
    )
    .expect("error body should be valid json");
    assert_eq!(payload["code"], 40901);
    assert!(payload["detail"]
        .as_str()
        .expect("detail should be present")
        .contains("active storage provider"));
}

#[tokio::test]
async fn complete_upload_session_rejects_invalid_multipart_parts() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-invalid-parts', 'tenant-invalid-parts', 'user', 'user-invalid-parts', 'personal', 'Invalid Parts', 'active', 1, 'user-invalid-parts', 'user-invalid-parts')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-invalid-parts', 'tenant-invalid-parts', 'space-invalid-parts', NULL, 'file', 'upload.bin', 'uploading', 'active', 1, 'user-invalid-parts', 'user-invalid-parts')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-invalid-parts",
        "Invalid Parts S3",
        "https://s3.example.com",
        "bucket-invalid-parts",
        "active",
        "user-invalid-parts",
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'upload-invalid-parts', 'tenant-invalid-parts', 'space-invalid-parts',
            'node-invalid-parts', 'bucket-invalid-parts',
            'sdkwork-drive/v1/t/aa/tenants/tenant-invalid-parts/spaces/space-invalid-parts/nodes/n/bb/node-invalid-parts/versions/0000000001/upload-invalid-parts/content',
            'idem-invalid-parts', 'provider-invalid-parts', 'storage-upload-invalid-parts',
            'created', 1800000000000, 1, 'user-invalid-parts', 'user-invalid-parts'
        )",
    )
    .execute(&pool)
    .await
    .expect("upload session should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-invalid-parts", "user-invalid-parts", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-invalid-parts", "user-invalid-parts", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions/upload-invalid-parts/complete")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "contentType":"application/octet-stream",
                        "contentLength":12,
                        "checksumSha256Hex":"sha256:invalid",
                        "parts":[
                            {"partNo":2,"etag":"etag-2"},
                            {"partNo":1,"etag":"etag-1"}
                        ]
                    }"#,
                ))
                .expect("complete upload request should be built"),
        )
        .await
        .expect("complete upload request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("complete response body should be read"),
    )
    .expect("complete response json should be valid");
    assert_eq!(payload["code"], 40001);
    assert!(payload["detail"]
        .as_str()
        .expect("detail should be a string")
        .contains("ascending"));
}

#[tokio::test]
async fn upload_session_mutations_reject_trashed_node_before_storage_side_effects() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-upload-trashed', 'tenant-upload-trashed', 'user', 'user-upload',
            'personal', 'Upload Trashed', 'active', 1, 'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-upload-trashed', 'tenant-upload-trashed', 'space-upload-trashed',
            NULL, 'file', 'upload.bin', 'uploading', 'trashed', 1,
            'user-upload', 'user-upload'
        )",
    )
    .execute(&pool)
    .await
    .expect("trashed upload node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-upload-trashed",
        "Upload Trashed S3",
        &s3_endpoint,
        "bucket-upload-trashed",
        "active",
        "user-upload",
    )
    .await;
    for id in [
        "upload-trashed-presign",
        "upload-trashed-complete",
        "upload-trashed-abort",
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_upload_session (
                id, tenant_id, space_id, node_id, bucket, object_key,
                idempotency_key, storage_provider_id, storage_upload_id, state,
                expires_at_epoch_ms, version, created_by, updated_by
            ) VALUES (
                $1, 'tenant-upload-trashed', 'space-upload-trashed',
                'node-upload-trashed', 'bucket-upload-trashed', $2,
                $3, 'provider-upload-trashed', 'mock-s3-upload-id', 'created',
                1800000000000, 1, 'user-upload', 'user-upload'
            )",
        )
        .bind(id)
        .bind(format!(
            "sdkwork-drive/v1/t/aa/tenants/tenant-upload-trashed/spaces/space-upload-trashed/nodes/n/bb/node-upload-trashed/versions/0000000001/{id}/content"
        ))
        .bind(format!("idem-{id}"))
        .execute(&pool)
        .await
        .expect("upload session should be seeded");
    }

    let app = common::test_router_with_pool(pool.clone());
    let mut observed = Vec::<(&'static str, StatusCode)>::new();
    for (name, request) in [
        (
            "presign",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-upload-trashed", "user-upload", "appbase")),
            )
            .header("access-token", common::access_token("tenant-upload-trashed", "user-upload", "appbase"))
                .method(Method::PUT)
                .uri("/app/v3/api/drive/upload_sessions/upload-trashed-presign/parts/1")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "requestedTtlSeconds":120
                    }"#,
                ))
                .expect("presign upload part request should be built"),
        ),
        (
            "complete",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-upload-trashed", "user-upload", "appbase")),
            )
            .header("access-token", common::access_token("tenant-upload-trashed", "user-upload", "appbase"))
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions/upload-trashed-complete/complete")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "contentType":"application/octet-stream",
                        "contentLength":12,
                        "checksumSha256Hex":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        "parts":[{"partNo":1,"etag":"etag-upload-trashed-1"}]
                    }"#,
                ))
                .expect("complete upload request should be built"),
        ),
        (
            "abort",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-upload-trashed", "user-upload", "appbase")),
            )
            .header("access-token", common::access_token("tenant-upload-trashed", "user-upload", "appbase"))
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions/upload-trashed-abort/abort")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#,
                ))
                .expect("abort upload request should be built"),
        ),
    ] {
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("upload mutation request should be handled");
        observed.push((name, response.status()));
        if response.status() == StatusCode::NOT_FOUND {
            let payload: serde_json::Value = serde_json::from_slice(
                &to_bytes(response.into_body(), usize::MAX)
                    .await
                    .expect("not found response should be read"),
            )
            .expect("not found response should be valid json");
            assert_eq!(payload["code"].as_i64(), Some(40401), "{name}");
        }
    }

    assert_eq!(
        observed,
        vec![
            ("presign", StatusCode::NOT_FOUND),
            ("complete", StatusCode::NOT_FOUND),
            ("abort", StatusCode::NOT_FOUND),
        ]
    );

    let session_states: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT id, state, version
         FROM dr_drive_upload_session
         WHERE tenant_id='tenant-upload-trashed'
         ORDER BY id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("upload session states should be readable");
    assert_eq!(
        session_states,
        vec![
            ("upload-trashed-abort".to_string(), "created".to_string(), 1),
            (
                "upload-trashed-complete".to_string(),
                "created".to_string(),
                1
            ),
            (
                "upload-trashed-presign".to_string(),
                "created".to_string(),
                1
            ),
        ]
    );
    let object_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-upload-trashed'",
    )
    .fetch_one(&pool)
    .await
    .expect("storage object count should be readable");
    assert_eq!(object_count, 0);
    let change_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_change_log
         WHERE tenant_id='tenant-upload-trashed'",
    )
    .fetch_one(&pool)
    .await
    .expect("change count should be readable");
    assert_eq!(change_count, 0);
    assert!(
        captured_requests
            .lock()
            .expect("captured s3 requests mutex should not be poisoned")
            .is_empty(),
        "trashed node upload mutations should fail before object storage calls"
    );
}

#[tokio::test]
async fn complete_upload_session_rejects_invalid_object_metadata_before_storage_call() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-invalid-metadata', 'tenant-invalid-metadata', 'user', 'user-invalid-metadata', 'personal', 'Invalid Metadata', 'active', 1, 'user-invalid-metadata', 'user-invalid-metadata')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-invalid-metadata', 'tenant-invalid-metadata', 'space-invalid-metadata', NULL, 'file', 'upload.bin', 'uploading', 'active', 1, 'user-invalid-metadata', 'user-invalid-metadata')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-invalid-metadata', 's3_compatible', 'Metadata S3', $1, 'us-east-1',
            'bucket-invalid-metadata', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'user-invalid-metadata', 'user-invalid-metadata'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'upload-invalid-metadata', 'tenant-invalid-metadata', 'space-invalid-metadata',
            'node-invalid-metadata', 'bucket-invalid-metadata',
            'sdkwork-drive/v1/t/aa/tenants/tenant-invalid-metadata/spaces/space-invalid-metadata/nodes/n/bb/node-invalid-metadata/versions/0000000001/upload-invalid-metadata/content',
            'idem-invalid-metadata', 'provider-invalid-metadata', 'storage-upload-invalid-metadata',
            'created', 1800000000000, 1, 'user-invalid-metadata', 'user-invalid-metadata'
        )",
    )
    .execute(&pool)
    .await
    .expect("upload session should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-invalid-metadata",
                            "user-invalid-metadata",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token(
                        "tenant-invalid-metadata",
                        "user-invalid-metadata",
                        "appbase",
                    ),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions/upload-invalid-metadata/complete")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "contentType":"application/octet stream",
                        "contentLength":12,
                        "checksumSha256Hex":"sha256:not-hex",
                        "parts":[{"partNo":1,"etag":"etag-invalid-metadata-1"}]
                    }"#,
                ))
                .expect("complete upload request should be built"),
        )
        .await
        .expect("complete upload request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("complete response body should be read"),
    )
    .expect("complete response json should be valid");
    assert_eq!(payload["code"], 40001);
    let detail = payload["detail"]
        .as_str()
        .expect("detail should be a string");
    assert!(detail.contains("contentType") || detail.contains("checksumSha256Hex"));

    let object_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_storage_object WHERE tenant_id='tenant-invalid-metadata'",
    )
    .fetch_one(&pool)
    .await
    .expect("storage object count should be readable");
    assert_eq!(object_count, 0);
    assert!(
        captured_requests
            .lock()
            .expect("captured s3 requests mutex should not be poisoned")
            .is_empty(),
        "invalid metadata should fail before completing storage upload"
    );
}

#[tokio::test]
async fn list_spaces_route_returns_tenant_scoped_items() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $8)",
    )
    .bind("space-001")
    .bind("tenant-001")
    .bind("user")
    .bind("user-001")
    .bind("personal")
    .bind("Main")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed first space should succeed");
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $8)",
    )
    .bind("space-002")
    .bind("tenant-001")
    .bind("user")
    .bind("user-002")
    .bind("knowledge_base")
    .bind("Knowledge")
    .bind("user-002")
    .bind("user-002")
    .execute(&pool)
    .await
    .expect("seed second space should succeed");
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $8)",
    )
    .bind("space-003")
    .bind("tenant-002")
    .bind("user")
    .bind("user-003")
    .bind("team")
    .bind("Other")
    .bind("user-003")
    .bind("user-003")
    .execute(&pool)
    .await
    .expect("seed third space should succeed");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("list spaces request should be handled");

    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    assert_eq!(payload["code"], 0);
    assert_eq!(
        payload["data"]["items"]
            .as_array()
            .expect("items should be array")
            .len(),
        1
    );
    assert_eq!(
        payload["data"]["items"][0]["id"].as_str(),
        Some("space-001")
    );
}

#[tokio::test]
async fn create_download_url_and_resolve_token_redirects_to_signed_source() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $8)",
    )
    .bind("space-001")
    .bind("tenant-001")
    .bind("user")
    .bind("user-001")
    .bind("personal")
    .bind("Main")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed space should succeed");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, NULL, $4, $5, 'ready', 'active', 1, $6, $7)",
    )
    .bind("node-001")
    .bind("tenant-001")
    .bind("space-001")
    .bind("file")
    .bind("v1.bin")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed node should succeed");
    seed_object_store_provider_fixture(
        &pool,
        ObjectStoreProviderFixture {
            provider_id: "provider-download-001",
            provider_kind: "s3_compatible",
            provider_name: "MinIO",
            endpoint_url: "http://127.0.0.1:9000",
            region: "us-east-1",
            bucket: "bucket-001",
            path_style: true,
            actor_id: "user-001",
        },
    )
    .await;
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "obj-001",
            tenant_id: "tenant-001",
            node_id: "node-001",
            version_no: 1,
            provider_id: "provider-download-001",
            bucket: "bucket-001",
            object_key: "objects/node-001/v1.bin",
            content_type: "application/octet-stream",
            content_length: 128,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-001",
        },
    )
    .await
    .expect("seed storage object should succeed");

    let app = common::test_router_with_pool(pool);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_urls")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeId":"node-001",
                        "requestedTtlSeconds":120
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create download url request should be handled");
    assert_eq!(response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    let download_data = common::envelope_data(&payload);
    assert_eq!(download_data["method"], "GET");
    let signed_source_url = download_data["signedSourceUrl"]
        .as_str()
        .expect("signedSourceUrl should be present");
    assert!(signed_source_url.contains("http://127.0.0.1:9000/bucket-001/objects/node-001/v1.bin"));
    assert!(signed_source_url.contains("X-Amz-Signature="));

    let url = common::envelope_data(&payload)["downloadUrl"]
        .as_str()
        .expect("downloadUrl should be present");
    let token = url
        .rsplit('/')
        .next()
        .expect("token should be encoded in path segment");
    let resolve_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::GET)
                .uri(format!("/app/v3/api/drive/download_tokens/{token}"))
                .body(Body::empty())
                .expect("resolve request should be built"),
        )
        .await
        .expect("resolve token request should be handled");
    assert_eq!(resolve_response.status(), StatusCode::OK);
    let resolve_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(resolve_response.into_body(), usize::MAX)
            .await
            .expect("resolve token response should be read"),
    )
    .expect("resolve token response should be valid json");
    let resolve_data = common::envelope_data(&resolve_payload);
    let signed_source_url = resolve_data["signedSourceUrl"]
        .as_str()
        .expect("resolved download response should include signedSourceUrl");
    assert!(signed_source_url.contains("http://127.0.0.1:9000/bucket-001/objects/node-001/v1.bin"));
    assert!(signed_source_url.contains("X-Amz-Signature="));
}

#[tokio::test]
async fn create_download_grant_via_canonical_route_returns_created() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $8)",
    )
    .bind("space-001")
    .bind("tenant-001")
    .bind("user")
    .bind("user-001")
    .bind("personal")
    .bind("Main")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed space should succeed");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, NULL, $4, $5, 'ready', 'active', 1, $6, $7)",
    )
    .bind("node-001")
    .bind("tenant-001")
    .bind("space-001")
    .bind("file")
    .bind("v1.bin")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed node should succeed");
    seed_object_store_provider_fixture(
        &pool,
        ObjectStoreProviderFixture {
            provider_id: "provider-download-grant-001",
            provider_kind: "s3_compatible",
            provider_name: "MinIO",
            endpoint_url: "http://127.0.0.1:9000",
            region: "us-east-1",
            bucket: "bucket-001",
            path_style: true,
            actor_id: "user-001",
        },
    )
    .await;
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "obj-grant-001",
            tenant_id: "tenant-001",
            node_id: "node-001",
            version_no: 1,
            provider_id: "provider-download-grant-001",
            bucket: "bucket-001",
            object_key: "objects/node-001/v1.bin",
            content_type: "application/octet-stream",
            content_length: 128,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-001",
        },
    )
    .await
    .expect("seed storage object should succeed");

    let pool_for_assert = pool.clone();
    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-001/download_grants")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "requestedTtlSeconds":120
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create download grant request should be handled");
    assert_eq!(response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    let download_data = common::envelope_data(&payload);
    assert_eq!(download_data["method"], "GET");
    assert!(download_data["downloadUrl"]
        .as_str()
        .expect("downloadUrl should be present")
        .contains("/download_tokens/"));
    assert!(download_data["signedSourceUrl"]
        .as_str()
        .expect("signedSourceUrl should be present")
        .contains("http://127.0.0.1:9000/bucket-001/objects/node-001/v1.bin"));

    let change_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_change_log
         WHERE tenant_id='tenant-001'
           AND node_id='node-001'
           AND event_type='drive.download_grant.created'",
    )
    .fetch_one(&pool_for_assert)
    .await
    .expect("download grant change count should be readable");
    assert_eq!(change_count, 1);
}

#[tokio::test]
async fn create_download_url_rejects_ttl_outside_contract() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-download-ttl', 'tenant-download-ttl', 'user', 'user-download-ttl', 'personal', 'TTL', 'active', 1, 'user-download-ttl', 'user-download-ttl')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-download-ttl', 'tenant-download-ttl', 'space-download-ttl', NULL, 'file', 'download.bin', 'ready', 'active', 1, 'user-download-ttl', 'user-download-ttl')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-download-ttl",
        "TTL S3",
        "http://127.0.0.1:9000",
        "bucket-download-ttl",
        "active",
        "user-download-ttl",
    )
    .await;
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "obj-download-ttl",
            tenant_id: "tenant-download-ttl",
            node_id: "node-download-ttl",
            version_no: 1,
            provider_id: "provider-download-ttl",
            bucket: "bucket-download-ttl",
            object_key: "objects/download-ttl.bin",
            content_type: "application/octet-stream",
            content_length: 128,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-download-ttl",
        },
    )
    .await
    .expect("storage object should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-download-ttl", "user-download-ttl", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-download-ttl", "user-download-ttl", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_urls")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeId":"node-download-ttl",
                        "requestedTtlSeconds":0
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create download url request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    assert_eq!(payload["code"], 40001);
    assert!(payload["detail"]
        .as_str()
        .expect("detail should be present")
        .contains("requestedTtlSeconds"));
}

#[tokio::test]
async fn resolve_download_token_requires_active_object_store_provider() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-download-no-provider', 'tenant-download-no-provider', 'user', 'user-download-no-provider', 'personal', 'No Provider', 'active', 1, 'user-download-no-provider', 'user-download-no-provider')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-download-no-provider', 'tenant-download-no-provider', 'space-download-no-provider', NULL, 'file', 'download.bin', 'ready', 'active', 1, 'user-download-no-provider', 'user-download-no-provider')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-download-disabled",
        "Disabled Download S3",
        "http://127.0.0.1:9000",
        "bucket-download-no-provider",
        "disabled",
        "user-download-no-provider",
    )
    .await;
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "obj-download-no-provider",
            tenant_id: "tenant-download-no-provider",
            node_id: "node-download-no-provider",
            version_no: 1,
            provider_id: "provider-download-disabled",
            bucket: "bucket-download-no-provider",
            object_key: "sdkwork-drive/v1/t/aa/tenants/tenant-download-no-provider/spaces/space-download-no-provider/nodes/n/bb/node-download-no-provider/versions/0000000001/upload-download-no-provider/content",
            content_type: "application/octet-stream",
            content_length: 128,
            checksum_sha256_hex: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-download-no-provider",
        },
    )
    .await
    .expect("storage object should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-download-no-provider",
                            "user-download-no-provider",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token(
                        "tenant-download-no-provider",
                        "user-download-no-provider",
                        "appbase",
                    ),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_urls")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeId":"node-download-no-provider",
                        "requestedTtlSeconds":120
                    }"#,
                ))
                .expect("create download url request should be built"),
        )
        .await
        .expect("create download url request should be handled");
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("error body should be readable"),
    )
    .expect("error body should be valid json");
    assert_eq!(payload["code"], 40901);
    assert!(payload["detail"]
        .as_str()
        .expect("detail should be present")
        .contains("active storage provider"));
}

#[tokio::test]
async fn resolve_download_token_uses_active_s3_provider_configuration_when_present() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $8)",
    )
    .bind("space-001")
    .bind("tenant-001")
    .bind("user")
    .bind("user-001")
    .bind("personal")
    .bind("Main")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed space should succeed");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, NULL, $4, $5, 'ready', 'active', 1, $6, $7)",
    )
    .bind("node-001")
    .bind("tenant-001")
    .bind("space-001")
    .bind("file")
    .bind("v1.bin")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed node should succeed");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'active', 1, $11, $12)",
    )
    .bind("provider-s3-001")
    .bind("s3_compatible")
    .bind("Primary S3")
    .bind("https://s3.custom.local")
    .bind("us-east-1")
    .bind("bucket-001")
    .bind(true)
    .bind("plain:test-access-key:test-secret-key")
    .bind("AES256")
    .bind("STANDARD")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("seed storage provider should succeed");
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "obj-001",
            tenant_id: "tenant-001",
            node_id: "node-001",
            version_no: 1,
            provider_id: "provider-s3-001",
            bucket: "bucket-001",
            object_key: "objects/node-001/v1.bin",
            content_type: "application/octet-stream",
            content_length: 128,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-001",
        },
    )
    .await
    .expect("seed storage object should succeed");

    let app = common::test_router_with_pool(pool);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_urls")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeId":"node-001",
                        "requestedTtlSeconds":120
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create download url request should be handled");
    assert_eq!(response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");

    let url = common::envelope_data(&payload)["downloadUrl"]
        .as_str()
        .expect("downloadUrl should be present");
    let token = url
        .rsplit('/')
        .next()
        .expect("token should be encoded in path segment");
    let resolve_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::GET)
                .uri(format!("/app/v3/api/drive/download_tokens/{token}"))
                .body(Body::empty())
                .expect("resolve request should be built"),
        )
        .await
        .expect("resolve token request should be handled");
    assert_eq!(resolve_response.status(), StatusCode::OK);
    let resolve_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(resolve_response.into_body(), usize::MAX)
            .await
            .expect("resolve token response should be read"),
    )
    .expect("resolve token response should be valid json");
    let signed_source_url = common::envelope_data(&resolve_payload)["signedSourceUrl"]
        .as_str()
        .expect("resolved download response should include signedSourceUrl");
    assert!(signed_source_url.starts_with("https://s3.custom.local/"));
    assert!(signed_source_url.contains("/bucket-001/objects/node-001/v1.bin"));
    assert!(signed_source_url.contains("X-Amz-Signature"));
}

#[tokio::test]
async fn resolve_download_token_uses_aliyun_oss_provider_kind_with_s3_signer() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $8)",
    )
    .bind("space-oss-001")
    .bind("tenant-001")
    .bind("user")
    .bind("user-001")
    .bind("personal")
    .bind("Main")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed space should succeed");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, NULL, $4, $5, 'ready', 'active', 1, $6, $7)",
    )
    .bind("node-oss-001")
    .bind("tenant-001")
    .bind("space-oss-001")
    .bind("file")
    .bind("v1.bin")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed node should succeed");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'active', 1, $11, $12)",
    )
    .bind("provider-oss-001")
    .bind("aliyun_oss")
    .bind("Aliyun OSS")
    .bind("https://oss-cn-hangzhou.aliyuncs.com")
    .bind("cn-hangzhou")
    .bind("bucket-oss-001")
    .bind(false)
    .bind("plain:test-access-key:test-secret-key")
    .bind("AES256")
    .bind("STANDARD")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("seed storage provider should succeed");
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "obj-oss-001",
            tenant_id: "tenant-001",
            node_id: "node-oss-001",
            version_no: 1,
            provider_id: "provider-oss-001",
            bucket: "bucket-oss-001",
            object_key: "objects/node-oss-001/v1.bin",
            content_type: "application/octet-stream",
            content_length: 128,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-001",
        },
    )
    .await
    .expect("seed storage object should succeed");

    let app = common::test_router_with_pool(pool);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_urls")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeId":"node-oss-001",
                        "requestedTtlSeconds":120
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create download url request should be handled");
    assert_eq!(response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");

    let url = common::envelope_data(&payload)["downloadUrl"]
        .as_str()
        .expect("downloadUrl should be present");
    let token = url
        .rsplit('/')
        .next()
        .expect("token should be encoded in path segment");
    let resolve_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::GET)
                .uri(format!("/app/v3/api/drive/download_tokens/{token}"))
                .body(Body::empty())
                .expect("resolve request should be built"),
        )
        .await
        .expect("resolve token request should be handled");
    assert_eq!(resolve_response.status(), StatusCode::OK);
    let resolve_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(resolve_response.into_body(), usize::MAX)
            .await
            .expect("resolve token response should be read"),
    )
    .expect("resolve token response should be valid json");
    let signed_source_url = common::envelope_data(&resolve_payload)["signedSourceUrl"]
        .as_str()
        .expect("resolved download response should include signedSourceUrl");
    assert!(signed_source_url.contains("X-Amz-Signature"));
    assert!(signed_source_url.contains("objects/node-oss-001/v1.bin"));
}

#[tokio::test]
async fn resolve_download_token_uses_explicit_cloud_s3_provider_kinds_with_s3_signer() {
    for (provider_kind, endpoint_url, region, bucket, node_id) in [
        (
            "tencent_cos",
            "https://cos.ap-guangzhou.myqcloud.com",
            "ap-guangzhou",
            "bucket-cos-001",
            "node-cos-001",
        ),
        (
            "huawei_obs",
            "https://obs.cn-north-4.myhuaweicloud.com",
            "cn-north-4",
            "bucket-obs-001",
            "node-obs-001",
        ),
        (
            "volcengine_tos",
            "https://tos-cn-beijing.volces.com",
            "cn-beijing",
            "bucket-tos-001",
            "node-tos-001",
        ),
    ] {
        let Some((pool, _database_guard)) =
            sdkwork_drive_test_support::postgres_test_database().await
        else {
            return;
        };

        let space_id = format!("space-{provider_kind}");
        let object_id = format!("obj-{provider_kind}");
        let object_key = format!("objects/{node_id}/v1.bin");
        let provider_id = format!("provider-{provider_kind}");

        sqlx::query(
            "INSERT INTO dr_drive_space (
                id, tenant_id, owner_subject_type, owner_subject_id, space_type,
                display_name, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-cloud-s3', 'user', 'user-cloud-s3', 'personal',
                'Cloud S3', 'active', 1, 'user-cloud-s3', 'user-cloud-s3')",
        )
        .bind(&space_id)
        .execute(&pool)
        .await
        .expect("space should be seeded");
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-cloud-s3', $2, NULL, 'file', 'v1.bin',
                'ready', 'active', 1, 'user-cloud-s3', 'user-cloud-s3')",
        )
        .bind(node_id)
        .bind(&space_id)
        .execute(&pool)
        .await
        .expect("node should be seeded");
        sqlx::query(
            "INSERT INTO dr_drive_storage_provider (
                id, provider_kind, name, endpoint_url, region, bucket, path_style,
                strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
                status, version, created_by, updated_by
            ) VALUES ($1, $2, 'Cloud S3', $3, $4, $5, 0, $6,
                'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD',
                'active', 1, 'admin-cloud-s3', 'admin-cloud-s3')",
        )
        .bind(&provider_id)
        .bind(provider_kind)
        .bind(endpoint_url)
        .bind(region)
        .bind(bucket)
        .bind(
            !endpoint_url
                .trim()
                .to_ascii_lowercase()
                .starts_with("http://"),
        )
        .execute(&pool)
        .await
        .expect("storage provider should be seeded");
        seed_storage_object_fixture(
            &pool,
            StorageObjectFixture {
                object_id: &object_id,
                tenant_id: "tenant-cloud-s3",
                node_id,
                version_no: 1,
                provider_id: &provider_id,
                bucket,
                object_key: &object_key,
                content_type: "application/octet-stream",
                content_length: 128,
                checksum_sha256_hex:
                    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                lifecycle_status: "active",
                actor_id: "user-cloud-s3",
            },
        )
        .await
        .expect("storage object should be seeded");

        let app = common::test_router_with_pool(pool);
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-cloud-s3", "user-cloud-s3", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-cloud-s3", "user-cloud-s3", "appbase"),
                    )
                    .method(Method::POST)
                    .uri("/app/v3/api/drive/download_urls")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{
                                                        "nodeId":"{node_id}",
                            "requestedTtlSeconds":120
                        }}"#
                    )))
                    .expect("request should be built"),
            )
            .await
            .expect("create download url request should be handled");
        assert_eq!(
            create_response.status(),
            StatusCode::CREATED,
            "{provider_kind} should create a Drive download token"
        );
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(create_response.into_body(), usize::MAX)
                .await
                .expect("response body should be read"),
        )
        .expect("response json should be valid");
        let token = common::envelope_data(&payload)["downloadUrl"]
            .as_str()
            .expect("downloadUrl should be present")
            .rsplit('/')
            .next()
            .expect("token should be encoded in path segment")
            .to_string();

        let resolve_response = app
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-cloud-s3", "user-cloud-s3", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-cloud-s3", "user-cloud-s3", "appbase"),
                    )
                    .method(Method::GET)
                    .uri(format!("/app/v3/api/drive/download_tokens/{token}"))
                    .body(Body::empty())
                    .expect("resolve request should be built"),
            )
            .await
            .expect("resolve token request should be handled");
        assert_eq!(
            resolve_response.status(),
            StatusCode::OK,
            "{provider_kind} should be signed through the S3 adapter"
        );
        let resolve_payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(resolve_response.into_body(), usize::MAX)
                .await
                .expect("resolve token response should be read"),
        )
        .expect("resolve token response should be valid json");
        let signed_source_url = common::envelope_data(&resolve_payload)["signedSourceUrl"]
            .as_str()
            .expect("resolved download response should include signedSourceUrl");
        assert!(
            signed_source_url.contains("X-Amz-Signature"),
            "{provider_kind} response should expose an S3 presigned URL: {signed_source_url}"
        );
        assert!(
            signed_source_url.contains(&object_key),
            "{provider_kind} response should target the stored object key: {signed_source_url}"
        );
    }
}

#[tokio::test]
async fn create_download_package_for_multiple_files_writes_zip_archive_and_returns_download_url() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_download_package_fixture(&pool, &s3_endpoint)
        .await
        .expect("download package fixture should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-bulk", "user-bulk", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-bulk", "user-bulk", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_packages")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeIds":["node-file-a","node-file-b"],
                        "packageName":"Selected documents",
                        "requestedTtlSeconds":180
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create package request should be handled");
    assert_eq!(response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    let package_data = common::envelope_data(&payload);
    assert_eq!(package_data["state"].as_str(), Some("ready"));
    assert_eq!(
        package_data["contentType"].as_str(),
        Some("application/zip")
    );
    assert_eq!(package_data["fileCount"].as_i64(), Some(2));
    assert_eq!(package_data["totalBytes"].as_i64(), Some(19));
    assert_eq!(package_data["method"].as_str(), Some("GET"));
    assert!(package_data["downloadUrl"]
        .as_str()
        .is_some_and(|value| value.contains("download_packages/")));
    assert!(package_data["signedSourceUrl"]
        .as_str()
        .is_some_and(|value| value.contains("X-Amz-Signature=")));
    assert!(package_data["archiveObjectKey"]
        .as_str()
        .is_some_and(|value| value.contains("/download-packages/")));
    let items = package_data["items"]
        .as_array()
        .expect("items should be present");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["nodeId"].as_str(), Some("node-file-a"));
    assert_eq!(items[0]["archivePath"].as_str(), Some("alpha.txt"));
    assert_eq!(items[1]["nodeId"].as_str(), Some("node-file-b"));
    assert_eq!(items[1]["archivePath"].as_str(), Some("beta.txt"));

    let db_state: String = sqlx::query_scalar(
        "SELECT state FROM dr_drive_download_package WHERE tenant_id=$1 LIMIT 1",
    )
    .bind("tenant-bulk")
    .fetch_one(&pool)
    .await
    .expect("package row should be persisted");
    assert_eq!(db_state, "ready");

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(requests.iter().any(|request| {
        request.method == "GET" && request.path.ends_with("/objects/bulk/file-a.txt")
    }));
    assert!(requests.iter().any(|request| {
        request.method == "GET" && request.path.ends_with("/objects/bulk/file-b.txt")
    }));
    let archive_put = requests
        .iter()
        .find(|request| request.method == "PUT" && request.path.contains("/download-packages/"))
        .expect("archive should be written with PUT object");
    assert!(
        archive_put.body_bytes.starts_with(b"PK"),
        "archive body should be a zip file"
    );
    let archive_text = String::from_utf8_lossy(&archive_put.body_bytes);
    assert!(archive_text.contains("alpha.txt"));
    assert!(archive_text.contains("beta.txt"));
}

#[tokio::test]
async fn create_download_package_reads_objects_from_their_bound_provider_when_bucket_is_shared() {
    let (wrong_endpoint, wrong_requests) = start_s3_mock_server().await;
    let (bound_endpoint, bound_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-shared-provider', 'tenant-shared-provider', 'user', 'user-shared',
            'personal', 'Shared Provider', 'active', 1, 'user-shared', 'user-shared'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    seed_s3_provider_fixture(
        &pool,
        "provider-shared-wrong",
        "Wrong shared bucket provider",
        &wrong_endpoint,
        "bucket-s3",
        "active",
        "admin-shared",
    )
    .await;
    seed_s3_provider_fixture(
        &pool,
        "provider-shared-bound",
        "Bound shared bucket provider",
        &bound_endpoint,
        "bucket-s3",
        "active",
        "admin-shared",
    )
    .await;
    sqlx::query(
        "UPDATE dr_drive_storage_provider
         SET updated_at='2999-01-01 00:00:00'
         WHERE id='provider-shared-wrong'",
    )
    .execute(&pool)
    .await
    .expect("wrong provider should be made newest by bucket lookup");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-shared-bound', 'tenant-shared-provider', 'space-shared-provider',
            NULL, 'file', 'alpha.txt', 'ready', 'active', 1,
            'user-shared', 'user-shared'
        )",
    )
    .execute(&pool)
    .await
    .expect("file node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'upload-shared-bound', 'tenant-shared-provider', 'space-shared-provider',
            'node-shared-bound', 'bucket-s3', 'objects/bulk/file-a.txt',
            'idem-shared-bound', 'provider-shared-bound', 'storage-upload-shared-bound',
            'completed', 1800000000000, 1, 'user-shared', 'user-shared'
        )",
    )
    .execute(&pool)
    .await
    .expect("upload session should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES (
            'obj-shared-bound', 'tenant-shared-provider', 'node-shared-bound', 1,
            'provider-shared-bound', 'bucket-s3', 'objects/bulk/file-a.txt', 'text/plain', 10,
            'sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
            'active', 'user-shared', 'user-shared'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage object should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-shared-provider", "user-shared", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-shared-provider", "user-shared", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_packages")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeIds":["node-shared-bound"],
                        "packageName":"Provider-bound export",
                        "requestedTtlSeconds":180
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create package request should be handled");

    assert_eq!(response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    let package_data = common::envelope_data(&payload);
    assert_eq!(
        package_data["storageProviderId"].as_str(),
        Some("provider-shared-bound")
    );

    let wrong_requests = wrong_requests
        .lock()
        .expect("wrong provider requests mutex should not be poisoned")
        .clone();
    let bound_requests = bound_requests
        .lock()
        .expect("bound provider requests mutex should not be poisoned")
        .clone();

    assert!(
        wrong_requests.is_empty(),
        "shared bucket operations must not use an unrelated active provider endpoint"
    );
    assert!(bound_requests.iter().any(|request| {
        request.method == "GET" && request.path.ends_with("/objects/bulk/file-a.txt")
    }));
    assert!(bound_requests.iter().any(|request| {
        request.method == "PUT" && request.path.contains("/download-packages/")
    }));
}

#[tokio::test]
async fn list_archive_entries_reads_zip_contents_from_drive_storage() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_archive_fixture(&pool, &s3_endpoint)
        .await
        .expect("archive fixture should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-archive", "user-archive", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-archive", "user-archive", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-archive/archive_entries")
                .body(Body::empty())
                .expect("archive entries request should be built"),
        )
        .await
        .expect("archive entries request should be handled");

    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("archive entries response should be read"),
    )
    .expect("archive entries response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(0));
    assert!(payload["traceId"].as_str().is_some());
    let items = common::envelope_items(&payload)
        .as_array()
        .expect("archive entries should include items");
    assert_eq!(items.len(), 3);
    assert_eq!(items[0]["path"].as_str(), Some("docs/"));
    assert_eq!(items[0]["isDirectory"].as_bool(), Some(true));
    assert_eq!(items[1]["path"].as_str(), Some("docs/readme.txt"));
    assert_eq!(items[1]["name"].as_str(), Some("readme.txt"));
    assert_eq!(items[1]["isDirectory"].as_bool(), Some(false));
    assert_eq!(items[1]["uncompressedSizeBytes"].as_i64(), Some(18));
    assert_eq!(items[1]["contentType"].as_str(), Some("text/plain"));
    assert_eq!(items[2]["path"].as_str(), Some("images/logo.png"));
    assert_eq!(items[2]["contentType"].as_str(), Some("image/png"));

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(requests.iter().any(|request| {
        request.method == "GET" && request.path.ends_with("/objects/archive/report.zip")
    }));
}

#[tokio::test]
async fn extract_archive_entries_creates_drive_nodes_and_writes_objects_to_default_storage() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_archive_fixture(&pool, &s3_endpoint)
        .await
        .expect("archive fixture should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-archive", "user-archive", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-archive", "user-archive", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-archive/archive_entries/extract")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "entryPaths":["docs/readme.txt"]
                    }"#,
                ))
                .expect("archive extract request should be built"),
        )
        .await
        .expect("archive extract request should be handled");

    assert_eq!(response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("archive extract response should be read"),
    )
    .expect("archive extract response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(0));
    assert!(payload["traceId"].as_str().is_some());
    assert_eq!(
        common::envelope_field(&payload, "extractedCount").as_i64(),
        Some(1)
    );
    let items = common::envelope_items(&payload)
        .as_array()
        .expect("archive extract response should include items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["nodeName"].as_str(), Some("readme.txt"));
    assert_eq!(items[0]["nodeType"].as_str(), Some("file"));
    assert!(items[0]["id"]
        .as_str()
        .is_some_and(|value| value.starts_with("node_")));

    let docs_parent_id: String = sqlx::query_scalar(
        "SELECT id FROM dr_drive_node
         WHERE tenant_id=$1 AND space_id=$2 AND parent_node_id IS NULL
           AND node_name='docs' AND node_type='folder' AND lifecycle_status='active'",
    )
    .bind("tenant-archive")
    .bind("space-archive")
    .fetch_one(&pool)
    .await
    .expect("docs folder should be created");
    let extracted_node_id: String = sqlx::query_scalar(
        "SELECT id FROM dr_drive_node
         WHERE tenant_id=$1 AND space_id=$2 AND parent_node_id=$3
           AND node_name='readme.txt' AND node_type='file' AND content_state='ready'
           AND lifecycle_status='active'",
    )
    .bind("tenant-archive")
    .bind("space-archive")
    .bind(&docs_parent_id)
    .fetch_one(&pool)
    .await
    .expect("readme file should be created");
    assert!(extracted_node_id.starts_with("node_"));

    let object: (String, String, String, i64, String) = sqlx::query_as(
        "SELECT id, bucket, object_key, content_length, checksum_sha256_hex
         FROM dr_drive_storage_object
         WHERE tenant_id=$1 AND node_id=$2 AND lifecycle_status='active'",
    )
    .bind("tenant-archive")
    .bind(&extracted_node_id)
    .fetch_one(&pool)
    .await
    .expect("extracted file storage object should be inserted");
    assert!(object.0.starts_with("obj_"));
    assert_eq!(object.1, "bucket-archive");
    assert_rooted_standard_storage_object_key(
        &object.2,
        "sdkwork-drive/v1/tenants/tenant-archive/spaces/space-archive",
        "tenant-archive",
        "space-archive",
        &extracted_node_id,
    );
    assert_eq!(object.3, 18);
    assert_eq!(
        object.4,
        "sha256:195abb83ab055906c3b0ba13008b3663721309ad699203cf29cee3a3b3c3b038"
    );

    let extracted_version: (String, i64, String, String, Option<String>, Option<String>) =
        sqlx::query_as(
            "SELECT storage_object_id, version_no, version_kind, change_source, scene, source
             FROM dr_drive_node_version
             WHERE tenant_id=$1 AND node_id=$2 AND lifecycle_status='active'",
        )
        .bind("tenant-archive")
        .bind(&extracted_node_id)
        .fetch_one(&pool)
        .await
        .expect("archive extraction should create logical node version metadata");
    assert_eq!(extracted_version.0, object.0);
    assert_eq!(extracted_version.1, 1);
    assert_eq!(extracted_version.2, "import");
    assert_eq!(extracted_version.3, "import");
    assert_eq!(extracted_version.4.as_deref(), Some("archive_extract"));
    assert_eq!(extracted_version.5.as_deref(), Some("archive_entry"));

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    let archive_reads = requests
        .iter()
        .filter(|request| {
            request.method == "GET" && request.path.ends_with("/objects/archive/report.zip")
        })
        .count();
    assert_eq!(archive_reads, 1);
    let extracted_put = requests
        .iter()
        .find(|request| {
            request.method == "PUT"
                && request.path.starts_with("/bucket-archive/")
                && request.path.ends_with("/content")
        })
        .expect("extracted archive entry should be written to object storage");
    assert_eq!(extracted_put.body_bytes, b"hello from archive");
}

#[tokio::test]
async fn extract_archive_entries_rejects_trashed_source_before_storage_side_effects() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_archive_fixture(&pool, &s3_endpoint)
        .await
        .expect("archive fixture should be seeded");
    sqlx::query(
        "UPDATE dr_drive_node
         SET lifecycle_status='trashed', updated_by='user-archive', updated_at=CURRENT_TIMESTAMP
         WHERE tenant_id='tenant-archive' AND id='node-archive'",
    )
    .execute(&pool)
    .await
    .expect("archive node should be trashed");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-archive", "user-archive", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-archive", "user-archive", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-archive/archive_entries/extract")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "entryPaths":["docs/readme.txt"]
                    }"#,
                ))
                .expect("archive extract request should be built"),
        )
        .await
        .expect("archive extract request should be handled");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("archive extract error response should be read"),
    )
    .expect("archive extract error response should be valid json");
    assert_eq!(payload["detail"].as_str(), Some("node not found"));

    let active_created_nodes: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-archive'
           AND id != 'node-archive'
           AND lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("active created node count should be readable");
    assert_eq!(active_created_nodes, 0);

    let extracted_storage_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-archive'
           AND node_id != 'node-archive'",
    )
    .fetch_one(&pool)
    .await
    .expect("extracted storage count should be readable");
    assert_eq!(extracted_storage_count, 0);

    let change_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_change_log
         WHERE tenant_id='tenant-archive'",
    )
    .fetch_one(&pool)
    .await
    .expect("change count should be readable");
    assert_eq!(change_count, 0);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(requests.is_empty());
}

#[tokio::test]
async fn extract_archive_entries_auto_renames_file_conflicts_and_completes_atomically() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_archive_fixture(&pool, &s3_endpoint)
        .await
        .expect("archive fixture should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'folder-images-existing', 'tenant-archive', 'space-archive',
            NULL, 'folder', 'images', 'empty', 'active', 1,
            'user-archive', 'user-archive'
        )",
    )
    .execute(&pool)
    .await
    .expect("existing images folder should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'file-logo-existing', 'tenant-archive', 'space-archive',
            'folder-images-existing', 'file', 'logo.png', 'ready', 'active', 1,
            'user-archive', 'user-archive'
        )",
    )
    .execute(&pool)
    .await
    .expect("existing conflicting logo file should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-archive", "user-archive", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-archive", "user-archive", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-archive/archive_entries/extract")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "entryPaths":["docs/readme.txt","images/logo.png"]
                    }"#,
                ))
                .expect("archive extract conflict request should be built"),
        )
        .await
        .expect("archive extract request should be handled");

    assert_eq!(response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("archive extract response should be read"),
    )
    .expect("archive extract response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(0));
    assert_eq!(
        common::envelope_field(&payload, "extractedCount").as_i64(),
        Some(2)
    );

    let readme_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-archive'
           AND node_name='readme.txt'
           AND lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("readme count should be readable");
    assert_eq!(readme_count, 1);

    let renamed_logo_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-archive'
           AND node_name='logo (1).png'
           AND lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("renamed logo count should be readable");
    assert_eq!(renamed_logo_count, 1);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    let archive_reads = requests
        .iter()
        .filter(|request| {
            request.method == "GET" && request.path.ends_with("/objects/archive/report.zip")
        })
        .count();
    assert_eq!(archive_reads, 1);
    assert!(requests.iter().any(|request| {
        request.method == "PUT"
            && request.path.starts_with("/bucket-archive/")
            && request.path.ends_with("/content")
    }));
}

#[tokio::test]
async fn create_download_package_rejects_ttl_outside_contract_before_writing_package() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_download_package_fixture(&pool, &s3_endpoint)
        .await
        .expect("download package fixture should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-bulk", "user-bulk", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-bulk", "user-bulk", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_packages")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeIds":["node-file-a","node-file-b"],
                        "packageName":"Invalid TTL",
                        "requestedTtlSeconds":0
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create package request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    assert_eq!(payload["code"], 40001);
    assert!(payload["detail"]
        .as_str()
        .expect("detail should be present")
        .contains("requestedTtlSeconds"));

    let package_count: i64 =
        sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_download_package WHERE tenant_id=$1")
            .bind("tenant-bulk")
            .fetch_one(&pool)
            .await
            .expect("download package count should be readable");
    assert_eq!(package_count, 0);
    assert!(
        captured_requests
            .lock()
            .expect("captured s3 requests mutex should not be poisoned")
            .is_empty(),
        "invalid TTL should fail before reading or writing object storage"
    );
}

#[tokio::test]
async fn create_download_package_reads_files_from_multiple_storage_buckets() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_download_package_fixture(&pool, &s3_endpoint)
        .await
        .expect("download package fixture should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-bulk-alt', 's3_compatible', 'Bulk S3 Alt', $1, 'us-east-1',
            'bucket-s3-alt', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'user-bulk', 'user-bulk'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("alternate storage provider should be seeded");
    sqlx::query(
        "UPDATE dr_drive_storage_object
         SET bucket='bucket-s3-alt'
         WHERE tenant_id='tenant-bulk' AND node_id='node-file-b'",
    )
    .execute(&pool)
    .await
    .expect("file b should move to alternate bucket");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-bulk", "user-bulk", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-bulk", "user-bulk", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_packages")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeIds":["node-file-a","node-file-b"],
                        "packageName":"Cross bucket export"
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create package request should be handled");
    assert_eq!(response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    let package_data = common::envelope_data(&payload);
    assert_eq!(package_data["state"].as_str(), Some("ready"));
    assert_eq!(package_data["fileCount"].as_i64(), Some(2));
    assert_eq!(
        package_data["items"][0]["bucket"].as_str(),
        Some("bucket-s3")
    );
    assert_eq!(
        package_data["items"][1]["bucket"].as_str(),
        Some("bucket-s3-alt")
    );

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(requests.iter().any(|request| {
        request.method == "GET" && request.path.ends_with("/bucket-s3/objects/bulk/file-a.txt")
    }));
    assert!(requests.iter().any(|request| {
        request.method == "GET"
            && request
                .path
                .ends_with("/bucket-s3-alt/objects/bulk/file-b.txt")
    }));
    assert!(requests.iter().any(|request| {
        request.method == "PUT"
            && request.path.starts_with("/bucket-s3/")
            && request.path.contains("/download-packages/")
    }));
}

#[tokio::test]
async fn create_download_package_rejects_empty_node_selection() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-bulk", "user-bulk", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-bulk", "user-bulk", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_packages")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeIds":[]
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create package request should be handled");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    assert_eq!(payload["code"].as_i64(), Some(40001));
    assert!(payload["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("nodeIds")));
}

#[tokio::test]
async fn create_download_package_rejects_trashed_selected_node_before_reading_objects() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_download_package_fixture(&pool, &s3_endpoint)
        .await
        .expect("download package fixture should be seeded");
    sqlx::query(
        "UPDATE dr_drive_node
         SET lifecycle_status='trashed'
         WHERE tenant_id='tenant-bulk' AND id='node-file-a'",
    )
    .execute(&pool)
    .await
    .expect("selected node should be moved to trash");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-bulk", "user-bulk", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-bulk", "user-bulk", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_packages")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeIds":["node-file-a"],
                        "packageName":"Trashed export"
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create package request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    assert_eq!(payload["code"].as_i64(), Some(40001));
    assert!(payload["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("active files or folders")));
    assert!(
        captured_requests
            .lock()
            .expect("captured s3 requests mutex should not be poisoned")
            .is_empty(),
        "inactive node selection must fail before reading or writing object storage"
    );
}

#[tokio::test]
async fn create_download_package_rejects_folder_expansion_above_file_limit_before_reading_objects()
{
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-package-limit', 'tenant-package-limit', 'user', 'user-bulk', 'team', 'Package Limit', 'active', 1, 'user-bulk', 'user-bulk')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-package-limit', 's3_compatible', 'Bulk S3', $1, 'us-east-1',
            'bucket-package-limit', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'user-bulk', 'user-bulk'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('folder-package-limit', 'tenant-package-limit', 'space-package-limit', NULL, 'folder', 'Large Folder', 'empty', 'active', 1, 'user-bulk', 'user-bulk')",
    )
    .execute(&pool)
    .await
    .expect("folder should be seeded");

    for index in 0..501 {
        let node_id = format!("node-package-limit-{index:03}");
        let object_id = format!("obj-package-limit-{index:03}");
        let object_key = format!("objects/package-limit/{index:03}.txt");
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-package-limit', 'space-package-limit', 'folder-package-limit',
                'file', $2, 'ready', 'active', 1, 'user-bulk', 'user-bulk')",
        )
        .bind(&node_id)
        .bind(format!("{index:03}.txt"))
        .execute(&pool)
        .await
        .expect("file node should be seeded");
        seed_storage_object_fixture(
            &pool,
            StorageObjectFixture {
                object_id: &object_id,
                tenant_id: "tenant-package-limit",
                node_id: &node_id,
                version_no: 1,
                provider_id: "provider-package-limit",
                bucket: "bucket-package-limit",
                object_key: &object_key,
                content_type: "text/plain",
                content_length: 1,
                checksum_sha256_hex: &format!("sha256:{index:064x}"),
                lifecycle_status: "active",
                actor_id: "user-bulk",
            },
        )
        .await
        .expect("storage object should be seeded");
    }

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-package-limit", "user-bulk", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-package-limit", "user-bulk", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_packages")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeIds":["folder-package-limit"],
                        "packageName":"Too Large"
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create package request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    assert_eq!(payload["code"].as_i64(), Some(40001));
    assert!(payload["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("at most 500 files")));

    let object_reads = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .iter()
        .filter(|request| request.method == "GET")
        .count();
    assert_eq!(object_reads, 0);
}

#[tokio::test]
async fn create_download_package_expands_selected_folder_descendants() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_download_package_fixture(&pool, &s3_endpoint)
        .await
        .expect("download package fixture should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-bulk", "user-bulk", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-bulk", "user-bulk", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_packages")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeIds":["node-folder"],
                        "packageName":"Folder export"
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create package request should be handled");
    assert_eq!(response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    let package_data = common::envelope_data(&payload);
    assert_eq!(package_data["fileCount"].as_i64(), Some(1));
    assert_eq!(package_data["totalBytes"].as_i64(), Some(12));
    assert_eq!(
        package_data["items"][0]["archivePath"].as_str(),
        Some("Project/folder-child.txt")
    );

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    let archive_put = requests
        .iter()
        .find(|request| request.method == "PUT" && request.path.contains("/download-packages/"))
        .expect("archive should be written");
    let archive_text = String::from_utf8_lossy(&archive_put.body_bytes);
    assert!(archive_text.contains("Project/folder-child.txt"));
}

#[tokio::test]
async fn resolve_download_package_treats_subsecond_remaining_ttl_as_expired() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_download_package_fixture(&pool, &s3_endpoint)
        .await
        .expect("download package fixture should be seeded");

    let expires_at_epoch_ms = current_epoch_ms() + 500;
    let items = serde_json::to_string(&vec![serde_json::json!({
        "nodeId": "node-file-a",
        "nodeName": "alpha.txt",
        "archivePath": "alpha.txt",
        "storageProviderId": "provider-bulk",
        "bucket": "bucket-s3",
        "objectKey": "objects/bulk/file-a.txt",
        "contentType": "text/plain",
        "contentLength": 10,
        "checksumSha256Hex": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    })])
    .expect("manifest should serialize");
    sqlx::query(
        "INSERT INTO dr_drive_download_package (
            id, tenant_id, package_name, state, storage_provider_id, bucket,
            archive_object_key, content_type, file_count, total_bytes,
            archive_size_bytes, requested_node_ids_json, item_manifest_json,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'package-subsecond', 'tenant-bulk', 'Almost expired', 'ready',
            'provider-bulk', 'bucket-s3',
            'sdkwork-drive/v1/t/te/tenants/tenant-bulk/download-packages/package-subsecond/archive.zip',
            'application/zip', 1, 10, 200, '[\"node-file-a\"]', $1, $2,
            1, 'user-bulk', 'user-bulk'
        )",
    )
    .bind(items)
    .bind(expires_at_epoch_ms)
    .execute(&pool)
    .await
    .expect("download package should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-bulk", "user-bulk", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-bulk", "user-bulk", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/download_packages/package-subsecond/download_url")
                .body(Body::empty())
                .expect("resolve package request should be built"),
        )
        .await
        .expect("resolve package request should be handled");

    assert_eq!(response.status(), StatusCode::GONE);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("error body should be read"),
    )
    .expect("error response should be valid json");
    assert_eq!(payload["code"], 41001);
    assert!(
        captured_requests
            .lock()
            .expect("captured s3 requests mutex should not be poisoned")
            .is_empty(),
        "subsecond TTL should fail before signing against object storage"
    );
}

#[tokio::test]
async fn resolve_expired_download_token_returns_gone() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = common::test_router_with_pool(pool);
    let now_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_millis() as i64;
    let token = build_download_token("tenant-001", "node-001", now_epoch_ms - 1_000);

    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::GET)
                .uri(format!("/app/v3/api/drive/download_tokens/{token}"))
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("expired token request should be handled");
    assert_eq!(response.status(), StatusCode::GONE);
}

#[tokio::test]
async fn resolve_download_token_rejects_node_after_it_is_moved_to_trash() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-token-lifecycle', 'tenant-token-lifecycle', 'user', 'user-token-lifecycle', 'personal', 'Token Lifecycle', 'active', 1, 'user-token-lifecycle', 'user-token-lifecycle')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-token-lifecycle', 'tenant-token-lifecycle', 'space-token-lifecycle',
            NULL, 'file', 'token.bin', 'ready', 'active', 1,
            'user-token-lifecycle', 'user-token-lifecycle'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-token-lifecycle",
        "Token S3",
        "http://127.0.0.1:9000",
        "bucket-token-lifecycle",
        "active",
        "user-token-lifecycle",
    )
    .await;
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "obj-token-lifecycle",
            tenant_id: "tenant-token-lifecycle",
            node_id: "node-token-lifecycle",
            version_no: 1,
            provider_id: "provider-token-lifecycle",
            bucket: "bucket-token-lifecycle",
            object_key: "objects/token-lifecycle.bin",
            content_type: "application/octet-stream",
            content_length: 128,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-token-lifecycle",
        },
    )
    .await
    .expect("storage object should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-token-lifecycle",
                            "user-token-lifecycle",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token(
                        "tenant-token-lifecycle",
                        "user-token-lifecycle",
                        "appbase",
                    ),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_urls")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeId":"node-token-lifecycle",
                        "requestedTtlSeconds":120
                    }"#,
                ))
                .expect("create download url request should be built"),
        )
        .await
        .expect("create download url request should be handled");
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("create response body should be read"),
    )
    .expect("create response should be valid json");
    let token = common::envelope_data(&create_payload)["downloadUrl"]
        .as_str()
        .expect("downloadUrl should be present")
        .rsplit('/')
        .next()
        .expect("token should be encoded in path segment")
        .to_string();

    sqlx::query(
        "UPDATE dr_drive_node
         SET lifecycle_status='trashed'
         WHERE tenant_id='tenant-token-lifecycle' AND id='node-token-lifecycle'",
    )
    .execute(&pool)
    .await
    .expect("node should be moved to trash");

    let resolve_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-token-lifecycle",
                            "user-token-lifecycle",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token(
                        "tenant-token-lifecycle",
                        "user-token-lifecycle",
                        "appbase",
                    ),
                )
                .method(Method::GET)
                .uri(format!("/app/v3/api/drive/download_tokens/{token}"))
                .body(Body::empty())
                .expect("resolve token request should be built"),
        )
        .await
        .expect("resolve token request should be handled");

    assert_eq!(resolve_response.status(), StatusCode::NOT_FOUND);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(resolve_response.into_body(), usize::MAX)
            .await
            .expect("resolve error body should be read"),
    )
    .expect("resolve error response should be valid json");
    assert_eq!(payload["code"], 40401);
}

#[tokio::test]
async fn resolve_download_token_treats_subsecond_remaining_ttl_as_expired() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-token-subsecond', 'tenant-token-subsecond', 'user', 'user-token-subsecond', 'personal', 'TTL', 'active', 1, 'user-token-subsecond', 'user-token-subsecond')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-token-subsecond', 'tenant-token-subsecond', 'space-token-subsecond', NULL, 'file', 'ttl.bin', 'ready', 'active', 1, 'user-token-subsecond', 'user-token-subsecond')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-token-subsecond",
        "TTL S3",
        "http://127.0.0.1:9000",
        "bucket-token-subsecond",
        "active",
        "user-token-subsecond",
    )
    .await;
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "obj-token-subsecond",
            tenant_id: "tenant-token-subsecond",
            node_id: "node-token-subsecond",
            version_no: 1,
            provider_id: "provider-token-subsecond",
            bucket: "bucket-token-subsecond",
            object_key: "objects/token-subsecond.bin",
            content_type: "application/octet-stream",
            content_length: 128,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-token-subsecond",
        },
    )
    .await
    .expect("storage object should be seeded");

    let token = build_download_token(
        "tenant-token-subsecond",
        "node-token-subsecond",
        current_epoch_ms() + 500,
    );
    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-token-subsecond",
                            "user-token-subsecond",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token(
                        "tenant-token-subsecond",
                        "user-token-subsecond",
                        "appbase",
                    ),
                )
                .method(Method::GET)
                .uri(format!("/app/v3/api/drive/download_tokens/{token}"))
                .body(Body::empty())
                .expect("resolve token request should be built"),
        )
        .await
        .expect("resolve token request should be handled");

    assert_eq!(response.status(), StatusCode::GONE);
}

fn current_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_millis() as i64
}

async fn seed_storage_metadata_provider_fixture(
    pool: &sqlx::PgPool,
    provider_id: &str,
    bucket: &str,
    actor_id: &str,
) {
    sqlx::query(
        "INSERT OR IGNORE INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            $1, 's3_compatible', $1, 'https://s3.fixture.local', 'us-east-1',
            $2, 1, 1, 'plain:test-access-key:test-secret-key', 'AES256',
            'STANDARD', 'active', 1, $3, $3
        )",
    )
    .bind(provider_id)
    .bind(bucket)
    .bind(actor_id)
    .execute(pool)
    .await
    .expect("storage metadata provider should be seeded");
}

struct StorageObjectFixture<'a> {
    object_id: &'a str,
    tenant_id: &'a str,
    node_id: &'a str,
    version_no: i64,
    provider_id: &'a str,
    bucket: &'a str,
    object_key: &'a str,
    content_type: &'a str,
    content_length: i64,
    checksum_sha256_hex: &'a str,
    lifecycle_status: &'a str,
    actor_id: &'a str,
}

async fn seed_storage_object_fixture(
    pool: &sqlx::PgPool,
    fixture: StorageObjectFixture<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12)",
    )
    .bind(fixture.object_id)
    .bind(fixture.tenant_id)
    .bind(fixture.node_id)
    .bind(fixture.version_no)
    .bind(fixture.provider_id)
    .bind(fixture.bucket)
    .bind(fixture.object_key)
    .bind(fixture.content_type)
    .bind(fixture.content_length)
    .bind(fixture.checksum_sha256_hex)
    .bind(fixture.lifecycle_status)
    .bind(fixture.actor_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn seed_s3_provider_fixture(
    pool: &sqlx::PgPool,
    provider_id: &str,
    provider_name: &str,
    endpoint_url: &str,
    bucket: &str,
    status: &str,
    actor_id: &str,
) {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            $1, 's3_compatible', $2, $3, 'us-east-1',
            $4, 1, $5, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', $6, 1, $7, $7
        )",
    )
    .bind(provider_id)
    .bind(provider_name)
    .bind(endpoint_url)
    .bind(bucket)
    .bind(
        !endpoint_url
            .trim()
            .to_ascii_lowercase()
            .starts_with("http://"),
    )
    .bind(status)
    .bind(actor_id)
    .execute(pool)
    .await
    .expect("storage provider should be seeded");
}

struct ObjectStoreProviderFixture<'a> {
    provider_id: &'a str,
    provider_kind: &'a str,
    provider_name: &'a str,
    endpoint_url: &'a str,
    region: &'a str,
    bucket: &'a str,
    path_style: bool,
    actor_id: &'a str,
}

async fn seed_object_store_provider_fixture(
    pool: &sqlx::PgPool,
    fixture: ObjectStoreProviderFixture<'_>,
) {
    sqlx::query(
        "INSERT OR IGNORE INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8,
            'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD',
            'active', 1, $9, $9
        )",
    )
    .bind(fixture.provider_id)
    .bind(fixture.provider_kind)
    .bind(fixture.provider_name)
    .bind(fixture.endpoint_url)
    .bind(fixture.region)
    .bind(fixture.bucket)
    .bind(fixture.path_style)
    .bind(
        !fixture
            .endpoint_url
            .trim()
            .to_ascii_lowercase()
            .starts_with("http://"),
    )
    .bind(fixture.actor_id)
    .execute(pool)
    .await
    .expect("object store provider should be seeded");
}

async fn seed_download_package_fixture(
    pool: &sqlx::PgPool,
    s3_endpoint: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-bulk', 'tenant-bulk', 'user', 'user-bulk', 'personal', 'Bulk', 'active', 1, 'user-bulk', 'user-bulk')",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-bulk', 's3_compatible', 'Bulk S3', $1, 'us-east-1',
            'bucket-s3', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'user-bulk', 'user-bulk'
        )",
    )
    .bind(s3_endpoint)
    .execute(pool)
    .await?;
    for (id, parent_id, node_type, node_name) in [
        ("node-file-a", None, "file", "alpha.txt"),
        ("node-file-b", None, "file", "beta.txt"),
        ("node-folder", None, "folder", "Project"),
        (
            "node-folder-child",
            Some("node-folder"),
            "file",
            "folder-child.txt",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-bulk', 'space-bulk', $2, $3, $4, 'ready', 'active', 1, 'user-bulk', 'user-bulk')",
        )
        .bind(id)
        .bind(parent_id)
        .bind(node_type)
        .bind(node_name)
        .execute(pool)
        .await?;
    }
    for (id, node_id, object_key, size, checksum) in [
        (
            "obj-bulk-a",
            "node-file-a",
            "objects/bulk/file-a.txt",
            10_i64,
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ),
        (
            "obj-bulk-b",
            "node-file-b",
            "objects/bulk/file-b.txt",
            9_i64,
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ),
        (
            "obj-bulk-folder-child",
            "node-folder-child",
            "objects/bulk/folder-child.txt",
            12_i64,
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_storage_object (
                id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
                content_type, content_length, checksum_sha256_hex, lifecycle_status,
                created_by, updated_by
            ) VALUES ($1, 'tenant-bulk', $2, 1, 'provider-bulk', 'bucket-s3', $3, 'text/plain', $4, $5, 'active', 'user-bulk', 'user-bulk')",
        )
        .bind(id)
        .bind(node_id)
        .bind(object_key)
        .bind(size)
        .bind(checksum)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn seed_archive_fixture(pool: &sqlx::PgPool, s3_endpoint: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-archive', 'tenant-archive', 'user', 'user-archive', 'personal', 'Archive', 'active', 1, 'user-archive', 'user-archive')",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-archive', 's3_compatible', 'Archive S3', $1, 'us-east-1',
            'bucket-archive', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'user-archive', 'user-archive'
        )",
    )
    .bind(s3_endpoint)
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'binding-archive-primary', 'tenant-archive', 'space-archive',
            'provider-archive', 'space', 'primary',
            'sdkwork-drive/v1/tenants/tenant-archive/spaces/space-archive',
            'active', 1, 'user-archive', 'user-archive'
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-archive', 'tenant-archive', 'space-archive', NULL, 'file',
            'report.zip', 'ready', 'active', 1, 'user-archive', 'user-archive'
        )",
    )
    .execute(pool)
    .await?;
    seed_storage_object_fixture(
        pool,
        StorageObjectFixture {
            object_id: "obj-archive",
            tenant_id: "tenant-archive",
            node_id: "node-archive",
            version_no: 1,
            provider_id: "provider-archive",
            bucket: "bucket-archive",
            object_key: "objects/archive/report.zip",
            content_type: "application/zip",
            content_length: 512,
            checksum_sha256_hex:
                "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            lifecycle_status: "active",
            actor_id: "user-archive",
        },
    )
    .await?;
    Ok(())
}

fn build_download_token(tenant_id: &str, node_id: &str, expires_at_epoch_ms: i64) -> String {
    sdkwork_drive_workspace_service::application::download_service::build_download_token(
        tenant_id,
        node_id,
        expires_at_epoch_ms,
    )
    .expect("download token should be signed")
}

async fn fetch_paged_items_as(
    app: axum::Router,
    uri: &str,
    tenant: &str,
    user: &str,
) -> (Vec<serde_json::Value>, Option<String>) {
    let response = app
        .clone()
        .oneshot(common::authed_get(uri, tenant, user, "appbase"))
        .await
        .expect("paged request should be handled");
    assert_eq!(response.status(), StatusCode::OK, "{uri} should return OK");
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("paged response should be read"),
    )
    .expect("paged response should be valid json");
    let items = common::envelope_items(&payload)
        .as_array()
        .expect("items should be an array")
        .clone();
    let next_page_token = common::envelope_next_page_token(&payload);
    (items, next_page_token)
}

async fn fetch_paged_items(
    app: axum::Router,
    uri: &str,
    tenant: &str,
) -> (Vec<serde_json::Value>, Option<String>) {
    let user = common::user_from_uri(uri, tenant);
    fetch_paged_items_as(app, uri, tenant, &user).await
}

async fn fetch_json_as(
    app: axum::Router,
    uri: &str,
    tenant: &str,
    user: &str,
) -> serde_json::Value {
    let response = app
        .oneshot(common::authed_get(uri, tenant, user, "appbase"))
        .await
        .expect("json request should be handled");
    assert_eq!(response.status(), StatusCode::OK, "{uri} should return OK");
    serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("json response should be read"),
    )
    .expect("json response should be valid")
}

async fn fetch_resource_json_as(
    app: axum::Router,
    uri: &str,
    tenant: &str,
    user: &str,
) -> serde_json::Value {
    let payload = fetch_json_as(app, uri, tenant, user).await;
    common::envelope_body(&payload).clone()
}

async fn assert_git_repository_root_directory_error(response: Response) {
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("git repository root validation response should be read"),
    )
    .expect("git repository root validation response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40001));
    assert!(
        payload["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("git repository space root accepts only repository directories"),
        "unexpected git repository root validation detail: {payload}"
    );
}

#[tokio::test]
async fn create_file_rejects_existing_node_id_before_storage_side_effects() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-file-conflict', 'tenant-file-conflict', 'user', 'user-file',
            'personal', 'File Conflict', 'active', 1, 'user-file', 'user-file'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-file-conflict",
        "File Conflict S3",
        &s3_endpoint,
        "bucket-file-conflict",
        "active",
        "user-file",
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'default:space:tenant-file-conflict:space-file-conflict',
            'tenant-file-conflict', 'space-file-conflict',
            'provider-file-conflict', 'space', 'primary',
            'sdkwork-drive/v1/tenants/tenant-file-conflict/spaces/space-file-conflict',
            'active', 1, 'user-file', 'user-file'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage provider binding should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'file-conflict', 'tenant-file-conflict', 'space-file-conflict',
            NULL, 'file', 'existing.pdf', 'ready', 'active', 1,
            'user-file', 'user-file'
        )",
    )
    .execute(&pool)
    .await
    .expect("existing node should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-file-conflict", "user-file", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-file-conflict", "user-file", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/files")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"file-conflict",
                        "spaceId":"space-file-conflict",
                        "nodeName":"different.pdf",
                        "uploadSessionId":"upload-file-conflict",
                        "idempotencyKey":"idem-file-conflict",
                        "expiresAtEpochMs":1800000000000
                    }"#,
                ))
                .expect("create conflicting file request should be built"),
        )
        .await
        .expect("create conflicting file request should be handled");

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("conflict response should be read"),
    )
    .expect("conflict response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40901));

    let upload_session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_upload_session
         WHERE tenant_id='tenant-file-conflict'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload session count should be readable");
    assert_eq!(upload_session_count, 0);

    let change_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_change_log
         WHERE tenant_id='tenant-file-conflict'",
    )
    .fetch_one(&pool)
    .await
    .expect("change count should be readable");
    assert_eq!(change_count, 0);

    let existing_name: String = sqlx::query_scalar(
        "SELECT node_name
         FROM dr_drive_node
         WHERE tenant_id='tenant-file-conflict' AND id='file-conflict'",
    )
    .fetch_one(&pool)
    .await
    .expect("existing node should remain readable");
    assert_eq!(existing_name, "existing.pdf");

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(requests.is_empty());
}

#[tokio::test]
async fn create_file_rejects_past_expiration_before_storage_side_effects() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-file-expired', 'tenant-file-expired', 'user', 'user-file',
            'personal', 'File Expired', 'active', 1, 'user-file', 'user-file'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-file-expired",
        "File Expired S3",
        &s3_endpoint,
        "bucket-file-expired",
        "active",
        "user-file",
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'default:space:tenant-file-expired:space-file-expired',
            'tenant-file-expired', 'space-file-expired',
            'provider-file-expired', 'space', 'primary',
            'sdkwork-drive/v1/tenants/tenant-file-expired/spaces/space-file-expired',
            'active', 1, 'user-file', 'user-file'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage provider binding should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-file-expired", "user-file", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-file-expired", "user-file", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/files")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"file-expired",
                        "spaceId":"space-file-expired",
                        "nodeName":"expired.pdf",
                        "uploadSessionId":"upload-file-expired",
                        "idempotencyKey":"idem-file-expired",
                        "expiresAtEpochMs":1
                    }"#,
                ))
                .expect("create expired file request should be built"),
        )
        .await
        .expect("create expired file request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("expired file response should be read"),
    )
    .expect("expired file response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40001));
    assert!(payload["detail"]
        .as_str()
        .unwrap_or_default()
        .contains("expiresAtEpochMs must be in the future"));

    let node_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-file-expired'",
    )
    .fetch_one(&pool)
    .await
    .expect("node count should be readable");
    assert_eq!(node_count, 0);

    let upload_session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_upload_session
         WHERE tenant_id='tenant-file-expired'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload session count should be readable");
    assert_eq!(upload_session_count, 0);

    let change_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_change_log
         WHERE tenant_id='tenant-file-expired'",
    )
    .fetch_one(&pool)
    .await
    .expect("change count should be readable");
    assert_eq!(change_count, 0);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(requests.is_empty());
}

#[tokio::test]
async fn create_file_aborts_multipart_when_atomic_website_root_rejects_publication() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-file-atomic', 'tenant-file-atomic', 'user', 'user-file-atomic',
            'website', 'Atomic Website', 'active', 1,
            'user-file-atomic', 'user-file-atomic'
        )",
    )
    .execute(&pool)
    .await
    .expect("website space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, space_type, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'atomic-root-node', 'tenant-file-atomic', 'space-file-atomic', 'website',
            NULL, 'folder', '__website_root__', 'ready', 'active', 1,
            'user-file-atomic', 'user-file-atomic'
        )",
    )
    .execute(&pool)
    .await
    .expect("atomic root node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_website_root (
            id, uuid, tenant_id, space_id, root_key, display_name,
            source_root_mode, selected_folder_node_id, selector_key, content_mode,
            active_node_id, active_generation, root_status, last_switch_by,
            version, created_by, updated_by
        ) VALUES (
            'atomic-root', 'atomic-root-uuid', 'tenant-file-atomic', 'space-file-atomic',
            'default', 'Default', 'space_root', NULL, 'space_root', 'atomic_generation',
            'atomic-root-node', 1, 'active', 'user-file-atomic', 1,
            'user-file-atomic', 'user-file-atomic'
        )",
    )
    .execute(&pool)
    .await
    .expect("atomic WebsiteRoot should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-file-atomic",
        "Atomic Website S3",
        &s3_endpoint,
        "bucket-file-atomic",
        "active",
        "user-file-atomic",
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'default:space:tenant-file-atomic:space-file-atomic',
            'tenant-file-atomic', 'space-file-atomic',
            'provider-file-atomic', 'space', 'primary',
            'sdkwork-drive/v1/tenants/tenant-file-atomic/spaces/space-file-atomic',
            'active', 1, 'user-file-atomic', 'user-file-atomic'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage provider binding should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-file-atomic", "user-file-atomic", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-file-atomic", "user-file-atomic", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/files")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"file-atomic-rejected",
                        "spaceId":"space-file-atomic",
                        "nodeName":"rejected.html",
                        "uploadSessionId":"upload-file-atomic-rejected",
                        "idempotencyKey":"idem-file-atomic-rejected",
                        "expiresAtEpochMs":4102444800000
                    }"#,
                ))
                .expect("atomic website create file request should be built"),
        )
        .await
        .expect("atomic website create file request should be handled");
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let rejected_node_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-file-atomic' AND id='file-atomic-rejected'",
    )
    .fetch_one(&pool)
    .await
    .expect("rejected node count should be readable");
    let upload_session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_upload_session
         WHERE tenant_id='tenant-file-atomic'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload session count should be readable");
    assert_eq!(rejected_node_count, 0);
    assert_eq!(upload_session_count, 0);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    let multipart_create_count = requests
        .iter()
        .filter(|request| request.method == "POST" && request.query.contains("uploads"))
        .count();
    let multipart_abort_count = requests
        .iter()
        .filter(|request| {
            request.method == "DELETE" && request.query.contains("uploadId=mock-s3-upload-id")
        })
        .count();
    assert_eq!(multipart_create_count, 1);
    assert_eq!(multipart_abort_count, 1);
}

#[tokio::test]
async fn create_file_route_is_idempotent_without_repeating_storage_or_changes() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-file-idem', 'tenant-file-idem', 'user', 'user-file-idem',
            'personal', 'File Idempotency', 'active', 1,
            'user-file-idem', 'user-file-idem'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    seed_s3_provider_fixture(
        &pool,
        "provider-file-idem",
        "File Idempotency S3",
        &s3_endpoint,
        "bucket-file-idem",
        "active",
        "user-file-idem",
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'default:space:tenant-file-idem:space-file-idem',
            'tenant-file-idem', 'space-file-idem',
            'provider-file-idem', 'space', 'primary',
            'sdkwork-drive/v1/tenants/tenant-file-idem/spaces/space-file-idem',
            'active', 1, 'user-file-idem', 'user-file-idem'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage provider binding should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let body = r#"{
        "id":"file-idem",
                "spaceId":"space-file-idem",
        "nodeName":"idempotent.pdf",
        "uploadSessionId":"upload-file-idem",
        "idempotencyKey":"idem-file-idem",
        "expiresAtEpochMs":1800000000000
    }"#;

    let first_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-file-idem", "user-file-idem", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-file-idem", "user-file-idem", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/files")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .expect("first create file request should be built"),
        )
        .await
        .expect("first create file request should be handled");
    assert_eq!(first_response.status(), StatusCode::CREATED);
    let first_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(first_response.into_body(), usize::MAX)
            .await
            .expect("first response body should be read"),
    )
    .expect("first response json should be valid");

    let second_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-file-idem", "user-file-idem", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-file-idem", "user-file-idem", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/files")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .expect("second create file request should be built"),
        )
        .await
        .expect("second create file request should be handled");
    assert_eq!(second_response.status(), StatusCode::CREATED);
    let second_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(second_response.into_body(), usize::MAX)
            .await
            .expect("second response body should be read"),
    )
    .expect("second response json should be valid");

    assert_eq!(first_payload["node"]["id"], second_payload["node"]["id"]);
    assert_eq!(
        common::envelope_data(&first_payload)["uploadSession"]["id"],
        common::envelope_data(&second_payload)["uploadSession"]["id"]
    );
    assert_eq!(
        common::envelope_data(&first_payload)["uploadSession"]["storageUploadId"],
        common::envelope_data(&second_payload)["uploadSession"]["storageUploadId"]
    );

    let node_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-file-idem'",
    )
    .fetch_one(&pool)
    .await
    .expect("node count should be readable");
    assert_eq!(node_count, 1);

    let upload_session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_upload_session
         WHERE tenant_id='tenant-file-idem'",
    )
    .fetch_one(&pool)
    .await
    .expect("upload session count should be readable");
    assert_eq!(upload_session_count, 1);

    let change_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_change_log
         WHERE tenant_id='tenant-file-idem'
           AND event_type='drive.node.created'",
    )
    .fetch_one(&pool)
    .await
    .expect("change count should be readable");
    assert_eq!(change_count, 1);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    let multipart_create_count = requests
        .iter()
        .filter(|request| request.method == "POST" && request.query.contains("uploads"))
        .count();
    assert_eq!(multipart_create_count, 1);
}

#[tokio::test]
async fn app_drive_professional_file_create_upload_status_and_empty_trash_routes() {
    let (s3_endpoint, _captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-pro', 'tenant-pro', 'user', 'user-pro', 'personal', 'Professional', 'active', 1, 'user-pro', 'user-pro')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-pro', 's3_compatible', 'Primary S3', $1, 'us-east-1', 'bucket-pro', 1, 0, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-pro', 'admin-pro')",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES ('default:space:tenant-pro:space-pro', 'tenant-pro', 'space-pro', 'provider-pro', 'space', 'primary', 'sdkwork-drive/v1/tenants/tenant-pro/spaces/space-pro', 'active', 1, 'admin-pro', 'admin-pro')",
    )
    .execute(&pool)
    .await
    .expect("storage provider binding should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let create_file_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-pro", "user-pro", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-pro", "user-pro", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/files")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"file-pro",
                        "spaceId":"space-pro",
                        "nodeName":"proposal.pdf",
                        "uploadSessionId":"upload-pro",
                        "idempotencyKey":"idem-pro",
                        "expiresAtEpochMs":1800000000000
                    }"#,
                ))
                .expect("create file request should be built"),
        )
        .await
        .expect("create file request should be handled");
    assert_eq!(create_file_response.status(), StatusCode::CREATED);
    let create_file_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(create_file_response.into_body(), usize::MAX)
            .await
            .expect("create file response body should be readable"),
    )
    .expect("create file response body should be valid json");
    let create_file_data = common::envelope_data(&create_file_payload);
    assert_eq!(create_file_data["node"]["id"].as_str(), Some("file-pro"));
    assert_eq!(create_file_data["node"]["nodeType"].as_str(), Some("file"));
    assert_eq!(
        create_file_data["uploadSession"]["bucket"].as_str(),
        Some("bucket-pro")
    );
    assert_eq!(
        create_file_data["uploadSession"]["state"].as_str(),
        Some("created")
    );

    let get_upload_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-pro", "user-pro", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-pro", "user-pro", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/upload_sessions/upload-pro")
                .body(Body::empty())
                .expect("get upload session request should be built"),
        )
        .await
        .expect("get upload session request should be handled");
    assert_eq!(get_upload_response.status(), StatusCode::OK);
    let upload_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(get_upload_response.into_body(), usize::MAX)
            .await
            .expect("get upload response body should be readable"),
    )
    .expect("get upload response body should be valid json");
    let upload_item = common::envelope_body(&upload_payload);
    assert_eq!(upload_item["id"].as_str(), Some("upload-pro"));
    assert_rooted_standard_storage_object_key(
        upload_item["objectKey"]
            .as_str()
            .expect("upload objectKey should be present"),
        "sdkwork-drive/v1/tenants/tenant-pro/spaces/space-pro",
        "tenant-pro",
        "space-pro",
        "file-pro",
    );

    let trash_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-pro", "user-pro", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-pro", "user-pro", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/file-pro/trash")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("trash file request should be built"),
        )
        .await
        .expect("trash file request should be handled");
    assert_eq!(trash_response.status(), StatusCode::CREATED);

    let empty_trash_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-pro", "user-pro", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-pro", "user-pro", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/trash/empty")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "spaceId":"space-pro"
                    }"#,
                ))
                .expect("empty trash request should be built"),
        )
        .await
        .expect("empty trash request should be handled");
    assert_eq!(empty_trash_response.status(), StatusCode::OK);
    let empty_trash_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(empty_trash_response.into_body(), usize::MAX)
            .await
            .expect("empty trash response body should be readable"),
    )
    .expect("empty trash response body should be valid json");
    assert_eq!(
        common::envelope_field(&empty_trash_payload, "deletedCount").as_i64(),
        Some(1)
    );

    let deleted_node_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-pro'
           AND id='file-pro'
           AND lifecycle_status='deleted'",
    )
    .fetch_one(&pool)
    .await
    .expect("deleted node count should be queryable");
    assert_eq!(deleted_node_count, 1);

    let changes_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-pro", "user-pro", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-pro", "user-pro", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/changes?spaceId=space-pro")
                .body(Body::empty())
                .expect("changes request should be built"),
        )
        .await
        .expect("changes request should be handled");
    assert_eq!(changes_response.status(), StatusCode::OK);
    let changes_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(changes_response.into_body(), usize::MAX)
            .await
            .expect("changes response body should be readable"),
    )
    .expect("changes response body should be valid json");
    let events = common::envelope_items(&changes_payload)
        .as_array()
        .expect("changes items should be an array")
        .iter()
        .map(|item| item["eventType"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert!(events.contains(&"drive.node.created".to_string()));
    assert!(events.contains(&"drive.trash.emptied".to_string()));
}

#[tokio::test]
async fn empty_trash_rejects_missing_or_deleted_explicit_space_before_deleting_nodes() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES
            ('space-trash-active', 'tenant-trash-filter', 'user', 'user-trash', 'personal', 'Trash Active', 'active', 1, 'user-trash', 'user-trash'),
            ('space-trash-deleted', 'tenant-trash-filter', 'user', 'user-trash-deleted', 'personal', 'Trash Deleted', 'deleted', 1, 'user-trash', 'user-trash')",
    )
    .execute(&pool)
    .await
    .expect("spaces should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-trash-active', 'tenant-trash-filter', 'space-trash-active', NULL,
            'file', 'kept-in-trash.txt', 'ready', 'trashed', 1, 'user-trash', 'user-trash'
        )",
    )
    .execute(&pool)
    .await
    .expect("trashed node should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    for space_id in ["space-trash-missing", "space-trash-deleted"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-trash-filter", "user-trash", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-trash-filter", "user-trash", "appbase"),
                    )
                    .method(Method::POST)
                    .uri("/app/v3/api/drive/trash/empty")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{
                                                        "spaceId":"{space_id}"
                        }}"#
                    )))
                    .expect("empty trash request should be built"),
            )
            .await
            .expect("empty trash request should be handled");
        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "space filter {space_id} must be validated as an active drive space"
        );
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("empty trash response should be read"),
        )
        .expect("empty trash response should be valid json");
        assert_eq!(payload["code"].as_i64(), Some(40401));
    }

    let remaining_trashed_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-trash-filter'
           AND id='node-trash-active'
           AND lifecycle_status='trashed'",
    )
    .fetch_one(&pool)
    .await
    .expect("remaining trash count should be queryable");
    assert_eq!(remaining_trashed_count, 1);
}

#[tokio::test]
async fn create_folder_assigns_server_id_when_client_id_is_omitted() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-server-id', 'tenant-server-id', 'user', 'user-server-id', 'personal', 'Core', 'active', 1, 'user-server-id', 'user-server-id')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    let app = common::test_router_with_pool(pool);
    let create_folder_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-server-id", "user-server-id", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-server-id", "user-server-id", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/folders")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "spaceId":"space-server-id",
                        "nodeName":"Server Assigned"
                    }"#,
                ))
                .expect("create folder request should be built"),
        )
        .await
        .expect("create folder request should be handled");
    assert_eq!(create_folder_response.status(), StatusCode::CREATED);
    let folder_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(create_folder_response.into_body(), usize::MAX)
            .await
            .expect("create folder response body should be read"),
    )
    .expect("create folder response json should be valid");
    let assigned_id = common::envelope_item(&folder_payload)["id"]
        .as_str()
        .expect("created folder id should be returned");
    assert!(
        assigned_id.starts_with("folder_"),
        "expected server-generated folder id, got {assigned_id}"
    );
    assert_eq!(
        common::envelope_item(&folder_payload)["nodeName"],
        "Server Assigned"
    );
}

#[tokio::test]
async fn app_drive_core_routes_create_browse_share_search_and_emit_changes() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-core', 'tenant-core', 'user', 'user-core', 'personal', 'Core', 'active', 1, 'user-core', 'user-core')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let create_folder_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-core", "user-core", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-core", "user-core", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/folders")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"node-core-folder",
                        "spaceId":"space-core",
                        "nodeName":"Project Docs"
                    }"#,
                ))
                .expect("create folder request should be built"),
        )
        .await
        .expect("create folder request should be handled");
    assert_eq!(create_folder_response.status(), StatusCode::CREATED);
    let folder_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(create_folder_response.into_body(), usize::MAX)
            .await
            .expect("create folder response body should be read"),
    )
    .expect("create folder response json should be valid");
    assert_eq!(
        common::envelope_item(&folder_payload)["id"],
        "node-core-folder"
    );
    assert_eq!(common::envelope_item(&folder_payload)["nodeType"], "folder");

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-core", "user-core", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-core", "user-core", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces/space-core/nodes")
                .body(Body::empty())
                .expect("list nodes request should be built"),
        )
        .await
        .expect("list nodes request should be handled");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("list nodes response body should be read"),
    )
    .expect("list nodes response json should be valid");
    assert_eq!(
        common::envelope_items(&list_payload)[0]["id"],
        "node-core-folder"
    );

    let permission_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-core", "user-core", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-core", "user-core", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-core-folder/permissions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"perm-core",
                        "subjectType":"user",
                        "subjectId":"user-reviewer",
                        "role":"reader"
                    }"#,
                ))
                .expect("create permission request should be built"),
        )
        .await
        .expect("create permission request should be handled");
    assert_eq!(permission_response.status(), StatusCode::CREATED);

    let duplicate_permission_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-core", "user-core", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-core", "user-core", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-core-folder/permissions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"perm-core-duplicate",
                        "subjectType":"user",
                        "subjectId":"user-reviewer",
                        "role":"writer"
                    }"#,
                ))
                .expect("duplicate permission request should be built"),
        )
        .await
        .expect("duplicate permission request should be handled");
    assert_eq!(duplicate_permission_response.status(), StatusCode::CONFLICT);

    let share_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-core", "user-core", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-core", "user-core", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-core-folder/share_links")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"share-core",
                        "role":"reader",
                        "downloadLimit":3
                    }"#,
                ))
                .expect("create share link request should be built"),
        )
        .await
        .expect("create share link request should be handled");
    assert_eq!(share_response.status(), StatusCode::CREATED);
    let share_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(share_response.into_body(), usize::MAX)
            .await
            .expect("share response should be read"),
    )
    .expect("share response should be json");
    let created_token = common::envelope_data(&share_payload)["token"]
        .as_str()
        .expect("server-generated share token should be returned");
    assert!(created_token.len() >= 32);
    let token_hash: String =
        sqlx::query_scalar("SELECT token_hash FROM dr_drive_node_share_link WHERE id='share-core'")
            .fetch_one(&pool)
            .await
            .expect("share token hash should be stored");
    assert_eq!(
        token_hash,
        sdkwork_drive_workspace_service::drive_share_token_hash(created_token)
    );

    let search_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-core", "user-core", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-core", "user-core", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/search?q=Project")
                .body(Body::empty())
                .expect("search request should be built"),
        )
        .await
        .expect("search request should be handled");
    assert_eq!(search_response.status(), StatusCode::OK);
    let search_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(search_response.into_body(), usize::MAX)
            .await
            .expect("search response body should be read"),
    )
    .expect("search response json should be valid");
    assert_eq!(
        common::envelope_items(&search_payload)[0]["id"],
        "node-core-folder"
    );

    let changes_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-core", "user-core", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-core", "user-core", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/changes?spaceId=space-core")
                .body(Body::empty())
                .expect("changes request should be built"),
        )
        .await
        .expect("changes request should be handled");
    assert_eq!(changes_response.status(), StatusCode::OK);
    let changes_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(changes_response.into_body(), usize::MAX)
            .await
            .expect("changes response body should be read"),
    )
    .expect("changes response json should be valid");
    let events = common::envelope_items(&changes_payload)
        .as_array()
        .expect("changes items should be an array")
        .iter()
        .map(|item| item["eventType"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert!(events.contains(&"drive.node.created".to_string()));
    assert!(events.contains(&"drive.permission.created".to_string()));
    assert!(events.contains(&"drive.share_link.created".to_string()));
}

#[tokio::test]
async fn app_dr_drive_node_share_link_create_rejects_negative_download_limit_before_database_write()
{
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-share-validation', 'tenant-share-validation', 'user', 'user-owner', 'personal', 'Share Validation', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-share-validation', 'tenant-share-validation', 'space-share-validation', NULL, 'file', 'share.txt', 'ready', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-share-validation", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-share-validation", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-share-validation/share_links")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"share-negative-limit",
                        "role":"reader",
                        "downloadLimit":-1
                    }"#,
                ))
                .expect("create share link request should be built"),
        )
        .await
        .expect("create share link request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("share link validation response should be read"),
    )
    .expect("share link validation response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40001));

    let stored_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_node_share_link WHERE tenant_id='tenant-share-validation'",
    )
    .fetch_one(&pool)
    .await
    .expect("share link count should be queryable");
    assert_eq!(stored_count, 0);
}

#[tokio::test]
async fn app_dr_drive_node_share_link_create_stores_access_code_hash_and_reports_required_flag() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-share-access-code', 'tenant-share-access-code', 'user', 'user-owner', 'personal', 'Share Access Code', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-share-access-code', 'tenant-share-access-code', 'space-share-access-code', NULL, 'file', 'secret.txt', 'ready', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-share-access-code", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-share-access-code", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-share-access-code/share_links")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"share-access-code",
                        "role":"reader",
                        "accessCode":"extract-42"
                    }"#,
                ))
                .expect("create share link request should be built"),
        )
        .await
        .expect("create share link request should be handled");

    assert_eq!(response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("share link response should be read"),
    )
    .expect("share link response should be valid json");
    assert_eq!(common::envelope_data(&payload)["accessCodeRequired"], true);

    let stored_hash: Option<String> = sqlx::query_scalar(
        "SELECT access_code_hash FROM dr_drive_node_share_link WHERE id='share-access-code'",
    )
    .fetch_one(&pool)
    .await
    .expect("share link access code hash should be readable");
    assert_eq!(
        stored_hash,
        Some(sdkwork_drive_workspace_service::drive_share_access_code_hash("extract-42"))
    );
}

#[tokio::test]
async fn app_dr_drive_node_share_link_create_rejects_past_expiration_before_database_write() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-share-expired-create', 'tenant-share-expired-create', 'user', 'user-owner', 'personal', 'Share Expired Create', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-share-expired-create', 'tenant-share-expired-create', 'space-share-expired-create', NULL, 'file', 'share-expired.txt', 'ready', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-share-expired-create", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-share-expired-create", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-share-expired-create/share_links")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"share-expired-create",
                        "role":"reader",
                        "expiresAtEpochMs":1
                    }"#,
                ))
                .expect("create share link request should be built"),
        )
        .await
        .expect("create share link request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("share link expiration validation response should be read"),
    )
    .expect("share link expiration validation response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40001));

    let stored_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_node_share_link WHERE tenant_id='tenant-share-expired-create'",
    )
    .fetch_one(&pool)
    .await
    .expect("share link count should be queryable");
    assert_eq!(stored_count, 0);
}

#[tokio::test]
async fn app_dr_drive_node_share_link_update_rejects_past_expiration_before_database_write() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-share-expired-update', 'tenant-share-expired-update', 'user', 'user-owner', 'personal', 'Share Expired Update', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-share-expired-update', 'tenant-share-expired-update', 'space-share-expired-update', NULL, 'file', 'share-expired-update.txt', 'ready', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'share-expired-update', 'tenant-share-expired-update', 'node-share-expired-update',
            'share-expired-update-token-hash', 'reader', 4102444800000,
            NULL, 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("share link should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-share-expired-update", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-share-expired-update", "user-owner", "appbase"),
                )
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/share_links/share-expired-update")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "expiresAtEpochMs":1
                    }"#,
                ))
                .expect("update share link request should be built"),
        )
        .await
        .expect("update share link request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("share link expiration update validation response should be read"),
    )
    .expect("share link expiration update validation response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40001));

    let stored_expires_at: Option<i64> = sqlx::query_scalar(
        "SELECT expires_at_epoch_ms FROM dr_drive_node_share_link WHERE id='share-expired-update'",
    )
    .fetch_one(&pool)
    .await
    .expect("share link expiration should be queryable");
    assert_eq!(stored_expires_at, Some(4_102_444_800_000));
}

#[tokio::test]
async fn app_dr_drive_node_permission_create_rejects_invalid_dictionaries_before_database_constraints(
) {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-permission-validation', 'tenant-permission-validation', 'user', 'user-owner', 'team', 'Permission Validation', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-permission-validation', 'tenant-permission-validation', 'space-permission-validation', NULL, 'folder', 'Docs', 'ready', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let invalid_subject_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-permission-validation", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-permission-validation", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-permission-validation/permissions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"perm-invalid-subject",
                        "subjectType":"workspace",
                        "subjectId":"workspace-001",
                        "role":"reader"
                    }"#,
                ))
                .expect("invalid subject permission request should be built"),
        )
        .await
        .expect("invalid subject permission request should be handled");
    assert_eq!(invalid_subject_response.status(), StatusCode::BAD_REQUEST);
    let invalid_subject_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(invalid_subject_response.into_body(), usize::MAX)
            .await
            .expect("invalid subject response should be read"),
    )
    .expect("invalid subject response should be valid json");
    assert_eq!(invalid_subject_payload["code"].as_i64(), Some(40001));

    let invalid_role_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-permission-validation", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-permission-validation", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-permission-validation/permissions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"perm-invalid-role",
                        "subjectType":"user",
                        "subjectId":"user-reviewer",
                        "role":"viewer"
                    }"#,
                ))
                .expect("invalid role permission request should be built"),
        )
        .await
        .expect("invalid role permission request should be handled");
    assert_eq!(invalid_role_response.status(), StatusCode::BAD_REQUEST);
    let invalid_role_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(invalid_role_response.into_body(), usize::MAX)
            .await
            .expect("invalid role response should be read"),
    )
    .expect("invalid role response should be valid json");
    assert_eq!(invalid_role_payload["code"].as_i64(), Some(40001));

    let permission_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_node_permission")
        .fetch_one(&pool)
        .await
        .expect("permission count should be readable");
    assert_eq!(permission_count, 0);
}

#[tokio::test]
async fn app_dr_drive_space_resource_routes_get_update_delete_and_retire_contents() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-resource', 'tenant-resource', 'user', 'user-owner', 'team', 'Resource Space', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-resource', 'tenant-resource', 'space-resource', NULL, 'file', 'resource.pdf', 'ready', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_storage_metadata_provider_fixture(
        &pool,
        "provider-resource",
        "bucket-resource",
        "user-owner",
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex,
            lifecycle_status, created_by, updated_by
        ) VALUES ('storage-resource', 'tenant-resource', 'node-resource', 1, 'provider-resource', 'bucket-resource', 'objects/node-resource/v1.pdf', 'application/pdf', 128, 'sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'active', 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("storage object should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let get_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-resource", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-resource", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces/space-resource")
                .body(Body::empty())
                .expect("get space request should be built"),
        )
        .await
        .expect("get space request should be handled");
    assert_eq!(get_response.status(), StatusCode::OK);
    let get_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(get_response.into_body(), usize::MAX)
            .await
            .expect("get space response should be read"),
    )
    .expect("get space response should be valid json");
    let get_item = common::envelope_item(&get_payload);
    assert_eq!(get_item["id"].as_str(), Some("space-resource"));
    assert_eq!(get_item["displayName"].as_str(), Some("Resource Space"));
    assert_eq!(get_item["version"].as_i64(), Some(1));

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-resource", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-resource", "user-owner", "appbase"),
                )
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/spaces/space-resource")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "displayName":"Resource Space Updated"
                    }"#,
                ))
                .expect("update space request should be built"),
        )
        .await
        .expect("update space request should be handled");
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(update_response.into_body(), usize::MAX)
            .await
            .expect("update space response should be read"),
    )
    .expect("update space response should be valid json");
    let update_item = common::envelope_item(&update_payload);
    assert_eq!(
        update_item["displayName"].as_str(),
        Some("Resource Space Updated")
    );
    assert_eq!(update_item["version"].as_i64(), Some(2));

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-resource", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-resource", "user-owner", "appbase"),
                )
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/spaces/space-resource")
                .body(Body::empty())
                .expect("delete space request should be built"),
        )
        .await
        .expect("delete space request should be handled");
    common::assert_no_content_response(delete_response).await;

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-resource", "user-admin", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-resource", "user-admin", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces")
                .body(Body::empty())
                .expect("list spaces request should be built"),
        )
        .await
        .expect("list spaces request should be handled");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("list spaces response should be read"),
    )
    .expect("list spaces response should be valid json");
    assert_eq!(
        common::envelope_items(&list_payload)
            .as_array()
            .unwrap()
            .len(),
        0
    );

    let get_deleted_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-resource", "user-admin", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-resource", "user-admin", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces/space-resource")
                .body(Body::empty())
                .expect("get deleted space request should be built"),
        )
        .await
        .expect("get deleted space request should be handled");
    assert_eq!(get_deleted_response.status(), StatusCode::NOT_FOUND);

    let node_status: String = sqlx::query_scalar(
        "SELECT lifecycle_status FROM dr_drive_node WHERE tenant_id='tenant-resource' AND id='node-resource'",
    )
    .fetch_one(&pool)
    .await
    .expect("node status should be readable");
    assert_eq!(node_status, "deleted");
    let storage_status: String = sqlx::query_scalar(
        "SELECT lifecycle_status FROM dr_drive_storage_object WHERE tenant_id='tenant-resource' AND node_id='node-resource'",
    )
    .fetch_one(&pool)
    .await
    .expect("storage status should be readable");
    assert_eq!(storage_status, "deleted");

    let changes_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-resource", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-resource", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/changes?spaceId=space-resource")
                .body(Body::empty())
                .expect("changes request should be built"),
        )
        .await
        .expect("changes request should be handled");
    assert_eq!(changes_response.status(), StatusCode::OK);
    let changes_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(changes_response.into_body(), usize::MAX)
            .await
            .expect("changes response should be read"),
    )
    .expect("changes response should be valid json");
    let events = common::envelope_items(&changes_payload)
        .as_array()
        .expect("changes items should be an array")
        .iter()
        .map(|item| item["eventType"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert!(events.contains(&"drive.space.updated".to_string()));
    assert!(events.contains(&"drive.space.deleted".to_string()));
}

#[tokio::test]
async fn app_drive_collaboration_and_version_governance_routes_update_and_emit_changes() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-gov', 'tenant-gov', 'user', 'user-owner', 'personal', 'Governance', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-gov', 'tenant-gov', 'space-gov', NULL, 'file', 'governance.pdf', 'ready', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_storage_metadata_provider_fixture(&pool, "provider-gov", "bucket-gov", "user-owner").await;
    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-gov', 'tenant-gov', 'node-gov', 'user', 'user-reviewer',
            'reader', 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("permission should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version,
            created_by, updated_by
        ) VALUES (
            'share-gov', 'tenant-gov', 'node-gov', 'sha256:governance-secret',
            'reader', 1800000000000, 5, 1, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("share link should be seeded");
    for (id, version_no, checksum) in [
        (
            "version-gov-1",
            1_i64,
            "sha256:1111111111111111111111111111111111111111111111111111111111111111",
        ),
        (
            "version-gov-2",
            2_i64,
            "sha256:2222222222222222222222222222222222222222222222222222222222222222",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_storage_object (
                id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
                content_type, content_length, checksum_sha256_hex,
                lifecycle_status, created_by, updated_by
            ) VALUES ($1, 'tenant-gov', 'node-gov', $2, 'provider-gov', 'bucket-gov', $3, 'application/pdf', 256, $4, 'active', 'user-owner', 'user-owner')",
        )
        .bind(id)
        .bind(version_no)
        .bind(format!("objects/node-gov/v{version_no}.pdf"))
        .bind(checksum)
        .execute(&pool)
        .await
        .expect("storage object version should be seeded");
    }

    let app = common::test_router_with_pool(pool.clone());
    let permission_update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-gov", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-gov", "user-owner", "appbase"),
                )
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/nodes/node-gov/permissions/perm-gov")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "role":"writer"
                    }"#,
                ))
                .expect("permission update request should be built"),
        )
        .await
        .expect("permission update request should be handled");
    assert_eq!(permission_update_response.status(), StatusCode::OK);
    let permission_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(permission_update_response.into_body(), usize::MAX)
            .await
            .expect("permission update response should be read"),
    )
    .expect("permission update response should be valid json");
    assert_eq!(
        common::envelope_item(&permission_payload)["role"].as_str(),
        Some("writer")
    );
    assert_eq!(
        common::envelope_item(&permission_payload)["version"].as_i64(),
        Some(2)
    );

    let permission_detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-gov", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-gov", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-gov/permissions/perm-gov")
                .body(Body::empty())
                .expect("permission detail request should be built"),
        )
        .await
        .expect("permission detail request should be handled");
    assert_eq!(permission_detail_response.status(), StatusCode::OK);
    let permission_detail_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(permission_detail_response.into_body(), usize::MAX)
            .await
            .expect("permission detail response should be read"),
    )
    .expect("permission detail response should be valid json");
    assert_eq!(
        common::envelope_item(&permission_detail_payload)["subjectId"].as_str(),
        Some("user-reviewer")
    );
    assert_eq!(
        common::envelope_item(&permission_detail_payload)["role"].as_str(),
        Some("writer")
    );

    let share_list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-gov", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-gov", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-gov/share_links")
                .body(Body::empty())
                .expect("share link list request should be built"),
        )
        .await
        .expect("share link list request should be handled");
    assert_eq!(share_list_response.status(), StatusCode::OK);
    let share_list_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(share_list_response.into_body(), usize::MAX)
            .await
            .expect("share link list response should be read"),
    )
    .expect("share link list response should be valid json");
    assert_eq!(
        common::envelope_items(&share_list_payload)[0]["id"].as_str(),
        Some("share-gov")
    );
    assert_eq!(
        common::envelope_items(&share_list_payload)[0]["role"].as_str(),
        Some("reader")
    );
    assert!(
        common::envelope_items(&share_list_payload)[0]
            .get("token")
            .is_none(),
        "share link list response must not expose raw token"
    );
    assert!(
        common::envelope_items(&share_list_payload)[0]
            .get("tokenHash")
            .is_none(),
        "share link list response must not expose token hash"
    );

    let share_update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-gov", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-gov", "user-owner", "appbase"),
                )
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/share_links/share-gov")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "role":"commenter",
                        "expiresAtEpochMs":null,
                        "downloadLimit":9
                    }"#,
                ))
                .expect("share link update request should be built"),
        )
        .await
        .expect("share link update request should be handled");
    assert_eq!(share_update_response.status(), StatusCode::OK);
    let share_update_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(share_update_response.into_body(), usize::MAX)
            .await
            .expect("share link update response should be read"),
    )
    .expect("share link update response should be valid json");
    let share_update_item = common::envelope_item(&share_update_payload);
    assert_eq!(share_update_item["role"].as_str(), Some("commenter"));
    assert!(share_update_item["expiresAtEpochMs"].is_null());
    assert_eq!(share_update_item["downloadLimit"].as_i64(), Some(9));
    assert_eq!(share_update_item["version"].as_i64(), Some(2));
    assert!(
        share_update_item.get("token").is_none(),
        "share link update response must not expose raw token"
    );
    assert!(
        share_update_item.get("tokenHash").is_none(),
        "share link update response must not expose token hash"
    );

    let share_detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-gov", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-gov", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/share_links/share-gov")
                .body(Body::empty())
                .expect("share link detail request should be built"),
        )
        .await
        .expect("share link detail request should be handled");
    assert_eq!(share_detail_response.status(), StatusCode::OK);
    let share_detail_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(share_detail_response.into_body(), usize::MAX)
            .await
            .expect("share link detail response should be read"),
    )
    .expect("share link detail response should be valid json");
    let share_detail_item = common::envelope_item(&share_detail_payload);
    assert_eq!(share_detail_item["id"].as_str(), Some("share-gov"));
    assert_eq!(share_detail_item["role"].as_str(), Some("commenter"));
    assert!(
        share_detail_item.get("token").is_none(),
        "share link detail response must not expose raw token"
    );
    assert!(
        share_detail_item.get("tokenHash").is_none(),
        "share link detail response must not expose token hash"
    );

    let share_revoke_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-gov", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-gov", "user-owner", "appbase"),
                )
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/share_links/share-gov")
                .body(Body::empty())
                .expect("share link revoke request should be built"),
        )
        .await
        .expect("share link revoke request should be handled");
    common::assert_no_content_response(share_revoke_response).await;
    let revoked_share_status: String = sqlx::query_scalar(
        "SELECT lifecycle_status FROM dr_drive_node_share_link WHERE tenant_id='tenant-gov' AND id='share-gov'",
    )
    .fetch_one(&pool)
    .await
    .expect("revoked share link status should be readable");
    assert_eq!(revoked_share_status, "deleted");

    let share_list_after_revoke_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-gov", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-gov", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-gov/share_links")
                .body(Body::empty())
                .expect("share link list after revoke request should be built"),
        )
        .await
        .expect("share link list after revoke request should be handled");
    assert_eq!(share_list_after_revoke_response.status(), StatusCode::OK);
    let share_list_after_revoke_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(share_list_after_revoke_response.into_body(), usize::MAX)
            .await
            .expect("share link list after revoke response should be read"),
    )
    .expect("share link list after revoke response should be valid json");
    assert_eq!(
        common::envelope_items(&share_list_after_revoke_payload)
            .as_array()
            .expect("share links should be an array")
            .len(),
        0
    );

    let version_detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-gov", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-gov", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-gov/versions/version-gov-1")
                .body(Body::empty())
                .expect("version detail request should be built"),
        )
        .await
        .expect("version detail request should be handled");
    assert_eq!(version_detail_response.status(), StatusCode::OK);
    let version_detail_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(version_detail_response.into_body(), usize::MAX)
            .await
            .expect("version detail response should be read"),
    )
    .expect("version detail response should be valid json");
    assert_eq!(
        common::envelope_item(&version_detail_payload)["id"].as_str(),
        Some("version-gov-1")
    );
    assert_eq!(
        common::envelope_item(&version_detail_payload)["versionNo"].as_i64(),
        Some(1)
    );

    let version_delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-gov", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-gov", "user-owner", "appbase"),
                )
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/nodes/node-gov/versions/version-gov-1")
                .body(Body::empty())
                .expect("version delete request should be built"),
        )
        .await
        .expect("version delete request should be handled");
    common::assert_no_content_response(version_delete_response).await;

    let deleted_version_status: String = sqlx::query_scalar(
        "SELECT lifecycle_status
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-gov' AND node_id='node-gov' AND id='version-gov-1'",
    )
    .fetch_one(&pool)
    .await
    .expect("deleted version status should be queryable");
    assert_eq!(deleted_version_status, "deleted");

    let last_version_delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-gov", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-gov", "user-owner", "appbase"),
                )
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/nodes/node-gov/versions/version-gov-2")
                .body(Body::empty())
                .expect("last version delete request should be built"),
        )
        .await
        .expect("last version delete request should be handled");
    assert_eq!(last_version_delete_response.status(), StatusCode::CONFLICT);

    let changes_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-gov", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-gov", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/changes?spaceId=space-gov")
                .body(Body::empty())
                .expect("changes request should be built"),
        )
        .await
        .expect("changes request should be handled");
    assert_eq!(changes_response.status(), StatusCode::OK);
    let changes_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(changes_response.into_body(), usize::MAX)
            .await
            .expect("changes response should be read"),
    )
    .expect("changes response should be valid json");
    let events = common::envelope_items(&changes_payload)
        .as_array()
        .expect("changes items should be an array")
        .iter()
        .map(|item| item["eventType"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    for expected_event in [
        "drive.permission.updated",
        "drive.share_link.updated",
        "drive.share_link.revoked",
        "drive.file_version.deleted",
    ] {
        assert!(
            events.contains(&expected_event.to_string()),
            "changes should include {expected_event}"
        );
    }
}

#[tokio::test]
async fn app_dr_drive_node_comment_and_reply_routes_support_collaboration_lifecycle() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-comments', 'tenant-comments', 'user', 'user-owner', 'personal', 'Comments', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-comments', 'tenant-comments', 'space-comments', NULL, 'file', 'proposal.docx', 'ready', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-comments-reviewer', 'tenant-comments', 'node-comments', 'user', 'user-reviewer',
            'writer', 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("reviewer permission should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-comments-collaborator', 'tenant-comments', 'node-comments', 'user', 'user-collaborator',
            'writer', 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("collaborator permission should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    for (id, content) in [
        ("comment-one", "Please review the first section."),
        ("comment-two", "Resolve the open question."),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-comments", "user-reviewer", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-comments", "user-reviewer", "appbase"),
                    )
                    .method(Method::POST)
                    .uri("/app/v3/api/drive/nodes/node-comments/comments")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{
                            "id":"{id}",
                                                        "content":"{content}",
                            "anchor":"$.body[0]"
                        }}"#
                    )))
                    .expect("create comment request should be built"),
            )
            .await
            .expect("create comment request should be handled");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let first_page = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comments", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comments", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-comments/comments?page_size=1")
                .body(Body::empty())
                .expect("comment list request should be built"),
        )
        .await
        .expect("comment list request should be handled");
    assert_eq!(first_page.status(), StatusCode::OK);
    let first_page_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(first_page.into_body(), usize::MAX)
            .await
            .expect("comment list response should be read"),
    )
    .expect("comment list response should be valid json");
    assert_eq!(
        common::envelope_items(&first_page_payload)
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        common::envelope_items(&first_page_payload)[0]["id"].as_str(),
        Some("comment-two")
    );
    let next_page_token = common::envelope_next_page_token(&first_page_payload)
        .expect("comment list should expose nextPageToken");

    let second_page = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comments", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comments", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri(format!(
            "/app/v3/api/drive/nodes/node-comments/comments?page_size=1&cursor={next_page_token}"
        ))
                .body(Body::empty())
                .expect("comment second page request should be built"),
        )
        .await
        .expect("comment second page request should be handled");
    assert_eq!(second_page.status(), StatusCode::OK);
    let second_page_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(second_page.into_body(), usize::MAX)
            .await
            .expect("comment second page response should be read"),
    )
    .expect("comment second page response should be valid json");
    assert_eq!(
        common::envelope_items(&second_page_payload)[0]["id"].as_str(),
        Some("comment-one")
    );
    assert!(common::envelope_next_page_token(&second_page_payload).is_none());

    let comment_detail = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comments", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comments", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-comments/comments/comment-one")
                .body(Body::empty())
                .expect("comment detail request should be built"),
        )
        .await
        .expect("comment detail request should be handled");
    assert_eq!(comment_detail.status(), StatusCode::OK);
    let comment_detail_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(comment_detail.into_body(), usize::MAX)
            .await
            .expect("comment detail response should be read"),
    )
    .expect("comment detail response should be valid json");
    let comment_detail_item = common::envelope_item(&comment_detail_payload);
    assert_eq!(
        comment_detail_item["content"].as_str(),
        Some("Please review the first section.")
    );
    assert_eq!(comment_detail_item["resolved"].as_bool(), Some(false));

    let update_comment_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comments", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comments", "user-reviewer", "appbase"),
                )
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/nodes/node-comments/comments/comment-one")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "content":"Reviewed and resolved.",
                        "resolved":true
                    }"#,
                ))
                .expect("comment update request should be built"),
        )
        .await
        .expect("comment update request should be handled");
    assert_eq!(update_comment_response.status(), StatusCode::OK);
    let update_comment_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(update_comment_response.into_body(), usize::MAX)
            .await
            .expect("comment update response should be read"),
    )
    .expect("comment update response should be valid json");
    let update_comment_item = common::envelope_item(&update_comment_payload);
    assert_eq!(
        update_comment_item["content"].as_str(),
        Some("Reviewed and resolved.")
    );
    assert_eq!(update_comment_item["resolved"].as_bool(), Some(true));
    assert_eq!(update_comment_item["version"].as_i64(), Some(2));

    for (id, content) in [
        ("reply-one", "I can take this."),
        ("reply-two", "Resolution confirmed."),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-comments", "user-collaborator", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-comments", "user-collaborator", "appbase"),
                    )
                    .method(Method::POST)
                    .uri("/app/v3/api/drive/nodes/node-comments/comments/comment-one/replies")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{
                            "id":"{id}",
                                                        "content":"{content}"
                        }}"#
                    )))
                    .expect("create reply request should be built"),
            )
            .await
            .expect("create reply request should be handled");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let replies_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comments", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comments", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri(
                    "/app/v3/api/drive/nodes/node-comments/comments/comment-one/replies?page_size=1",
                )
                .body(Body::empty())
                .expect("reply list request should be built"),
        )
        .await
        .expect("reply list request should be handled");
    assert_eq!(replies_response.status(), StatusCode::OK);
    let replies_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(replies_response.into_body(), usize::MAX)
            .await
            .expect("reply list response should be read"),
    )
    .expect("reply list response should be valid json");
    assert_eq!(
        common::envelope_items(&replies_payload)
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        common::envelope_items(&replies_payload)[0]["id"].as_str(),
        Some("reply-one")
    );
    assert!(
        common::envelope_next_page_token(&replies_payload).is_some(),
        "reply list should expose nextPageToken when more rows exist"
    );

    let reply_detail = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comments", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comments", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-comments/comments/comment-one/replies/reply-one")
                .body(Body::empty())
                .expect("reply detail request should be built"),
        )
        .await
        .expect("reply detail request should be handled");
    assert_eq!(reply_detail.status(), StatusCode::OK);
    let reply_detail_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(reply_detail.into_body(), usize::MAX)
            .await
            .expect("reply detail response should be read"),
    )
    .expect("reply detail response should be valid json");
    let reply_detail_item = common::envelope_item(&reply_detail_payload);
    assert_eq!(
        reply_detail_item["content"].as_str(),
        Some("I can take this.")
    );

    let reply_update = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comments", "user-collaborator", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comments", "user-collaborator", "appbase"),
                )
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/nodes/node-comments/comments/comment-one/replies/reply-one")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "content":"I handled this."
                    }"#,
                ))
                .expect("reply update request should be built"),
        )
        .await
        .expect("reply update request should be handled");
    assert_eq!(reply_update.status(), StatusCode::OK);
    let reply_update_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(reply_update.into_body(), usize::MAX)
            .await
            .expect("reply update response should be read"),
    )
    .expect("reply update response should be valid json");
    let reply_update_item = common::envelope_item(&reply_update_payload);
    assert_eq!(
        reply_update_item["content"].as_str(),
        Some("I handled this.")
    );
    assert_eq!(reply_update_item["version"].as_i64(), Some(2));

    let delete_reply_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comments", "user-collaborator", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comments", "user-collaborator", "appbase"),
                )
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/nodes/node-comments/comments/comment-one/replies/reply-one")
                .body(Body::empty())
                .expect("delete reply request should be built"),
        )
        .await
        .expect("delete reply request should be handled");
    common::assert_no_content_response(delete_reply_response).await;

    let deleted_reply_detail = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comments", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comments", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-comments/comments/comment-one/replies/reply-one")
                .body(Body::empty())
                .expect("deleted reply detail request should be built"),
        )
        .await
        .expect("deleted reply detail request should be handled");
    assert_eq!(deleted_reply_detail.status(), StatusCode::NOT_FOUND);

    let delete_comment_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comments", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comments", "user-reviewer", "appbase"),
                )
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/nodes/node-comments/comments/comment-one")
                .body(Body::empty())
                .expect("delete comment request should be built"),
        )
        .await
        .expect("delete comment request should be handled");
    common::assert_no_content_response(delete_comment_response).await;

    let deleted_comment_detail = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comments", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comments", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-comments/comments/comment-one")
                .body(Body::empty())
                .expect("deleted comment detail request should be built"),
        )
        .await
        .expect("deleted comment detail request should be handled");
    assert_eq!(deleted_comment_detail.status(), StatusCode::NOT_FOUND);

    let changes_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comments", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comments", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/changes?spaceId=space-comments")
                .body(Body::empty())
                .expect("changes request should be built"),
        )
        .await
        .expect("changes request should be handled");
    assert_eq!(changes_response.status(), StatusCode::OK);
    let changes_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(changes_response.into_body(), usize::MAX)
            .await
            .expect("changes response should be read"),
    )
    .expect("changes response should be valid json");
    let events = common::envelope_items(&changes_payload)
        .as_array()
        .expect("changes items should be an array")
        .iter()
        .map(|item| item["eventType"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    for expected_event in [
        "drive.comment.created",
        "drive.comment.updated",
        "drive.comment.deleted",
        "drive.comment_reply.created",
        "drive.comment_reply.updated",
        "drive.comment_reply.deleted",
    ] {
        assert!(
            events.contains(&expected_event.to_string()),
            "changes should include {expected_event}"
        );
    }
}

#[tokio::test]
async fn app_drive_changes_support_start_page_token_and_standard_pagination() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-changes', 'tenant-changes', 'user', 'user-owner', 'personal', 'Changes', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    for sequence_no in 1_i64..=3_i64 {
        sqlx::query(
            "INSERT INTO dr_drive_change_log (
                id, tenant_id, space_id, node_id, sequence_no, event_type, actor_id
             ) VALUES ($1, 'tenant-changes', 'space-changes', $2, $3, $4, 'user-owner')",
        )
        .bind(10_470_000_i64 + sequence_no)
        .bind(format!("node-change-{sequence_no}"))
        .bind(sequence_no)
        .bind(format!("node.changed.{sequence_no}"))
        .execute(&pool)
        .await
        .expect("change row should be seeded");
    }

    let app = common::test_router_with_pool(pool.clone());
    let start_token_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-changes", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-changes", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/changes/start_page_token?spaceId=space-changes")
                .body(Body::empty())
                .expect("start page token request should be built"),
        )
        .await
        .expect("start page token request should be handled");
    assert_eq!(start_token_response.status(), StatusCode::OK);
    let start_token_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(start_token_response.into_body(), usize::MAX)
            .await
            .expect("start page token response should be read"),
    )
    .expect("start page token response should be valid json");
    let start_page_token = common::envelope_data(&start_token_payload)["startPageToken"]
        .as_str()
        .expect("start page token should be present");
    assert_ne!(start_page_token, "3");
    assert!(
        !start_page_token.bytes().all(|byte| byte.is_ascii_digit()),
        "start page token must be opaque"
    );

    let first_page = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-changes", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-changes", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/changes?spaceId=space-changes&page_size=1")
                .body(Body::empty())
                .expect("changes first page request should be built"),
        )
        .await
        .expect("changes first page request should be handled");
    assert_eq!(first_page.status(), StatusCode::OK);
    let first_page_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(first_page.into_body(), usize::MAX)
            .await
            .expect("changes first page response should be read"),
    )
    .expect("changes first page response should be valid json");
    assert_eq!(
        common::envelope_items(&first_page_payload)
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        common::envelope_items(&first_page_payload)[0]["sequenceNo"].as_i64(),
        Some(1)
    );
    let first_page_token = common::envelope_next_page_token(&first_page_payload)
        .expect("first page should expose continuation token");
    assert_ne!(first_page_token, "1");
    assert!(
        !first_page_token.bytes().all(|byte| byte.is_ascii_digit()),
        "change continuation token must be opaque"
    );

    let second_page = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-changes", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-changes", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri(format!(
                    "/app/v3/api/drive/changes?spaceId=space-changes&page_size=2&cursor={first_page_token}"
                ))
                .body(Body::empty())
                .expect("changes second page request should be built"),
        )
        .await
        .expect("changes second page request should be handled");
    assert_eq!(second_page.status(), StatusCode::OK);
    let second_page_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(second_page.into_body(), usize::MAX)
            .await
            .expect("changes second page response should be read"),
    )
    .expect("changes second page response should be valid json");
    let second_page_sequences = common::envelope_items(&second_page_payload)
        .as_array()
        .expect("changes second page items should be an array")
        .iter()
        .map(|item| item["sequenceNo"].as_i64().unwrap_or_default())
        .collect::<Vec<_>>();
    assert_eq!(second_page_sequences, vec![2, 3]);
    assert!(common::envelope_next_page_token(&second_page_payload).is_none());
}

#[tokio::test]
async fn app_drive_changes_validate_explicit_space_filter() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-changes-deleted', 'tenant-changes-filter', 'user', 'user-deleted-owner',
            'personal', 'Deleted Changes', 'deleted', 1, 'user-deleted-owner', 'user-deleted-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("deleted space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_change_log (
            id, tenant_id, space_id, node_id, sequence_no, event_type, actor_id
        ) VALUES (
            10588001, 'tenant-changes-filter', 'space-changes-deleted', NULL, 1, 'drive.space.deleted', 'user-deleted-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("deleted space history should be seeded");

    let app = common::test_router_with_pool(pool);

    for uri in [
        "/app/v3/api/drive/changes?spaceId=space-changes-missing",
        "/app/v3/api/drive/changes/start_page_token?spaceId=space-changes-missing",
        "/app/v3/api/drive/changes/start_page_token?spaceId=space-changes-deleted",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token(
                                "tenant-changes-filter",
                                "user-deleted-owner",
                                "appbase"
                            )
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token(
                            "tenant-changes-filter",
                            "user-deleted-owner",
                            "appbase",
                        ),
                    )
                    .method(Method::GET)
                    .uri(uri)
                    .body(Body::empty())
                    .expect("changes invalid space request should be built"),
            )
            .await
            .expect("changes invalid space request should be handled");
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("changes invalid space response should be read"),
        )
        .expect("changes invalid space response should be valid json");
        assert_eq!(payload["detail"].as_str(), Some("space not found"));
    }

    let deleted_history_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-changes-filter",
                            "user-deleted-owner",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-changes-filter", "user-deleted-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/changes?spaceId=space-changes-deleted")
                .body(Body::empty())
                .expect("deleted changes request should be built"),
        )
        .await
        .expect("deleted changes request should be handled");
    assert_eq!(deleted_history_response.status(), StatusCode::OK);
    let deleted_history_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(deleted_history_response.into_body(), usize::MAX)
            .await
            .expect("deleted changes response should be read"),
    )
    .expect("deleted changes response should be valid json");
    let events = common::envelope_items(&deleted_history_payload)
        .as_array()
        .expect("deleted changes items should be an array")
        .iter()
        .map(|item| item["eventType"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert_eq!(events, vec!["drive.space.deleted".to_string()]);
}

#[tokio::test]
async fn app_drive_changes_rejects_page_size_outside_contract() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-change-page-size', 'tenant-change-page-size', 'user', 'user-change-page-size', 'personal', 'Change Page Size', 'active', 1, 'user-change-page-size', 'user-change-page-size')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-change-page-size",
                            "user-change-page-size",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token(
                        "tenant-change-page-size",
                        "user-change-page-size",
                        "appbase",
                    ),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/changes?spaceId=space-change-page-size&page_size=0")
                .body(Body::empty())
                .expect("changes request should be built"),
        )
        .await
        .expect("changes request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("error body should be readable"),
    )
    .expect("error body should be valid json");
    assert_eq!(payload["code"], 40001);
    assert!(payload["detail"]
        .as_str()
        .expect("detail should be present")
        .contains("page_size"));
}

#[tokio::test]
async fn app_drive_change_feed_allocates_unique_sequences_for_concurrent_writes() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-change-concurrent', 'tenant-change-concurrent', 'user', 'user-change',
            'personal', 'Change Concurrent', 'active', 1, 'user-change', 'user-change'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let mut tasks = Vec::new();
    for index in 0..12 {
        let app = app.clone();
        tasks.push(tokio::spawn(async move {
            let body = format!(
                r#"{{
                    "id":"folder-change-{index}",
                                        "spaceId":"space-change-concurrent",
                    "nodeName":"Folder {index}"
                }}"#
            );
            app.oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token(
                                "tenant-change-concurrent",
                                "user-change",
                                "appbase"
                            )
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-change-concurrent", "user-change", "appbase"),
                    )
                    .method(Method::POST)
                    .uri("/app/v3/api/drive/nodes/folders")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .expect("create folder request should be built"),
            )
            .await
            .expect("create folder request should be handled")
            .status()
        }));
    }

    for task in tasks {
        assert_eq!(
            task.await.expect("create task should complete"),
            StatusCode::CREATED
        );
    }

    let sequences: Vec<i64> = sqlx::query_scalar(
        "SELECT sequence_no
         FROM dr_drive_change_log
         WHERE tenant_id='tenant-change-concurrent'
           AND space_id='space-change-concurrent'
         ORDER BY sequence_no ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("change sequences should be queryable");

    assert_eq!(
        sequences,
        (1_i64..=12_i64).collect::<Vec<_>>(),
        "concurrent writes must allocate gap-free unique per-space change sequences"
    );
}

#[tokio::test]
async fn app_dr_drive_node_path_route_returns_ordered_breadcrumbs() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-path', 'tenant-path', 'user', 'user-owner', 'personal', 'Path', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    for (id, parent_id, node_type, node_name) in [
        ("node-root-folder", Option::<&str>::None, "folder", "Root"),
        (
            "node-child-folder",
            Some("node-root-folder"),
            "folder",
            "Project",
        ),
        (
            "node-leaf-file",
            Some("node-child-folder"),
            "file",
            "brief.pdf",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-path', 'space-path', $2, $3, $4, 'ready', 'active', 1, 'user-owner', 'user-owner')",
        )
        .bind(id)
        .bind(parent_id)
        .bind(node_type)
        .bind(node_name)
        .execute(&pool)
        .await
        .expect("path node should be seeded");
    }

    let app = common::test_router_with_pool(pool.clone());
    let path_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-path", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-path", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-leaf-file/path")
                .body(Body::empty())
                .expect("node path request should be built"),
        )
        .await
        .expect("node path request should be handled");
    assert_eq!(path_response.status(), StatusCode::OK);
    let path_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(path_response.into_body(), usize::MAX)
            .await
            .expect("node path response should be read"),
    )
    .expect("node path response should be valid json");
    let ids = common::envelope_items(&path_payload)
        .as_array()
        .expect("path items should be an array")
        .iter()
        .map(|item| item["id"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![
            "node-root-folder".to_string(),
            "node-child-folder".to_string(),
            "node-leaf-file".to_string()
        ]
    );
    let path_segments = common::envelope_data(&path_payload)["pathSegments"]
        .as_array()
        .expect("pathSegments should be an array")
        .iter()
        .map(|item| item.as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        path_segments,
        vec![
            "Root".to_string(),
            "Project".to_string(),
            "brief.pdf".to_string()
        ]
    );

    let missing_path_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-path", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-path", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/missing-node/path")
                .body(Body::empty())
                .expect("missing node path request should be built"),
        )
        .await
        .expect("missing node path request should be handled");
    assert_eq!(missing_path_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn app_drive_standard_views_list_trash_recent_shared_and_favorites() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-views', 'tenant-views', 'user', 'user-owner', 'personal', 'Views', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    for (id, node_name, lifecycle_status) in [
        ("node-active", "active.txt", "active"),
        ("node-trashed", "trashed.txt", "trashed"),
        ("node-shared", "shared.txt", "active"),
        ("node-favorite", "favorite.txt", "active"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-views', 'space-views', NULL, 'file', $2, 'ready', $3, 1, 'user-owner', 'user-owner')",
        )
        .bind(id)
        .bind(node_name)
        .bind(lifecycle_status)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }

    sqlx::query(
        "UPDATE dr_drive_node
         SET updated_at='2026-06-04 10:00:00'
         WHERE id='node-active'",
    )
    .execute(&pool)
    .await
    .expect("active node timestamp should be updated");
    sqlx::query(
        "UPDATE dr_drive_node
         SET updated_at='2026-06-04 11:00:00'
         WHERE id='node-shared'",
    )
    .execute(&pool)
    .await
    .expect("shared node timestamp should be updated");
    sqlx::query(
        "UPDATE dr_drive_node
         SET updated_at='2026-06-04 12:00:00'
         WHERE id='node-favorite'",
    )
    .execute(&pool)
    .await
    .expect("favorite node timestamp should be updated");

    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-shared', 'tenant-views', 'node-shared', 'user', 'user-reviewer',
            'reader', 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("permission should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-trashed', 'tenant-views', 'node-trashed', 'user', 'user-reviewer',
            'reader', 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("trashed permission should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-favorite', 'tenant-views', 'node-favorite', 'user', 'user-reviewer',
            'reader', 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("favorite permission should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-active', 'tenant-views', 'node-active', 'user', 'user-reviewer',
            'reader', 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("active permission should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_favorite (
            id, tenant_id, node_id, subject_type, subject_id,
            lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'favorite-seeded', 'tenant-views', 'node-favorite', 'user', 'user-reviewer',
            'active', 1, 'user-reviewer', 'user-reviewer'
        )",
    )
    .execute(&pool)
    .await
    .expect("favorite should be seeded");

    let app = common::test_router_with_pool(pool.clone());

    let trash_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-views", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-views", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/trash?spaceId=space-views")
                .body(Body::empty())
                .expect("trash list request should be built"),
        )
        .await
        .expect("trash list request should be handled");
    assert_eq!(trash_response.status(), StatusCode::OK);
    let trash_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(trash_response.into_body(), usize::MAX)
            .await
            .expect("trash response should be read"),
    )
    .expect("trash response should be valid json");
    assert_eq!(
        common::envelope_items(&trash_payload)
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        common::envelope_items(&trash_payload)[0]["id"].as_str(),
        Some("node-trashed")
    );

    let recent_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-views", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-views", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/recent?spaceId=space-views")
                .body(Body::empty())
                .expect("recent list request should be built"),
        )
        .await
        .expect("recent list request should be handled");
    assert_eq!(recent_response.status(), StatusCode::OK);
    let recent_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(recent_response.into_body(), usize::MAX)
            .await
            .expect("recent response should be read"),
    )
    .expect("recent response should be valid json");
    let recent_ids = common::envelope_items(&recent_payload)
        .as_array()
        .expect("recent items should be an array")
        .iter()
        .map(|item| item["id"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        recent_ids,
        vec![
            "node-favorite".to_string(),
            "node-shared".to_string(),
            "node-active".to_string()
        ]
    );
    let recent_items = common::envelope_items(&recent_payload)
        .as_array()
        .expect("recent items should be an array");
    assert!(
        recent_items.iter().all(|item| {
            item["createdAt"]
                .as_str()
                .is_some_and(|timestamp| timestamp.contains('T') && timestamp.ends_with('Z'))
        }),
        "recent nodes should expose their persisted creation timestamp as RFC3339 UTC"
    );
    assert_eq!(
        recent_items[0]["updatedAt"].as_str(),
        Some("2026-06-04T12:00:00Z"),
        "recent nodes should expose the persisted ordering timestamp as RFC3339 UTC"
    );

    let shared_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-views", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-views", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/shared_with_me")
                .body(Body::empty())
                .expect("shared with me request should be built"),
        )
        .await
        .expect("shared with me request should be handled");
    assert_eq!(shared_response.status(), StatusCode::OK);
    let shared_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(shared_response.into_body(), usize::MAX)
            .await
            .expect("shared response should be read"),
    )
    .expect("shared response should be valid json");
    assert_eq!(
        common::envelope_items(&shared_payload)
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let shared_ids = common::envelope_items(&shared_payload)
        .as_array()
        .expect("shared items should be an array")
        .iter()
        .map(|item| item["id"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        shared_ids,
        vec![
            "node-favorite".to_string(),
            "node-shared".to_string(),
            "node-active".to_string()
        ]
    );

    let favorites_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-views", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-views", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/favorites")
                .body(Body::empty())
                .expect("favorites request should be built"),
        )
        .await
        .expect("favorites request should be handled");
    assert_eq!(favorites_response.status(), StatusCode::OK);
    let favorites_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(favorites_response.into_body(), usize::MAX)
            .await
            .expect("favorites response should be read"),
    )
    .expect("favorites response should be valid json");
    assert_eq!(
        common::envelope_items(&favorites_payload)
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        common::envelope_items(&favorites_payload)[0]["id"].as_str(),
        Some("node-favorite")
    );

    let set_favorite_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-views", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-views", "user-reviewer", "appbase"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/nodes/node-active/favorite")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("set favorite request should be built"),
        )
        .await
        .expect("set favorite request should be handled");
    assert_eq!(set_favorite_response.status(), StatusCode::OK);
    let set_favorite_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(set_favorite_response.into_body(), usize::MAX)
            .await
            .expect("set favorite response should be read"),
    )
    .expect("set favorite response should be valid json");
    assert_eq!(
        common::envelope_data(&set_favorite_payload)["favorited"].as_bool(),
        Some(true)
    );

    let unset_favorite_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-views", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-views", "user-reviewer", "appbase"),
                )
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/nodes/node-active/favorite")
                .body(Body::empty())
                .expect("unset favorite request should be built"),
        )
        .await
        .expect("unset favorite request should be handled");
    common::assert_no_content_response(unset_favorite_response).await;

    let active_favorite_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node_favorite
         WHERE tenant_id='tenant-views'
           AND node_id='node-active'
           AND subject_type='user'
           AND subject_id='user-reviewer'
           AND lifecycle_status='active'",
    )
    .fetch_one(&pool)
    .await
    .expect("favorite count should be queryable");
    assert_eq!(active_favorite_count, 0);

    let changes_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-views", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-views", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/changes?spaceId=space-views")
                .body(Body::empty())
                .expect("changes request should be built"),
        )
        .await
        .expect("changes request should be handled");
    assert_eq!(changes_response.status(), StatusCode::OK);
    let changes_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(changes_response.into_body(), usize::MAX)
            .await
            .expect("changes response should be read"),
    )
    .expect("changes response should be valid json");
    let events = common::envelope_items(&changes_payload)
        .as_array()
        .expect("changes items should be an array")
        .iter()
        .map(|item| item["eventType"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert!(events.contains(&"drive.favorite.created".to_string()));
    assert!(events.contains(&"drive.favorite.deleted".to_string()));
}

#[tokio::test]
async fn app_drive_standard_views_validate_explicit_space_filter() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-views-deleted', 'tenant-view-filter', 'user', 'user-deleted-owner',
            'personal', 'Deleted Views', 'deleted', 1, 'user-deleted-owner', 'user-deleted-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("deleted space should be seeded");

    let app = common::test_router_with_pool(pool);
    for space_id in ["space-views-missing", "space-views-deleted"] {
        for uri in [
            format!("/app/v3/api/drive/trash?spaceId={space_id}"),
            format!("/app/v3/api/drive/recent?spaceId={space_id}"),
            format!("/app/v3/api/drive/search?spaceId={space_id}&q=doc"),
            format!("/app/v3/api/drive/shared_with_me?spaceId={space_id}"),
            format!("/app/v3/api/drive/favorites?spaceId={space_id}"),
        ] {
            let response = app
                .clone()
                .oneshot(common::authed_get_uri(uri.as_str(), "tenant-view-filter"))
                .await
                .expect("standard view invalid space request should be handled");
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
            let payload: serde_json::Value = serde_json::from_slice(
                &to_bytes(response.into_body(), usize::MAX)
                    .await
                    .expect("standard view invalid space response should be read"),
            )
            .expect("standard view invalid space response should be valid json");
            assert_eq!(payload["detail"].as_str(), Some("space not found"));
        }
    }
}

#[tokio::test]
async fn app_drive_list_routes_support_standard_page_tokens() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-page', 'tenant-page', 'user', 'user-page', 'personal', 'Paging', 'active', 1, 'user-page', 'user-page')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    seed_storage_metadata_provider_fixture(&pool, "provider-page", "bucket-page", "user-page")
        .await;
    for (id, node_name, updated_at) in [
        ("node-page-a", "a.txt", "2026-06-04 10:00:00"),
        ("node-page-b", "b.txt", "2026-06-04 11:00:00"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by, updated_at
            ) VALUES ($1, 'tenant-page', 'space-page', NULL, 'file', $2, 'ready', 'active', 1, 'user-page', 'user-page', $3)",
        )
        .bind(id)
        .bind(node_name)
        .bind(updated_at)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }
    for (id, subject_id) in [
        ("permission-page-a", "user-a"),
        ("permission-page-b", "user-b"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node_permission (
                id, tenant_id, node_id, subject_type, subject_id, role,
                inherited, lifecycle_status, version, created_by, updated_by
            ) VALUES (
                $1, 'tenant-page', 'node-page-a', 'user', $2,
                'reader', 0, 'active', 1, 'user-page', 'user-page'
            )",
        )
        .bind(id)
        .bind(subject_id)
        .execute(&pool)
        .await
        .expect("permission should be seeded");
    }
    for (id, token_hash, created_at) in [
        ("share-page-a", "sha256:share-page-a", "2026-06-04 10:00:00"),
        ("share-page-b", "sha256:share-page-b", "2026-06-04 11:00:00"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node_share_link (
                id, tenant_id, node_id, token_hash, role, download_count,
                lifecycle_status, version, created_by, updated_by, created_at
            ) VALUES (
                $1, 'tenant-page', 'node-page-a', $2, 'reader', 0,
                'active', 1, 'user-page', 'user-page', $3
            )",
        )
        .bind(id)
        .bind(token_hash)
        .bind(created_at)
        .execute(&pool)
        .await
        .expect("share link should be seeded");
    }
    for (id, version_no) in [("version-page-1", 1_i64), ("version-page-2", 2_i64)] {
        sqlx::query(
            "INSERT INTO dr_drive_storage_object (
                id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
                content_type, content_length, checksum_sha256_hex,
                lifecycle_status, created_by, updated_by
            ) VALUES ($1, 'tenant-page', 'node-page-a', $2, 'provider-page', 'bucket-page', $3, 'text/plain', 10, $4, 'active', 'user-page', 'user-page')",
        )
        .bind(id)
        .bind(version_no)
        .bind(format!("objects/node-page-a/v{version_no}.txt"))
        .bind(format!("sha256:{}", version_no.to_string().repeat(64)))
        .execute(&pool)
        .await
        .expect("version should be seeded");
    }

    let app = common::test_router_with_pool(pool);
    let (first_nodes, next_nodes_token) = fetch_paged_items(
        app.clone(),
        "/app/v3/api/drive/spaces/space-page/nodes?page_size=1",
        "tenant-page",
    )
    .await;
    assert_eq!(first_nodes[0]["id"].as_str(), Some("node-page-a"));
    let next_nodes_token = next_nodes_token.expect("nodes first page should have nextPageToken");
    let (second_nodes, _) = fetch_paged_items(
        app.clone(),
        &format!("/app/v3/api/drive/spaces/space-page/nodes?page_size=1&cursor={next_nodes_token}"),
        "tenant-page",
    )
    .await;
    assert_eq!(second_nodes[0]["id"].as_str(), Some("node-page-b"));

    let (first_recent, next_recent_token) = fetch_paged_items(
        app.clone(),
        "/app/v3/api/drive/recent?page_size=1",
        "tenant-page",
    )
    .await;
    assert_eq!(first_recent[0]["id"].as_str(), Some("node-page-b"));
    let next_recent_token = next_recent_token.expect("recent first page should have nextPageToken");
    let (second_recent, _) = fetch_paged_items(
        app.clone(),
        &format!("/app/v3/api/drive/recent?page_size=1&cursor={next_recent_token}"),
        "tenant-page",
    )
    .await;
    assert_eq!(second_recent[0]["id"].as_str(), Some("node-page-a"));

    let (first_permissions, next_permissions_token) = fetch_paged_items(
        app.clone(),
        "/app/v3/api/drive/nodes/node-page-a/permissions?page_size=1",
        "tenant-page",
    )
    .await;
    assert_eq!(
        first_permissions[0]["id"].as_str(),
        Some("permission-page-a")
    );
    let next_permissions_token =
        next_permissions_token.expect("permissions first page should have nextPageToken");
    let (second_permissions, _) = fetch_paged_items(
        app.clone(),
        &format!(
            "/app/v3/api/drive/nodes/node-page-a/permissions?page_size=1&cursor={next_permissions_token}"
        ),
    "tenant-page",
    )
    .await;
    assert_eq!(
        second_permissions[0]["id"].as_str(),
        Some("permission-page-b")
    );

    let (first_share_links, next_share_links_token) = fetch_paged_items(
        app.clone(),
        "/app/v3/api/drive/nodes/node-page-a/share_links?page_size=1",
        "tenant-page",
    )
    .await;
    assert_eq!(first_share_links[0]["id"].as_str(), Some("share-page-b"));
    let next_share_links_token =
        next_share_links_token.expect("share links first page should have nextPageToken");
    let (second_share_links, _) = fetch_paged_items(
        app.clone(),
        &format!(
            "/app/v3/api/drive/nodes/node-page-a/share_links?page_size=1&cursor={next_share_links_token}"
        ),
    "tenant-page",
    )
    .await;
    assert_eq!(second_share_links[0]["id"].as_str(), Some("share-page-a"));

    let (first_versions, next_versions_token) = fetch_paged_items(
        app.clone(),
        "/app/v3/api/drive/nodes/node-page-a/versions?page_size=1",
        "tenant-page",
    )
    .await;
    assert_eq!(first_versions[0]["id"].as_str(), Some("version-page-2"));
    let next_versions_token =
        next_versions_token.expect("versions first page should have nextPageToken");
    let (second_versions, _) = fetch_paged_items(
        app,
        &format!(
            "/app/v3/api/drive/nodes/node-page-a/versions?page_size=1&cursor={next_versions_token}"
        ),
        "tenant-page",
    )
    .await;
    assert_eq!(second_versions[0]["id"].as_str(), Some("version-page-1"));
}

#[tokio::test]
async fn app_drive_move_destinations_pages_without_replaying_prior_folders() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-move-destination-page', 'tenant-move-destination-page',
            'user', 'user-move-destination-page', 'personal',
            'Move Destination Paging', 'active', 1,
            'user-move-destination-page', 'user-move-destination-page'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    for (id, node_name) in [
        ("folder-move-page-alpha", "Alpha"),
        ("folder-move-page-beta", "Beta"),
        ("folder-move-page-gamma", "Gamma"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES (
                $1, 'tenant-move-destination-page', 'space-move-destination-page',
                NULL, 'folder', $2, 'ready', 'active', 1,
                'user-move-destination-page', 'user-move-destination-page'
            )",
        )
        .bind(id)
        .bind(node_name)
        .execute(&pool)
        .await
        .expect("destination folder should be seeded");
    }

    let app = common::test_router_with_pool(pool);
    let (first_items, first_next) = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/spaces/space-move-destination-page/move_destinations?page_size=1",
        "tenant-move-destination-page",
        "user-move-destination-page",
    )
    .await;
    assert_eq!(first_items.len(), 1);
    assert_eq!(
        first_items[0]["id"].as_str(),
        Some("folder-move-page-alpha")
    );
    let first_next = first_next.expect("first page should expose next cursor");

    let (second_items, second_next) = fetch_paged_items_as(
        app.clone(),
        &format!(
            "/app/v3/api/drive/spaces/space-move-destination-page/move_destinations?page_size=1&cursor={first_next}"
        ),
        "tenant-move-destination-page",
        "user-move-destination-page",
    )
    .await;
    assert_eq!(second_items.len(), 1);
    assert_eq!(
        second_items[0]["id"].as_str(),
        Some("folder-move-page-beta")
    );
    let second_next = second_next.expect("second page should expose next cursor");

    let (third_items, third_next) = fetch_paged_items_as(
        app,
        &format!(
            "/app/v3/api/drive/spaces/space-move-destination-page/move_destinations?page_size=1&cursor={second_next}"
        ),
        "tenant-move-destination-page",
        "user-move-destination-page",
    )
    .await;
    assert_eq!(third_items.len(), 1);
    assert_eq!(
        third_items[0]["id"].as_str(),
        Some("folder-move-page-gamma")
    );
    assert!(third_next.is_none());
}

#[tokio::test]
async fn app_drive_list_routes_reject_page_size_outside_contract() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-list-page-size', 'tenant-list-page-size', 'user', 'user-list-page-size', 'personal', 'List Page Size', 'active', 1, 'user-list-page-size', 'user-list-page-size')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-list-page-size",
                            "user-list-page-size",
                            "appbase"
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-list-page-size", "user-list-page-size", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces/space-list-page-size/nodes?page_size=0")
                .body(Body::empty())
                .expect("nodes list request should be built"),
        )
        .await
        .expect("nodes list request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("error body should be readable"),
    )
    .expect("error body should be valid json");
    assert_eq!(payload["code"], 40001);
    assert!(payload["detail"]
        .as_str()
        .expect("detail should be present")
        .contains("page_size"));
}

#[tokio::test]
async fn app_drive_effective_permissions_include_direct_inherited_acl_and_page_tokens() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-effective-perm', 'tenant-effective-perm', 'user', 'user-owner', 'personal', 'Effective Permissions', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    for (id, parent_node_id, node_type, node_name) in [
        ("node-effective-root", None, "folder", "Root"),
        (
            "node-effective-folder",
            Some("node-effective-root"),
            "folder",
            "Folder",
        ),
        (
            "node-effective-file",
            Some("node-effective-folder"),
            "file",
            "report.pdf",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-effective-perm', 'space-effective-perm', $2, $3, $4, 'ready', 'active', 1, 'user-owner', 'user-owner')",
        )
        .bind(id)
        .bind(parent_node_id)
        .bind(node_type)
        .bind(node_name)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }
    for (id, node_id, subject_type, subject_id, role) in [
        (
            "perm-effective-root",
            "node-effective-root",
            "user",
            "user-root-reader",
            "reader",
        ),
        (
            "perm-effective-folder",
            "node-effective-folder",
            "group",
            "group-editors",
            "writer",
        ),
        (
            "perm-effective-file",
            "node-effective-file",
            "user",
            "user-file-commenter",
            "commenter",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node_permission (
                id, tenant_id, node_id, subject_type, subject_id, role,
                inherited, lifecycle_status, version, created_by, updated_by
            ) VALUES (
                $1, 'tenant-effective-perm', $2, $3, $4,
                $5, 0, 'active', 1, 'user-owner', 'user-owner'
            )",
        )
        .bind(id)
        .bind(node_id)
        .bind(subject_type)
        .bind(subject_id)
        .bind(role)
        .execute(&pool)
        .await
        .expect("permission should be seeded");
    }

    let app = common::test_router_with_pool(pool);
    let (first_items, next_token) = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/nodes/node-effective-file/permissions/effective?page_size=2",
        "tenant-effective-perm",
        "user-owner",
    )
    .await;

    assert_eq!(first_items.len(), 2);
    assert_eq!(first_items[0]["id"].as_str(), Some("perm-effective-file"));
    assert_eq!(
        first_items[0]["targetNodeId"].as_str(),
        Some("node-effective-file")
    );
    assert_eq!(
        first_items[0]["nodeId"].as_str(),
        Some("node-effective-file")
    );
    assert_eq!(first_items[0]["inherited"].as_bool(), Some(false));
    assert!(
        first_items[0].get("inheritedFromNodeId").is_some(),
        "direct permissions should expose inheritedFromNodeId as null"
    );
    assert!(first_items[0]["inheritedFromNodeId"].is_null());
    assert_eq!(first_items[1]["id"].as_str(), Some("perm-effective-folder"));
    assert_eq!(
        first_items[1]["targetNodeId"].as_str(),
        Some("node-effective-file")
    );
    assert_eq!(
        first_items[1]["nodeId"].as_str(),
        Some("node-effective-folder")
    );
    assert_eq!(first_items[1]["inherited"].as_bool(), Some(true));
    assert_eq!(
        first_items[1]["inheritedFromNodeId"].as_str(),
        Some("node-effective-folder")
    );

    let next_token = next_token.expect("effective permissions first page should have token");
    let (second_items, final_token) = fetch_paged_items_as(
        app,
        &format!(
            "/app/v3/api/drive/nodes/node-effective-file/permissions/effective?page_size=2&cursor={next_token}"
        ),
        "tenant-effective-perm",
        "user-owner",
    )
    .await;
    assert_eq!(second_items.len(), 1);
    assert_eq!(second_items[0]["id"].as_str(), Some("perm-effective-root"));
    assert_eq!(
        second_items[0]["nodeId"].as_str(),
        Some("node-effective-root")
    );
    assert_eq!(second_items[0]["inherited"].as_bool(), Some(true));
    assert_eq!(
        second_items[0]["inheritedFromNodeId"].as_str(),
        Some("node-effective-root")
    );
    assert!(final_token.is_none());
}

#[tokio::test]
async fn app_drive_effective_permissions_prefer_direct_then_nearest_acl_for_same_subject() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-effective-override', 'tenant-effective-override', 'user', 'user-owner', 'personal', 'Effective Overrides', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    for (id, parent_node_id, node_type, node_name) in [
        ("node-override-root", None, "folder", "Root"),
        (
            "node-override-folder",
            Some("node-override-root"),
            "folder",
            "Folder",
        ),
        (
            "node-override-file",
            Some("node-override-folder"),
            "file",
            "report.pdf",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-effective-override', 'space-effective-override', $2, $3, $4, 'ready', 'active', 1, 'user-owner', 'user-owner')",
        )
        .bind(id)
        .bind(parent_node_id)
        .bind(node_type)
        .bind(node_name)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }
    for (id, node_id, role) in [
        ("perm-overlap-root", "node-override-root", "reader"),
        ("perm-overlap-folder", "node-override-folder", "writer"),
        ("perm-overlap-file", "node-override-file", "commenter"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node_permission (
                id, tenant_id, node_id, subject_type, subject_id, role,
                inherited, lifecycle_status, version, created_by, updated_by
            ) VALUES (
                $1, 'tenant-effective-override', $2, 'user', 'user-overlap',
                $3, 0, 'active', 1, 'user-owner', 'user-owner'
            )",
        )
        .bind(id)
        .bind(node_id)
        .bind(role)
        .execute(&pool)
        .await
        .expect("permission should be seeded");
    }

    let app = common::test_router_with_pool(pool);
    let (items, next_token) = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/nodes/node-override-file/permissions/effective",
        "tenant-effective-override",
        "user-owner",
    )
    .await;

    assert_eq!(
        items.len(),
        1,
        "effective permissions should collapse duplicate principals"
    );
    assert_eq!(items[0]["id"].as_str(), Some("perm-overlap-file"));
    assert_eq!(items[0]["role"].as_str(), Some("commenter"));
    assert_eq!(items[0]["inherited"].as_bool(), Some(false));
    assert!(items[0]["inheritedFromNodeId"].is_null());
    assert!(next_token.is_none());
}

#[tokio::test]
async fn app_dr_drive_node_capabilities_resolve_direct_inherited_owner_and_missing_access() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-capability', 'tenant-capability', 'user', 'user-owner', 'team', 'Capabilities', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    for (id, parent_node_id, node_type, node_name) in [
        ("node-cap-root", None, "folder", "Root"),
        ("node-cap-folder", Some("node-cap-root"), "folder", "Folder"),
        (
            "node-cap-file",
            Some("node-cap-folder"),
            "file",
            "capabilities.pdf",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-capability', 'space-capability', $2, $3, $4, 'ready', 'active', 1, 'user-owner', 'user-owner')",
        )
        .bind(id)
        .bind(parent_node_id)
        .bind(node_type)
        .bind(node_name)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }
    for (id, node_id, subject_id, role) in [
        (
            "perm-cap-inherited",
            "node-cap-folder",
            "user-inherited-writer",
            "writer",
        ),
        (
            "perm-cap-direct",
            "node-cap-file",
            "user-direct-commenter",
            "commenter",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node_permission (
                id, tenant_id, node_id, subject_type, subject_id, role,
                inherited, lifecycle_status, version, created_by, updated_by
            ) VALUES (
                $1, 'tenant-capability', $2, 'user', $3,
                $4, 0, 'active', 1, 'user-owner', 'user-owner'
            )",
        )
        .bind(id)
        .bind(node_id)
        .bind(subject_id)
        .bind(role)
        .execute(&pool)
        .await
        .expect("permission should be seeded");
    }

    let app = common::test_router_with_pool(pool);
    let direct = fetch_resource_json_as(
        app.clone(),
        "/app/v3/api/drive/nodes/node-cap-file/capabilities",
        "tenant-capability",
        "user-direct-commenter",
    )
    .await;
    assert_eq!(direct["role"].as_str(), Some("commenter"));
    assert_eq!(direct["source"].as_str(), Some("permission"));
    assert_eq!(direct["inherited"].as_bool(), Some(false));
    assert!(direct["inheritedFromNodeId"].is_null());
    assert_eq!(direct["canRead"].as_bool(), Some(true));
    assert_eq!(direct["canComment"].as_bool(), Some(true));
    assert_eq!(direct["canWrite"].as_bool(), Some(false));
    assert_eq!(direct["canDownload"].as_bool(), Some(true));
    assert_eq!(direct["canManagePermissions"].as_bool(), Some(false));

    let inherited = fetch_resource_json_as(
        app.clone(),
        "/app/v3/api/drive/nodes/node-cap-file/capabilities",
        "tenant-capability",
        "user-inherited-writer",
    )
    .await;
    assert_eq!(inherited["role"].as_str(), Some("writer"));
    assert_eq!(inherited["source"].as_str(), Some("permission"));
    assert_eq!(inherited["inherited"].as_bool(), Some(true));
    assert_eq!(
        inherited["inheritedFromNodeId"].as_str(),
        Some("node-cap-folder")
    );
    assert_eq!(inherited["canWrite"].as_bool(), Some(true));
    assert_eq!(inherited["canShare"].as_bool(), Some(true));
    assert_eq!(inherited["canManageVersions"].as_bool(), Some(true));
    assert_eq!(inherited["canDelete"].as_bool(), Some(false));

    let owner = fetch_resource_json_as(
        app.clone(),
        "/app/v3/api/drive/nodes/node-cap-file/capabilities",
        "tenant-capability",
        "user-owner",
    )
    .await;
    assert_eq!(owner["role"].as_str(), Some("owner"));
    assert_eq!(owner["source"].as_str(), Some("space_owner"));
    assert_eq!(owner["canManagePermissions"].as_bool(), Some(true));
    assert_eq!(owner["canDelete"].as_bool(), Some(true));

    let missing_response = app
        .clone()
        .oneshot(common::authed_get(
            "/app/v3/api/drive/nodes/node-cap-file/capabilities",
            "tenant-capability",
            "user-missing",
            "appbase",
        ))
        .await
        .expect("missing capabilities request should be handled");
    assert_eq!(missing_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn app_dr_drive_node_capabilities_support_trashed_nodes_with_restore_only_actions() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-cap-trash', 'tenant-cap-trash', 'user', 'user-owner',
            'team', 'Trash Capabilities', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-cap-trash', 'tenant-cap-trash', 'space-cap-trash',
            NULL, 'file', 'trashed.pdf', 'ready', 'trashed', 1,
            'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("trashed node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-cap-trash-writer', 'tenant-cap-trash', 'node-cap-trash',
            'user', 'user-writer', 'writer', 0, 'active', 1,
            'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("writer permission should be seeded");

    let app = common::test_router_with_pool(pool);
    let writer = fetch_resource_json_as(
        app.clone(),
        "/app/v3/api/drive/nodes/node-cap-trash/capabilities",
        "tenant-cap-trash",
        "user-writer",
    )
    .await;
    assert_eq!(writer["role"].as_str(), Some("writer"));
    assert_eq!(writer["source"].as_str(), Some("permission"));
    assert_eq!(writer["canRead"].as_bool(), Some(true));
    assert_eq!(writer["canRestore"].as_bool(), Some(true));
    assert_eq!(writer["canDelete"].as_bool(), Some(false));
    for key in [
        "canComment",
        "canWrite",
        "canDownload",
        "canCopy",
        "canMove",
        "canTrash",
        "canShare",
        "canManagePermissions",
        "canManageVersions",
    ] {
        assert_eq!(writer[key].as_bool(), Some(false), "{key} should be false");
    }

    let owner = fetch_resource_json_as(
        app.clone(),
        "/app/v3/api/drive/nodes/node-cap-trash/capabilities",
        "tenant-cap-trash",
        "user-owner",
    )
    .await;
    assert_eq!(owner["role"].as_str(), Some("owner"));
    assert_eq!(owner["source"].as_str(), Some("space_owner"));
    assert_eq!(owner["canRead"].as_bool(), Some(true));
    assert_eq!(owner["canRestore"].as_bool(), Some(true));
    assert_eq!(owner["canDelete"].as_bool(), Some(true));
    assert_eq!(owner["canWrite"].as_bool(), Some(false));
    assert_eq!(owner["canShare"].as_bool(), Some(false));
    assert_eq!(owner["canManagePermissions"].as_bool(), Some(false));
    assert_eq!(owner["canManageVersions"].as_bool(), Some(false));
}

#[tokio::test]
async fn app_dr_drive_node_properties_support_custom_metadata_lifecycle_and_page_tokens() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-property', 'tenant-property', 'user', 'user-owner', 'team', 'Properties', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-property', 'tenant-property', 'space-property', NULL, 'file', 'metadata.pdf', 'ready', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");

    let app = common::test_router_with_pool(pool);
    let first_set = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-property", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-property", "user-owner", "appbase"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/nodes/node-property/properties/customerId")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "value":"cust-001",
                        "visibility":"private"
                    }"#,
                ))
                .expect("set property request should be built"),
        )
        .await
        .expect("set property request should be handled");
    assert_eq!(first_set.status(), StatusCode::OK);
    let first_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(first_set.into_body(), usize::MAX)
            .await
            .expect("set property response should be read"),
    )
    .expect("set property response should be valid json");
    let first_item = common::envelope_item(&first_payload);
    assert_eq!(first_item["propertyKey"].as_str(), Some("customerId"));
    assert_eq!(first_item["propertyValue"].as_str(), Some("cust-001"));
    assert_eq!(first_item["visibility"].as_str(), Some("private"));
    assert_eq!(first_item["version"].as_i64(), Some(1));

    let second_set = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-property", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-property", "user-owner", "appbase"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/nodes/node-property/properties/orderId")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "value":"order-001",
                        "visibility":"app_public"
                    }"#,
                ))
                .expect("set second property request should be built"),
        )
        .await
        .expect("set second property request should be handled");
    assert_eq!(second_set.status(), StatusCode::OK);

    let update_set = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-property", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-property", "user-owner", "appbase"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/nodes/node-property/properties/customerId")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "value":"cust-002",
                        "visibility":"private"
                    }"#,
                ))
                .expect("update property request should be built"),
        )
        .await
        .expect("update property request should be handled");
    assert_eq!(update_set.status(), StatusCode::OK);
    let update_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(update_set.into_body(), usize::MAX)
            .await
            .expect("update property response should be read"),
    )
    .expect("update property response should be valid json");
    let update_item = common::envelope_item(&update_payload);
    assert_eq!(update_item["propertyValue"].as_str(), Some("cust-002"));
    assert_eq!(update_item["version"].as_i64(), Some(2));

    let (first_items, next_token) = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/nodes/node-property/properties?page_size=1",
        "tenant-property",
        "user-owner",
    )
    .await;
    assert_eq!(first_items.len(), 1);
    assert_eq!(first_items[0]["propertyKey"].as_str(), Some("customerId"));
    let next_token = next_token.expect("node properties should expose next page token");
    let (second_items, final_token) = fetch_paged_items_as(
        app.clone(),
        &format!(
            "/app/v3/api/drive/nodes/node-property/properties?page_size=1&cursor={next_token}"
        ),
        "tenant-property",
        "user-owner",
    )
    .await;
    assert_eq!(second_items.len(), 1);
    assert_eq!(second_items[0]["propertyKey"].as_str(), Some("orderId"));
    assert!(final_token.is_none());

    let private_only = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/nodes/node-property/properties?visibility=private",
        "tenant-property",
        "user-owner",
    )
    .await
    .0;
    assert_eq!(private_only.len(), 1);
    assert_eq!(private_only[0]["propertyKey"].as_str(), Some("customerId"));

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-property", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-property", "user-owner", "appbase"),
                )
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/nodes/node-property/properties/customerId")
                .body(Body::empty())
                .expect("delete property request should be built"),
        )
        .await
        .expect("delete property request should be handled");
    common::assert_no_content_response(delete_response).await;

    let remaining = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/nodes/node-property/properties",
        "tenant-property",
        "user-owner",
    )
    .await
    .0;
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0]["propertyKey"].as_str(), Some("orderId"));

    let changes = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/changes?spaceId=space-property",
        "tenant-property",
        "user-owner",
    )
    .await
    .0;
    let events = changes
        .iter()
        .map(|item| item["eventType"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert!(events.contains(&"drive.node_property.set".to_string()));
    assert!(events.contains(&"drive.node_property.deleted".to_string()));
}

#[tokio::test]
async fn app_drive_collaboration_and_metadata_writes_reject_trashed_nodes_without_side_effects() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-trashed-write', 'tenant-trashed-write', 'user', 'user-owner',
            'team', 'Trashed Write', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-trashed-write', 'tenant-trashed-write', 'space-trashed-write',
            NULL, 'file', 'trashed-write.pdf', 'ready', 'trashed', 1,
            'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("trashed node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_label (
            id, tenant_id, label_key, display_name, color, description,
            lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'label-trashed-write', 'tenant-trashed-write',
            'classification.trashed', 'Trashed', '#344054', NULL,
            'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("label should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let mut observed = Vec::<(&'static str, StatusCode)>::new();
    for (name, request) in [
        (
            "property",
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-trashed-write", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-trashed-write", "user-owner", "appbase"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/nodes/node-trashed-write/properties/customerId")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "value":"cust-trashed"
                    }"#,
                ))
                .expect("set property request should be built"),
        ),
        (
            "label",
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-trashed-write", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-trashed-write", "user-owner", "appbase"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/nodes/node-trashed-write/labels/label-trashed-write")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("apply label request should be built"),
        ),
        (
            "favorite",
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-trashed-write", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-trashed-write", "user-owner", "appbase"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/nodes/node-trashed-write/favorite")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("favorite request should be built"),
        ),
        (
            "permission",
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-trashed-write", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-trashed-write", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-trashed-write/permissions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"permission-trashed-write",
                        "subjectType":"user",
                        "subjectId":"user-reviewer",
                        "role":"reader"
                    }"#,
                ))
                .expect("permission request should be built"),
        ),
        (
            "share_link",
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-trashed-write", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-trashed-write", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-trashed-write/share_links")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"share-trashed-write"
                    }"#,
                ))
                .expect("share link request should be built"),
        ),
        (
            "comment",
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-trashed-write", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-trashed-write", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-trashed-write/comments")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"comment-trashed-write",
                        "content":"This should not be written."
                    }"#,
                ))
                .expect("comment request should be built"),
        ),
    ] {
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("trashed node write request should be handled");
        observed.push((name, response.status()));
        if response.status() == StatusCode::NOT_FOUND {
            let payload: serde_json::Value = serde_json::from_slice(
                &to_bytes(response.into_body(), usize::MAX)
                    .await
                    .expect("not found response should be read"),
            )
            .expect("not found response should be valid json");
            assert_eq!(payload["code"].as_i64(), Some(40401), "{name}");
        }
    }
    assert_eq!(
        observed,
        vec![
            ("property", StatusCode::NOT_FOUND),
            ("label", StatusCode::NOT_FOUND),
            ("favorite", StatusCode::NOT_FOUND),
            ("permission", StatusCode::NOT_FOUND),
            ("share_link", StatusCode::NOT_FOUND),
            ("comment", StatusCode::NOT_FOUND),
        ]
    );

    for (table, expected_count) in [
        ("dr_drive_node_property", 0_i64),
        ("dr_drive_node_label", 0_i64),
        ("dr_drive_node_favorite", 0_i64),
        ("dr_drive_node_permission", 0_i64),
        ("dr_drive_node_share_link", 0_i64),
        ("dr_drive_node_comment", 0_i64),
    ] {
        let query = format!("SELECT COUNT(1) FROM {table} WHERE tenant_id='tenant-trashed-write'");
        let count: i64 = sqlx::query_scalar(&query)
            .fetch_one(&pool)
            .await
            .expect("side effect count should be queryable");
        assert_eq!(count, expected_count, "{table} should not be mutated");
    }
}

#[tokio::test]
async fn app_drive_metadata_deletes_reject_trashed_nodes_without_side_effects() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-trashed-delete', 'tenant-trashed-delete', 'user', 'user-owner',
            'team', 'Trashed Delete', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-trashed-delete', 'tenant-trashed-delete', 'space-trashed-delete',
            NULL, 'file', 'trashed-delete.pdf', 'ready', 'trashed', 1,
            'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("trashed node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_property (
            id, tenant_id, node_id, property_key, property_value, visibility,
            lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'property-trashed-delete', 'tenant-trashed-delete', 'node-trashed-delete',
            'customerId', 'cust-original', 'private', 'active', 1,
            'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("property should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_label (
            id, tenant_id, label_key, display_name, color, description,
            lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'label-trashed-delete', 'tenant-trashed-delete',
            'classification.delete', 'Delete Guard', '#344054', NULL,
            'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("label should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_label (
            id, tenant_id, node_id, label_id, lifecycle_status,
            version, created_by, updated_by
        ) VALUES (
            'node-label-trashed-delete', 'tenant-trashed-delete',
            'node-trashed-delete', 'label-trashed-delete',
            'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("node label should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_favorite (
            id, tenant_id, node_id, subject_type, subject_id,
            lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'favorite-trashed-delete', 'tenant-trashed-delete',
            'node-trashed-delete', 'user', 'user-owner',
            'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("favorite should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let mut observed = Vec::<(&'static str, StatusCode)>::new();
    for (name, request) in [
        (
            "property.delete",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-trashed-delete", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-trashed-delete", "user-owner", "appbase"))
                .method(Method::DELETE)
                .uri(
                    "/app/v3/api/drive/nodes/node-trashed-delete/properties/customerId?visibility=private",
                )
                .body(Body::empty())
                .expect("property delete request should be built"),
        ),
        (
            "label.remove",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-trashed-delete", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-trashed-delete", "user-owner", "appbase"))
                .method(Method::DELETE)
                .uri(
                    "/app/v3/api/drive/nodes/node-trashed-delete/labels/label-trashed-delete",
                )
                .body(Body::empty())
                .expect("label remove request should be built"),
        ),
        (
            "favorite.unset",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-trashed-delete", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-trashed-delete", "user-owner", "appbase"))
                .method(Method::DELETE)
                .uri(
                    "/app/v3/api/drive/nodes/node-trashed-delete/favorite",
                )
                .body(Body::empty())
                .expect("favorite unset request should be built"),
        ),
    ] {
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("trashed node metadata delete request should be handled");
        observed.push((name, response.status()));
        if response.status() == StatusCode::NOT_FOUND {
            let payload: serde_json::Value = serde_json::from_slice(
                &to_bytes(response.into_body(), usize::MAX)
                    .await
                    .expect("not found response should be read"),
            )
            .expect("not found response should be valid json");
            assert_eq!(payload["code"].as_i64(), Some(40401), "{name}");
        }
    }

    assert_eq!(
        observed,
        vec![
            ("property.delete", StatusCode::NOT_FOUND),
            ("label.remove", StatusCode::NOT_FOUND),
            ("favorite.unset", StatusCode::NOT_FOUND),
        ]
    );

    for (table, expected_status) in [
        ("dr_drive_node_property", "active"),
        ("dr_drive_node_label", "active"),
        ("dr_drive_node_favorite", "active"),
    ] {
        let query =
            format!("SELECT lifecycle_status FROM {table} WHERE tenant_id='tenant-trashed-delete'");
        let status: String = sqlx::query_scalar(&query)
            .fetch_one(&pool)
            .await
            .expect("metadata status should be queryable");
        assert_eq!(status, expected_status, "{table} should not be mutated");
    }

    let change_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_change_log
         WHERE tenant_id='tenant-trashed-delete'",
    )
    .fetch_one(&pool)
    .await
    .expect("change count should be readable");
    assert_eq!(change_count, 0);
}

#[tokio::test]
async fn app_drive_collaboration_updates_and_versions_reject_trashed_nodes_without_side_effects() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-trashed-update', 'tenant-trashed-update', 'user', 'user-owner',
            'team', 'Trashed Update', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-trashed-update', 'tenant-trashed-update', 'space-trashed-update',
            NULL, 'file', 'trashed-update.pdf', 'ready', 'trashed', 1,
            'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("trashed node should be seeded");
    seed_storage_metadata_provider_fixture(
        &pool,
        "provider-trashed-update",
        "bucket-trashed-update",
        "user-owner",
    )
    .await;
    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'permission-trashed-update', 'tenant-trashed-update', 'node-trashed-update',
            'user', 'user-reviewer', 'reader', 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("permission should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version,
            created_by, updated_by
        ) VALUES (
            'share-trashed-update', 'tenant-trashed-update', 'node-trashed-update',
            'sha256:trashed-update-token', 'reader', 1800000000000,
            5, 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("share link should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_comment (
            id, tenant_id, node_id, content, anchor, resolved, lifecycle_status,
            version, created_by, updated_by
        ) VALUES (
            'comment-trashed-update', 'tenant-trashed-update', 'node-trashed-update',
            'Original comment', '$.body[0]', 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("comment should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_comment_reply (
            id, tenant_id, node_id, comment_id, content, lifecycle_status,
            version, created_by, updated_by
        ) VALUES (
            'reply-trashed-update', 'tenant-trashed-update', 'node-trashed-update',
            'comment-trashed-update', 'Original reply', 'active', 1,
            'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("comment reply should be seeded");
    for (id, version_no, lifecycle_status, checksum) in [
        (
            "version-trashed-restore",
            1_i64,
            "deleted",
            "sha256:1111111111111111111111111111111111111111111111111111111111111111",
        ),
        (
            "version-trashed-delete-a",
            2_i64,
            "active",
            "sha256:2222222222222222222222222222222222222222222222222222222222222222",
        ),
        (
            "version-trashed-delete-b",
            3_i64,
            "active",
            "sha256:3333333333333333333333333333333333333333333333333333333333333333",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_storage_object (
                id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
                content_type, content_length, checksum_sha256_hex, lifecycle_status,
                created_by, updated_by
            ) VALUES (
                $1, 'tenant-trashed-update', 'node-trashed-update', $2,
                'provider-trashed-update', 'bucket-trashed-update', $3, 'application/pdf', 128, $4, $5,
                'user-owner', 'user-owner'
            )",
        )
        .bind(id)
        .bind(version_no)
        .bind(format!("objects/node-trashed-update/v{version_no}.pdf"))
        .bind(checksum)
        .bind(lifecycle_status)
        .execute(&pool)
        .await
        .expect("storage version should be seeded");
    }

    let app = common::test_router_with_pool(pool.clone());
    let mut observed = Vec::<(&'static str, StatusCode)>::new();
    for (name, request) in [
        (
            "permission.update",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-trashed-update", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-trashed-update", "user-owner", "appbase"))
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/nodes/node-trashed-update/permissions/permission-trashed-update")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "role":"writer"
                }"#,
                ))
                .expect("permission update request should be built"),
        ),
        (
            "permission.delete",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-trashed-update", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-trashed-update", "user-owner", "appbase"))
                .method(Method::DELETE)
                .uri(
                    "/app/v3/api/drive/nodes/node-trashed-update/permissions/permission-trashed-update",
                )
                .body(Body::empty())
                .expect("permission delete request should be built"),
        ),
        (
            "share_link.update",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-trashed-update", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-trashed-update", "user-owner", "appbase"))
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/share_links/share-trashed-update")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "role":"commenter",
                        "downloadLimit":9
                }"#,
                ))
                .expect("share link update request should be built"),
        ),
        (
            "share_link.revoke",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-trashed-update", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-trashed-update", "user-owner", "appbase"))
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/share_links/share-trashed-update")
                .body(Body::empty())
                .expect("share link revoke request should be built"),
        ),
        (
            "comment.update",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-trashed-update", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-trashed-update", "user-owner", "appbase"))
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/nodes/node-trashed-update/comments/comment-trashed-update")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "content":"Mutated comment",
                        "resolved":true
                }"#,
                ))
                .expect("comment update request should be built"),
        ),
        (
            "comment.delete",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-trashed-update", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-trashed-update", "user-owner", "appbase"))
                .method(Method::DELETE)
                .uri(
                    "/app/v3/api/drive/nodes/node-trashed-update/comments/comment-trashed-update",
                )
                .body(Body::empty())
                .expect("comment delete request should be built"),
        ),
        (
            "comment_reply.create",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-trashed-update", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-trashed-update", "user-owner", "appbase"))
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-trashed-update/comments/comment-trashed-update/replies")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "id":"reply-trashed-new",
                        "content":"New reply"
                }"#,
                ))
                .expect("comment reply create request should be built"),
        ),
        (
            "comment_reply.update",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-trashed-update", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-trashed-update", "user-owner", "appbase"))
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/nodes/node-trashed-update/comments/comment-trashed-update/replies/reply-trashed-update")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                        "content":"Mutated reply"
                }"#,
                ))
                .expect("comment reply update request should be built"),
        ),
        (
            "comment_reply.delete",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-trashed-update", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-trashed-update", "user-owner", "appbase"))
                .method(Method::DELETE)
                .uri(
                    "/app/v3/api/drive/nodes/node-trashed-update/comments/comment-trashed-update/replies/reply-trashed-update",
                )
                .body(Body::empty())
                .expect("comment reply delete request should be built"),
        ),
        (
            "version.restore",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-trashed-update", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-trashed-update", "user-owner", "appbase"))
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-trashed-update/versions/version-trashed-restore/restore")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#,
                ))
                .expect("version restore request should be built"),
        ),
        (
            "version.delete",
            Request::builder()
            .header(
                "authorization",
                format!("Bearer {}", common::auth_token("tenant-trashed-update", "user-owner", "appbase")),
            )
            .header("access-token", common::access_token("tenant-trashed-update", "user-owner", "appbase"))
                .method(Method::DELETE)
                .uri(
                    "/app/v3/api/drive/nodes/node-trashed-update/versions/version-trashed-delete-a",
                )
                .body(Body::empty())
                .expect("version delete request should be built"),
        ),
    ] {
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("trashed node update request should be handled");
        observed.push((name, response.status()));
        if response.status() == StatusCode::NOT_FOUND {
            let payload: serde_json::Value = serde_json::from_slice(
                &to_bytes(response.into_body(), usize::MAX)
                    .await
                    .expect("not found response should be read"),
            )
            .expect("not found response should be valid json");
            assert_eq!(payload["code"].as_i64(), Some(40401), "{name}");
        }
    }
    assert_eq!(
        observed,
        vec![
            ("permission.update", StatusCode::NOT_FOUND),
            ("permission.delete", StatusCode::NOT_FOUND),
            ("share_link.update", StatusCode::NOT_FOUND),
            ("share_link.revoke", StatusCode::NOT_FOUND),
            ("comment.update", StatusCode::NOT_FOUND),
            ("comment.delete", StatusCode::NOT_FOUND),
            ("comment_reply.create", StatusCode::NOT_FOUND),
            ("comment_reply.update", StatusCode::NOT_FOUND),
            ("comment_reply.delete", StatusCode::NOT_FOUND),
            ("version.restore", StatusCode::NOT_FOUND),
            ("version.delete", StatusCode::NOT_FOUND),
        ]
    );

    let permission: (String, String, i64) = sqlx::query_as(
        "SELECT role, lifecycle_status, version
         FROM dr_drive_node_permission
         WHERE tenant_id='tenant-trashed-update' AND id='permission-trashed-update'",
    )
    .fetch_one(&pool)
    .await
    .expect("permission should be readable");
    assert_eq!(permission, ("reader".to_string(), "active".to_string(), 1));

    let share_link: (String, i64, String, i64) = sqlx::query_as(
        "SELECT role, download_limit, lifecycle_status, version
         FROM dr_drive_node_share_link
         WHERE tenant_id='tenant-trashed-update' AND id='share-trashed-update'",
    )
    .fetch_one(&pool)
    .await
    .expect("share link should be readable");
    assert_eq!(
        share_link,
        ("reader".to_string(), 5, "active".to_string(), 1)
    );

    let comment: (String, i64, String, i64) = sqlx::query_as(
        "SELECT content, resolved, lifecycle_status, version
         FROM dr_drive_node_comment
         WHERE tenant_id='tenant-trashed-update' AND id='comment-trashed-update'",
    )
    .fetch_one(&pool)
    .await
    .expect("comment should be readable");
    assert_eq!(
        comment,
        ("Original comment".to_string(), 0, "active".to_string(), 1)
    );

    let reply: (String, String, i64) = sqlx::query_as(
        "SELECT content, lifecycle_status, version
         FROM dr_drive_node_comment_reply
         WHERE tenant_id='tenant-trashed-update' AND id='reply-trashed-update'",
    )
    .fetch_one(&pool)
    .await
    .expect("reply should be readable");
    assert_eq!(
        reply,
        ("Original reply".to_string(), "active".to_string(), 1)
    );

    let new_reply_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node_comment_reply
         WHERE tenant_id='tenant-trashed-update' AND id='reply-trashed-new'",
    )
    .fetch_one(&pool)
    .await
    .expect("new reply count should be readable");
    assert_eq!(new_reply_count, 0);

    let restored_version_status: String = sqlx::query_scalar(
        "SELECT lifecycle_status
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-trashed-update' AND id='version-trashed-restore'",
    )
    .fetch_one(&pool)
    .await
    .expect("restored version status should be readable");
    assert_eq!(restored_version_status, "deleted");

    let deleted_version_status: String = sqlx::query_scalar(
        "SELECT lifecycle_status
         FROM dr_drive_storage_object
         WHERE tenant_id='tenant-trashed-update' AND id='version-trashed-delete-a'",
    )
    .fetch_one(&pool)
    .await
    .expect("deleted version status should be readable");
    assert_eq!(deleted_version_status, "active");

    let change_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_change_log
         WHERE tenant_id='tenant-trashed-update'",
    )
    .fetch_one(&pool)
    .await
    .expect("change count should be readable");
    assert_eq!(change_count, 0);
}

#[tokio::test]
async fn app_drive_shortcuts_create_and_resolve_target_metadata() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-shortcut', 'tenant-shortcut', 'user', 'user-owner', 'team', 'Shortcuts', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-shortcut-other', 'tenant-shortcut', 'user', 'user-other-owner', 'team', 'Other Shortcuts', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("other space should be seeded");
    for (id, parent_node_id, node_type, node_name) in [
        (
            "folder-shortcut",
            Option::<&str>::None,
            "folder",
            "Shortcut Folder",
        ),
        ("node-target", None, "file", "source.pdf"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-shortcut', 'space-shortcut', $2, $3, $4, 'ready', 'active', 1, 'user-owner', 'user-owner')",
        )
        .bind(id)
        .bind(parent_node_id)
        .bind(node_type)
        .bind(node_name)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-target-other-space', 'tenant-shortcut', 'space-shortcut-other', NULL, 'file', 'external.pdf', 'ready', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("other space target node should be seeded");

    let app = common::test_router_with_pool(pool);
    let cross_space_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-shortcut", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-shortcut", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/shortcuts")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"shortcut-cross-space",
                        "spaceId":"space-shortcut",
                        "parentNodeId":"folder-shortcut",
                        "nodeName":"external shortcut",
                        "targetNodeId":"node-target-other-space"
                    }"#,
                ))
                .expect("cross-space shortcut request should be built"),
        )
        .await
        .expect("cross-space shortcut request should be handled");
    assert_eq!(cross_space_response.status(), StatusCode::BAD_REQUEST);

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-shortcut", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-shortcut", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/shortcuts")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"shortcut-001",
                        "spaceId":"space-shortcut",
                        "parentNodeId":"folder-shortcut",
                        "nodeName":"source shortcut",
                        "targetNodeId":"node-target"
                    }"#,
                ))
                .expect("create shortcut request should be built"),
        )
        .await
        .expect("create shortcut request should be handled");
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("create shortcut response should be read"),
    )
    .expect("create shortcut response should be valid json");
    let create_item = common::envelope_item(&create_payload);
    assert_eq!(create_item["id"].as_str(), Some("shortcut-001"));
    assert_eq!(create_item["nodeType"].as_str(), Some("shortcut"));
    assert_eq!(
        create_item["shortcutTargetNodeId"].as_str(),
        Some("node-target")
    );

    let detail = fetch_json_as(
        app.clone(),
        "/app/v3/api/drive/nodes/shortcut-001",
        "tenant-shortcut",
        "user-owner",
    )
    .await;
    assert_eq!(
        common::envelope_item(&detail)["shortcutTargetNodeId"].as_str(),
        Some("node-target")
    );

    let (listed, _) = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/spaces/space-shortcut/nodes?parentNodeId=folder-shortcut",
        "tenant-shortcut",
        "user-owner",
    )
    .await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["id"].as_str(), Some("shortcut-001"));
    assert_eq!(
        listed[0]["shortcutTargetNodeId"].as_str(),
        Some("node-target")
    );

    let changes = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/changes?spaceId=space-shortcut",
        "tenant-shortcut",
        "user-owner",
    )
    .await
    .0;
    let events = changes
        .iter()
        .map(|item| item["eventType"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert!(events.contains(&"drive.node.created".to_string()));
}

#[tokio::test]
async fn app_dr_drive_node_hierarchy_mutations_validate_parent_type_and_name_conflicts() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-hierarchy', 'tenant-hierarchy', 'user', 'user-owner', 'team', 'Hierarchy', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    for (id, parent_node_id, node_type, node_name) in [
        ("folder-alpha", Option::<&str>::None, "folder", "Alpha"),
        ("folder-beta", None, "folder", "Beta"),
        ("file-parent", None, "file", "not-a-folder.pdf"),
        ("node-child", Some("folder-alpha"), "folder", "Child"),
        (
            "node-grandchild",
            Some("node-child"),
            "folder",
            "Grandchild",
        ),
        ("node-existing", Some("folder-alpha"), "folder", "Existing"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-hierarchy', 'space-hierarchy', $2, $3, $4, 'ready', 'active', 1, 'user-owner', 'user-owner')",
        )
        .bind(id)
        .bind(parent_node_id)
        .bind(node_type)
        .bind(node_name)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }

    let app = common::test_router_with_pool(pool.clone());
    let create_under_file = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-hierarchy", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-hierarchy", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/folders")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"folder-under-file",
                        "spaceId":"space-hierarchy",
                        "parentNodeId":"file-parent",
                        "nodeName":"Invalid"
                    }"#,
                ))
                .expect("create under file request should be built"),
        )
        .await
        .expect("create under file request should be handled");
    assert_eq!(create_under_file.status(), StatusCode::BAD_REQUEST);

    let move_under_file = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-hierarchy", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-hierarchy", "user-owner", "appbase"),
                )
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/nodes/node-child")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "parentNodeId":"file-parent"
                    }"#,
                ))
                .expect("move under file request should be built"),
        )
        .await
        .expect("move under file request should be handled");
    assert_eq!(move_under_file.status(), StatusCode::BAD_REQUEST);

    let self_parent = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-hierarchy", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-hierarchy", "user-owner", "appbase"),
                )
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/nodes/node-child")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "parentNodeId":"node-child"
                    }"#,
                ))
                .expect("self parent request should be built"),
        )
        .await
        .expect("self parent request should be handled");
    assert_eq!(self_parent.status(), StatusCode::BAD_REQUEST);

    let move_under_descendant = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-hierarchy", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-hierarchy", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/folder-alpha/move")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "targetParentNodeId":"node-grandchild"
                    }"#,
                ))
                .expect("move under descendant request should be built"),
        )
        .await
        .expect("move under descendant request should be handled");
    assert_eq!(move_under_descendant.status(), StatusCode::BAD_REQUEST);
    let alpha_parent_after_cycle_attempt: Option<String> = sqlx::query_scalar(
        "SELECT parent_node_id FROM dr_drive_node WHERE tenant_id='tenant-hierarchy' AND id='folder-alpha'",
    )
    .fetch_one(&pool)
    .await
    .expect("folder alpha parent should be readable");
    assert_eq!(alpha_parent_after_cycle_attempt, None);

    let rename_conflict = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-hierarchy", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-hierarchy", "user-owner", "appbase"),
                )
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/nodes/node-child")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeName":"Existing"
                    }"#,
                ))
                .expect("rename conflict request should be built"),
        )
        .await
        .expect("rename conflict request should be handled");
    assert_eq!(rename_conflict.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn app_drive_node_mutations_reject_trashed_sources_and_shortcut_targets_without_side_effects()
{
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-node-mutation-trash', 'tenant-node-mutation-trash', 'user', 'user-owner',
            'team', 'Node Mutation Trash', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    for (id, parent_node_id, node_type, node_name, lifecycle_status) in [
        (
            "folder-node-mutation-target",
            Option::<&str>::None,
            "folder",
            "Target",
            "active",
        ),
        (
            "node-mutation-trashed",
            None,
            "file",
            "trashed-source.pdf",
            "trashed",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES (
                $1, 'tenant-node-mutation-trash', 'space-node-mutation-trash',
                $2, $3, $4, 'ready', $5, 1, 'user-owner', 'user-owner'
            )",
        )
        .bind(id)
        .bind(parent_node_id)
        .bind(node_type)
        .bind(node_name)
        .bind(lifecycle_status)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }

    let app = common::test_router_with_pool(pool.clone());
    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-node-mutation-trash", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-node-mutation-trash", "user-owner", "appbase"),
                )
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/nodes/node-mutation-trashed")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeName":"renamed-while-trashed.pdf"
                    }"#,
                ))
                .expect("update trashed node request should be built"),
        )
        .await
        .expect("update trashed node request should be handled");
    assert_eq!(update_response.status(), StatusCode::NOT_FOUND);

    let move_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-node-mutation-trash", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-node-mutation-trash", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-mutation-trashed/move")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "targetParentNodeId":"folder-node-mutation-target"
                    }"#,
                ))
                .expect("move trashed node request should be built"),
        )
        .await
        .expect("move trashed node request should be handled");
    assert_eq!(move_response.status(), StatusCode::NOT_FOUND);

    let copy_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-node-mutation-trash", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-node-mutation-trash", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-mutation-trashed/copy")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"node-mutation-copy"
                    }"#,
                ))
                .expect("copy trashed node request should be built"),
        )
        .await
        .expect("copy trashed node request should be handled");
    assert_eq!(copy_response.status(), StatusCode::NOT_FOUND);

    let shortcut_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-node-mutation-trash", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-node-mutation-trash", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/shortcuts")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"shortcut-to-trashed",
                        "spaceId":"space-node-mutation-trash",
                        "parentNodeId":"folder-node-mutation-target",
                        "nodeName":"Trashed shortcut",
                        "targetNodeId":"node-mutation-trashed"
                    }"#,
                ))
                .expect("shortcut to trashed node request should be built"),
        )
        .await
        .expect("shortcut to trashed node request should be handled");
    assert_eq!(shortcut_response.status(), StatusCode::NOT_FOUND);

    let original: (String, Option<String>, String, i64) = sqlx::query_as(
        "SELECT node_name, parent_node_id, lifecycle_status, version
         FROM dr_drive_node
         WHERE tenant_id='tenant-node-mutation-trash'
           AND id='node-mutation-trashed'",
    )
    .fetch_one(&pool)
    .await
    .expect("trashed node should remain readable");
    assert_eq!(original.0, "trashed-source.pdf");
    assert_eq!(original.1, None);
    assert_eq!(original.2, "trashed");
    assert_eq!(original.3, 1);

    let created_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-node-mutation-trash'
           AND id IN ('node-mutation-copy', 'shortcut-to-trashed')",
    )
    .fetch_one(&pool)
    .await
    .expect("side effect count should be readable");
    assert_eq!(created_count, 0);
}

#[tokio::test]
async fn app_dr_drive_git_repository_space_root_accepts_only_repository_directories() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    for (space_id, space_type, display_name) in [
        (
            "space-git-repository-root",
            "git_repository",
            "Git Repositories",
        ),
        ("space-team-root", "team", "Team"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_space (
                id, tenant_id, owner_subject_type, owner_subject_id, space_type,
                display_name, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-git-repository-root', 'user', 'user-owner', $2, $3, 'active', 1, 'user-owner', 'user-owner')",
        )
        .bind(space_id)
        .bind(space_type)
        .bind(display_name)
        .execute(&pool)
        .await
        .expect("space should be seeded");
    }
    for (id, space_id, parent_node_id, node_type, node_name, content_state) in [
        (
            "folder-repository-alpha",
            "space-git-repository-root",
            Option::<&str>::None,
            "folder",
            "repository-alpha",
            "ready",
        ),
        (
            "file-repository-alpha",
            "space-git-repository-root",
            Some("folder-repository-alpha"),
            "file",
            "source.zip",
            "ready",
        ),
        (
            "file-team-source",
            "space-team-root",
            None,
            "file",
            "team-source.zip",
            "ready",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-git-repository-root', $2, $3, $4, $5, $6, 'active', 1, 'user-owner', 'user-owner')",
        )
        .bind(id)
        .bind(space_id)
        .bind(parent_node_id)
        .bind(node_type)
        .bind(node_name)
        .bind(content_state)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }

    let app = common::test_router_with_pool(pool.clone());
    let create_repository_directory = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-git-repository-root", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-git-repository-root", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/folders")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"folder-repository-beta",
                        "spaceId":"space-git-repository-root",
                        "nodeName":"repository-beta"
                    }"#,
                ))
                .expect("create repository directory request should be built"),
        )
        .await
        .expect("create repository directory request should be handled");
    if create_repository_directory.status() != StatusCode::CREATED {
        let status = create_repository_directory.status();
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(create_repository_directory.into_body(), usize::MAX)
                .await
                .expect("create repository directory error response should be read"),
        )
        .expect("create repository directory error response should be valid json");
        panic!("create repository directory returned {status}: {payload}");
    }

    let create_file_at_root = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-git-repository-root", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-git-repository-root", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/files")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"file-git-repository-root",
                        "spaceId":"space-git-repository-root",
                        "nodeName":"root-file.zip",
                        "uploadSessionId":"upload-git-repository-root",
                        "idempotencyKey":"idem-git-repository-root",
                        "expiresAtEpochMs":1800000000000
                    }"#,
                ))
                .expect("create root file request should be built"),
        )
        .await
        .expect("create root file request should be handled");
    assert_git_repository_root_directory_error(create_file_at_root).await;

    let create_shortcut_at_root = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-git-repository-root", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-git-repository-root", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/shortcuts")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"shortcut-git-repository-root",
                        "spaceId":"space-git-repository-root",
                        "nodeName":"root-shortcut",
                        "targetNodeId":"file-repository-alpha"
                    }"#,
                ))
                .expect("create root shortcut request should be built"),
        )
        .await
        .expect("create root shortcut request should be handled");
    assert_git_repository_root_directory_error(create_shortcut_at_root).await;

    let move_file_to_root = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-git-repository-root", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-git-repository-root", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/file-repository-alpha/move")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("move file to root request should be built"),
        )
        .await
        .expect("move file to root request should be handled");
    assert_git_repository_root_directory_error(move_file_to_root).await;

    let copy_file_to_git_repository_root = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-git-repository-root", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-git-repository-root", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/file-team-source/copy")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"file-copy-git-repository-root",
                        "targetSpaceId":"space-git-repository-root"
                    }"#,
                ))
                .expect("copy file to git repository root request should be built"),
        )
        .await
        .expect("copy file to git repository root request should be handled");
    assert_git_repository_root_directory_error(copy_file_to_git_repository_root).await;

    let root_file_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-git-repository-root'
           AND space_id='space-git-repository-root'
           AND parent_node_id IS NULL
           AND node_type != 'folder'
           AND lifecycle_status != 'deleted'",
    )
    .fetch_one(&pool)
    .await
    .expect("git repository root node count should be readable");
    assert_eq!(root_file_count, 0);
}

#[tokio::test]
async fn app_drive_copy_shortcut_preserves_target_node_reference() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-shortcut-copy', 'tenant-shortcut-copy', 'user', 'user-owner', 'team', 'Shortcut Copy', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-target-copy', 'tenant-shortcut-copy', 'space-shortcut-copy', NULL, 'file', 'source.pdf', 'ready', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("target node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, shortcut_target_node_id,
            node_type, node_name, content_state, lifecycle_status, version,
            created_by, updated_by
        ) VALUES ('shortcut-copy-source', 'tenant-shortcut-copy', 'space-shortcut-copy', NULL, 'node-target-copy', 'shortcut', 'source shortcut', 'ready', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("shortcut node should be seeded");

    let app = common::test_router_with_pool(pool);
    let copy_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-shortcut-copy", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-shortcut-copy", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/shortcut-copy-source/copy")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"shortcut-copy-destination",
                        "nodeName":"copied shortcut"
                    }"#,
                ))
                .expect("copy shortcut request should be built"),
        )
        .await
        .expect("copy shortcut request should be handled");
    assert_eq!(copy_response.status(), StatusCode::CREATED);
    let copy_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(copy_response.into_body(), usize::MAX)
            .await
            .expect("copy shortcut response should be read"),
    )
    .expect("copy shortcut response should be valid json");
    let copy_item = common::envelope_item(&copy_payload);
    assert_eq!(copy_item["nodeType"].as_str(), Some("shortcut"));
    assert_eq!(
        copy_item["shortcutTargetNodeId"].as_str(),
        Some("node-target-copy")
    );
}

#[tokio::test]
async fn app_drive_copy_node_rejects_missing_or_deleted_target_space() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-copy-source', 'tenant-copy-space', 'user', 'user-owner', 'team', 'Copy Source', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("source space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-copy-deleted', 'tenant-copy-space', 'user', 'user-deleted-owner', 'team', 'Deleted Copy Target', 'deleted', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("deleted target space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('folder-copy-source', 'tenant-copy-space', 'space-copy-source', NULL, 'folder', 'Source Folder', 'empty', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("source folder should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    for (copy_id, target_space_id) in [
        ("folder-copy-missing-target", "space-copy-missing"),
        ("folder-copy-deleted-target", "space-copy-deleted"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-copy-space", "user-owner", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-copy-space", "user-owner", "appbase"),
                    )
                    .method(Method::POST)
                    .uri("/app/v3/api/drive/nodes/folder-copy-source/copy")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{
                            "id":"{copy_id}",
                                                        "targetSpaceId":"{target_space_id}"
                        }}"#
                    )))
                    .expect("copy to invalid target space request should be built"),
            )
            .await
            .expect("copy to invalid target space request should be handled");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("copy invalid target space response should be read"),
        )
        .expect("copy invalid target space response should be valid json");
        assert_eq!(payload["detail"].as_str(), Some("space not found"));
    }

    let copied_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id='tenant-copy-space'
           AND id IN ('folder-copy-missing-target', 'folder-copy-deleted-target')",
    )
    .fetch_one(&pool)
    .await
    .expect("copied node count should be readable");
    assert_eq!(copied_count, 0);
}

#[tokio::test]
async fn app_dr_drive_node_labels_apply_list_filter_remove_and_emit_changes() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-label', 'tenant-label', 'user', 'user-owner', 'team', 'Labels', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-label', 'tenant-label', 'space-label', NULL, 'file', 'classified.pdf', 'ready', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    for (id, label_key, display_name, color) in [
        (
            "label-confidential",
            "classification.confidential",
            "Confidential",
            "#D92D20",
        ),
        ("label-public", "classification.public", "Public", "#027A48"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_label (
                id, tenant_id, label_key, display_name, color, description,
                lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-label', $2, $3, $4, NULL, 'active', 1, 'admin-label', 'admin-label')",
        )
        .bind(id)
        .bind(label_key)
        .bind(display_name)
        .bind(color)
        .execute(&pool)
        .await
        .expect("label should be seeded");
    }

    let app = common::test_router_with_pool(pool);
    let apply_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-label", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-label", "user-owner", "appbase"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/nodes/node-label/labels/label-confidential")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("apply label request should be built"),
        )
        .await
        .expect("apply label request should be handled");
    assert_eq!(apply_response.status(), StatusCode::OK);
    let apply_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(apply_response.into_body(), usize::MAX)
            .await
            .expect("apply label response should be read"),
    )
    .expect("apply label response should be valid json");
    let apply_item = common::envelope_item(&apply_payload);
    assert_eq!(apply_item["nodeId"].as_str(), Some("node-label"));
    assert_eq!(
        apply_item["label"]["labelKey"].as_str(),
        Some("classification.confidential")
    );
    assert_eq!(apply_item["lifecycleStatus"].as_str(), Some("active"));

    let apply_second = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-label", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-label", "user-owner", "appbase"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/nodes/node-label/labels/label-public")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("apply second label request should be built"),
        )
        .await
        .expect("apply second label request should be handled");
    assert_eq!(apply_second.status(), StatusCode::OK);

    let (first_items, next_token) = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/nodes/node-label/labels?page_size=1",
        "tenant-label",
        "user-owner",
    )
    .await;
    assert_eq!(first_items.len(), 1);
    assert_eq!(
        first_items[0]["label"]["labelKey"].as_str(),
        Some("classification.confidential")
    );
    let next_token = next_token.expect("node label list should expose next page token");
    let (second_items, final_token) = fetch_paged_items_as(
        app.clone(),
        &format!("/app/v3/api/drive/nodes/node-label/labels?page_size=1&cursor={next_token}"),
        "tenant-label",
        "user-owner",
    )
    .await;
    assert_eq!(second_items.len(), 1);
    assert_eq!(
        second_items[0]["label"]["labelKey"].as_str(),
        Some("classification.public")
    );
    assert!(final_token.is_none());

    let filtered = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/nodes/node-label/labels?labelKey=classification.public",
        "tenant-label",
        "user-owner",
    )
    .await
    .0;
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0]["labelId"].as_str(), Some("label-public"));

    let remove_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-label", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-label", "user-owner", "appbase"),
                )
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/nodes/node-label/labels/label-confidential")
                .body(Body::empty())
                .expect("remove label request should be built"),
        )
        .await
        .expect("remove label request should be handled");
    common::assert_no_content_response(remove_response).await;

    let remaining = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/nodes/node-label/labels",
        "tenant-label",
        "user-owner",
    )
    .await
    .0;
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0]["labelId"].as_str(), Some("label-public"));

    let changes = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/changes?spaceId=space-label",
        "tenant-label",
        "user-owner",
    )
    .await
    .0;
    let events = changes
        .iter()
        .map(|item| item["eventType"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert!(events.contains(&"drive.node_label.applied".to_string()));
    assert!(events.contains(&"drive.node_label.removed".to_string()));
}

#[tokio::test]
async fn app_dr_drive_watch_channels_create_list_get_stop_and_emit_changes() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-watch', 'tenant-watch', 'user', 'user-owner', 'team', 'Watch', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-watch', 'tenant-watch', 'space-watch', NULL, 'file', 'watched.pdf', 'ready', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let create_changes_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-watch", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-watch", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/changes/watch")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"watch-changes-001",
                        "spaceId":"space-watch",
                        "address":"https://hooks.example.com/drive/changes",
                        "expirationEpochMs":1800000000000,
                        "token":"notify-secret-thirty-two-characters-minimum"
                    }"#,
                ))
                .expect("create changes watch request should be built"),
        )
        .await
        .expect("create changes watch request should be handled");
    assert_eq!(create_changes_response.status(), StatusCode::CREATED);
    let changes_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(create_changes_response.into_body(), usize::MAX)
            .await
            .expect("changes watch response should be read"),
    )
    .expect("changes watch response should be valid json");
    let changes_item = common::envelope_item(&changes_payload);
    assert_eq!(changes_item["id"].as_str(), Some("watch-changes-001"));
    assert_eq!(changes_item["resourceType"].as_str(), Some("changes"));
    assert_eq!(changes_item["spaceId"].as_str(), Some("space-watch"));
    assert_eq!(changes_item["channelType"].as_str(), Some("web_hook"));
    assert_eq!(changes_item["lifecycleStatus"].as_str(), Some("active"));
    assert_eq!(changes_item["version"].as_i64(), Some(1));
    assert!(
        changes_item.get("token").is_none(),
        "watch channel response must not echo notification token"
    );

    let stored_token_hash: String = sqlx::query_scalar(
        "SELECT token_hash FROM dr_drive_watch_channel WHERE id='watch-changes-001'",
    )
    .fetch_one(&pool)
    .await
    .expect("token hash should be stored");
    assert_eq!(stored_token_hash.len(), 64);
    assert_ne!(
        stored_token_hash,
        "notify-secret-thirty-two-characters-minimum"
    );

    let create_node_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-watch", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-watch", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-watch/watch")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"watch-node-001",
                        "address":"https://hooks.example.com/drive/node",
                        "expirationEpochMs":1800000005000
                    }"#,
                ))
                .expect("create node watch request should be built"),
        )
        .await
        .expect("create node watch request should be handled");
    assert_eq!(create_node_response.status(), StatusCode::CREATED);
    let node_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(create_node_response.into_body(), usize::MAX)
            .await
            .expect("node watch response should be read"),
    )
    .expect("node watch response should be valid json");
    let node_item = common::envelope_item(&node_payload);
    assert_eq!(node_item["resourceType"].as_str(), Some("node"));
    assert_eq!(node_item["resourceId"].as_str(), Some("node-watch"));
    assert_eq!(node_item["nodeId"].as_str(), Some("node-watch"));

    let (first_items, next_token) = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/watch_channels?page_size=1",
        "tenant-watch",
        "user-owner",
    )
    .await;
    assert_eq!(first_items.len(), 1);
    assert_eq!(first_items[0]["id"].as_str(), Some("watch-changes-001"));
    let next_token = next_token.expect("watch channel list should expose next page token");
    let (second_items, final_token) = fetch_paged_items_as(
        app.clone(),
        &format!("/app/v3/api/drive/watch_channels?page_size=1&cursor={next_token}"),
        "tenant-watch",
        "user-owner",
    )
    .await;
    assert_eq!(second_items.len(), 1);
    assert_eq!(second_items[0]["id"].as_str(), Some("watch-node-001"));
    assert!(final_token.is_none());

    let node_filtered = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/watch_channels?resourceType=node",
        "tenant-watch",
        "user-owner",
    )
    .await
    .0;
    assert_eq!(node_filtered.len(), 1);
    assert_eq!(node_filtered[0]["id"].as_str(), Some("watch-node-001"));

    let get_payload = fetch_json_as(
        app.clone(),
        "/app/v3/api/drive/watch_channels/watch-node-001",
        "tenant-watch",
        "user-owner",
    )
    .await;
    let get_item = common::envelope_item(&get_payload);
    assert_eq!(
        get_item["address"].as_str(),
        Some("https://hooks.example.com/drive/node")
    );
    assert_eq!(get_item["lifecycleStatus"].as_str(), Some("active"));

    let stop_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-watch", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-watch", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/watch_channels/watch-node-001/stop")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("stop watch request should be built"),
        )
        .await
        .expect("stop watch request should be handled");
    assert_eq!(stop_response.status(), StatusCode::OK);
    let stop_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(stop_response.into_body(), usize::MAX)
            .await
            .expect("stop watch response should be read"),
    )
    .expect("stop watch response should be valid json");
    let stop_data = common::envelope_data(&stop_payload);
    assert_eq!(stop_data["stopped"].as_bool(), Some(true));
    assert_eq!(
        stop_data["channel"]["lifecycleStatus"].as_str(),
        Some("stopped")
    );
    assert_eq!(stop_data["channel"]["version"].as_i64(), Some(2));

    let active_after_stop = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/watch_channels",
        "tenant-watch",
        "user-owner",
    )
    .await
    .0;
    assert_eq!(active_after_stop.len(), 1);
    assert_eq!(
        active_after_stop[0]["id"].as_str(),
        Some("watch-changes-001")
    );

    let stopped_filtered = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/watch_channels?lifecycleStatus=stopped",
        "tenant-watch",
        "user-owner",
    )
    .await
    .0;
    assert_eq!(stopped_filtered.len(), 1);
    assert_eq!(stopped_filtered[0]["id"].as_str(), Some("watch-node-001"));

    let changes = fetch_paged_items_as(
        app.clone(),
        "/app/v3/api/drive/changes?spaceId=space-watch",
        "tenant-watch",
        "user-owner",
    )
    .await
    .0;
    let events = changes
        .iter()
        .map(|item| item["eventType"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert!(events.contains(&"drive.watch_channel.created".to_string()));
    assert!(events.contains(&"drive.watch_channel.stopped".to_string()));
}

#[tokio::test]
async fn app_dr_drive_watch_channel_create_rejects_past_expiration_before_database_write() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-watch-validation', 'tenant-watch-validation', 'user', 'user-owner', 'team', 'Watch Validation', 'active', 1, 'user-owner', 'user-owner')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    let past_expiration_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after UNIX epoch")
        .as_millis() as i64
        - 1_000;
    let request_body = serde_json::json!({
            "id": "watch-past-expiration",
                    "spaceId": "space-watch-validation",
            "address": "https://hooks.example.com/drive/past",
            "expirationEpochMs": past_expiration_epoch_ms
    })
    .to_string();

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-watch-validation", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-watch-validation", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/changes/watch")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .expect("create watch request should be built"),
        )
        .await
        .expect("create watch request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("watch validation response should be read"),
    )
    .expect("watch validation response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40001));

    let stored_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_watch_channel WHERE tenant_id='tenant-watch-validation'",
    )
    .fetch_one(&pool)
    .await
    .expect("watch channel count should be queryable");
    assert_eq!(stored_count, 0);
}

#[tokio::test]
async fn app_dr_drive_watch_node_rejects_trashed_node_before_creating_channel() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-watch-trashed', 'tenant-watch-trashed', 'user', 'user-owner',
            'team', 'Watch Trashed', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-watch-trashed', 'tenant-watch-trashed', 'space-watch-trashed',
            NULL, 'file', 'trashed.pdf', 'ready', 'trashed', 1,
            'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("trashed node should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-watch-trashed", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-watch-trashed", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-watch-trashed/watch")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"watch-node-trashed",
                        "address":"https://hooks.example.com/drive/node",
                        "expirationEpochMs":1800000005000
                    }"#,
                ))
                .expect("create node watch request should be built"),
        )
        .await
        .expect("create node watch request should be handled");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("watch response should be read"),
    )
    .expect("watch response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40401));

    let watch_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_watch_channel
         WHERE tenant_id='tenant-watch-trashed'",
    )
    .fetch_one(&pool)
    .await
    .expect("watch channel count should be queryable");
    assert_eq!(watch_count, 0);
}

#[tokio::test]
async fn app_drive_share_link_routes_enforce_acl_roles() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-share-acl', 'tenant-share-acl', 'user', 'user-owner', 'personal',
            'Share ACL', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-share-acl', 'tenant-share-acl', 'space-share-acl', NULL, 'folder', 'Shared Folder',
            'empty', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version,
            created_by, updated_by
        ) VALUES (
            'share-acl-existing', 'tenant-share-acl', 'node-share-acl', $1, 'reader',
            1800000000000, NULL, 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .bind(drive_share_token_hash("share-acl-token"))
    .execute(&pool)
    .await
    .expect("share link should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    for (method, uri) in [
        (
            Method::GET,
            "/app/v3/api/drive/nodes/node-share-acl/share_links",
        ),
        (
            Method::GET,
            "/app/v3/api/drive/share_links/share-acl-existing",
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-share-acl", "user-outsider", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-share-acl", "user-outsider", "appbase"),
                    )
                    .method(method.clone())
                    .uri(uri)
                    .body(Body::empty())
                    .expect("denied share link request should be built"),
            )
            .await
            .expect("denied share link request should be handled");
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{uri}");
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("denied share link response should be read"),
        )
        .expect("denied share link response should be valid json");
        assert_eq!(payload["code"].as_i64(), Some(40301));
    }

    let create_denied_before_reader = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-share-acl", "user-outsider", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-share-acl", "user-outsider", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-share-acl/share_links")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"share-denied-create",
                        "role":"reader"
                    }"#,
                ))
                .expect("denied share link create request should be built"),
        )
        .await
        .expect("denied share link create request should be handled");
    assert_eq!(create_denied_before_reader.status(), StatusCode::FORBIDDEN);

    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-share-reader', 'tenant-share-acl', 'node-share-acl', 'user', 'user-outsider',
            'reader', 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("reader permission should be seeded");

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-share-acl", "user-outsider", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-share-acl", "user-outsider", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-share-acl/share_links")
                .body(Body::empty())
                .expect("allowed share link list request should be built"),
        )
        .await
        .expect("allowed share link list request should be handled");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("allowed share link list response should be read"),
    )
    .expect("allowed share link list response should be valid json");
    let ids = common::envelope_items(&list_payload)
        .as_array()
        .expect("share links should be an array")
        .iter()
        .filter_map(|item| item["id"].as_str())
        .collect::<Vec<_>>();
    assert!(ids.contains(&"share-acl-existing"));

    let create_denied = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-share-acl", "user-outsider", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-share-acl", "user-outsider", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-share-acl/share_links")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"share-reader-cannot-create",
                        "role":"reader"
                    }"#,
                ))
                .expect("reader create share link request should be built"),
        )
        .await
        .expect("reader create share link request should be handled");
    assert_eq!(create_denied.status(), StatusCode::FORBIDDEN);

    let create_allowed = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-share-acl", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-share-acl", "user-owner", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-share-acl/share_links")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"share-owner-created",
                        "role":"reader"
                    }"#,
                ))
                .expect("owner create share link request should be built"),
        )
        .await
        .expect("owner create share link request should be handled");
    assert_eq!(create_allowed.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn app_drive_search_skips_nodes_without_reader_acl_and_paginates_visible_results() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-search-acl', 'tenant-search-acl', 'user', 'user-owner', 'personal',
            'Search ACL', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    for (id, name) in [
        ("node-search-hidden", "secret-hidden.txt"),
        ("node-search-visible-a", "secret-visible-a.txt"),
        ("node-search-visible-b", "secret-visible-b.txt"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES (
                $1, 'tenant-search-acl', 'space-search-acl', NULL, 'file', $2,
                'ready', 'active', 1, 'user-owner', 'user-owner'
            )",
        )
        .bind(id)
        .bind(name)
        .execute(&pool)
        .await
        .expect("search node should be seeded");
    }

    for node_id in ["node-search-visible-a", "node-search-visible-b"] {
        sqlx::query(
            "INSERT INTO dr_drive_node_permission (
                id, tenant_id, node_id, subject_type, subject_id, role,
                inherited, lifecycle_status, version, created_by, updated_by
            ) VALUES (
                $1, 'tenant-search-acl', $2, 'user', 'user-reviewer', 'reader',
                0, 'active', 1, 'user-owner', 'user-owner'
            )",
        )
        .bind(format!("perm-{node_id}"))
        .bind(node_id)
        .execute(&pool)
        .await
        .expect("reader permission should be seeded");
    }

    let app = common::test_router_with_pool(pool);
    let first_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-search-acl", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-search-acl", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/search?spaceId=space-search-acl&q=secret&page_size=1")
                .body(Body::empty())
                .expect("search first page request should be built"),
        )
        .await
        .expect("search first page request should be handled");
    assert_eq!(first_response.status(), StatusCode::OK);
    let first_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(first_response.into_body(), usize::MAX)
            .await
            .expect("search first page response should be read"),
    )
    .expect("search first page response should be valid json");
    let first_items = common::envelope_items(&first_payload)
        .as_array()
        .expect("search items should be an array");
    assert_eq!(first_items.len(), 1);
    let first_id = first_items[0]["id"]
        .as_str()
        .expect("search item id should be present");
    assert!(
        first_id == "node-search-visible-a" || first_id == "node-search-visible-b",
        "search must only return ACL-visible nodes"
    );
    assert!(
        first_id != "node-search-hidden",
        "search must not leak nodes without reader access"
    );
    let next_page_token = common::envelope_next_page_token(&first_payload)
        .expect("search first page should expose nextPageToken when more visible rows exist");

    let second_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-search-acl", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-search-acl", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri(format!(
                    "/app/v3/api/drive/search?spaceId=space-search-acl&q=secret&page_size=1&cursor={next_page_token}"
                ))
                .body(Body::empty())
                .expect("search second page request should be built"),
        )
        .await
        .expect("search second page request should be handled");
    assert_eq!(second_response.status(), StatusCode::OK);
    let second_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(second_response.into_body(), usize::MAX)
            .await
            .expect("search second page response should be read"),
    )
    .expect("search second page response should be valid json");
    let second_items = common::envelope_items(&second_payload)
        .as_array()
        .expect("second search items should be an array");
    assert_eq!(second_items.len(), 1);
    let second_id = second_items[0]["id"]
        .as_str()
        .expect("second search item id should be present");
    assert_ne!(second_id, first_id);
    assert!(
        second_id == "node-search-visible-a" || second_id == "node-search-visible-b",
        "second search page must remain ACL-filtered"
    );
    assert!(common::envelope_next_page_token(&second_payload).is_none());
}

#[tokio::test]
async fn app_drive_list_skips_nodes_without_reader_acl_and_paginates_visible_results() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-list-acl', 'tenant-list-acl', 'user', 'user-owner', 'personal',
            'List ACL', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'folder-list-acl-anchor', 'tenant-list-acl', 'space-list-acl',
            NULL, 'folder', 'Root', 'ready', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("root folder anchor should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-folder-list-acl-anchor', 'tenant-list-acl', 'folder-list-acl-anchor',
            'user', 'user-reviewer', 'reader', 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("anchor reader permission should be seeded");

    for (id, name) in [
        ("node-list-hidden", "secret-hidden.txt"),
        ("node-list-visible-a", "secret-visible-a.txt"),
        ("node-list-visible-b", "secret-visible-b.txt"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES (
                $1, 'tenant-list-acl', 'space-list-acl', NULL, 'file', $2,
                'ready', 'active', 1, 'user-owner', 'user-owner'
            )",
        )
        .bind(id)
        .bind(name)
        .execute(&pool)
        .await
        .expect("list node should be seeded");
    }

    for node_id in ["node-list-visible-a", "node-list-visible-b"] {
        sqlx::query(
            "INSERT INTO dr_drive_node_permission (
                id, tenant_id, node_id, subject_type, subject_id, role,
                inherited, lifecycle_status, version, created_by, updated_by
            ) VALUES (
                $1, 'tenant-list-acl', $2, 'user', 'user-reviewer', 'reader',
                0, 'active', 1, 'user-owner', 'user-owner'
            )",
        )
        .bind(format!("perm-{node_id}"))
        .bind(node_id)
        .execute(&pool)
        .await
        .expect("reader permission should be seeded");
    }

    let app = common::test_router_with_pool(pool);
    let first_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-list-acl", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-list-acl", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces/space-list-acl/nodes?page_size=10")
                .body(Body::empty())
                .expect("list request should be built"),
        )
        .await
        .expect("list request should be handled");
    assert_eq!(first_response.status(), StatusCode::OK);
    let first_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(first_response.into_body(), usize::MAX)
            .await
            .expect("list response should be read"),
    )
    .expect("list response should be valid json");
    let first_items = common::envelope_items(&first_payload)
        .as_array()
        .expect("list items should be an array");
    let listed_ids = first_items
        .iter()
        .filter_map(|item| item["id"].as_str())
        .collect::<Vec<_>>();
    assert!(
        !listed_ids.contains(&"node-list-hidden"),
        "list must not leak nodes without reader access"
    );
    assert!(listed_ids.contains(&"node-list-visible-a"));
    assert!(listed_ids.contains(&"node-list-visible-b"));
    assert!(listed_ids.contains(&"folder-list-acl-anchor"));
    assert!(common::envelope_next_page_token(&first_payload).is_none());

    let paged_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-list-acl", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-list-acl", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces/space-list-acl/nodes?page_size=1")
                .body(Body::empty())
                .expect("list first page request should be built"),
        )
        .await
        .expect("list first page request should be handled");
    assert_eq!(paged_response.status(), StatusCode::OK);
    let paged_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(paged_response.into_body(), usize::MAX)
            .await
            .expect("list first page response should be read"),
    )
    .expect("list first page response should be valid json");
    let paged_items = common::envelope_items(&paged_payload)
        .as_array()
        .expect("paged list items should be an array");
    assert_eq!(paged_items.len(), 1);
    let first_id = paged_items[0]["id"]
        .as_str()
        .expect("list item id should be present");
    assert_ne!(first_id, "node-list-hidden");
    let next_page_token = common::envelope_next_page_token(&paged_payload)
        .expect("list first page should expose nextPageToken when more visible rows exist");

    let second_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-list-acl", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-list-acl", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri(format!(
                    "/app/v3/api/drive/spaces/space-list-acl/nodes?page_size=1&cursor={next_page_token}"
                ))
                .body(Body::empty())
                .expect("list second page request should be built"),
        )
        .await
        .expect("list second page request should be handled");
    assert_eq!(second_response.status(), StatusCode::OK);
    let second_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(second_response.into_body(), usize::MAX)
            .await
            .expect("list second page response should be read"),
    )
    .expect("list second page response should be valid json");
    let second_items = common::envelope_items(&second_payload)
        .as_array()
        .expect("second list items should be an array");
    assert_eq!(second_items.len(), 1);
    let second_id = second_items[0]["id"]
        .as_str()
        .expect("second list item id should be present");
    assert_ne!(second_id, first_id);
    assert_ne!(second_id, "node-list-hidden");
}

#[tokio::test]
async fn uploader_mark_part_uploaded_requires_writer_acl() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-uploader-part-deny', 'tenant-uploader-part-deny', 'user',
            'user-owner', 'personal', 'Upload Deny', 'active', 1,
            'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-uploader-part-deny', 'tenant-uploader-part-deny', 'space-uploader-part-deny',
            NULL, 'file', 'deny.bin', 'uploading', 'active', 1,
            'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-uploader-part-deny', 's3_compatible', 'Uploader S3',
            'https://s3.example.com', 'us-east-1', 'bucket-s3', 1, 1,
            'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD',
            'active', 1, 'admin-uploader', 'admin-uploader'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES (
            'session-uploader-part-deny', 'tenant-uploader-part-deny', 'space-uploader-part-deny',
            'node-uploader-part-deny', 'bucket-s3', 'objects/deny.bin',
            'idem-uploader-part-deny', 'provider-uploader-part-deny', 'storage-upload-deny',
            'uploading', 1800000000000, 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("upload session should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_upload_item (
            id, task_id, tenant_id, organization_id, user_id, actor_type, actor_id,
            app_id, app_resource_type, app_resource_id, upload_profile_code,
            file_fingerprint, space_id, node_id, upload_session_id,
            storage_provider_id, storage_upload_id, original_file_name, file_extension,
            content_type, content_type_group, detected_content_type, content_length,
            checksum_sha256_hex, chunk_size_bytes, total_parts, uploaded_parts_count,
            uploaded_bytes, status, retention_mode, retention_expires_at_epoch_ms,
            cleanup_action, hard_delete_after_epoch_ms, cleanup_status,
            post_process_status, created_by, updated_by
        ) VALUES (
            'upload-item-uploader-part-deny', 'task-uploader-part-deny',
            'tenant-uploader-part-deny', NULL, 'user-owner', 'user',
            'user-owner', 'drive-pc', 'desktop-file-browser',
            'root', 'generic', 'fp-uploader-part-deny', 'space-uploader-part-deny',
            'node-uploader-part-deny', 'session-uploader-part-deny',
            'provider-uploader-part-deny', 'storage-upload-deny', 'deny.bin',
            'bin', 'application/octet-stream', 'binary',
            'application/octet-stream', 12, NULL, 8, 2, 0, 0,
            'uploading', 'long_term', NULL, NULL, NULL, 'active',
            'not_required', 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("upload item should be seeded");

    let app = common::test_router_with_pool(pool);
    let denied_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token(
                            "tenant-uploader-part-deny",
                            "user-no-access",
                            "appbase",
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token(
                        "tenant-uploader-part-deny",
                        "user-no-access",
                        "appbase",
                    ),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/uploader/uploads/upload-item-uploader-part-deny/parts/1")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "uploadSessionId":"session-uploader-part-deny",
                        "offsetBytes":0,
                        "sizeBytes":8,
                        "etag":"etag-part-deny",
                        "checksumSha256Hex":"sha256:3333333333333333333333333333333333333333333333333333333333333333",
                        "uploadedAtEpochMs":1700000000000
                    }"#,
                ))
                .expect("denied mark part request should be built"),
        )
        .await
        .expect("denied mark part request should be handled");
    assert_eq!(denied_response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn app_drive_comment_routes_enforce_acl_roles() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-comment-acl', 'tenant-comment-acl', 'user', 'user-owner', 'personal',
            'Comment ACL', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-comment-acl', 'tenant-comment-acl', 'space-comment-acl', NULL, 'file', 'notes.txt',
            'ready', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");

    let app = common::test_router_with_pool(pool.clone());
    for (method, uri, body) in [
        (
            Method::GET,
            "/app/v3/api/drive/nodes/node-comment-acl/comments",
            None,
        ),
        (
            Method::POST,
            "/app/v3/api/drive/nodes/node-comment-acl/comments",
            Some(
                r#"{
                    "id":"comment-denied",
                    "content":"Should be denied"
                }"#,
            ),
        ),
    ] {
        let request = match body {
            Some(payload) => Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comment-acl", "user-outsider", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comment-acl", "user-outsider", "appbase"),
                )
                .method(method.clone())
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .expect("denied comment request should be built"),
            None => Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comment-acl", "user-outsider", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comment-acl", "user-outsider", "appbase"),
                )
                .method(method.clone())
                .uri(uri)
                .body(Body::empty())
                .expect("denied comment request should be built"),
        };
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("denied comment request should be handled");
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{uri}");
    }

    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-comment-reader', 'tenant-comment-acl', 'node-comment-acl', 'user', 'user-outsider',
            'reader', 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("reader permission should be seeded");

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comment-acl", "user-outsider", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comment-acl", "user-outsider", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/nodes/node-comment-acl/comments")
                .body(Body::empty())
                .expect("reader comment list request should be built"),
        )
        .await
        .expect("reader comment list request should be handled");
    assert_eq!(list_response.status(), StatusCode::OK);

    let create_denied = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-comment-acl", "user-outsider", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-comment-acl", "user-outsider", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-comment-acl/comments")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"comment-reader-denied",
                        "content":"Reader cannot create"
                    }"#,
                ))
                .expect("reader create comment request should be built"),
        )
        .await
        .expect("reader create comment request should be handled");
    assert_eq!(create_denied.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn app_drive_legacy_download_url_route_enforces_reader_acl() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-download-acl', 'tenant-download-acl', 'user', 'user-owner', 'personal',
            'Download ACL', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-download-acl', 'tenant-download-acl', 'space-download-acl', NULL, 'file', 'secret.bin',
            'ready', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");

    let app = common::test_router_with_pool(pool);
    let denied_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-download-acl", "user-outsider", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-download-acl", "user-outsider", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/download_urls")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeId":"node-download-acl",
                        "requestedTtlSeconds":120
                    }"#,
                ))
                .expect("denied download url request should be built"),
        )
        .await
        .expect("denied download url request should be handled");
    assert_eq!(denied_response.status(), StatusCode::FORBIDDEN);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(denied_response.into_body(), usize::MAX)
            .await
            .expect("denied download url response should be read"),
    )
    .expect("denied download url response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40301));
}

#[tokio::test]
async fn app_drive_changes_require_space_id() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = common::test_router_with_pool(pool);
    for uri in [
        "/app/v3/api/drive/changes",
        "/app/v3/api/drive/changes/start_page_token",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-changes-required", "user-owner", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-changes-required", "user-owner", "appbase"),
                    )
                    .method(Method::GET)
                    .uri(uri)
                    .body(Body::empty())
                    .expect("changes missing space request should be built"),
            )
            .await
            .expect("changes missing space request should be handled");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{uri}");
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("changes missing space response should be read"),
        )
        .expect("changes missing space response should be valid json");
        assert_eq!(payload["code"].as_i64(), Some(40001));
        assert_eq!(payload["detail"].as_str(), Some("spaceId is required"));
    }
}

#[tokio::test]
async fn app_drive_change_feed_skips_nodes_without_reader_acl_and_paginates_visible_results() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-change-acl', 'tenant-change-acl', 'user', 'user-owner', 'personal',
            'Change ACL', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'folder-change-root', 'tenant-change-acl', 'space-change-acl', NULL, 'folder', 'Root',
            'empty', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("change feed root folder should be seeded");
    for (node_id, seq, event_type) in [
        ("node-change-hidden", 1_i64, "drive.node.created"),
        ("node-change-visible-a", 2_i64, "drive.node.updated"),
        ("node-change-visible-b", 3_i64, "drive.node.updated"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES (
                $1, 'tenant-change-acl', 'space-change-acl', NULL, 'file', $1,
                'ready', 'active', 1, 'user-owner', 'user-owner'
            )",
        )
        .bind(node_id)
        .execute(&pool)
        .await
        .expect("change node should be seeded");
        sqlx::query(
            "INSERT INTO dr_drive_change_log (
                id, tenant_id, space_id, node_id, sequence_no, event_type, actor_id
            ) VALUES (
                $1, 'tenant-change-acl', 'space-change-acl', $2, $3, $4, 'user-owner'
            )",
        )
        .bind(format!("change-{seq}"))
        .bind(node_id)
        .bind(seq)
        .bind(event_type)
        .execute(&pool)
        .await
        .expect("change log row should be seeded");
    }
    sqlx::query(
        "INSERT INTO dr_drive_change_log (
            id, tenant_id, space_id, node_id, sequence_no, event_type, actor_id
        ) VALUES (
            'change-space-level', 'tenant-change-acl', 'space-change-acl', NULL, 4, 'drive.space.updated', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space-level change should be seeded");

    for node_id in ["node-change-visible-a", "node-change-visible-b"] {
        sqlx::query(
            "INSERT INTO dr_drive_node_permission (
                id, tenant_id, node_id, subject_type, subject_id, role,
                inherited, lifecycle_status, version, created_by, updated_by
            ) VALUES (
                $1, 'tenant-change-acl', $2, 'user', 'user-reviewer', 'reader',
                0, 'active', 1, 'user-owner', 'user-owner'
            )",
        )
        .bind(format!("perm-{node_id}"))
        .bind(node_id)
        .execute(&pool)
        .await
        .expect("reader permission should be seeded");
    }
    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-change-feed-reader', 'tenant-change-acl', 'folder-change-root', 'user', 'user-reviewer', 'reader',
            0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("change feed reader permission should be seeded");

    let app = common::test_router_with_pool(pool);
    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-change-acl", "user-outsider", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-change-acl", "user-outsider", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/changes?spaceId=space-change-acl")
                .body(Body::empty())
                .expect("denied change feed request should be built"),
        )
        .await
        .expect("denied change feed request should be handled");
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let first_page = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-change-acl", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-change-acl", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/changes?spaceId=space-change-acl&page_size=1")
                .body(Body::empty())
                .expect("change feed first page request should be built"),
        )
        .await
        .expect("change feed first page request should be handled");
    assert_eq!(first_page.status(), StatusCode::OK);
    let first_page_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(first_page.into_body(), usize::MAX)
            .await
            .expect("change feed first page response should be read"),
    )
    .expect("change feed first page response should be valid json");
    assert_eq!(
        common::envelope_items(&first_page_payload)[0]["nodeId"].as_str(),
        Some("node-change-visible-a")
    );
    let next_page_token = common::envelope_next_page_token(&first_page_payload)
        .expect("change feed should expose nextPageToken");

    let second_page = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-change-acl", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-change-acl", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri(format!(
                    "/app/v3/api/drive/changes?spaceId=space-change-acl&page_size=1&cursor={next_page_token}"
                ))
                .body(Body::empty())
                .expect("change feed second page request should be built"),
        )
        .await
        .expect("change feed second page request should be handled");
    assert_eq!(second_page.status(), StatusCode::OK);
    let second_page_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(second_page.into_body(), usize::MAX)
            .await
            .expect("change feed second page response should be read"),
    )
    .expect("change feed second page response should be valid json");
    let visible_node_ids = common::envelope_items(&second_page_payload)
        .as_array()
        .expect("change feed items should be array")
        .iter()
        .filter_map(|item| item["nodeId"].as_str())
        .collect::<Vec<_>>();
    assert!(visible_node_ids.contains(&"node-change-visible-b"));
    assert!(!visible_node_ids.contains(&"node-change-hidden"));
}

#[tokio::test]
async fn app_drive_standard_views_hide_nodes_without_reader_acl() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-view-acl', 'tenant-view-acl', 'user', 'user-owner', 'personal',
            'View ACL', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    for (id, lifecycle_status) in [
        ("node-recent-hidden", "active"),
        ("node-trash-hidden", "trashed"),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, 'tenant-view-acl', 'space-view-acl', NULL, 'file', $1, 'ready', $2, 1, 'user-owner', 'user-owner')",
        )
        .bind(id)
        .bind(lifecycle_status)
        .execute(&pool)
        .await
        .expect("node should be seeded");
    }

    let app = common::test_router_with_pool(pool);
    for uri in [
        "/app/v3/api/drive/recent?spaceId=space-view-acl",
        "/app/v3/api/drive/trash?spaceId=space-view-acl",
        "/app/v3/api/drive/search?spaceId=space-view-acl&q=hidden",
        "/app/v3/api/drive/shared_with_me?spaceId=space-view-acl",
        "/app/v3/api/drive/favorites?spaceId=space-view-acl",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-view-acl", "user-outsider", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-view-acl", "user-outsider", "appbase"),
                    )
                    .method(Method::GET)
                    .uri(uri)
                    .body(Body::empty())
                    .expect("denied view request should be built"),
            )
            .await
            .expect("denied view request should be handled");
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{uri}");
    }

    let favorite_denied = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-view-acl", "user-outsider", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-view-acl", "user-outsider", "appbase"),
                )
                .method(Method::PUT)
                .uri("/app/v3/api/drive/nodes/node-recent-hidden/favorite")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .expect("denied favorite request should be built"),
        )
        .await
        .expect("denied favorite request should be handled");
    assert_eq!(favorite_denied.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn app_drive_shared_with_me_includes_inherited_and_share_link_nodes() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-shared-view', 'tenant-shared-view', 'user', 'user-owner', 'personal',
            'Shared View', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by, updated_at
        ) VALUES
        ('folder-shared', 'tenant-shared-view', 'space-shared-view', NULL, 'folder', 'Shared Folder', 'ready', 'active', 1, 'user-owner', 'user-owner', '2026-06-04 10:00:00'),
        ('file-inherited', 'tenant-shared-view', 'space-shared-view', 'folder-shared', 'file', 'inherited.txt', 'ready', 'active', 1, 'user-owner', 'user-owner', '2026-06-04 11:00:00'),
        ('file-link-only', 'tenant-shared-view', 'space-shared-view', NULL, 'file', 'link-only.txt', 'ready', 'active', 1, 'user-owner', 'user-owner', '2026-06-04 12:00:00')",
    )
    .execute(&pool)
    .await
    .expect("nodes should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-folder-shared', 'tenant-shared-view', 'folder-shared', 'user', 'user-reviewer',
            'reader', 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("folder permission should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'share-link-only', 'tenant-shared-view', 'file-link-only',
            'sha256:share-link-only-token', 'reader', NULL, NULL, 0,
            'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("share link should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-shared-view", "user-reviewer", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-shared-view", "user-reviewer", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/shared_with_me?spaceId=space-shared-view")
                .body(Body::empty())
                .expect("shared with me request should be built"),
        )
        .await
        .expect("shared with me request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("shared with me response should be read"),
    )
    .expect("shared with me response should be valid json");
    let ids = common::envelope_items(&payload)
        .as_array()
        .expect("shared with me items should be an array")
        .iter()
        .map(|item| item["id"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert!(ids.contains(&"folder-shared".to_string()));
    assert!(ids.contains(&"file-inherited".to_string()));
    assert!(!ids.contains(&"file-link-only".to_string()));
}

#[tokio::test]
async fn app_drive_space_routes_enforce_acl_roles() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-space-acl', 'tenant-space-acl', 'user', 'user-owner', 'personal',
            'Owner Space', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-space-acl-root', 'tenant-space-acl', 'space-space-acl', NULL, 'folder',
            'Root', 'ready', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("root folder should be seeded");

    let app = common::test_router_with_pool(pool);
    for (method, uri, body) in [
        (
            Method::GET,
            "/app/v3/api/drive/spaces/space-space-acl",
            Body::empty(),
        ),
        (
            Method::PATCH,
            "/app/v3/api/drive/spaces/space-space-acl",
            Body::from(
                r#"{
                    "displayName":"Denied"
                }"#,
            ),
        ),
        (
            Method::DELETE,
            "/app/v3/api/drive/spaces/space-space-acl",
            Body::empty(),
        ),
    ] {
        let mut builder = Request::builder()
            .header(
                "authorization",
                format!(
                    "Bearer {}",
                    common::auth_token("tenant-space-acl", "user-outsider", "appbase")
                ),
            )
            .header(
                "access-token",
                common::access_token("tenant-space-acl", "user-outsider", "appbase"),
            )
            .method(method.clone())
            .uri(uri);
        if method != Method::GET && method != Method::DELETE {
            builder = builder.header("content-type", "application/json");
        }
        let response = app
            .clone()
            .oneshot(
                builder
                    .body(body)
                    .expect("denied space request should be built"),
            )
            .await
            .expect("denied space request should be handled");
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{uri}");
    }

    let list_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-space-acl", "user-outsider", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-space-acl", "user-outsider", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces")
                .body(Body::empty())
                .expect("denied list spaces request should be built"),
        )
        .await
        .expect("denied list spaces request should be handled");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("list spaces response should be read"),
    )
    .expect("list spaces response should be valid json");
    assert_eq!(
        common::envelope_items(&list_payload)
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

#[tokio::test]
async fn app_drive_list_spaces_includes_collaborator_granted_team_space() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-team-shared', 'tenant-list-collab', 'group', 'group-team',
            'team', 'Team Shared', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("team space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'folder-team-root', 'tenant-list-collab', 'space-team-shared', NULL, 'folder',
            'Root', 'ready', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("team root folder should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node_permission (
            id, tenant_id, node_id, subject_type, subject_id, role,
            inherited, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'perm-team-collab', 'tenant-list-collab', 'folder-team-root', 'user', 'user-collab',
            'reader', 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("collaborator permission should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-list-collab", "user-collab", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-list-collab", "user-collab", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces")
                .body(Body::empty())
                .expect("list spaces request should be built"),
        )
        .await
        .expect("list spaces request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("list spaces response should be read"),
    )
    .expect("list spaces response should be valid json");
    assert_eq!(payload["code"], 0);
    let ids = payload["data"]["items"]
        .as_array()
        .expect("items should be an array")
        .iter()
        .map(|item| item["id"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["space-team-shared".to_string()]);
}

#[tokio::test]
async fn app_drive_get_space_denies_outsider_on_empty_space_without_leaking_metadata() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-empty-acl', 'tenant-empty-acl', 'user', 'user-owner', 'personal',
            'Empty Space', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-empty-acl", "user-outsider", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-empty-acl", "user-outsider", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces/space-empty-acl")
                .body(Body::empty())
                .expect("get empty space request should be built"),
        )
        .await
        .expect("get empty space request should be handled");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn app_drive_create_space_rejects_foreign_owner_for_personal_space() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-create-acl", "user-caller", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-create-acl", "user-caller", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"space-foreign-owner",
                        "ownerSubjectType":"user",
                        "ownerSubjectId":"user-other",
                        "displayName":"Foreign",
                        "spaceType":"personal"
                    }"#,
                ))
                .expect("create space request should be built"),
        )
        .await
        .expect("create space request should be handled");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn app_drive_create_team_space_rejects_foreign_group_owner() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token_for_organization(
                            "tenant-team-acl",
                            "user-creator",
                            "org-001",
                            "appbase",
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token_for_organization(
                        "tenant-team-acl",
                        "user-creator",
                        "org-001",
                        "appbase",
                    ),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"space-team-foreign",
                        "ownerSubjectType":"group",
                        "ownerSubjectId":"group-foreign",
                        "displayName":"Foreign Team",
                        "spaceType":"team"
                    }"#,
                ))
                .expect("create team space request should be built"),
        )
        .await
        .expect("create team space request should be handled");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn app_drive_create_team_space_bootstraps_creator_owner_access() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token_for_organization(
                            "tenant-team-bootstrap",
                            "user-creator",
                            "org-bootstrap",
                            "appbase",
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token_for_organization(
                        "tenant-team-bootstrap",
                        "user-creator",
                        "org-bootstrap",
                        "appbase",
                    ),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"org-bootstrap:space-team-bootstrap",
                        "ownerSubjectType":"group",
                        "ownerSubjectId":"org-bootstrap:space-team-bootstrap",
                        "displayName":"Engineering",
                        "spaceType":"team"
                    }"#,
                ))
                .expect("create team space request should be built"),
        )
        .await
        .expect("create team space request should be handled");
    assert_eq!(response.status(), StatusCode::CREATED);

    let root_folder_id: String = sqlx::query_scalar(
        "SELECT id FROM dr_drive_node
         WHERE tenant_id='tenant-team-bootstrap'
           AND space_id='org-bootstrap:space-team-bootstrap'
           AND parent_node_id IS NULL
           AND lifecycle_status='active'
         LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("team space root folder should exist");

    let permission_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_node_permission
         WHERE tenant_id='tenant-team-bootstrap'
           AND node_id=$1
           AND subject_type='user'
           AND subject_id='user-creator'
           AND role='owner'
           AND lifecycle_status='active'",
    )
    .bind(&root_folder_id)
    .fetch_one(&pool)
    .await
    .expect("creator owner permission should exist");
    assert_eq!(permission_count, 1);

    let list_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token_for_organization(
                            "tenant-team-bootstrap",
                            "user-creator",
                            "org-bootstrap",
                            "appbase",
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token_for_organization(
                        "tenant-team-bootstrap",
                        "user-creator",
                        "org-bootstrap",
                        "appbase",
                    ),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces")
                .body(Body::empty())
                .expect("list spaces request should be built"),
        )
        .await
        .expect("list spaces request should be handled");
    assert_eq!(list_response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("list spaces response should be read"),
    )
    .expect("list spaces response should be json");
    let space_ids = common::envelope_items(&payload)
        .as_array()
        .expect("items array")
        .iter()
        .filter_map(|item| item["id"].as_str().map(str::to_owned))
        .collect::<Vec<_>>();
    assert!(space_ids.contains(&"org-bootstrap:space-team-bootstrap".to_string()));
}

#[tokio::test]
async fn app_drive_delete_team_space_allows_root_owner_creator() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = common::test_router_with_pool(pool.clone());
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token_for_organization(
                            "tenant-team-delete",
                            "user-creator",
                            "org-delete",
                            "appbase",
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token_for_organization(
                        "tenant-team-delete",
                        "user-creator",
                        "org-delete",
                        "appbase",
                    ),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"org-delete:space-team-delete",
                        "ownerSubjectType":"group",
                        "ownerSubjectId":"org-delete:space-team-delete",
                        "displayName":"Delete Me",
                        "spaceType":"team"
                    }"#,
                ))
                .expect("create team space request should be built"),
        )
        .await
        .expect("create team space request should be handled");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let delete_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token_for_organization(
                            "tenant-team-delete",
                            "user-creator",
                            "org-delete",
                            "appbase",
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token_for_organization(
                        "tenant-team-delete",
                        "user-creator",
                        "org-delete",
                        "appbase",
                    ),
                )
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/spaces/org-delete:space-team-delete")
                .body(Body::empty())
                .expect("delete team space request should be built"),
        )
        .await
        .expect("delete team space request should be handled");
    common::assert_no_content_response(delete_response).await;

    let lifecycle_status: String = sqlx::query_scalar(
        "SELECT lifecycle_status FROM dr_drive_space
         WHERE tenant_id='tenant-team-delete' AND id='org-delete:space-team-delete'",
    )
    .fetch_one(&pool)
    .await
    .expect("deleted team space row should exist");
    assert_eq!(lifecycle_status, "deleted");
}

#[tokio::test]
async fn app_drive_create_team_space_allows_organization_owner_matching_token() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = common::test_router_with_pool(pool.clone());
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token_for_organization(
                            "tenant-team-org",
                            "user-creator",
                            "org-001",
                            "appbase",
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token_for_organization(
                        "tenant-team-org",
                        "user-creator",
                        "org-001",
                        "appbase",
                    ),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"space-team-org",
                        "ownerSubjectType":"organization",
                        "ownerSubjectId":"org-001",
                        "displayName":"Org Team",
                        "spaceType":"team"
                    }"#,
                ))
                .expect("create organization team space request should be built"),
        )
        .await
        .expect("create organization team space request should be handled");
    assert_eq!(response.status(), StatusCode::CREATED);

    let list_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token_for_organization(
                            "tenant-team-org",
                            "user-creator",
                            "org-001",
                            "appbase",
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token_for_organization(
                        "tenant-team-org",
                        "user-creator",
                        "org-001",
                        "appbase",
                    ),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces")
                .body(Body::empty())
                .expect("list spaces request should be built"),
        )
        .await
        .expect("list spaces request should be handled");
    assert_eq!(list_response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("list spaces response should be read"),
    )
    .expect("list spaces response should be json");
    let space_ids = common::envelope_items(&payload)
        .as_array()
        .expect("items array")
        .iter()
        .filter_map(|item| item["id"].as_str().map(str::to_owned))
        .collect::<Vec<_>>();
    assert!(space_ids.contains(&"space-team-org".to_string()));
}

#[tokio::test]
async fn app_drive_create_team_space_rejects_foreign_organization_owner() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token_for_organization(
                            "tenant-team-org-guard",
                            "user-creator",
                            "org-001",
                            "appbase",
                        )
                    ),
                )
                .header(
                    "access-token",
                    common::access_token_for_organization(
                        "tenant-team-org-guard",
                        "user-creator",
                        "org-001",
                        "appbase",
                    ),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"space-team-org-foreign",
                        "ownerSubjectType":"organization",
                        "ownerSubjectId":"org-other",
                        "displayName":"Foreign Org Team",
                        "spaceType":"team"
                    }"#,
                ))
                .expect("create organization team space request should be built"),
        )
        .await
        .expect("create organization team space request should be handled");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn app_drive_claim_share_link_grants_access_and_lists_in_shared_with_me() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-claim', 'tenant-claim', 'user', 'user-owner', 'personal',
            'Claim Space', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by, updated_at
        ) VALUES (
            'node-claim-target', 'tenant-claim', 'space-claim', NULL, 'file', 'shared.docx',
            'ready', 'active', 1, 'user-owner', 'user-owner', '2026-06-04 10:00:00'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'share-claim', 'tenant-claim', 'node-claim-target', $1, 'reader',
            1800000000000, NULL, 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .bind(drive_share_token_hash("claim-share-token"))
    .execute(&pool)
    .await
    .expect("share link should be seeded");

    let app = common::test_router_with_pool(pool);
    let claim_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-claim", "user-collaborator", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-claim", "user-collaborator", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/share_links/claim-share-token/claim")
                .body(Body::empty())
                .expect("claim share link request should be built"),
        )
        .await
        .expect("claim share link request should be handled");
    assert_eq!(claim_response.status(), StatusCode::CREATED);
    let claim_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(claim_response.into_body(), usize::MAX)
            .await
            .expect("claim share link response should be read"),
    )
    .expect("claim share link response should be valid json");
    let claim_data = common::envelope_data(&claim_payload);
    assert_eq!(claim_data["nodeId"].as_str(), Some("node-claim-target"));
    assert_eq!(claim_data["alreadyClaimed"].as_bool(), Some(false));

    let shared_response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-claim", "user-collaborator", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-claim", "user-collaborator", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/shared_with_me?spaceId=space-claim")
                .body(Body::empty())
                .expect("shared with me request should be built"),
        )
        .await
        .expect("shared with me request should be handled");
    assert_eq!(shared_response.status(), StatusCode::OK);
    let shared_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(shared_response.into_body(), usize::MAX)
            .await
            .expect("shared with me response should be read"),
    )
    .expect("shared with me response should be valid json");
    let ids = common::envelope_items(&shared_payload)
        .as_array()
        .expect("shared with me items should be an array")
        .iter()
        .map(|item| item["id"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert!(ids.contains(&"node-claim-target".to_string()));
}

#[tokio::test]
async fn app_drive_claim_share_link_is_idempotent_and_rejects_cross_tenant() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-claim-idempotent', 'tenant-claim-idempotent', 'user', 'user-owner', 'personal',
            'Claim Space', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-claim-idempotent', 'tenant-claim-idempotent', 'space-claim-idempotent', NULL, 'file', 'shared.docx',
            'ready', 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'share-claim-idempotent', 'tenant-claim-idempotent', 'node-claim-idempotent', $1, 'reader',
            1800000000000, NULL, 0, 'active', 1, 'user-owner', 'user-owner'
        )",
    )
    .bind(drive_share_token_hash("claim-idempotent-token"))
    .execute(&pool)
    .await
    .expect("share link should be seeded");

    let app = common::test_router_with_pool(pool);
    for expected_status in [StatusCode::CREATED, StatusCode::OK] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token(
                                "tenant-claim-idempotent",
                                "user-collaborator",
                                "appbase",
                            )
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token(
                            "tenant-claim-idempotent",
                            "user-collaborator",
                            "appbase",
                        ),
                    )
                    .method(Method::POST)
                    .uri("/app/v3/api/drive/share_links/claim-idempotent-token/claim")
                    .body(Body::empty())
                    .expect("claim share link request should be built"),
            )
            .await
            .expect("claim share link request should be handled");
        assert_eq!(response.status(), expected_status);
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("claim share link response should be read"),
        )
        .expect("claim share link response should be valid json");
        assert_eq!(
            common::envelope_data(&payload)["alreadyClaimed"].as_bool(),
            Some(expected_status == StatusCode::OK)
        );
    }

    let denied = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-other", "user-outsider", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-other", "user-outsider", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/share_links/claim-idempotent-token/claim")
                .body(Body::empty())
                .expect("cross tenant claim request should be built"),
        )
        .await
        .expect("cross tenant claim request should be handled");
    assert_eq!(denied.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn drive_problem_responses_include_correlation_ids() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let app = common::test_router_with_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-corr", "user-owner", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-corr", "user-owner", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces/space-missing-corr/nodes")
                .body(Body::empty())
                .expect("correlation request should be built"),
        )
        .await
        .expect("correlation request should be handled");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("correlation response should be read"),
    )
    .expect("correlation response should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40401));
    assert!(payload["traceId"]
        .as_str()
        .is_some_and(|value| !value.is_empty() && value != "request-unset"));
    assert!(payload["traceId"]
        .as_str()
        .is_some_and(|value| !value.is_empty() && value != "trace-unset"));
}
