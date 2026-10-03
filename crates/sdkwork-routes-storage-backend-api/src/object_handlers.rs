use crate::app_context::DriveRequestContext;
use crate::audit::record_storage_provider_audit;
use crate::dto::{
    AbortProviderObjectMultipartUploadRequest, CompleteProviderObjectMultipartUploadRequest,
    CopyProviderObjectRequest, CreateProviderObjectMultipartUploadRequest,
    ListProviderObjectsQuery, PresignProviderObjectUploadPartsRequest, ProviderObjectBucketQuery,
    ProviderObjectContentResponse, ProviderObjectMultipartUploadResponse,
    ProviderObjectMutationResponse, ProviderObjectResponse,
    ProviderObjectUploadPartGrantResponse, ProviderObjectUploadPartGrantsResponse,
    UpdateProviderObjectContentRequest,
};
use crate::error::{
    invalid_json_problem, map_object_store_route_error, payload_too_large_problem,
    validation_problem, ProblemDetail,
};
use crate::object_store::build_object_store_for_provider;
use crate::provider_lookup::get_active_provider;
use crate::response::{
    no_content, success_cursor_list_page, success_item, StorageItemHttpResponse,
    StorageListHttpResponse,
};
use crate::state::AdminStorageState;
use crate::validators::{
    decode_path_object_key, resolve_object_bucket, resolve_object_region,
    validate_completed_upload_parts, validate_grant_lifetime, validate_object_delimiter,
    validate_object_key, validate_object_prefix, validate_page_size_u16, validate_upload_id,
    validate_upload_part_numbers,
};
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_contract::drive::domain_events::admin_audit;
use sdkwork_drive_storage_contract::{
    AbortMultipartUploadRequest, CompleteMultipartUploadRequest, CompletedMultipartPart,
    CopyObjectRequest, CreateMultipartUploadRequest, DeleteObjectRequest, DriveByteRange,
    DriveObjectLocator, DriveObjectStoreError, DriveObjectStoreErrorKind, HeadObjectRequest,
    ListObjectsRequest,
    PresignUploadPartRequest, PutObjectRequest, ReadObjectRangeRequest,
};
use sdkwork_utils_rust::{DEFAULT_LIST_PAGE_SIZE, MAX_LIST_PAGE_SIZE};

pub(crate) async fn list_storage_provider_objects(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_id): Path<String>,
    Query(query): Query<ListProviderObjectsQuery>,
) -> Result<StorageListHttpResponse<ProviderObjectResponse>, (StatusCode, Json<ProblemDetail>)> {
    let max_keys = validate_page_size_u16(
        query.page_size,
        DEFAULT_LIST_PAGE_SIZE as u16,
        1,
        MAX_LIST_PAGE_SIZE as u16,
        "page_size",
    )?;
    let prefix = validate_object_prefix(query.prefix, "prefix")?;
    let tenant_id = ctx.resolve_tenant_id()?;

    let delimiter = validate_object_delimiter(query.delimiter, "delimiter")?;
    let provider = get_active_provider(&state, &tenant_id, &provider_id).await?;
    let bucket = resolve_object_bucket(&provider.bucket, query.bucket.clone(), "bucket")?;
    let bucket_region = resolve_object_region(query.region, "region")?;
    let object_store =
        build_object_store_for_provider(&state, &provider, bucket_region.as_deref()).await?;
    let result = object_store
        .list_objects(ListObjectsRequest {
            bucket,
            prefix,
            delimiter,
            continuation_token: query.page_token,
            max_keys,
        })
        .await
        .map_err(map_object_store_route_error)?;
    let mut items: Vec<ProviderObjectResponse> = result
        .prefixes
        .into_iter()
        .map(|prefix| ProviderObjectResponse {
            provider_id: provider_id.clone(),
            bucket: result.bucket.clone(),
            object_kind: "prefix".to_string(),
            object_key: prefix,
            content_length: 0,
            content_type: None,
            etag: None,
            version_id: None,
            storage_class: None,
            last_modified_epoch_ms: None,
        })
        .chain(result.items.into_iter().map(|item| ProviderObjectResponse {
            provider_id: provider_id.clone(),
            bucket: result.bucket.clone(),
            object_kind: "object".to_string(),
            object_key: item.object_key,
            content_length: item.content_length,
            content_type: None,
            etag: item.etag,
            version_id: None,
            storage_class: item.storage_class,
            last_modified_epoch_ms: item.last_modified_epoch_ms,
        }))
        .collect();
    items.sort_by(|left, right| left.object_key.cmp(&right.object_key));
    Ok(success_cursor_list_page(
        items,
        i32::from(max_keys),
        result.next_continuation_token,
    ))
}

pub(crate) async fn head_storage_provider_object(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((provider_id, object_key)): Path<(String, String)>,
    Query(query): Query<ProviderObjectBucketQuery>,
) -> Result<StorageItemHttpResponse<ProviderObjectResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let provider = get_active_provider(&state, &tenant_id, &provider_id).await?;
    let bucket = resolve_object_bucket(&provider.bucket, query.bucket, "bucket")?;
    let bucket_region = resolve_object_region(query.region, "region")?;
    let object_store =
        build_object_store_for_provider(&state, &provider, bucket_region.as_deref()).await?;
    let object_key = decode_path_object_key(&object_key)?;
    let result = object_store
        .head_object(HeadObjectRequest {
            locator: DriveObjectLocator {
                bucket,
                object_key,
            },
        })
        .await
        .map_err(map_object_store_route_error)?;
    Ok(success_item(ProviderObjectResponse {
        provider_id,
        bucket: result.locator.bucket,
        object_kind: "object".to_string(),
        object_key: result.locator.object_key,
        content_length: result.content_length,
        content_type: result.content_type,
        etag: result.etag,
        version_id: result.version_id,
        storage_class: None,
        last_modified_epoch_ms: None,
    }))
}

pub(crate) async fn delete_storage_provider_object(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((provider_id, object_key)): Path<(String, String)>,
    Query(query): Query<ProviderObjectBucketQuery>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let operator_id = ctx.resolve_operator_id()?;
    let tenant_id = ctx.resolve_tenant_id()?;

    let provider = get_active_provider(&state, &tenant_id, &provider_id).await?;
    let bucket = resolve_object_bucket(&provider.bucket, query.bucket, "bucket")?;
    let bucket_region = resolve_object_region(query.region, "region")?;
    let object_store =
        build_object_store_for_provider(&state, &provider, bucket_region.as_deref()).await?;
    let object_key = decode_path_object_key(&object_key)?;
    let result = object_store
        .delete_object(DeleteObjectRequest {
            locator: DriveObjectLocator {
                bucket,
                object_key: object_key.clone(),
            },
        })
        .await
        .map_err(map_object_store_route_error)?;
    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::OBJECT_DELETED,
        &format!("{provider_id}/{object_key}"),
        &operator_id,
        &tenant_id,
    )
    .await?;
    let _deleted = result.deleted;
    Ok(no_content())
}

pub(crate) async fn copy_storage_provider_object(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_id): Path<String>,
    payload: Result<Json<CopyProviderObjectRequest>, JsonRejection>,
) -> Result<
    StorageItemHttpResponse<ProviderObjectMutationResponse>,
    (StatusCode, Json<ProblemDetail>),
> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let source_key = validate_object_key(payload.source_object_key, "sourceObjectKey")?;
    let destination_key =
        validate_object_key(payload.destination_object_key, "destinationObjectKey")?;
    let operator_id = ctx.resolve_operator_id()?;
    let tenant_id = ctx.resolve_tenant_id()?;

    let provider = get_active_provider(&state, &tenant_id, &provider_id).await?;
    let source_bucket = resolve_object_bucket(&provider.bucket, payload.source_bucket, "sourceBucket")?;
    let destination_bucket = resolve_object_bucket(
        &provider.bucket,
        payload.destination_bucket,
        "destinationBucket",
    )?;
    let bucket_region = resolve_object_region(payload.region, "region")?;
    let object_store =
        build_object_store_for_provider(&state, &provider, bucket_region.as_deref()).await?;
    let result = object_store
        .copy_object(CopyObjectRequest {
            source: DriveObjectLocator {
                bucket: source_bucket,
                object_key: source_key,
            },
            destination: DriveObjectLocator {
                bucket: destination_bucket,
                object_key: destination_key.clone(),
            },
            metadata_directive: payload.metadata_directive,
        })
        .await
        .map_err(map_object_store_route_error)?;
    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::OBJECT_COPIED,
        &format!("{provider_id}/{destination_key}"),
        &operator_id,
        &tenant_id,
    )
    .await?;
    Ok(success_item(ProviderObjectMutationResponse {
        provider_id,
        bucket: result.locator.bucket,
        object_key: result.locator.object_key,
        changed: true,
    }))
}

/// 单个对象内容读取上限（字节）。超过该限制的下载应改用大文件通道（presign）。
const MAX_OBJECT_CONTENT_BYTES: usize = 8 * 1024 * 1024;
/// base64 编码内容的字符上限：`ceil(MAX_OBJECT_CONTENT_BYTES / 3) * 4`。
const MAX_OBJECT_CONTENT_BASE64_CHARS: usize = 11_184_812;

/// 对象内容写入请求的**请求体**上限（字节）。
///
/// 业务上限是"对象 8 MiB"，但请求体不是 8 MiB：内容以 base64 装在 JSON 里，`4/3` 放大
/// 之后约 11.2 MB，再加字段名/编码等信封。路由必须显式声明这个上限（见 `route_paths`），
/// 否则用的是 axum 的默认 2 MB —— 1.4 MB 以上的文件根本到不了这里的业务校验，客户端
/// 看到的是一句没有上下文的 413。
///
/// 这里按"base64 上限 + 信封与括号的余量"取整，留足余量而不放开无界请求体。
pub(crate) const MAX_OBJECT_CONTENT_REQUEST_BYTES: usize =
    MAX_OBJECT_CONTENT_BASE64_CHARS + 64 * 1024;

pub(crate) async fn read_storage_provider_object_content(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((provider_id, object_key)): Path<(String, String)>,
    Query(query): Query<ProviderObjectBucketQuery>,
) -> Result<StorageItemHttpResponse<ProviderObjectContentResponse>, (StatusCode, Json<ProblemDetail>)>
{
    let tenant_id = ctx.resolve_tenant_id()?;
    let provider = get_active_provider(&state, &tenant_id, &provider_id).await?;
    let bucket = resolve_object_bucket(&provider.bucket, query.bucket, "bucket")?;
    let bucket_region = resolve_object_region(query.region, "region")?;
    let object_store =
        build_object_store_for_provider(&state, &provider, bucket_region.as_deref()).await?;
    let object_key = decode_path_object_key(&object_key)?;
    let head = object_store
        .head_object(HeadObjectRequest {
            locator: DriveObjectLocator {
                bucket,
                object_key: object_key.clone(),
            },
        })
        .await
        .map_err(map_object_store_route_error)?;
    if head.content_length > MAX_OBJECT_CONTENT_BYTES as u64 {
        return Err(payload_too_large_problem(format!(
            "object content exceeds the {MAX_OBJECT_CONTENT_BYTES} byte read limit"
        )));
    }
    let mut bytes = Vec::with_capacity(head.content_length as usize);
    if head.content_length > 0 {
        let (_, mut stream) = object_store
            .read_object_range(ReadObjectRangeRequest {
                locator: DriveObjectLocator {
                    bucket: head.locator.bucket.clone(),
                    object_key: object_key.clone(),
                },
                range: DriveByteRange {
                    start_inclusive: 0,
                    end_inclusive: head.content_length - 1,
                },
            })
            .await
            .map_err(map_object_store_route_error)?;
        while let Some(chunk) = stream
            .next_chunk()
            .await
            .map_err(map_object_store_route_error)?
        {
            bytes.extend_from_slice(&chunk);
        }
        if bytes.len() as u64 != head.content_length {
            return Err(map_object_store_route_error(DriveObjectStoreError {
                kind: DriveObjectStoreErrorKind::IntegrityFailed,
                message: format!(
                    "object content read length {} does not match expected {}",
                    bytes.len(),
                    head.content_length
                ),
            }));
        }
    }
    Ok(success_item(ProviderObjectContentResponse {
        provider_id,
        bucket: head.locator.bucket,
        object_key,
        content_type: head.content_type,
        size_bytes: head.content_length,
        encoding: "base64".to_string(),
        content: sdkwork_utils_rust::base64_encode(&bytes),
        checksum_sha256: sdkwork_utils_rust::sha256_hash(&bytes),
    }))
}

pub(crate) async fn write_storage_provider_object_content(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((provider_id, object_key)): Path<(String, String)>,
    Query(query): Query<ProviderObjectBucketQuery>,
    payload: Result<Json<UpdateProviderObjectContentRequest>, JsonRejection>,
) -> Result<StorageItemHttpResponse<ProviderObjectResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let encoding = payload.encoding.as_deref().unwrap_or("utf8");
    let object_key = decode_path_object_key(&object_key)?;
    // base64 字符上限在解码前检查，避免为注定 413 的载荷分配解码缓冲。
    if encoding == "base64" && payload.content.len() > MAX_OBJECT_CONTENT_BASE64_CHARS {
        return Err(payload_too_large_problem(format!(
            "base64 content exceeds the {MAX_OBJECT_CONTENT_BASE64_CHARS} character limit"
        )));
    }
    let bytes = match encoding {
        "utf8" => payload.content.as_bytes().to_vec(),
        "base64" => sdkwork_utils_rust::base64_decode(&payload.content)
            .ok_or_else(|| validation_problem("content is not valid base64"))?,
        _ => return Err(validation_problem("encoding must be utf8 or base64")),
    };
    if bytes.len() > MAX_OBJECT_CONTENT_BYTES {
        return Err(payload_too_large_problem(format!(
            "object content exceeds the {MAX_OBJECT_CONTENT_BYTES} byte write limit"
        )));
    }
    let content_type = payload
        .content_type
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let tenant_id = ctx.resolve_tenant_id()?;
    let provider = get_active_provider(&state, &tenant_id, &provider_id).await?;
    let bucket = resolve_object_bucket(&provider.bucket, query.bucket, "bucket")?;
    let bucket_region = resolve_object_region(query.region, "region")?;
    let object_store =
        build_object_store_for_provider(&state, &provider, bucket_region.as_deref()).await?;
    let checksum = sdkwork_utils_rust::sha256_hash(&bytes);
    object_store
        .put_object(PutObjectRequest {
            locator: DriveObjectLocator {
                bucket: bucket.clone(),
                object_key: object_key.clone(),
            },
            content_type,
            metadata: Default::default(),
            body: bytes,
            checksum_sha256_hex: Some(checksum),
        })
        .await
        .map_err(map_object_store_route_error)?;
    let operator_id = ctx.resolve_operator_id()?;
    let tenant_id = ctx.resolve_tenant_id()?;

    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::OBJECT_PUT,
        &format!("{provider_id}/{object_key}"),
        &operator_id,
        &tenant_id,
    )
    .await?;
    let head = object_store
        .head_object(HeadObjectRequest {
            locator: DriveObjectLocator {
                bucket,
                object_key,
            },
        })
        .await
        .map_err(map_object_store_route_error)?;
    Ok(success_item(ProviderObjectResponse {
        provider_id,
        bucket: head.locator.bucket,
        object_kind: "object".to_string(),
        object_key: head.locator.object_key,
        content_length: head.content_length,
        content_type: head.content_type,
        etag: head.etag,
        version_id: head.version_id,
        storage_class: None,
        last_modified_epoch_ms: None,
    }))
}

/// 开启一个分片上传。
///
/// 单次内容接口（8 MiB）之外的对象从这里进入：开启 →（客户端按分片直传厂商）→ 完成/中止。
/// 这里只做"开启"这一步，字节不经过本 API。
pub(crate) async fn create_storage_provider_object_multipart_upload(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_id): Path<String>,
    Query(query): Query<ProviderObjectBucketQuery>,
    payload: Result<Json<CreateProviderObjectMultipartUploadRequest>, JsonRejection>,
) -> Result<StorageItemHttpResponse<ProviderObjectMultipartUploadResponse>, (StatusCode, Json<ProblemDetail>)>
{
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let object_key = validate_object_key(payload.object_key, "objectKey")?;
    let content_type = payload
        .content_type
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let checksum_sha256_hex = payload
        .checksum_sha256_hex
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());

    let tenant_id = ctx.resolve_tenant_id()?;
    let provider = get_active_provider(&state, &tenant_id, &provider_id).await?;
    let bucket = resolve_object_bucket(&provider.bucket, query.bucket, "bucket")?;
    let bucket_region = resolve_object_region(query.region, "region")?;
    let object_store =
        build_object_store_for_provider(&state, &provider, bucket_region.as_deref()).await?;

    let created = object_store
        .create_multipart_upload(CreateMultipartUploadRequest {
            locator: DriveObjectLocator {
                bucket: bucket.clone(),
                object_key: object_key.clone(),
            },
            content_type,
            metadata: Default::default(),
            checksum_sha256_hex,
        })
        .await
        .map_err(map_object_store_route_error)?;

    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::MULTIPART_UPLOAD_CREATED,
        &format!("{provider_id}/{object_key}"),
        &ctx.resolve_operator_id()?,
        &tenant_id,
    )
    .await?;

    Ok(success_item(ProviderObjectMultipartUploadResponse {
        provider_id,
        bucket: created.locator.bucket,
        object_key: created.locator.object_key,
        upload_id: created.upload_id,
    }))
}

/// 为一批分片签发直传授权。
///
/// 返回的 `headers` 是签名的一部分，客户端必须原样回放；`url` 指向厂商地址，字节不回本服务。
pub(crate) async fn presign_storage_provider_object_upload_parts(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_id): Path<String>,
    Query(query): Query<ProviderObjectBucketQuery>,
    payload: Result<Json<PresignProviderObjectUploadPartsRequest>, JsonRejection>,
) -> Result<StorageItemHttpResponse<ProviderObjectUploadPartGrantsResponse>, (StatusCode, Json<ProblemDetail>)>
{
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let object_key = validate_object_key(payload.object_key, "objectKey")?;
    let upload_id = validate_upload_id(&payload.upload_id)?;
    validate_upload_part_numbers(&payload.part_numbers)?;
    let expires_in_seconds = validate_grant_lifetime(payload.expires_in_seconds)?;

    let tenant_id = ctx.resolve_tenant_id()?;
    let provider = get_active_provider(&state, &tenant_id, &provider_id).await?;
    let bucket = resolve_object_bucket(&provider.bucket, query.bucket, "bucket")?;
    let bucket_region = resolve_object_region(query.region, "region")?;
    let object_store =
        build_object_store_for_provider(&state, &provider, bucket_region.as_deref()).await?;

    // 逐片签发：厂商的签名按分片号参与计算，无法一次签出多个号。
    let mut parts = Vec::with_capacity(payload.part_numbers.len());
    for part_number in &payload.part_numbers {
        let grant = object_store
            .presign_upload_part(PresignUploadPartRequest {
                locator: DriveObjectLocator {
                    bucket: bucket.clone(),
                    object_key: object_key.clone(),
                },
                upload_id: upload_id.clone(),
                part_number: *part_number,
                expires_in_seconds,
            })
            .await
            .map_err(map_object_store_route_error)?;
        parts.push(ProviderObjectUploadPartGrantResponse {
            part_number: *part_number,
            method: grant.method,
            url: grant.url,
            headers: grant.headers,
            expires_at_epoch_ms: grant.expires_at_epoch_ms,
        });
    }

    Ok(success_item(ProviderObjectUploadPartGrantsResponse {
        provider_id,
        bucket,
        object_key,
        upload_id,
        parts,
    }))
}

/// 完成分片上传，并把落盘后的对象信息回给客户端（与单次写入的响应保持同一形状）。
pub(crate) async fn complete_storage_provider_object_multipart_upload(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_id): Path<String>,
    Query(query): Query<ProviderObjectBucketQuery>,
    payload: Result<Json<CompleteProviderObjectMultipartUploadRequest>, JsonRejection>,
) -> Result<StorageItemHttpResponse<ProviderObjectResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let object_key = validate_object_key(payload.object_key, "objectKey")?;
    let upload_id = validate_upload_id(&payload.upload_id)?;
    let parts: Vec<(u16, String)> = payload
        .parts
        .iter()
        .map(|part| (part.part_number, part.etag.trim().to_string()))
        .collect();
    validate_completed_upload_parts(&parts)?;

    let tenant_id = ctx.resolve_tenant_id()?;
    let provider = get_active_provider(&state, &tenant_id, &provider_id).await?;
    let bucket = resolve_object_bucket(&provider.bucket, query.bucket, "bucket")?;
    let bucket_region = resolve_object_region(query.region, "region")?;
    let object_store =
        build_object_store_for_provider(&state, &provider, bucket_region.as_deref()).await?;

    let completed = object_store
        .complete_multipart_upload(CompleteMultipartUploadRequest {
            locator: DriveObjectLocator {
                bucket: bucket.clone(),
                object_key: object_key.clone(),
            },
            upload_id,
            parts: parts
                .into_iter()
                .map(|(part_number, etag)| CompletedMultipartPart { part_number, etag })
                .collect(),
        })
        .await
        .map_err(map_object_store_route_error)?;

    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::MULTIPART_UPLOAD_COMPLETED,
        &format!("{provider_id}/{object_key}"),
        &ctx.resolve_operator_id()?,
        &tenant_id,
    )
    .await?;

    // 与单次写入一致：完成后再 head 一次，把大小/类型/ETag 这些真实值回给客户端。
    let head = object_store
        .head_object(HeadObjectRequest {
            locator: DriveObjectLocator {
                bucket: completed.locator.bucket,
                object_key: completed.locator.object_key,
            },
        })
        .await
        .map_err(map_object_store_route_error)?;
    Ok(success_item(ProviderObjectResponse {
        provider_id,
        bucket: head.locator.bucket,
        object_kind: "object".to_string(),
        object_key: head.locator.object_key,
        content_length: head.content_length,
        content_type: head.content_type,
        etag: head.etag.or(completed.etag),
        version_id: head.version_id.or(completed.version_id),
        storage_class: None,
        last_modified_epoch_ms: None,
    }))
}

/// 中止分片上传：运营商取消或关闭上传时必须调用，否则已上传的分片会一直计费。
pub(crate) async fn abort_storage_provider_object_multipart_upload(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_id): Path<String>,
    Query(query): Query<ProviderObjectBucketQuery>,
    payload: Result<Json<AbortProviderObjectMultipartUploadRequest>, JsonRejection>,
) -> Result<StorageItemHttpResponse<ProviderObjectMutationResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let object_key = validate_object_key(payload.object_key, "objectKey")?;
    let upload_id = validate_upload_id(&payload.upload_id)?;

    let tenant_id = ctx.resolve_tenant_id()?;
    let provider = get_active_provider(&state, &tenant_id, &provider_id).await?;
    let bucket = resolve_object_bucket(&provider.bucket, query.bucket, "bucket")?;
    let bucket_region = resolve_object_region(query.region, "region")?;
    let object_store =
        build_object_store_for_provider(&state, &provider, bucket_region.as_deref()).await?;

    object_store
        .abort_multipart_upload(AbortMultipartUploadRequest {
            locator: DriveObjectLocator {
                bucket: bucket.clone(),
                object_key: object_key.clone(),
            },
            upload_id,
        })
        .await
        .map_err(map_object_store_route_error)?;

    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::MULTIPART_UPLOAD_ABORTED,
        &format!("{provider_id}/{object_key}"),
        &ctx.resolve_operator_id()?,
        &tenant_id,
    )
    .await?;

    Ok(success_item(ProviderObjectMutationResponse {
        provider_id,
        bucket,
        object_key,
        changed: true,
    }))
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_OBJECT_CONTENT_BASE64_CHARS, MAX_OBJECT_CONTENT_BYTES,
        MAX_OBJECT_CONTENT_REQUEST_BYTES,
    };

    /// 请求体上限必须容得下"合法对象 + base64 放大 + 信封"。
    ///
    /// 这条不变式曾经不成立：路由没有声明请求体上限，axum 的默认 2 MB 先于业务校验生效，
    /// 于是 1.4 MB 以上的上传全部失败，客户端只看到一句 `Payload too large`。测试把它锁住：
    /// 只要有人把业务上限调大而不动请求体上限，这里立刻失败。
    #[test]
    fn request_body_limit_covers_the_encoded_form_of_a_legal_object() {
        // base64 上限本身要与 8 MiB 的业务上限自洽（ceil(n/3)*4）。
        let expected_base64 = MAX_OBJECT_CONTENT_BYTES.div_ceil(3) * 4;
        assert!(
            MAX_OBJECT_CONTENT_BASE64_CHARS >= expected_base64,
            "base64 character limit {MAX_OBJECT_CONTENT_BASE64_CHARS} must cover {expected_base64}",
        );
        assert!(
            MAX_OBJECT_CONTENT_REQUEST_BYTES > MAX_OBJECT_CONTENT_BASE64_CHARS,
            "request body limit {} must exceed the base64 payload {} to leave room for the JSON envelope",
            MAX_OBJECT_CONTENT_REQUEST_BYTES,
            MAX_OBJECT_CONTENT_BASE64_CHARS,
        );
        // 余量要能装下字段名、编码值与引号，但也不能大到失去意义。
        let headroom = MAX_OBJECT_CONTENT_REQUEST_BYTES - MAX_OBJECT_CONTENT_BASE64_CHARS;
        assert!(
            (1024..=1024 * 1024).contains(&headroom),
            "JSON envelope headroom {headroom} should be between 1 KiB and 1 MiB",
        );
        // 默认的 2 MB 必须已经被显式上限取代：这是这条 bug 的核心教训。
        assert!(MAX_OBJECT_CONTENT_REQUEST_BYTES > 2 * 1024 * 1024);
    }

    /// int64 在网线上的形状是**字符串**（`API_SPEC.md` §13.6）。
    ///
    /// 对象响应里的 `lastModifiedEpochMs` 曾经直接序列化成数字：契约声明的是
    /// `type: string, format: int64`，生成的 Go/Rust/Java SDK 按字符串反序列化，于是它们解析
    /// 这条响应会失败——而浏览器的 `numberField` 两种都收，这个偏差在 Web 上根本看不出来。
    /// 新加的 `expiresAtEpochMs` 与它一起锁住。
    #[test]
    fn epoch_millisecond_fields_serialize_as_strings() {
        let object = crate::dto::ProviderObjectResponse {
            provider_id: "provider-1".to_string(),
            bucket: "bucket-1".to_string(),
            object_kind: "object".to_string(),
            object_key: "photos/a.png".to_string(),
            content_length: 1024,
            content_type: Some("image/png".to_string()),
            etag: Some("etag-1".to_string()),
            version_id: None,
            storage_class: None,
            last_modified_epoch_ms: Some(1_767_225_600_000),
        };
        let value = serde_json::to_value(&object).expect("object response should serialize");
        assert_eq!(
            value["lastModifiedEpochMs"],
            serde_json::Value::String("1767225600000".to_string()),
            "int64 wire fields must be strings: {value}"
        );

        let grant = crate::dto::ProviderObjectUploadPartGrantResponse {
            part_number: 3,
            method: "PUT".to_string(),
            url: "https://cos.example.com/bucket/photos/a.png?partNumber=3".to_string(),
            headers: std::collections::BTreeMap::new(),
            expires_at_epoch_ms: 1_767_225_600_000,
        };
        let value = serde_json::to_value(&grant).expect("part grant should serialize");
        assert_eq!(
            value["expiresAtEpochMs"],
            serde_json::Value::String("1767225600000".to_string()),
            "int64 wire fields must be strings: {value}"
        );
    }
}
