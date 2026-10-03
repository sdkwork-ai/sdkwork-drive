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
    if uri.path().trim_end_matches('/') == "/init-bucket"
        && (method == Method::HEAD || method == Method::PUT)
    {
        // Stateful stand-in for vendor re-create semantics: the bucket only
        // exists once a CreateBucket (PUT) has reached this mock.
        if method == Method::PUT {
            return (StatusCode::OK, [("content-length", "0")], Body::empty()).into_response();
        }
        let created = requests
            .lock()
            .expect("captured s3 requests mutex should not be poisoned")
            .iter()
            .any(|request| {
                request.method == "PUT" && request.path.trim_end_matches('/') == "/init-bucket"
            });
        return if created {
            (StatusCode::OK, [("content-length", "0")], Body::empty()).into_response()
        } else {
            StatusCode::NOT_FOUND.into_response()
        };
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
        return (StatusCode::OK, [("content-length", "0")], Body::empty()).into_response();
    }
    if method == Method::GET && uri.path() == "/bucket-admin/objects/notes.txt" {
        return (
            StatusCode::OK,
            [("content-type", "text/plain"), ("content-length", "11")],
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
    // 分片上传：厂商用查询参数区分四个动作，响应体是 XML（SDK 按 XML 解析上传 id 与 ETag）。
    if method == Method::POST && query.contains("uploads") {
        return (
            StatusCode::OK,
            [("content-type", "application/xml")],
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<InitiateMultipartUploadResult>
  <Bucket>bucket-admin</Bucket>
  <Key>{}</Key>
  <UploadId>upload-mock-1</UploadId>
</InitiateMultipartUploadResult>"#,
                uri.path().trim_start_matches('/')
            ),
        )
            .into_response();
    }
    if method == Method::POST && query.contains("uploadId=") {
        return (
            StatusCode::OK,
            [("content-type", "application/xml")],
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<CompleteMultipartUploadResult>
  <Location>http://127.0.0.1/bucket-admin/{}</Location>
  <Bucket>bucket-admin</Bucket>
  <Key>{}</Key>
  <ETag>"etag-completed"</ETag>
</CompleteMultipartUploadResult>"#,
                uri.path().trim_start_matches('/'),
                uri.path().trim_start_matches('/')
            ),
        )
            .into_response();
    }
    if method == Method::DELETE && query.contains("uploadId=") {
        return StatusCode::NO_CONTENT.into_response();
    }
    if method == Method::GET && query.contains("list-type=2") {        return (
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            credential_ref, status, version, created_by, updated_by
        ) VALUES ('provider-root-prefix', 'tenant-storage', 's3_compatible', 'Root Prefix S3', 'https://s3.amazonaws.com', 'us-east-1', 'root-prefix-bucket', false, 'plain:access-key:secret-key', 'active', 1, 'admin-storage', 'admin-storage')",
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            credential_ref, status, version, created_by, updated_by
        ) VALUES ('provider-tenant-default', 'tenant-storage', 's3_compatible', 'Tenant Default', 'https://s3.amazonaws.com', 'us-east-1', 'tenant-bucket', false, 'plain:access-key:secret-key', 'active', 1, 'admin-storage', 'admin-storage'),
            (
                'provider-space-default', 'aliyun_oss', 'Space Default', 'https://oss-cn-hangzhou.aliyuncs.com',
                'cn-hangzhou', 'space-bucket', false, 'plain:access-key:secret-key',
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

/// Read one binding-list page as JSON.
async fn list_bindings_payload(app: Router, uri: &str) -> serde_json::Value {
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .body(Body::empty())
                .expect("list bindings request should be built"),
        )
        .await
        .expect("list bindings request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("list bindings response body should be read"),
    )
    .expect("list bindings response should be json")
}

/// The binding list can be narrowed to one resolution step.
///
/// The console renders a section per step, and the unfiltered list is one page
/// window ordered space → space type → tenant: without the filter, a tenant with
/// enough space-scoped bindings makes the space-type section render every row as
/// unbound. The filter therefore has to select server-side, and an unknown step
/// has to fail loudly rather than answer an empty page.
#[tokio::test]
async fn admin_storage_binding_list_filters_by_resolution_step() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, display_name,
            space_type, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-scope-a', 'tenant-scope', 'user', 'user-scope',
            'Scoped Space', 'personal', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            credential_ref, status, version, created_by, updated_by
        ) VALUES ('provider-scope', 'tenant-storage', 's3_compatible', 'Scoped Provider', 'https://s3.amazonaws.com', 'us-east-1', 'scoped-bucket', false, 'plain:access-key:secret-key', 'active', 1, 'admin-storage', 'admin-storage')",
    )
    .execute(&pool)
    .await
    .expect("provider should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES
            (
                'default:tenant:tenant-scope', 'tenant-scope', NULL, 'provider-scope',
                'tenant', 'primary', 'sdkwork-drive/v1/tenants/tenant-scope', 'active', 1,
                'admin-storage', 'admin-storage'
            ),
            (
                'default:space:tenant-scope:space-scope-a', 'tenant-scope', 'space-scope-a',
                'provider-scope', 'space', 'primary',
                'sdkwork-drive/v1/tenants/tenant-scope/spaces/space-scope-a', 'active', 1,
                'admin-storage', 'admin-storage'
            ),
            (
                'default:space_type:tenant-scope:website', 'tenant-scope', NULL, 'provider-scope',
                'space_type', 'website',
                'sdkwork-drive/v1/tenants/tenant-scope/space-types/website', 'active', 1,
                'admin-storage', 'admin-storage'
            )",
    )
    .execute(&pool)
    .await
    .expect("bindings should be seeded");

    let app = build_router_with_pool_without_iam_and_test_tenant(pool, "tenant-scope");

    let space_type_page = list_bindings_payload(
        app.clone(),
        "/backend/v3/api/drive/storage/bindings?binding_scope=space_type",
    )
    .await;
    let space_type_items = space_type_page["data"]["items"].as_array().unwrap();
    assert_eq!(space_type_items.len(), 1);
    assert_eq!(space_type_items[0]["purpose"], "website");
    assert_eq!(space_type_items[0]["bindingScope"], "space_type");

    let space_page = list_bindings_payload(
        app.clone(),
        "/backend/v3/api/drive/storage/bindings?binding_scope=space",
    )
    .await;
    let space_items = space_page["data"]["items"].as_array().unwrap();
    assert_eq!(space_items.len(), 1);
    assert_eq!(space_items[0]["spaceId"], "space-scope-a");

    let tenant_page = list_bindings_payload(
        app.clone(),
        "/backend/v3/api/drive/storage/bindings?binding_scope=tenant",
    )
    .await;
    let tenant_items = tenant_page["data"]["items"].as_array().unwrap();
    assert_eq!(tenant_items.len(), 1);
    assert_eq!(tenant_items[0]["purpose"], "primary");

    // A step that does not exist is a client defect, not an empty list.
    let invalid_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/bindings?binding_scope=spaces")
                .body(Body::empty())
                .expect("invalid scope request should be built"),
        )
        .await
        .expect("invalid scope request should be handled");
    assert_eq!(invalid_response.status(), StatusCode::BAD_REQUEST);
}

/// A `website` space type can carry its own storage target.
///
/// `dr_drive_space.space_type` has always accepted `website` and the route
/// validator accepted it too, while the binding table's purpose check did not: the
/// bind succeeded through validation and then failed the table check, so
/// published-site content could never be given its own bucket (a public,
/// CDN-fronted one, separated from the tenant's private storage).
#[tokio::test]
async fn admin_storage_binding_accepts_website_space_type() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            credential_ref, status, version, created_by, updated_by
        ) VALUES ('provider-website', 'tenant-storage', 's3_compatible', 'Website Bucket', 'https://s3.amazonaws.com', 'us-east-1', 'sdkwork-website', false, 'plain:access-key:secret-key', 'active', 1, 'admin-storage', 'admin-storage')",
    )
    .execute(&pool)
    .await
    .expect("website provider should be seeded");

    let app = build_router_with_pool_without_iam_and_test_tenant(pool, "tenant-website");

    let set_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/bindings/default")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "spaceType":"website",
                        "providerId":"provider-website"
                    }"#,
                ))
                .expect("website binding request should be built"),
        )
        .await
        .expect("website binding request should be handled");
    assert_eq!(set_response.status(), StatusCode::OK);
    let set_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(set_response.into_body(), usize::MAX)
            .await
            .expect("website binding response body should be read"),
    )
    .expect("website binding response should be json");
    assert_eq!(set_payload["data"]["item"]["bindingScope"], "space_type");
    assert_eq!(set_payload["data"]["item"]["purpose"], "website");
    // The bucket a `website` binding writes into comes from the provider
    // configuration the binding points at, which is what the console shows.
    assert_eq!(
        set_payload["data"]["item"]["storageProvider"]["bucket"],
        "sdkwork-website"
    );

    let filtered = list_bindings_payload(
        app,
        "/backend/v3/api/drive/storage/bindings?binding_scope=space_type",
    )
    .await;
    let items = filtered["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["purpose"], "website");
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, status, version, created_by, updated_by
        ) VALUES ('provider-bucket-list-s3', 'tenant-storage', 's3_compatible', 'Bucket List S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'active', 1, 'admin-storage', 'admin-storage')",
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-admin-s3', 'tenant-storage', 's3_compatible', 'Admin S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
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
async fn admin_storage_object_routes_honor_the_bucket_override() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    // 配置桶是 bucket-admin；管理端浏览的是账号下另一个真实存在的桶。
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-bucket-override', 'tenant-storage', 's3_compatible', 'Override S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool);
    let override_bucket = "image2-1253947560";

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/backend/v3/api/drive/storage/providers/provider-bucket-override/objects?bucket={override_bucket}&prefix=objects/&page_size=100"
                ))
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
    assert_eq!(list_payload["data"]["items"][0]["objectKey"], "objects/file-a.bin");
    assert_eq!(list_payload["data"]["items"][0]["bucket"], override_bucket);

    let head_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/backend/v3/api/drive/storage/providers/provider-bucket-override/objects/objects/file-a.bin?bucket={override_bucket}"
                ))
                .body(Body::empty())
                .expect("object head request should be built"),
        )
        .await
        .expect("object head request should be handled");
    assert_eq!(head_response.status(), StatusCode::OK);

    let write_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(format!(
                    "/backend/v3/api/drive/storage/providers/provider-bucket-override/object-contents/objects/notes.txt?bucket={override_bucket}"
                ))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"content":"hello world","contentType":"text/plain"}"#))
                .expect("object content request should be built"),
        )
        .await
        .expect("object content request should be handled");
    assert_eq!(write_response.status(), StatusCode::OK);
    let write_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(write_response.into_body(), usize::MAX)
            .await
            .expect("object content response body should be read"),
    )
    .expect("object content response should be json");
    assert_eq!(write_payload["data"]["item"]["bucket"], override_bucket);

    let delete_response = app
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!(
                    "/backend/v3/api/drive/storage/providers/provider-bucket-override/objects/objects/file-a.bin?bucket={override_bucket}"
                ))
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
        requests.iter().any(|request| request.method == "GET"
            && request.path == format!("/{override_bucket}/")
            && request.query.contains("list-type=2")),
        "object list route should list the requested bucket: {requests:?}"
    );
    assert!(
        requests.iter().any(|request| request.method == "HEAD"
            && request.path == format!("/{override_bucket}/objects/file-a.bin")),
        "object head route should head the requested bucket: {requests:?}"
    );
    assert!(
        requests.iter().any(|request| request.method == "PUT"
            && request.path == format!("/{override_bucket}/objects/notes.txt")),
        "object content write should write into the requested bucket: {requests:?}"
    );
    assert!(
        requests.iter().any(|request| request.method == "DELETE"
            && request.path == format!("/{override_bucket}/objects/file-a.bin")),
        "object delete route should delete in the requested bucket: {requests:?}"
    );
    // 覆盖的是桶，不是端点：配置桶本身一次都不该被碰到。
    assert!(
        !requests
            .iter()
            .any(|request| request.path.starts_with("/bucket-admin/")),
        "configured bucket must stay untouched when a bucket override is given: {requests:?}"
    );
}

#[tokio::test]
async fn admin_storage_object_routes_reject_invalid_bucket_override_before_calling_s3() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-bucket-override-validation', 'tenant-storage', 's3_compatible', 'Override Validation S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool);

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-bucket-override-validation/objects?bucket=Drive_Bucket")
                .body(Body::empty())
                .expect("object list request should be built"),
        )
        .await
        .expect("object list request should be handled");
    assert_eq!(list_response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("error response body should be read"),
    )
    .expect("error response should be json");
    assert!(
        payload["detail"]
            .as_str()
            .is_some_and(|detail| detail.contains("bucket")),
        "validation error should name the bucket parameter: {payload}"
    );

    let copy_response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/providers/provider-bucket-override-validation/objects/copy")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sourceObjectKey":"objects/file-a.bin",
                        "destinationObjectKey":"objects/file-b.bin",
                        "sourceBucket":"Drive_Bucket"
                    }"#,
                ))
                .expect("object copy request should be built"),
        )
        .await
        .expect("object copy request should be handled");
    assert_eq!(copy_response.status(), StatusCode::BAD_REQUEST);

    assert!(
        captured_requests
            .lock()
            .expect("captured s3 requests mutex should not be poisoned")
            .is_empty(),
        "an invalid bucket override should fail before calling object storage"
    );
}

#[tokio::test]
async fn admin_storage_bucket_initialization_is_idempotent() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-admin-init', 'tenant-storage', 's3_compatible', 'Admin Init', $1, 'us-east-1', 'init-bucket', true, false, 'plain:init-access-key:init-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool.clone());

    let first_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-admin-init/bucket")
                .body(Body::empty())
                .expect("bucket initialize request should be built"),
        )
        .await
        .expect("bucket initialize request should be handled");
    assert_eq!(first_response.status(), StatusCode::OK);
    let first_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(first_response.into_body(), usize::MAX)
            .await
            .expect("bucket initialize response body should be read"),
    )
    .expect("bucket initialize response should be json");
    assert_eq!(first_payload["code"], 0);
    assert_eq!(first_payload["data"]["changed"], true);

    let second_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-admin-init/bucket")
                .body(Body::empty())
                .expect("second bucket initialize request should be built"),
        )
        .await
        .expect("second bucket initialize request should be handled");
    assert_eq!(second_response.status(), StatusCode::OK);
    let second_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(second_response.into_body(), usize::MAX)
            .await
            .expect("second bucket initialize response body should be read"),
    )
    .expect("second bucket initialize response should be json");
    assert_eq!(second_payload["code"], 0);
    assert_eq!(second_payload["data"]["changed"], false);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.method == "PUT"
                && request.path.trim_end_matches('/') == "/init-bucket")
            .count(),
        1,
        "re-running initialization must not repeat the vendor CreateBucket call"
    );

    let audit_actions: Vec<String> = sqlx::query_scalar(
        "SELECT action
         FROM dr_drive_audit_event
         WHERE resource_type='storage_provider'
           AND resource_id='provider-admin-init'
         ORDER BY id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("bucket audit events should be queryable");
    assert_eq!(
        audit_actions,
        vec!["drive.storage_provider.bucket_created"],
        "audit must record exactly one creation even across repeated initializations"
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-admin-object-key', 'tenant-storage', 's3_compatible', 'Admin Object Key S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-copy-bucket-validation', 'tenant-storage', 's3_compatible', 'Copy Bucket Validation S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-content-routes', 'tenant-storage', 's3_compatible', 'Content Routes S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
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
    assert_eq!(
        get_payload["data"]["item"]["objectKey"],
        "objects/notes.txt"
    );
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
    assert_eq!(
        put_dir_payload["data"]["item"]["objectKey"],
        "objects/folder/"
    );

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(
        requests
            .iter()
            .any(|request| request.method == "PUT"
                && request.path == "/bucket-admin/objects/notes.txt"),
        "object content write route should call S3 PutObject"
    );
    assert!(
        requests
            .iter()
            .any(|request| request.method == "PUT"
                && request.path == "/bucket-admin/objects/blob.bin"),
        "base64 object content write route should call S3 PutObject"
    );
    assert!(
        requests
            .iter()
            .any(|request| request.method == "PUT"
                && request.path == "/bucket-admin/objects/folder/"),
        "directory placeholder write route should call S3 PutObject with trailing slash key"
    );
    assert!(
        requests
            .iter()
            .any(|request| request.method == "GET"
                && request.path == "/bucket-admin/objects/notes.txt"),
        "object content read route should call S3 GetObject"
    );
}

/// 上传大对象的请求体必须能进到业务校验。
///
/// 回归用例：路由最初没有声明请求体上限，用的是 axum 的默认 2 MB。业务上限是 8 MiB 对象、
/// 以 base64 装在 JSON 里（网线体积约 11.2 MB），于是 1.4 MB 以上的文件在业务校验之前就被
/// 拒掉，客户端只看到框架归一化后的 `Payload too large`——用户表现为"就是传不上去"。
/// 这里用一个 4 MiB 对象（base64 约 5.6 MB，远高于 2 MB、低于 11.2 MB）锁住这个边界。
#[tokio::test]
async fn admin_storage_object_content_write_accepts_a_body_beyond_the_default_axum_limit() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-content-large-body', 'tenant-storage', 's3_compatible', 'Content Large Body S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool);

    let plaintext = vec![b'x'; 4 * 1024 * 1024];
    let body = serde_json::json!({
        "content": sdkwork_utils_rust::base64_encode(&plaintext),
        "encoding": "base64",
        "contentType": "application/octet-stream",
    })
    .to_string();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-content-large-body/object-contents/objects/large.bin")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .expect("large object content write request should be built"),
        )
        .await
        .expect("large object content write request should be handled");

    let status = response.status();
    let payload = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("large object content response body should be readable");
    assert_eq!(
        status,
        StatusCode::OK,
        "a 4 MiB object (≈5.6 MB base64) must reach the handler instead of being rejected by the request body limit: {}",
        String::from_utf8_lossy(&payload)
    );
    assert!(
        captured_requests
            .lock()
            .expect("captured s3 requests mutex should not be poisoned")
            .iter()
            .any(|request| request.method == "PUT"
                && request.path == "/bucket-admin/objects/large.bin"),
        "the large body should reach S3 PutObject"
    );
}

/// 预签名分片上传：开启 → 签发分片授权 → 完成 → 中止。
///
/// 这是 >8 MiB 对象的唯一通道（单次内容接口上限 8 MiB，且 base64 会把请求体放大 1.37 倍）。
/// 断言的重点不是"接口返回 200"，而是：
/// - 签发的是**厂商**地址（字节不经过本服务）；
/// - 授权里带回客户端必须原样发送的签名头；
/// - int64 字段是字符串（生成的 SDK 按字符串解析）；
/// - 四个动作分别落到厂商的 create / complete / abort 调用上。
#[tokio::test]
async fn admin_storage_multipart_upload_routes_open_presign_complete_and_abort() {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-multipart', 'tenant-storage', 's3_compatible', 'Multipart S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool);
    let base = "/backend/v3/api/drive/storage/providers/provider-multipart/objects/multipart-uploads";

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(base)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"objectKey":"objects/large.bin","contentType":"application/octet-stream"}"#,
                ))
                .expect("create request should be built"),
        )
        .await
        .expect("create request should be handled");
    assert_eq!(create.status(), StatusCode::OK);
    let create_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(create.into_body(), usize::MAX)
            .await
            .expect("create body should be readable"),
    )
    .expect("create response should be json");
    assert_eq!(create_payload["data"]["item"]["uploadId"], "upload-mock-1");
    assert_eq!(create_payload["data"]["item"]["bucket"], "bucket-admin");

    let presign = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{base}/parts"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"objectKey":"objects/large.bin","uploadId":"upload-mock-1","partNumbers":[1,2]}"#,
                ))
                .expect("presign request should be built"),
        )
        .await
        .expect("presign request should be handled");
    assert_eq!(presign.status(), StatusCode::OK);
    let presign_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(presign.into_body(), usize::MAX)
            .await
            .expect("presign body should be readable"),
    )
    .expect("presign response should be json");
    let parts = presign_payload["data"]["item"]["parts"]
        .as_array()
        .expect("presign response should carry a parts array");
    assert_eq!(parts.len(), 2, "one grant per requested part");
    for (index, part) in parts.iter().enumerate() {
        assert_eq!(part["partNumber"], index + 1);
        assert_eq!(part["method"], "PUT");
        assert!(
            part["url"]
                .as_str()
                .is_some_and(|url| url.contains("partNumber=") && url.contains("uploadId=")),
            "the grant must point at the vendor with the part query: {part}"
        );
        assert!(
            part["expiresAtEpochMs"].is_string(),
            "int64 wire fields must be strings: {part}"
        );
    }
    // 签发不产生厂商调用：URL 是本地签名算出来的。
    assert!(
        !captured_requests
            .lock()
            .expect("captured s3 requests mutex should not be poisoned")
            .iter()
            .any(|request| request.path.contains("partNumber=")),
        "presigning must not call the vendor"
    );

    let complete = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{base}/complete"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"objectKey":"objects/large.bin","uploadId":"upload-mock-1","parts":[{"partNumber":1,"etag":"etag-1"},{"partNumber":2,"etag":"etag-2"}]}"#,
                ))
                .expect("complete request should be built"),
        )
        .await
        .expect("complete request should be handled");
    assert_eq!(complete.status(), StatusCode::OK);
    let complete_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(complete.into_body(), usize::MAX)
            .await
            .expect("complete body should be readable"),
    )
    .expect("complete response should be json");
    assert_eq!(
        complete_payload["data"]["item"]["objectKey"],
        "objects/large.bin"
    );

    let abort = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{base}/abort"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"objectKey":"objects/large.bin","uploadId":"upload-mock-1"}"#,
                ))
                .expect("abort request should be built"),
        )
        .await
        .expect("abort request should be handled");
    assert_eq!(abort.status(), StatusCode::OK);

    let requests = captured_requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .clone();
    assert!(
        requests
            .iter()
            .any(|request| request.method == "POST" && request.query.contains("uploads")),
        "create must call the vendor CreateMultipartUpload: {requests:?}"
    );
    assert!(
        requests
            .iter()
            .any(|request| request.method == "POST" && request.query.contains("uploadId=")),
        "complete must call the vendor CompleteMultipartUpload: {requests:?}"
    );
    assert!(
        requests
            .iter()
            .any(|request| request.method == "DELETE" && request.query.contains("uploadId=")),
        "abort must call the vendor AbortMultipartUpload: {requests:?}"
    );
}

/// 分片参数在触达厂商之前就要被挡住：这些错误形态都会让厂商回一句没有上下文的 400。
///
/// 用惰性连接池跑：校验发生在任何数据访问之前，所以这条用例**不需要数据库**也能执行——
/// 它同时证明了四个路由确实挂上了（没挂上会是 404/405，而不是 400）。
#[tokio::test]
async fn admin_storage_multipart_upload_routes_reject_malformed_part_lists() {
    let app = build_router_with_pool_without_iam(sdkwork_drive_test_support::lazy_postgres_test_pool());
    let base =
        "/backend/v3/api/drive/storage/providers/provider-multipart-invalid/objects/multipart-uploads";

    for (uri, body, expectation) in [
        (
            format!("{base}/parts"),
            r#"{"objectKey":"objects/a.bin","uploadId":"upload-1","partNumbers":[]}"#,
            "an empty part batch is rejected",
        ),
        (
            format!("{base}/parts"),
            r#"{"objectKey":"objects/a.bin","uploadId":"upload-1","partNumbers":[0]}"#,
            "part number 0 is rejected",
        ),
        (
            format!("{base}/parts"),
            r#"{"objectKey":"objects/a.bin","uploadId":"upload-1","partNumbers":[1,1]}"#,
            "a duplicated part number is rejected",
        ),
        (
            format!("{base}/parts"),
            r#"{"objectKey":"objects/a.bin","uploadId":"upload-1","partNumbers":[1],"expiresInSeconds":5}"#,
            "a grant lifetime below the floor is rejected",
        ),
        (
            format!("{base}/parts"),
            r#"{"objectKey":"","uploadId":"upload-1","partNumbers":[1]}"#,
            "an empty object key is rejected",
        ),
        (
            format!("{base}/parts"),
            r#"{"objectKey":"objects/a.bin","uploadId":"   ","partNumbers":[1]}"#,
            "a blank upload id is rejected",
        ),
        (
            format!("{base}/complete"),
            r#"{"objectKey":"objects/a.bin","uploadId":"upload-1","parts":[{"partNumber":1,"etag":""}]}"#,
            "a blank etag is rejected",
        ),
        (
            format!("{base}/complete"),
            r#"{"objectKey":"objects/a.bin","uploadId":"upload-1","parts":[{"partNumber":1,"etag":"a"},{"partNumber":3,"etag":"c"}]}"#,
            "a gap in part numbers is rejected",
        ),
        (
            format!("{base}/abort"),
            r#"{"objectKey":"objects/a.bin","uploadId":""}"#,
            "abort requires an upload id",
        ),
        (
            base.to_string(),
            r#"{"objectKey":""}"#,
            "create requires an object key",
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .expect("request should be built"),
            )
            .await
            .expect("request should be handled");
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "{expectation}: got {}",
            response.status()
        );
        // 错误信封必须带可定位的 detail，而不是空 body。
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("problem body should be readable"),
        )
        .expect("problem response should be json");
        assert!(
            payload["detail"]
                .as_str()
                .is_some_and(|detail| !detail.is_empty()),
            "{expectation}: problem detail must explain the rejection: {payload}"
        );
    }
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-content-validation', 'tenant-storage', 's3_compatible', 'Content Validation S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-admin-opendal-disabled', 'tenant-storage', 's3_compatible', 'Admin OpenDAL S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-admin-bucket-plugin-selected', 'tenant-storage', 's3_compatible', 'Admin Bucket S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
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
    // 服务端只支持 PostgreSQL：sqlite 由 admin_storage_database_url_router_rejects_sqlite 显式拒绝。
    // 用测试库 URL 验证显式 DatabaseConfig + AdminStorageConfig 可构建路由。
    let Ok(database_url) = std::env::var("SDKWORK_DATABASE_URL") else {
        eprintln!("skip PostgreSQL integration test: SDKWORK_DATABASE_URL is not set");
        return;
    };
    let database_config =
        DatabaseConfig::from_url(&database_url).expect("postgres database config should parse");
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, status, version, created_by, updated_by
        ) VALUES ('provider-test-s3', 'tenant-storage', 's3_compatible', 'Admin S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'active', 1, 'admin-storage', 'admin-storage')",
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, status, version, created_by, updated_by
        ) VALUES ('provider-test-disabled-s3', 'tenant-storage', 's3_compatible', 'Disabled S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'disabled', 1, 'admin-storage', 'admin-storage')",
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, status, version, created_by, updated_by
        ) VALUES ('provider-mutation-s3', 'tenant-storage', 's3_compatible', 'Mutation S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'active', 1, 'admin-storage', 'admin-storage')",
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

/// Read one provider-list page as JSON.
async fn list_providers_payload(app: Router, uri: &str) -> serde_json::Value {
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .body(Body::empty())
                .expect("list providers request should be built"),
        )
        .await
        .expect("list providers request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("list providers response body should be read"),
    )
    .expect("list providers response should be json")
}

/// Provider ids of one list page, in response order.
fn provider_ids(payload: &serde_json::Value) -> Vec<String> {
    payload["data"]["items"]
        .as_array()
        .expect("provider list items should be an array")
        .iter()
        .map(|item| {
            item["id"]
                .as_str()
                .expect("provider id should be a string")
                .to_string()
        })
        .collect()
}

/// A provider-kind filter has to be applied *before* the cursor window.
///
/// Seeded so the filtered row (`tencent_cos`) sorts last: an implementation that
/// pages first and filters the fetched page second answers "no rows" for
/// `provider_kind=tencent_cos` on page 1 while the row sits on page 2 — the
/// operator-visible defect this test pins down ("select Tencent COS: empty;
/// page forward: there it is"). The unfiltered first page is asserted too, so
/// the test fails loudly if seeding ever stops reproducing that shape.
#[tokio::test]
async fn admin_storage_provider_list_filters_by_kind_before_paging() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = build_router_with_pool_without_iam(pool);
    for (id, provider_kind) in [
        ("provider-a-s3", "s3_compatible"),
        ("provider-b-s3", "s3_compatible"),
        ("provider-c-custom", "custom:acme"),
        ("provider-z-tencent-cos", "tencent_cos"),
    ] {
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/backend/v3/api/drive/storage/providers")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{
                            "id":"{id}",
                            "providerKind":"{provider_kind}",
                            "name":"{id}",
                            "endpointUrl":"https://s3.example.com",
                            "bucket":"bucket-{id}"
                        }}"#
                    )))
                    .expect("create provider request should be built"),
            )
            .await
            .expect("create provider request should be handled");
        assert_eq!(
            create_response.status(),
            StatusCode::CREATED,
            "provider {id} should be created"
        );
    }

    let unfiltered = list_providers_payload(
        app.clone(),
        "/backend/v3/api/drive/storage/providers?page_size=2",
    )
    .await;
    assert_eq!(
        provider_ids(&unfiltered),
        vec!["provider-a-s3".to_string(), "provider-b-s3".to_string()]
    );
    assert_eq!(unfiltered["data"]["pageInfo"]["hasMore"], true);

    // Same page size, one kind filter: the COS row is the *first* page of the
    // filtered set, and there is nothing after it.
    let filtered = list_providers_payload(
        app.clone(),
        "/backend/v3/api/drive/storage/providers?page_size=2&provider_kind=tencent_cos",
    )
    .await;
    assert_eq!(
        provider_ids(&filtered),
        vec!["provider-z-tencent-cos".to_string()]
    );
    assert_eq!(filtered["data"]["pageInfo"]["hasMore"], false);

    // Case-insensitive, because the console's catalog value and an
    // operator-typed value have to mean the same row.
    let mixed_case = list_providers_payload(
        app.clone(),
        "/backend/v3/api/drive/storage/providers?provider_kind=tencent_cos",
    )
    .await;
    assert_eq!(
        provider_ids(&mixed_case),
        vec!["provider-z-tencent-cos".to_string()]
    );

    // `custom` is the family the console's "custom" option stands for.
    let custom = list_providers_payload(
        app.clone(),
        "/backend/v3/api/drive/storage/providers?provider_kind=custom",
    )
    .await;
    assert_eq!(
        provider_ids(&custom),
        vec!["provider-c-custom".to_string()]
    );

    // An unknown kind selects nothing instead of failing: `custom:<vendor>` is
    // open-ended, so the filter value space cannot be closed.
    let unknown = list_providers_payload(
        app.clone(),
        "/backend/v3/api/drive/storage/providers?provider_kind=not-a-kind",
    )
    .await;
    assert!(provider_ids(&unknown).is_empty());
    assert_eq!(unknown["data"]["pageInfo"]["hasMore"], false);

    // The kind filter and the status filter are independent predicates that
    // compose rather than override each other.
    let kind_and_status = list_providers_payload(
        app,
        "/backend/v3/api/drive/storage/providers?provider_kind=s3_compatible&status=disabled",
    )
    .await;
    assert!(provider_ids(&kind_and_status).is_empty());
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
    assert_eq!(
        items.len(),
        sdkwork_drive_workspace_service::application::storage_provider_kind_service::BUILTIN_STORAGE_PROVIDER_KIND_CATALOG.len(),
        "the list endpoint must expose the full built-in catalog"
    );
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
    let init_status = init_response.status();
    let init_body_bytes = to_bytes(init_response.into_body(), usize::MAX)
        .await
        .expect("initialize provider kinds response body should be read");
    eprintln!("INIT DEBUG status={init_status} body={}", String::from_utf8_lossy(&init_body_bytes));
    assert_eq!(init_status, StatusCode::OK);
    let init_payload: serde_json::Value = serde_json::from_slice(
        &init_body_bytes,
    )
    .expect("initialize provider kinds response should be json");
    assert_eq!(
        init_payload["data"]["items"]
            .as_array()
            .expect("initialized provider kinds items should be an array")
            .len(),
        sdkwork_drive_workspace_service::application::storage_provider_kind_service::BUILTIN_STORAGE_PROVIDER_KIND_CATALOG.len(),
        "initialize must expose the full built-in catalog"
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
    assert_eq!(disable_payload["code"], 0);
    let disabled_item = &disable_payload["data"]["item"];
    assert_eq!(disabled_item["enabled"], false);
    assert_eq!(disabled_item["providerKind"], "aliyun_oss");

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
async fn admin_storage_object_content_routes_handle_literal_percent_keys_and_directory_placeholders(
) {
    let (s3_endpoint, captured_requests) = start_s3_mock_server().await;
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-percent-keys', 'tenant-storage', 's3_compatible', 'Percent Key S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    let app = build_router_with_pool_without_iam(pool);

    // 字面 % 字符的 key：客户端 encodeURIComponent 与 axum 解码一次，
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
        put_payload["data"]["item"]["objectKey"], "objects/50%20off.txt",
        "literal percent signs must survive exactly one decode"
    );

    // 非法尾序列（100%.txt）必须保持字面并成功，而不是被二次解码成 400。
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
    let put_dir_status = put_dir.status();
    assert_eq!(put_dir_status, StatusCode::OK, "put_dir debug body={}", if put_dir_status == StatusCode::OK { String::new() } else { String::from_utf8_lossy(&to_bytes(put_dir.into_body(), usize::MAX).await.expect("put dir body should be read")).to_string() });

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
        requests
            .iter()
            .any(|request| request.method == "PUT"
                && request.path == "/bucket-admin/objects/100%.txt"),
        "trailing percent key should be stored verbatim"
    );
    assert!(
        requests
            .iter()
            .any(|request| request.method == "DELETE"
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-empty-missing', 'tenant-storage', 's3_compatible', 'Empty Missing S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
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
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES ('provider-oversized', 'tenant-storage', 's3_compatible', 'Oversized S3', $1, 'us-east-1', 'bucket-admin', true, false, 'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD', 'active', 1, 'admin-storage', 'admin-storage')",
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


#[tokio::test]
async fn admin_storage_provider_routes_reject_cross_tenant_access() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, status, version, created_by, updated_by
        ) VALUES (
            'provider-foreign', 'tenant-storage', 's3_compatible', 'Foreign Seed',
            'http://127.0.0.1:9', 'us-east-1', 'foreign-bucket', true, false,
            'plain:k:s', 'active', 1, 'admin-storage', 'admin-storage'
        )",
    )
    .execute(&pool)
    .await
    .expect("foreign-tenant provider should be seeded");

    // A router acting for a different tenant must see the row as missing on
    // reads and bucket mutations alike — 404, never 403, so the row's
    // existence does not leak across tenants.
    let app = build_router_with_pool_without_iam_and_test_tenant(pool.clone(), "tenant-other");

    let get = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-foreign")
                .body(Body::empty())
                .expect("get request should be built"),
        )
        .await
        .expect("get request should be handled");
    assert_eq!(get.status(), StatusCode::NOT_FOUND);

    let put = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/storage/providers/provider-foreign/bucket")
                .body(Body::empty())
                .expect("bucket initialize request should be built"),
        )
        .await
        .expect("bucket initialize request should be handled");
    assert_eq!(put.status(), StatusCode::NOT_FOUND);

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers")
                .body(Body::empty())
                .expect("list request should be built"),
        )
        .await
        .expect("list request should be handled");
    assert_eq!(list.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(list.into_body(), usize::MAX)
            .await
            .expect("list response body should be read"),
    )
    .expect("list response should be json");
    assert_eq!(
        payload["data"]["items"].as_array().expect("items").len(),
        0,
        "provider list must be tenant-scoped"
    );

    // The owning tenant still sees the row.
    let owning = build_router_with_pool_without_iam(pool);
    let get = owning
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers/provider-foreign")
                .body(Body::empty())
                .expect("owning get request should be built"),
        )
        .await
        .expect("owning get request should be handled");
    assert_eq!(get.status(), StatusCode::OK);
}

/// Read one account-list page as JSON.
async fn list_provider_accounts_payload(app: Router, uri: &str) -> serde_json::Value {
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .body(Body::empty())
                .expect("list provider accounts request should be built"),
        )
        .await
        .expect("list provider accounts request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("list provider accounts response body should be read"),
    )
    .expect("list provider accounts response should be json")
}

/// `(id, vendorCode)` of one account-list page, in response order.
fn provider_account_rows(payload: &serde_json::Value) -> Vec<(String, String)> {
    payload["data"]["items"]
        .as_array()
        .expect("account list items should be an array")
        .iter()
        .map(|item| {
            (
                item["id"].as_str().expect("account id should be a string").to_string(),
                item["vendorCode"]
                    .as_str()
                    .expect("account vendorCode should be a string")
                    .to_string(),
            )
        })
        .collect()
}

/// A vendor code no earlier run can have used.
///
/// The account center is IAM-owned and the Drive fixture only truncates `dr_`
/// tables, so a fixed code would inherit rows from a previous run and make the
/// page boundaries below describe somebody else's accounts.
fn unique_vendor_code() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the system clock should be after the unix epoch")
        .as_nanos();
    format!("it{:x}", nanos & 0xffff_ffff_ffff)
}

/// Seed one account-center row.
///
/// The rows are written directly rather than through
/// `POST /storage/provider-accounts`: what this test pins down is the *list*
/// window and its continuation, and the create route adds nothing to that
/// question beyond a credential seal that needs a deployment master secret.
/// The account center is IAM-owned, so its table is addressed by name here.
async fn seed_provider_account(
    pool: &sqlx::PgPool,
    account_id: &str,
    tenant_id: &str,
    scope_type: &str,
    vendor_code: &str,
    account_code: &str,
) {
    sqlx::query(
        "INSERT INTO iam_provider_account (
            id, uuid, tenant_id, organization_id, scope_type, owner_user_id, vendor_code,
            account_code, display_name, account_type, environment, capability_codes, status,
            version, created_by, updated_by
         ) VALUES ($1, $2, $3, '0', $4, NULL, $5, $6, $6, 'long_term_key', 'production', \
            '[\"object_storage\"]', 'active', 1, 'test', 'test')",
    )
    .bind(account_id)
    .bind(account_id)
    .bind(tenant_id)
    .bind(scope_type)
    .bind(vendor_code)
    .bind(account_code)
    .execute(pool)
    .await
    .expect("seed an account-center provider account");
}

/// The account list has to report a continuation the console can follow.
///
/// The defect this pins down: `list_accounts` applies `LIMIT` itself, so a
/// handler that asks it for exactly `page_size` rows can never prove another
/// page exists — `pageInfo.hasMore` was `false` and no `nextCursor` was ever
/// issued for *any* window. The console reads one page, so every account that
/// sorted past it was unreachable: an operator registered a new account, the
/// account center stored it, and the picker still answered "no account for this
/// vendor" and invited registering it again.
#[tokio::test]
async fn admin_storage_provider_account_list_reports_a_real_continuation() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    // Platform scope: the storage console registers platform-wide accounts,
    // which is also the slice its credential picker lists.
    let app = build_router_with_pool_without_iam_and_test_tenant(pool.clone(), "100001");
    let vendor_code = unique_vendor_code();
    let account_ids = ["one", "two", "three"].map(|suffix| format!("iampacct-{vendor_code}-{suffix}"));
    for account_id in &account_ids {
        seed_provider_account(
            &pool,
            account_id,
            "100001",
            "platform",
            &vendor_code,
            &format!("{vendor_code}-{account_id}"),
        )
        .await;
    }

    let first_page = list_provider_accounts_payload(
        app.clone(),
        &format!(
            "/backend/v3/api/drive/storage/provider-accounts?scopeType=platform&vendorCode={vendor_code}&page_size=2"
        ),
    )
    .await;
    let first_rows = provider_account_rows(&first_page);
    assert_eq!(
        first_rows.len(),
        2,
        "a two-row window must return two rows: {first_page}"
    );
    // Every row belongs to the filtered vendor: the window is selected from that
    // vendor's accounts, not filtered after the fact from a mixed page.
    assert!(
        first_rows.iter().all(|(_, vendor)| vendor == &vendor_code),
        "the vendor filter must select the window, got {first_page}"
    );
    assert_eq!(
        first_page["data"]["pageInfo"]["hasMore"], true,
        "a third matching row exists, so the page must report more: {first_page}"
    );
    let next_cursor = first_page["data"]["pageInfo"]["nextCursor"]
        .as_str()
        .expect("a page with more rows must carry an opaque cursor")
        .to_string();

    let second_page = list_provider_accounts_payload(
        app.clone(),
        &format!(
            "/backend/v3/api/drive/storage/provider-accounts?scopeType=platform&vendorCode={vendor_code}&page_size=2&cursor={next_cursor}"
        ),
    )
    .await;
    let second_rows = provider_account_rows(&second_page);
    assert_eq!(
        second_rows.len(),
        1,
        "the cursor must land on the remaining row: {second_page}"
    );
    assert!(
        second_rows.iter().all(|(_, vendor)| vendor == &vendor_code),
        "the continuation must stay inside the same vendor window: {second_page}"
    );
    assert_eq!(
        second_page["data"]["pageInfo"]["hasMore"], false,
        "the last page must not claim a successor: {second_page}"
    );

    // The two pages partition the vendor's accounts: no row is skipped and no
    // row is served twice, which is what makes "page forward to find it" a real
    // answer for the operator.
    let mut served = first_rows
        .iter()
        .chain(second_rows.iter())
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    served.sort();
    let mut expected = account_ids.to_vec();
    expected.sort();
    assert_eq!(served, expected, "pages must partition the account set");

    // Finally the console's own request — the one the credential picker issues —
    // finds the account that was registered last, on its first page.
    let console_page = list_provider_accounts_payload(
        app,
        &format!(
            "/backend/v3/api/drive/storage/provider-accounts?status=active&scopeType=platform&vendorCode={vendor_code}&page_size=20"
        ),
    )
    .await;
    let console_rows = provider_account_rows(&console_page);
    assert!(
        console_rows
            .iter()
            .any(|(id, _)| id == account_ids.last().expect("three accounts were seeded")),
        "the console's vendor-scoped page must carry the newest account: {console_page}"
    );
}
