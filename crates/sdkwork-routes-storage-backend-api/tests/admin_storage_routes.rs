use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use axum::Router;
use http::{Method, Request, StatusCode};
use sdkwork_drive_config::DatabaseConfig;
use sdkwork_routes_storage_backend_api::{
    build_router_with_database_config_and_admin_storage_config,
    build_router_with_pool_config_without_iam, build_router_with_pool_without_iam,
    build_router_with_pool_without_iam_and_test_tenant, AdminStorageConfig,
    DriveAdminStorageObjectStoreAdapter,
};
use std::sync::{Arc, Mutex};
use tower::util::ServiceExt;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapturedS3Request {
    method: String,
    path: String,
    query: String,
}

type CapturedS3Requests = Arc<Mutex<Vec<CapturedS3Request>>>;

async fn assert_no_content_response(response: Response) {
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("204 response body should be readable");
    assert!(
        body.is_empty(),
        "204 delete response must not include a JSON body"
    );
}

async fn start_s3_mock_server() -> (String, CapturedS3Requests) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let router = Router::new()
        .fallback(mock_s3_endpoint)
        .with_state(requests.clone());
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
    State(requests): State<CapturedS3Requests>,
    method: Method,
    uri: Uri,
    body: Body,
) -> Response {
    let query = uri.query().unwrap_or_default().to_string();
    let _ = to_bytes(body, usize::MAX)
        .await
        .expect("mock s3 request body should be readable");
    requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .push(CapturedS3Request {
            method: method.as_str().to_string(),
            path: uri.path().to_string(),
            query: query.clone(),
        });

    if method == Method::HEAD && uri.path() == "/bucket-admin/missing.txt" {
        return StatusCode::NOT_FOUND.into_response();
    }
    if method == Method::HEAD && uri.path() == "/bucket-admin/oversized.bin" {
        return (
            StatusCode::OK,
            [("content-length", "8388609")],
            Body::empty(),
        )
            .into_response();
    }
    if method == Method::HEAD {
        return (
            StatusCode::OK,
            [("content-length", "0")],
            Body::empty(),
        )
            .into_response();
    }
    if method == Method::GET && uri.path() == "/bucket-admin/objects/notes.txt" {
        return (
            StatusCode::OK,
            [
                ("content-type", "text/plain"),
                ("content-length", "11"),
            ],
            Body::from("hello world"),
        )
            .into_response();
    }
    if method == Method::GET
        && uri.path() == "/"
        && (query.is_empty() || query.contains("x-id=ListBuckets"))
    {
        return (
            StatusCode::OK,
            [("content-type", "application/xml")],
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ListAllMyBucketsResult>
  <Buckets>
    <Bucket>
      <Name>bucket-admin</Name>
      <CreationDate>2026-06-04T00:00:00.000Z</CreationDate>
    </Bucket>
    <Bucket>
      <Name>bucket-archive</Name>
      <CreationDate>2026-06-05T00:00:00.000Z</CreationDate>
    </Bucket>
  </Buckets>
</ListAllMyBucketsResult>"#,
        )
            .into_response();
    }
    if method == Method::GET && query.contains("list-type=2") {
        return (
            StatusCode::OK,
            [("content-type", "application/xml")],
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult>
  <Name>bucket-admin</Name>
  <Prefix>objects/</Prefix>
  <KeyCount>1</KeyCount>
  <MaxKeys>100</MaxKeys>
  <IsTruncated>false</IsTruncated>
  <Contents>
    <Key>objects/file-a.bin</Key>
    <LastModified>2026-06-04T00:00:00.000Z</LastModified>
    <ETag>"etag-a"</ETag>
    <Size>128</Size>
    <StorageClass>STANDARD</StorageClass>
  </Contents>
</ListBucketResult>"#,
        )
            .into_response();
    }
    StatusCode::OK.into_response()
}

#[tokio::test]
async fn admin_storage_provider_routes_mask_credentials_and_report_capabilities() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = build_router_with_pool_without_iam(pool);
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"provider-tencent-cos",
                        "providerKind":"tencent_cos",
                        "name":"Tencent COS",
                        "endpointUrl":"https://cos.ap-guangzhou.myqcloud.com",
                        "region":"ap-guangzhou",
                        "bucket":"drive-bucket",
                        "strictTls":true,
                        "credentialRef":"plain:secret-id:secret-key",
                        "serverSideEncryptionMode":"AES256",
                        "defaultStorageClass":"STANDARD"
                    }"#,
                ))
                .expect("create provider request should be built"),
        )
        .await
        .expect("create provider request should be handled");
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("create provider response body should be read"),
    )
    .expect("create provider response should be json");
    assert_eq!(create_payload["providerKind"], "tencent_cos");
    assert_eq!(create_payload["credentialConfigured"], true);
    assert_eq!(create_payload["credentialRef"], "plain:***");
    assert_eq!(create_payload["pathStyle"], false);
    assert_eq!(create_payload["strictTls"], true);

    let get_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-tencent-cos")
                .body(Body::empty())
                .expect("get provider request should be built"),
        )
        .await
        .expect("get provider request should be handled");
    assert_eq!(get_response.status(), StatusCode::OK);
    let get_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(get_response.into_body(), usize::MAX)
            .await
            .expect("get provider response body should be read"),
    )
    .expect("get provider response should be json");
    assert_eq!(get_payload["credentialRef"], "plain:***");
    assert_eq!(get_payload["strictTls"], true);

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/backend/v3/api/drive/storage/providers/provider-tencent-cos")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "endpointUrl":"https://cos.ap-guangzhou.myqcloud.com",
                        "strictTls":false
                    }"#,
                ))
                .expect("update provider request should be built"),
        )
        .await
        .expect("update provider request should be handled");
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(update_response.into_body(), usize::MAX)
            .await
            .expect("update provider response body should be read"),
    )
    .expect("update provider response should be json");
    assert_eq!(update_payload["strictTls"], false);

    let capabilities_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-tencent-cos/capabilities")
                .body(Body::empty())
                .expect("capabilities request should be built"),
        )
        .await
        .expect("capabilities request should be handled");
    assert_eq!(capabilities_response.status(), StatusCode::OK);
    let capabilities_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(capabilities_response.into_body(), usize::MAX)
            .await
            .expect("capabilities response body should be read"),
    )
    .expect("capabilities response should be json");
    assert_eq!(capabilities_payload["supportsMultipartUpload"], true);
    assert_eq!(capabilities_payload["supportsPresignedUploadPart"], true);
    assert_eq!(capabilities_payload["supportsCredentialRotation"], true);
}

#[tokio::test]
async fn admin_storage_default_binding_can_mount_provider_to_tenant_or_space() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, display_name,
            space_type, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-git-repositories', 'tenant-storage', 'user', 'user-storage',
            'Repositories', 'git_repository', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    let app = build_router_with_pool_without_iam(pool);
    let create_provider = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"provider-binding",
                        "providerKind":"volcengine_tos",
                        "name":"Volcengine TOS",
                        "endpointUrl":"https://tos-cn-beijing.volces.com",
                        "region":"cn-beijing",
                        "bucket":"drive-bucket",
                        "credentialRef":"plain:access-key:secret-key"
                    }"#,
                ))
                .expect("create provider request should be built"),
        )
        .await
        .expect("create provider request should be handled");
    assert_eq!(create_provider.status(), StatusCode::CREATED);

    let set_binding = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/bindings/default")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "spaceId":"space-git-repositories",
                        "providerId":"provider-binding"
                    }"#,
                ))
                .expect("set binding request should be built"),
        )
        .await
        .expect("set binding request should be handled");
    assert_eq!(set_binding.status(), StatusCode::OK);
    let binding_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(set_binding.into_body(), usize::MAX)
            .await
            .expect("set binding response body should be read"),
    )
    .expect("set binding response should be json");
    assert_eq!(binding_payload["bindingScope"], "space");
    assert_eq!(binding_payload["spaceId"], "space-git-repositories");
    assert_eq!(
        binding_payload["storageRootPrefix"],
        "sdkwork-drive/v1/tenants/tenant-storage/spaces/space-git-repositories"
    );
    assert_eq!(
        binding_payload["storageProvider"]["providerKind"],
        "volcengine_tos"
    );

    let get_binding = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(
                    "/backend/v3/api/drive/storage/bindings/default?spaceId=space-git-repositories",
                )
                .body(Body::empty())
                .expect("get binding request should be built"),
        )
        .await
        .expect("get binding request should be handled");
    assert_eq!(get_binding.status(), StatusCode::OK);
}

#[tokio::test]
async fn admin_storage_delete_provider_rejects_active_provider_bindings() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, display_name,
            space_type, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-provider-delete', 'tenant-provider-delete', 'user', 'user-provider-delete',
            'Provider Delete Space', 'personal', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    let app =
        build_router_with_pool_without_iam_and_test_tenant(pool.clone(), "tenant-provider-delete");
    let create_provider = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"provider-bound-delete",
                        "providerKind":"s3_compatible",
                        "name":"Bound Delete S3",
                        "endpointUrl":"https://s3.amazonaws.com",
                        "region":"us-east-1",
                        "bucket":"bound-delete-bucket",
                        "credentialRef":"plain:access-key:secret-key"
                    }"#,
                ))
                .expect("create provider request should be built"),
        )
        .await
        .expect("create provider request should be handled");
    assert_eq!(create_provider.status(), StatusCode::CREATED);

    let set_binding = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/bindings/default")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "spaceId":"space-provider-delete",
                        "providerId":"provider-bound-delete"
                    }"#,
                ))
                .expect("set binding request should be built"),
        )
        .await
        .expect("set binding request should be handled");
    assert_eq!(set_binding.status(), StatusCode::OK);

    let patch_bucket_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/backend/v3/api/drive/storage/providers/provider-bound-delete")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"bucket":"bound-delete-bucket-updated"}"#))
                .expect("patch provider bucket request should be built"),
        )
        .await
        .expect("patch provider bucket request should be handled");
    assert_eq!(patch_bucket_response.status(), StatusCode::CONFLICT);
    let patch_bucket_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(patch_bucket_response.into_body(), usize::MAX)
            .await
            .expect("patch provider bucket conflict response body should be read"),
    )
    .expect("patch provider bucket conflict response should be json");
    assert_eq!(patch_bucket_payload["code"], 40901);
    assert!(patch_bucket_payload["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("bucket cannot be changed")));

    let patch_deleted_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/backend/v3/api/drive/storage/providers/provider-bound-delete")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"status":"deleted"}"#))
                .expect("patch provider deleted request should be built"),
        )
        .await
        .expect("patch provider deleted request should be handled");
    assert_eq!(patch_deleted_response.status(), StatusCode::CONFLICT);
    let patch_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(patch_deleted_response.into_body(), usize::MAX)
            .await
            .expect("patch provider conflict response body should be read"),
    )
    .expect("patch provider conflict response should be json");
    assert_eq!(patch_payload["code"], 40901);
    assert!(patch_payload["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("storage provider has active bindings")));

    let delete_response = app
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/backend/v3/api/drive/storage/providers/provider-bound-delete")
                .body(Body::empty())
                .expect("delete provider request should be built"),
        )
        .await
        .expect("delete provider request should be handled");
    assert_eq!(delete_response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(delete_response.into_body(), usize::MAX)
            .await
            .expect("delete provider conflict response body should be read"),
    )
    .expect("delete provider conflict response should be json");
    assert_eq!(payload["code"], 40901);
    assert!(payload["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("storage provider has active bindings")));

    let provider_status: String =
        sqlx::query_scalar("SELECT status FROM dr_drive_storage_provider WHERE id=$1")
            .bind("provider-bound-delete")
            .fetch_one(&pool)
            .await
            .expect("provider status should be readable");
    assert_eq!(provider_status, "active");
}

#[tokio::test]
async fn admin_storage_activate_rejects_deleted_provider() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = build_router_with_pool_without_iam(pool.clone());
    let create_provider = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"provider-deleted-terminal",
                        "providerKind":"s3_compatible",
                        "name":"Deleted Terminal S3",
                        "endpointUrl":"https://s3.amazonaws.com",
                        "region":"us-east-1",
                        "bucket":"deleted-terminal-bucket",
                        "credentialRef":"plain:access-key:secret-key"
                    }"#,
                ))
                .expect("create provider request should be built"),
        )
        .await
        .expect("create provider request should be handled");
    assert_eq!(create_provider.status(), StatusCode::CREATED);

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/backend/v3/api/drive/storage/providers/provider-deleted-terminal")
                .body(Body::empty())
                .expect("delete provider request should be built"),
        )
        .await
        .expect("delete provider request should be handled");
    assert_no_content_response(delete_response).await;

    let rotate_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers/provider-deleted-terminal/credentials/rotate")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"credentialRef":"plain:new-access:new-secret"}"#,
                ))
                .expect("rotate deleted provider credential request should be built"),
        )
        .await
        .expect("rotate deleted provider credential request should be handled");
    assert_eq!(rotate_response.status(), StatusCode::CONFLICT);
    let rotate_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(rotate_response.into_body(), usize::MAX)
            .await
            .expect("rotate deleted provider conflict response body should be read"),
    )
    .expect("rotate deleted provider conflict response should be json");
    assert_eq!(rotate_payload["code"], 40901);
    assert!(rotate_payload["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("deleted storage provider cannot be modified")));

    let activate_response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers/provider-deleted-terminal/activate")
                .body(Body::empty())
                .expect("activate deleted provider request should be built"),
        )
        .await
        .expect("activate deleted provider request should be handled");
    assert_eq!(activate_response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(activate_response.into_body(), usize::MAX)
            .await
            .expect("activate deleted provider conflict response body should be read"),
    )
    .expect("activate deleted provider conflict response should be json");
    assert_eq!(payload["code"], 40901);
    assert!(payload["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("deleted storage provider cannot be reactivated")));

    let provider_status: String =
        sqlx::query_scalar("SELECT status FROM dr_drive_storage_provider WHERE id=$1")
            .bind("provider-deleted-terminal")
            .fetch_one(&pool)
            .await
            .expect("provider status should be readable");
    assert_eq!(provider_status, "deleted");
}

#[tokio::test]
async fn admin_storage_binding_rejects_invalid_storage_root_prefix() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            credential_ref, status, version, created_by, updated_by
        ) VALUES (
            'provider-root-prefix', 's3_compatible', 'Root Prefix S3', 'https://s3.amazonaws.com',
            'us-east-1', 'root-prefix-bucket', 0, 'plain:access-key:secret-key',
            'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/bindings/default")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "providerId":"provider-root-prefix",
                        "storageRootPrefix":"../escape"
                    }"#,
                ))
                .expect("invalid binding request should be built"),
        )
        .await
        .expect("invalid binding request should be handled");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("invalid binding response body should be read"),
    )
    .expect("invalid binding response should be json");
    assert_eq!(payload["code"], 40001);
    assert!(payload["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("storageRootPrefix")));
}

#[tokio::test]
async fn admin_storage_binding_routes_list_and_delete_space_mounts_with_audit() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, display_name,
            space_type, lifecycle_status, version, created_by, updated_by
        ) VALUES
            (
                'space-admin-a', 'tenant-storage', 'user', 'user-storage',
                'Admin Space A', 'personal', 'active', 1, 'admin-storage', 'admin-storage'
            ),
            (
                'space-admin-b', 'tenant-storage', 'user', 'user-storage',
                'Admin Space B', 'git_repository', 'active', 1, 'admin-storage', 'admin-storage'
            )",
    )
    .execute(&pool)
    .await
    .expect("spaces should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            credential_ref, status, version, created_by, updated_by
        ) VALUES
            (
                'provider-tenant-default', 's3_compatible', 'Tenant Default', 'https://s3.amazonaws.com',
                'us-east-1', 'tenant-bucket', 0, 'plain:access-key:secret-key',
                'active', 1, 'admin-storage', 'admin-storage'
            ),
            (
                'provider-space-default', 'aliyun_oss', 'Space Default', 'https://oss-cn-hangzhou.aliyuncs.com',
                'cn-hangzhou', 'space-bucket', 0, 'plain:access-key:secret-key',
                'active', 1, 'admin-storage', 'admin-storage'
            )",
    )
    .execute(&pool)
    .await
    .expect("storage providers should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES
            (
                'default:tenant:tenant-storage', 'tenant-storage', NULL,
                'provider-tenant-default', 'tenant', 'primary',
                'sdkwork-drive/v1/tenants/tenant-storage', 'active', 1,
                'admin-storage', 'admin-storage'
            ),
            (
                'default:space:tenant-storage:space-admin-a', 'tenant-storage', 'space-admin-a',
                'provider-space-default', 'space', 'primary',
                'sdkwork-drive/v1/tenants/tenant-storage/spaces/space-admin-a',
                'active', 1,
                'admin-storage', 'admin-storage'
            ),
            (
                'default:space:tenant-storage:space-admin-b', 'tenant-storage', 'space-admin-b',
                'provider-tenant-default', 'space', 'primary',
                'sdkwork-drive/v1/tenants/tenant-storage/spaces/space-admin-b',
                'deleted', 2,
                'admin-storage', 'admin-storage'
            )",
    )
    .execute(&pool)
    .await
    .expect("storage provider bindings should be seeded");

    let app = build_router_with_pool_without_iam(pool.clone());
    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/bindings")
                .body(Body::empty())
                .expect("list bindings request should be built"),
        )
        .await
        .expect("list bindings request should be handled");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("list bindings response body should be read"),
    )
    .expect("list bindings response should be json");
    assert_eq!(list_payload["code"], 0);
    assert_eq!(list_payload["data"]["items"].as_array().unwrap().len(), 2);
    assert_eq!(list_payload["data"]["items"][0]["bindingScope"], "space");
    assert_eq!(list_payload["data"]["items"][0]["spaceId"], "space-admin-a");
    assert_eq!(
        list_payload["data"]["items"][0]["storageProvider"]["providerKind"],
        "aliyun_oss"
    );
    assert_eq!(list_payload["data"]["items"][1]["bindingScope"], "tenant");
    assert_eq!(
        list_payload["data"]["items"][1]["spaceId"],
        serde_json::Value::Null
    );

    let filtered_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/bindings?providerId=provider-space-default")
                .body(Body::empty())
                .expect("filtered bindings request should be built"),
        )
        .await
        .expect("filtered bindings request should be handled");
    assert_eq!(filtered_response.status(), StatusCode::OK);
    let filtered_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(filtered_response.into_body(), usize::MAX)
            .await
            .expect("filtered bindings response body should be read"),
    )
    .expect("filtered bindings response should be json");
    assert_eq!(filtered_payload["code"], 0);
    assert_eq!(
        filtered_payload["data"]["items"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        filtered_payload["data"]["items"][0]["providerId"],
        "provider-space-default"
    );

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/backend/v3/api/drive/storage/bindings/default?spaceId=space-admin-a")
                .body(Body::empty())
                .expect("delete binding request should be built"),
        )
        .await
        .expect("delete binding request should be handled");
    assert_no_content_response(delete_response).await;

    let get_deleted_binding = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/bindings/default?spaceId=space-admin-a")
                .body(Body::empty())
                .expect("get deleted binding request should be built"),
        )
        .await
        .expect("get deleted binding request should be handled");
    assert_eq!(get_deleted_binding.status(), StatusCode::NOT_FOUND);

    let binding_actions: Vec<String> = sqlx::query_scalar(
        "SELECT action
         FROM dr_drive_audit_event
         WHERE resource_type='storage_provider_binding'
           AND resource_id='space-admin-a'
         ORDER BY id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("binding audit events should be queryable");
    assert_eq!(
        binding_actions,
        vec!["drive.storage_provider_binding.default_deleted"]
    );
}

#[tokio::test]
async fn admin_storage_provider_bucket_routes_list_account_buckets() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, status, version, created_by, updated_by
        ) VALUES (
            'provider-bucket-list-s3', 's3_compatible', 'Bucket List S3', $1, 'us-east-1',
            'bucket-admin', 1, 0, 'plain:test-access-key:test-secret-key',
            'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool);
    let list_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-bucket-list-s3/buckets")
                .body(Body::empty())
                .expect("bucket list request should be built"),
        )
        .await
        .expect("bucket list request should be handled");
    let status = list_response.status();
    let body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("bucket list response body should be read");
    assert_eq!(
        status,
        StatusCode::OK,
        "bucket list body: {}",
        String::from_utf8_lossy(&body)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("bucket list response should be json");
    assert_eq!(payload["code"], 0);
    let items = payload["data"]["items"]
        .as_array()
        .expect("bucket list items");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["bucket"], "bucket-admin");
    assert_eq!(items[0]["configured"], true);
    assert_eq!(items[0]["creationDateEpochMs"], 1780531200000_i64);
    assert_eq!(items[1]["bucket"], "bucket-archive");
    assert_eq!(items[1]["configured"], false);

    let captured = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(
        captured.iter().any(|request| request.method == "GET"
            && request.path == "/"
            && (request.query.is_empty() || request.query.contains("x-id=ListBuckets"))),
        "admin bucket listing should call S3 ListBuckets: {captured:?}"
    );
}

#[tokio::test]
async fn admin_storage_bucket_and_object_routes_use_configured_s3_store() {
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
            'provider-admin-s3', 's3_compatible', 'Admin S3', $1, 'us-east-1',
            'bucket-admin', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool);
    let bucket_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-admin-s3/bucket")
                .body(Body::empty())
                .expect("bucket request should be built"),
        )
        .await
        .expect("bucket request should be handled");
    assert_eq!(bucket_response.status(), StatusCode::OK);

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-admin-s3/objects?prefix=objects/&page_size=100")
                .body(Body::empty())
                .expect("object list request should be built"),
        )
        .await
        .expect("object list request should be handled");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("object list response body should be read"),
    )
    .expect("object list response should be json");
    assert_eq!(list_payload["code"], 0);
    assert_eq!(
        list_payload["data"]["items"][0]["objectKey"],
        "objects/file-a.bin"
    );
    assert_eq!(list_payload["data"]["items"][0]["objectKind"], "object");
    assert_eq!(list_payload["data"]["items"][0]["contentLength"], 128);

    let head_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-admin-s3/objects/objects/file-a.bin")
                .body(Body::empty())
                .expect("object head request should be built"),
        )
        .await
        .expect("object head request should be handled");
    assert_eq!(head_response.status(), StatusCode::OK);

    let delete_response = app
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/backend/v3/api/drive/storage/providers/provider-admin-s3/objects/objects/file-a.bin")
                .body(Body::empty())
                .expect("object delete request should be built"),
        )
        .await
        .expect("object delete request should be handled");
    assert_no_content_response(delete_response).await;

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(
        requests
            .iter()
            .any(|request| request.method == "HEAD" && request.path == "/bucket-admin/"),
        "bucket route should call S3 HeadBucket"
    );
    assert!(
        requests.iter().any(|request| request.method == "GET"
            && request.path == "/bucket-admin/"
            && request.query.contains("list-type=2")),
        "object list route should call S3 ListObjectsV2"
    );
    assert!(
        requests.iter().any(|request| request.method == "HEAD"
            && request.path == "/bucket-admin/objects/file-a.bin"),
        "object head route should call S3 HeadObject"
    );
    assert!(
        requests.iter().any(|request| request.method == "DELETE"
            && request.path == "/bucket-admin/objects/file-a.bin"),
        "object delete route should call S3 DeleteObject"
    );
}

#[tokio::test]
async fn admin_storage_object_routes_reject_leading_slash_object_keys_before_calling_s3() {
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
            'provider-admin-object-key', 's3_compatible', 'Admin Object Key S3', $1, 'us-east-1',
            'bucket-admin', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let response = build_router_with_pool_without_iam(pool)
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-admin-object-key/objects/%2Fleading-slash")
                .body(Body::empty())
                .expect("object head request should be built"),
        )
        .await
        .expect("object head request should be handled");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    assert!(
        captured_requests
            .lock()
            .expect("captured s3 requests mutex should not be poisoned")
            .is_empty(),
        "invalid object key should fail before calling object storage"
    );
}

#[tokio::test]
async fn admin_storage_copy_object_rejects_invalid_destination_bucket_before_calling_s3() {
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
            'provider-copy-bucket-validation', 's3_compatible', 'Copy Bucket Validation S3',
            $1, 'us-east-1', 'bucket-admin', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let response = build_router_with_pool_without_iam(pool)
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers/provider-copy-bucket-validation/objects/copy")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sourceObjectKey":"objects/file-a.bin",
                        "destinationObjectKey":"objects/file-b.bin",
                        "destinationBucket":"Drive_Bucket"
                    }"#,
                ))
                .expect("object copy request should be built"),
        )
        .await
        .expect("object copy request should be handled");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("error response body should be read"),
    )
    .expect("error response should be json");
    assert!(
        payload["detail"]
            .as_str()
            .is_some_and(|detail| detail.contains("destinationBucket")),
        "validation error should name destinationBucket: {payload}"
    );

    assert!(
        captured_requests
            .lock()
            .expect("captured s3 requests mutex should not be poisoned")
            .is_empty(),
        "invalid destination bucket should fail before calling object storage"
    );
}

#[tokio::test]
async fn admin_storage_object_content_routes_write_then_read_through_configured_s3_store() {
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
            'provider-content-routes', 's3_compatible', 'Content Routes S3', $1, 'us-east-1',
            'bucket-admin', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool);

    let put_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-content-routes/object-contents/objects/notes.txt")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"content":"hello world","contentType":"text/plain"}"#,
                ))
                .expect("object content write request should be built"),
        )
        .await
        .expect("object content write request should be handled");
    assert_eq!(put_response.status(), StatusCode::OK);
    let put_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(put_response.into_body(), usize::MAX)
            .await
            .expect("object content write response body should be read"),
    )
    .expect("object content write response should be json");
    assert_eq!(put_payload["code"], 0);
    assert_eq!(
        put_payload["data"]["item"]["objectKey"],
        "objects/notes.txt"
    );

    let get_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-content-routes/object-contents/objects/notes.txt")
                .body(Body::empty())
                .expect("object content read request should be built"),
        )
        .await
        .expect("object content read request should be handled");
    assert_eq!(get_response.status(), StatusCode::OK);
    let get_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(get_response.into_body(), usize::MAX)
            .await
            .expect("object content read response body should be read"),
    )
    .expect("object content read response should be json");
    assert_eq!(get_payload["data"]["item"]["objectKey"], "objects/notes.txt");
    assert_eq!(get_payload["data"]["item"]["sizeBytes"], 11);
    assert_eq!(get_payload["data"]["item"]["encoding"], "base64");
    assert_eq!(get_payload["data"]["item"]["content"], "aGVsbG8gd29ybGQ=");
    assert_eq!(
        get_payload["data"]["item"]["checksumSha256"],
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );

    let put_b64_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-content-routes/object-contents/objects/blob.bin")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"content":"aGVsbG8=","encoding":"base64"}"#))
                .expect("object content base64 write request should be built"),
        )
        .await
        .expect("object content base64 write request should be handled");
    assert_eq!(put_b64_response.status(), StatusCode::OK);

    let put_dir_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-content-routes/object-contents/objects/folder/")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"content":""}"#))
                .expect("object directory placeholder request should be built"),
        )
        .await
        .expect("object directory placeholder request should be handled");
    assert_eq!(put_dir_response.status(), StatusCode::OK);
    let put_dir_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(put_dir_response.into_body(), usize::MAX)
            .await
            .expect("object directory placeholder response body should be read"),
    )
    .expect("object directory placeholder response should be json");
    assert_eq!(put_dir_payload["data"]["item"]["objectKey"], "objects/folder/");

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(
        requests.iter().any(|request| request.method == "PUT"
            && request.path == "/bucket-admin/objects/notes.txt"),
        "object content write route should call S3 PutObject"
    );
    assert!(
        requests.iter().any(|request| request.method == "PUT"
            && request.path == "/bucket-admin/objects/blob.bin"),
        "base64 object content write route should call S3 PutObject"
    );
    assert!(
        requests.iter().any(|request| request.method == "PUT"
            && request.path == "/bucket-admin/objects/folder/"),
        "directory placeholder write route should call S3 PutObject with trailing slash key"
    );
    assert!(
        requests.iter().any(|request| request.method == "GET"
            && request.path == "/bucket-admin/objects/notes.txt"),
        "object content read route should call S3 GetObject"
    );
}

#[tokio::test]
async fn admin_storage_object_content_routes_reject_invalid_keys_and_encodings() {
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
            'provider-content-validation', 's3_compatible', 'Content Validation S3', $1, 'us-east-1',
            'bucket-admin', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool);

    let invalid_dir_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-content-validation/object-contents/objects/folder/")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"content":"not empty"}"#))
                .expect("object content request should be built"),
        )
        .await
        .expect("object content request should be handled");
    assert_eq!(invalid_dir_response.status(), StatusCode::BAD_REQUEST);

    let invalid_encoding_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-content-validation/object-contents/objects/notes.txt")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"content":"AAAA","encoding":"hex"}"#))
                .expect("object content request should be built"),
        )
        .await
        .expect("object content request should be handled");
    assert_eq!(invalid_encoding_response.status(), StatusCode::BAD_REQUEST);

    let invalid_base64_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-content-validation/object-contents/objects/notes.txt")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"content":"!!!not-base64!!!","encoding":"base64"}"#))
                .expect("object content request should be built"),
        )
        .await
        .expect("object content request should be handled");
    assert_eq!(invalid_base64_response.status(), StatusCode::BAD_REQUEST);

    let leading_slash_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-content-validation/object-contents/%2Fleading")
                .body(Body::empty())
                .expect("object content read request should be built"),
        )
        .await
        .expect("object content read request should be handled");
    assert_eq!(leading_slash_response.status(), StatusCode::BAD_REQUEST);

    assert!(
        captured_requests
            .lock()
            .expect("captured s3 requests mutex should not be poisoned")
            .is_empty(),
        "invalid content requests should fail before calling object storage"
    );
}

#[cfg(not(feature = "opendal-s3-plugin"))]
#[tokio::test]
async fn admin_storage_opendal_plugin_adapter_is_default_disabled_without_feature() {
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
            'provider-admin-opendal-disabled', 's3_compatible', 'Admin OpenDAL S3', $1, 'us-east-1',
            'bucket-admin', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_config_without_iam(
        pool,
        AdminStorageConfig {
            object_store_adapter: DriveAdminStorageObjectStoreAdapter::OpendalS3,
        },
    );
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-admin-opendal-disabled/objects?prefix=objects/&page_size=10")
                .body(Body::empty())
                .expect("object list request should be built"),
        )
        .await
        .expect("object list request should be handled");
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("error response body should be read"),
    )
    .expect("error response should be json");
    assert!(
        payload["detail"]
            .as_str()
            .expect("problem detail should be a string")
            .contains("OpenDAL S3 plugin is not enabled"),
        "problem detail should explain that the optional plugin is disabled: {payload}"
    );
    assert!(
        captured_requests
            .lock()
            .expect("captured s3 requests mutex should not be poisoned")
            .is_empty(),
        "disabled OpenDAL plugin should fail before calling object storage"
    );
}

#[tokio::test]
async fn admin_storage_bucket_admin_uses_full_s3_adapter_even_when_object_plugin_is_selected() {
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
            'provider-admin-bucket-plugin-selected', 's3_compatible', 'Admin Bucket S3', $1, 'us-east-1',
            'bucket-admin', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_config_without_iam(
        pool,
        AdminStorageConfig {
            object_store_adapter: DriveAdminStorageObjectStoreAdapter::OpendalS3,
        },
    );
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-admin-bucket-plugin-selected/buckets")
                .body(Body::empty())
                .expect("bucket list request should be built"),
        )
        .await
        .expect("bucket list request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("bucket list response body should be read"),
    )
    .expect("bucket list response should be json");
    assert_eq!(payload["code"], 0);
    assert_eq!(payload["data"]["items"].as_array().unwrap().len(), 2);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(
        requests
            .iter()
            .any(|request| request.method == "GET" && request.path == "/"),
        "bucket admin should use the full S3 adapter list-buckets operation"
    );
}

#[test]
fn admin_storage_config_reads_object_store_adapter_from_env() {
    let default_config = AdminStorageConfig::from_env_pairs(Vec::<(&str, &str)>::new())
        .expect("empty env should use default admin storage config");
    assert_eq!(
        default_config.object_store_adapter,
        DriveAdminStorageObjectStoreAdapter::AwsSdkS3
    );

    let opendal_config = AdminStorageConfig::from_env_pairs([(
        "SDKWORK_DRIVE_ADMIN_STORAGE_OBJECT_STORE_ADAPTER",
        "opendal_s3",
    )])
    .expect("opendal adapter env should parse");
    assert_eq!(
        opendal_config.object_store_adapter,
        DriveAdminStorageObjectStoreAdapter::OpendalS3
    );

    let invalid = AdminStorageConfig::from_env_pairs([(
        "SDKWORK_DRIVE_ADMIN_STORAGE_OBJECT_STORE_ADAPTER",
        "handwritten_xml_s3",
    )])
    .expect_err("invalid adapter env should fail");
    assert!(
        invalid.contains("SDKWORK_DRIVE_ADMIN_STORAGE_OBJECT_STORE_ADAPTER"),
        "error should identify the invalid env var: {invalid}"
    );
}

#[tokio::test]
async fn admin_storage_database_router_can_receive_explicit_plugin_config() {
    // 服务端只支持 PostgreSQL（sqlite 被 admin_storage_database_url_router_rejects_sqlite
    // 显式拒绝）：用测试库 URL 验证显式 DatabaseConfig + AdminStorageConfig 可构建路由。
    let Ok(database_url) = std::env::var("SDKWORK_DATABASE_URL") else {
        eprintln!("skip PostgreSQL integration test: SDKWORK_DATABASE_URL is not set");
        return;
    };
    let database_config = DatabaseConfig::from_url(&database_url)
        .expect("postgres database config should parse");
    let router = build_router_with_database_config_and_admin_storage_config(
        &database_config,
        AdminStorageConfig {
            object_store_adapter: DriveAdminStorageObjectStoreAdapter::AwsSdkS3,
        },
    )
    .await
    .expect("admin storage router should build with explicit storage config");

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/healthz")
                .body(Body::empty())
                .expect("health request should be built"),
        )
        .await
        .expect("health request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn admin_storage_database_url_router_rejects_sqlite() {
    let error = sdkwork_routes_storage_backend_api::build_router_with_database_url(
        "sqlite://target/drive-admin-storage-tests/missing-parent/router.sqlite",
    )
    .await
    .expect_err("server runtime must reject SQLite URLs");

    assert!(
        matches!(error, sqlx::Error::Configuration(_)),
        "SQLite rejection must remain a configuration error: {error:?}"
    );
}

#[tokio::test]
async fn admin_storage_provider_test_route_checks_configured_s3_bucket() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, status, version, created_by, updated_by
        ) VALUES (
            'provider-test-s3', 's3_compatible', 'Admin S3', $1, 'us-east-1',
            'bucket-admin', 1, 0, 'plain:test-access-key:test-secret-key',
            'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers/provider-test-s3/test")
                .body(Body::empty())
                .expect("test provider request should be built"),
        )
        .await
        .expect("test provider request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("test provider response body should be read"),
    )
    .expect("test provider response should be json");
    assert_eq!(payload["reachable"], true);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(
        requests
            .iter()
            .any(|request| request.method == "HEAD" && request.path == "/bucket-admin/"),
        "provider test route should call S3 HeadBucket"
    );
}

#[tokio::test]
async fn admin_storage_provider_test_route_checks_disabled_s3_provider_bucket() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, status, version, created_by, updated_by
        ) VALUES (
            'provider-test-disabled-s3', 's3_compatible', 'Disabled S3', $1, 'us-east-1',
            'bucket-admin', 1, 0, 'plain:test-access-key:test-secret-key',
            'disabled', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers/provider-test-disabled-s3/test")
                .body(Body::empty())
                .expect("test provider request should be built"),
        )
        .await
        .expect("test provider request should be handled");
    assert_eq!(response.status(), StatusCode::OK);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(
        requests
            .iter()
            .any(|request| request.method == "HEAD" && request.path == "/bucket-admin/"),
        "provider test route should call S3 HeadBucket for disabled providers"
    );
}

#[tokio::test]
async fn admin_storage_provider_and_binding_routes_emit_audit_events() {
    let (s3_endpoint, _captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, display_name,
            space_type, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-audit', 'tenant-audit', 'user', 'user-audit',
            'Audit Space', 'git_repository', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    let app = build_router_with_pool_without_iam_and_test_tenant(pool.clone(), "tenant-audit");
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"provider-audit-s3",
                        "providerKind":"s3_compatible",
                        "name":"Audit S3",
                        "endpointUrl":"__S3_ENDPOINT__",
                        "region":"us-east-1",
                        "bucket":"bucket-audit",
                        "pathStyle":true,
                        "credentialRef":"plain:test-access-key:test-secret-key"
                    }"#
                    .replace("__S3_ENDPOINT__", &s3_endpoint),
                ))
                .expect("create provider request should be built"),
        )
        .await
        .expect("create provider request should be handled");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/backend/v3/api/drive/storage/providers/provider-audit-s3")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "name":"Audit S3 Updated",
                        "status":"disabled"
                    }"#,
                ))
                .expect("update provider request should be built"),
        )
        .await
        .expect("update provider request should be handled");
    assert_eq!(update_response.status(), StatusCode::OK);

    let test_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers/provider-audit-s3/test")
                .body(Body::empty())
                .expect("test provider request should be built"),
        )
        .await
        .expect("test provider request should be handled");
    assert_eq!(test_response.status(), StatusCode::OK);

    let activate_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers/provider-audit-s3/activate")
                .body(Body::empty())
                .expect("activate provider request should be built"),
        )
        .await
        .expect("activate provider request should be handled");
    assert_eq!(activate_response.status(), StatusCode::OK);

    let rotate_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers/provider-audit-s3/credentials/rotate")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"credentialRef":"plain:rotated-access-key:rotated-secret-key"}"#,
                ))
                .expect("rotate provider credential request should be built"),
        )
        .await
        .expect("rotate provider credential request should be handled");
    assert_eq!(rotate_response.status(), StatusCode::OK);

    let binding_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/bindings/default")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "spaceId":"space-audit",
                        "providerId":"provider-audit-s3"
                    }"#,
                ))
                .expect("set default binding request should be built"),
        )
        .await
        .expect("set default binding request should be handled");
    assert_eq!(binding_response.status(), StatusCode::OK);

    let delete_binding_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/backend/v3/api/drive/storage/bindings/default?spaceId=space-audit")
                .body(Body::empty())
                .expect("delete default binding request should be built"),
        )
        .await
        .expect("delete default binding request should be handled");
    assert_no_content_response(delete_binding_response).await;

    let delete_response = app
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/backend/v3/api/drive/storage/providers/provider-audit-s3")
                .body(Body::empty())
                .expect("delete provider request should be built"),
        )
        .await
        .expect("delete provider request should be handled");
    assert_no_content_response(delete_response).await;

    let provider_actions: Vec<String> = sqlx::query_scalar(
        "SELECT action
         FROM dr_drive_audit_event
         WHERE resource_type='storage_provider'
           AND resource_id='provider-audit-s3'
         ORDER BY id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("provider audit events should be queryable");
    assert_eq!(
        provider_actions,
        vec![
            "drive.storage_provider.created",
            "drive.storage_provider.updated",
            "drive.storage_provider.tested",
            "drive.storage_provider.activated",
            "drive.storage_provider.credentials_rotated",
            "drive.storage_provider.deleted"
        ]
    );

    let binding_actions: Vec<String> = sqlx::query_scalar(
        "SELECT action
         FROM dr_drive_audit_event
         WHERE resource_type='storage_provider_binding'
           AND resource_id='space-audit'
         ORDER BY id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("binding audit events should be queryable");
    assert_eq!(
        binding_actions,
        vec![
            "drive.storage_provider_binding.default_set",
            "drive.storage_provider_binding.default_deleted"
        ]
    );
}

#[tokio::test]
async fn admin_storage_bucket_and_object_mutations_audit_authenticated_actor() {
    let (s3_endpoint, _captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, status, version, created_by, updated_by
        ) VALUES (
            'provider-mutation-s3', 's3_compatible', 'Mutation S3', $1, 'us-east-1',
            'bucket-admin', 1, 0, 'plain:test-access-key:test-secret-key',
            'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool.clone());
    let create_bucket = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-mutation-s3/bucket")
                .body(Body::empty())
                .expect("bucket create request should be built"),
        )
        .await
        .expect("bucket create request should be handled");
    assert_eq!(create_bucket.status(), StatusCode::OK);

    let delete_object = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/backend/v3/api/drive/storage/providers/provider-mutation-s3/objects/objects/file-a.bin")
                .body(Body::empty())
                .expect("object delete request should be built"),
        )
        .await
        .expect("object delete request should be handled");
    assert_no_content_response(delete_object).await;

    let forged_copy = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers/provider-mutation-s3/objects/copy")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sourceObjectKey":"objects/file-a.bin",
                        "destinationObjectKey":"objects/file-b.bin",
                        "operatorId":"forged-operator"
                    }"#,
                ))
                .expect("forged object copy request should be built"),
        )
        .await
        .expect("forged object copy request should be handled");
    assert_eq!(forged_copy.status(), StatusCode::BAD_REQUEST);

    let copy_object = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers/provider-mutation-s3/objects/copy")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sourceObjectKey":"objects/file-a.bin",
                        "destinationObjectKey":"objects/file-b.bin"
                    }"#,
                ))
                .expect("object copy request should be built"),
        )
        .await
        .expect("object copy request should be handled");
    assert_eq!(copy_object.status(), StatusCode::OK);

    let delete_bucket = app
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/backend/v3/api/drive/storage/providers/provider-mutation-s3/bucket")
                .body(Body::empty())
                .expect("bucket delete request should be built"),
        )
        .await
        .expect("bucket delete request should be handled");
    assert_no_content_response(delete_bucket).await;

    let audit_rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT action, operator_id
         FROM dr_drive_audit_event
         WHERE resource_type='storage_provider'
           AND resource_id='provider-mutation-s3'
         ORDER BY id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("mutation audit events should be queryable");
    assert_eq!(
        audit_rows,
        vec![
            (
                "drive.storage_provider.bucket_created".to_string(),
                "admin-storage".to_string()
            ),
            (
                "drive.storage_provider.object_deleted".to_string(),
                "admin-storage".to_string()
            ),
            (
                "drive.storage_provider.object_copied".to_string(),
                "admin-storage".to_string()
            ),
            (
                "drive.storage_provider.bucket_deleted".to_string(),
                "admin-storage".to_string()
            )
        ]
    );
}

#[tokio::test]
async fn admin_storage_legacy_admin_prefix_remains_compatible() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = build_router_with_pool_without_iam(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v3/api/drive/storage/providers")
                .body(Body::empty())
                .expect("legacy list providers request should be built"),
        )
        .await
        .expect("legacy list providers request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("legacy list providers response body should be read"),
    )
    .expect("legacy list providers response should be json");
    assert_eq!(payload["code"], 0);
    assert!(payload["data"]["items"].is_array());
}

#[tokio::test]
async fn admin_storage_provider_kinds_list_and_initialize() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = build_router_with_pool_without_iam(pool);
    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/provider-kinds")
                .body(Body::empty())
                .expect("list provider kinds request should be built"),
        )
        .await
        .expect("list provider kinds request should be handled");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("list provider kinds response body should be read"),
    )
    .expect("list provider kinds response should be json");
    assert_eq!(list_payload["code"], 0);
    let items = list_payload["data"]["items"]
        .as_array()
        .expect("provider kinds items should be an array");
    assert_eq!(items.len(), 7);
    assert!(items.iter().all(|item| item["enabled"] == true));
    assert!(items.iter().all(|item| item["configCount"] == 0));
    let kinds = items
        .iter()
        .map(|item| item["providerKind"].as_str().unwrap_or(""))
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"aliyun_oss"));
    assert!(kinds.contains(&"tencent_cos"));

    // Initialize is idempotent and preserves the catalog.
    let init_response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/provider-kinds/initialize")
                .body(Body::empty())
                .expect("initialize provider kinds request should be built"),
        )
        .await
        .expect("initialize provider kinds request should be handled");
    assert_eq!(init_response.status(), StatusCode::OK);
    let init_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(init_response.into_body(), usize::MAX)
            .await
            .expect("initialize provider kinds response body should be read"),
    )
    .expect("initialize provider kinds response should be json");
    assert_eq!(
        init_payload["data"]["items"]
            .as_array()
            .expect("initialized provider kinds items should be an array")
            .len(),
        7
    );
}

#[tokio::test]
async fn admin_storage_provider_kind_disable_blocks_new_configurations() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = build_router_with_pool_without_iam(pool);
    let disable_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/backend/v3/api/drive/storage/provider-kinds/aliyun_oss")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"enabled":false}"#))
                .expect("disable provider kind request should be built"),
        )
        .await
        .expect("disable provider kind request should be handled");
    assert_eq!(disable_response.status(), StatusCode::OK);
    let disable_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(disable_response.into_body(), usize::MAX)
            .await
            .expect("disable provider kind response body should be read"),
    )
    .expect("disable provider kind response should be json");
    assert_eq!(disable_payload["enabled"], false);
    assert_eq!(disable_payload["providerKind"], "aliyun_oss");

    // Creating a configuration for a disabled kind is rejected with 409.
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"provider-disabled-kind",
                        "providerKind":"aliyun_oss",
                        "name":"Disabled Kind OSS",
                        "endpointUrl":"https://oss-cn-hangzhou.aliyuncs.com",
                        "region":"cn-hangzhou",
                        "bucket":"drive-bucket",
                        "credentialRef":"plain:access-key:secret-key"
                    }"#,
                ))
                .expect("create provider request should be built"),
        )
        .await
        .expect("create provider request should be handled");
    assert_eq!(create_response.status(), StatusCode::CONFLICT);

    // Custom kinds remain usable while a built-in kind is disabled.
    let custom_create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"provider-custom-minio",
                        "providerKind":"custom:minio",
                        "name":"MinIO",
                        "endpointUrl":"https://minio.example.com",
                        "region":"us-east-1",
                        "bucket":"drive-bucket",
                        "credentialRef":"plain:access-key:secret-key"
                    }"#,
                ))
                .expect("create custom provider request should be built"),
        )
        .await
        .expect("create custom provider request should be handled");
    assert_eq!(custom_create_response.status(), StatusCode::CREATED);

    // Re-enabling the kind allows creating new configurations again.
    let enable_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/backend/v3/api/drive/storage/provider-kinds/aliyun_oss")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"enabled":true}"#))
                .expect("enable provider kind request should be built"),
        )
        .await
        .expect("enable provider kind request should be handled");
    assert_eq!(enable_response.status(), StatusCode::OK);
    let create_after_enable = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"provider-reenabled-kind",
                        "providerKind":"aliyun_oss",
                        "name":"Re-enabled OSS",
                        "endpointUrl":"https://oss-cn-hangzhou.aliyuncs.com",
                        "region":"cn-hangzhou",
                        "bucket":"drive-bucket",
                        "credentialRef":"plain:access-key:secret-key"
                    }"#,
                ))
                .expect("create provider request should be built"),
        )
        .await
        .expect("create provider request should be handled");
    assert_eq!(create_after_enable.status(), StatusCode::CREATED);
}
#[tokio::test]
async fn admin_storage_provider_kind_disable_blocks_reactivation() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = build_router_with_pool_without_iam(pool.clone());
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"provider-reactivation-guard",
                        "providerKind":"tencent_cos",
                        "name":"COS Guard",
                        "endpointUrl":"https://cos.ap-guangzhou.myqcloud.com",
                        "region":"ap-guangzhou",
                        "bucket":"drive-bucket",
                        "credentialRef":"plain:secret-id:secret-key"
                    }"#,
                ))
                .expect("create provider request should be built"),
        )
        .await
        .expect("create provider request should be handled");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let deactivate_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers/provider-reactivation-guard/deactivate")
                .body(Body::empty())
                .expect("deactivate provider request should be built"),
        )
        .await
        .expect("deactivate provider request should be handled");
    assert_eq!(deactivate_response.status(), StatusCode::OK);

    // Disable the provider kind while an existing configuration is disabled.
    let disable_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/backend/v3/api/drive/storage/provider-kinds/tencent_cos")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"enabled":false}"#))
                .expect("disable provider kind request should be built"),
        )
        .await
        .expect("disable provider kind request should be handled");
    assert_eq!(disable_response.status(), StatusCode::OK);

    // Reactivation of the existing configuration is rejected with 409.
    let activate_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers/provider-reactivation-guard/activate")
                .body(Body::empty())
                .expect("activate provider request should be built"),
        )
        .await
        .expect("activate provider request should be handled");
    assert_eq!(activate_response.status(), StatusCode::CONFLICT);

    // Re-enabling the kind allows the configuration to be activated again.
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/backend/v3/api/drive/storage/provider-kinds/tencent_cos")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"enabled":true}"#))
                .expect("enable provider kind request should be built"),
        )
        .await
        .expect("enable provider kind request should be handled");
    let activate_after_enable = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers/provider-reactivation-guard/activate")
                .body(Body::empty())
                .expect("activate provider request should be built"),
        )
        .await
        .expect("activate provider request should be handled");
    assert_eq!(activate_after_enable.status(), StatusCode::OK);
}

#[tokio::test]
async fn admin_storage_object_content_routes_handle_literal_percent_keys_and_directory_placeholders() {
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
            'provider-percent-keys', 's3_compatible', 'Percent Key S3', $1, 'us-east-1',
            'bucket-admin', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool);

    // 字面 % 字符的 key：客户端 encodeURIComponent 后 axum 解码一次，
    // 服务端不得二次解码（50%20off.txt 必须保持字面，不能变成 50 off.txt）。
    let put_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-percent-keys/object-contents/objects/50%2520off.txt")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"content":"literal percent"}"#))
                .expect("object content request should be built"),
        )
        .await
        .expect("object content request should be handled");
    assert_eq!(put_response.status(), StatusCode::OK);
    let put_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(put_response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response should be json");
    assert_eq!(
        put_payload["data"]["item"]["objectKey"],
        "objects/50%20off.txt",
        "literal percent signs must survive exactly one decode"
    );

    // 非法尾序列（100%.txt）必须保持字面并成功，而不是被二次解码报 400。
    let trailing_percent = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-percent-keys/object-contents/objects/100%25.txt")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"content":"x"}"#))
                .expect("object content request should be built"),
        )
        .await
        .expect("object content request should be handled");
    assert_eq!(trailing_percent.status(), StatusCode::OK);

    // 双斜杠 key 一律拒绝（避免幽灵空段对象）。
    let double_slash = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-percent-keys/object-contents/objects/folder/")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"content":""}"#))
                .expect("object content request should be built"),
        )
        .await
        .expect("object content request should be handled");
    assert_eq!(double_slash.status(), StatusCode::BAD_REQUEST);

    // 目录占位对象（docs/）可通过 DELETE 管理（尾斜杠 key 合法）。
    let put_dir = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-percent-keys/object-contents/objects/docs/")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"content":""}"#))
                .expect("object content request should be built"),
        )
        .await
        .expect("object content request should be handled");
    assert_eq!(put_dir.status(), StatusCode::OK);

    let delete_dir = app
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/backend/v3/api/drive/storage/providers/provider-percent-keys/objects/objects/docs/")
                .body(Body::empty())
                .expect("object delete request should be built"),
        )
        .await
        .expect("object delete request should be handled");
    assert_no_content_response(delete_dir).await;

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(
        requests.iter().any(|request| request.method == "PUT"
            && request.path == "/bucket-admin/objects/50%20off.txt"),
        "literal percent key should be stored verbatim"
    );
    assert!(
        requests.iter().any(|request| request.method == "PUT"
            && request.path == "/bucket-admin/objects/100%.txt"),
        "trailing percent key should be stored verbatim"
    );
    assert!(
        requests.iter().any(|request| request.method == "DELETE"
            && request.path == "/bucket-admin/objects/docs/"),
        "directory placeholder object should be deletable"
    );
}

#[tokio::test]
async fn admin_storage_object_content_routes_map_missing_and_empty_objects() {
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
            'provider-empty-missing', 's3_compatible', 'Empty Missing S3', $1, 'us-east-1',
            'bucket-admin', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool);

    // 空对象读取：head 返回 content-length 0，跳过 read，返回空 base64。
    let empty_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-empty-missing/object-contents/objects/empty.bin")
                .body(Body::empty())
                .expect("object content request should be built"),
        )
        .await
        .expect("object content request should be handled");
    assert_eq!(empty_response.status(), StatusCode::OK);
    let empty_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(empty_response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response should be json");
    assert_eq!(empty_payload["data"]["item"]["sizeBytes"], 0);
    assert_eq!(empty_payload["data"]["item"]["content"], "");

    // 不存在的对象读取 → 404（mock HEAD 对 missing.txt 返回 404）。
    let missing_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-empty-missing/object-contents/objects/missing.txt")
                .body(Body::empty())
                .expect("object content request should be built"),
        )
        .await
        .expect("object content request should be handled");
    assert_eq!(missing_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn admin_storage_object_content_routes_reject_oversized_reads() {
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
            'provider-oversized', 's3_compatible', 'Oversized S3', $1, 'us-east-1',
            'bucket-admin', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    // 对象超过 8 MiB 读取上限：head 报 8 MiB+1 → 413，不发起 GetObject。
    let response = build_router_with_pool_without_iam(pool)
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-oversized/object-contents/objects/oversized.bin")
                .body(Body::empty())
                .expect("object content request should be built"),
        )
        .await
        .expect("object content request should be handled");
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response should be json");
    assert_eq!(413, payload["status"].as_i64().unwrap_or_default());
    assert!(
        payload["detail"]
            .as_str()
            .is_some_and(|detail| detail.contains("8 MiB") || detail.contains("8388608")),
        "oversized read should name the limit: {payload}"
    );
}
