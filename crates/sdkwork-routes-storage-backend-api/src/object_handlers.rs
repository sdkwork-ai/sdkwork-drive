use crate::app_context::DriveRequestContext;
use crate::audit::record_storage_provider_audit;
use crate::dto::{
    CopyProviderObjectRequest, ListProviderObjectsQuery, ProviderObjectContentResponse,
    ProviderObjectMutationResponse, ProviderObjectResponse, UpdateProviderObjectContentRequest,
};
use crate::error::{
    invalid_json_problem, map_object_store_route_error, payload_too_large_problem,
    validation_problem, ProblemDetail,
};
use crate::object_store::build_object_store_for_provider;
use crate::provider_lookup::get_active_provider;
use crate::response::{no_content, success_cursor_list_page, StorageListHttpResponse};
use crate::state::AdminStorageState;
use crate::validators::{
    decode_path_object_key, validate_object_delimiter, validate_object_key,
    validate_object_prefix, validate_page_size_u16,
};
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_contract::drive::domain_events::admin_audit;
use sdkwork_drive_storage_contract::{
    validate_s3_bucket_name, CopyObjectRequest, DeleteObjectRequest, DriveByteRange,
    DriveObjectLocator, DriveObjectStoreError, DriveObjectStoreErrorKind, HeadObjectRequest,
    ListObjectsRequest, PutObjectRequest, ReadObjectRangeRequest,
};
use sdkwork_utils_rust::{DEFAULT_LIST_PAGE_SIZE, MAX_LIST_PAGE_SIZE};

pub(crate) async fn list_storage_provider_objects(
    State(state): State<AdminStorageState>,
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
    let delimiter = validate_object_delimiter(query.delimiter, "delimiter")?;
    let provider = get_active_provider(&state, &provider_id).await?;
    let object_store = build_object_store_for_provider(&state.config, &provider).await?;
    let result = object_store
        .list_objects(ListObjectsRequest {
            bucket: provider.bucket.clone(),
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
    Path((provider_id, object_key)): Path<(String, String)>,
) -> Result<Json<ProviderObjectResponse>, (StatusCode, Json<ProblemDetail>)> {
    let provider = get_active_provider(&state, &provider_id).await?;
    let object_store = build_object_store_for_provider(&state.config, &provider).await?;
    let object_key = decode_path_object_key(&object_key)?;
    let result = object_store
        .head_object(HeadObjectRequest {
            locator: DriveObjectLocator {
                bucket: provider.bucket.clone(),
                object_key,
            },
        })
        .await
        .map_err(map_object_store_route_error)?;
    Ok(Json(ProviderObjectResponse {
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
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let operator_id = ctx.resolve_operator_id()?;
    let provider = get_active_provider(&state, &provider_id).await?;
    let object_store = build_object_store_for_provider(&state.config, &provider).await?;
    let object_key = decode_path_object_key(&object_key)?;
    let result = object_store
        .delete_object(DeleteObjectRequest {
            locator: DriveObjectLocator {
                bucket: provider.bucket.clone(),
                object_key,
            },
        })
        .await
        .map_err(map_object_store_route_error)?;
    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::OBJECT_DELETED,
        &provider_id,
        &operator_id,
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
) -> Result<Json<ProviderObjectMutationResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let source_key = validate_object_key(payload.source_object_key, "sourceObjectKey")?;
    let destination_key =
        validate_object_key(payload.destination_object_key, "destinationObjectKey")?;
    let operator_id = ctx.resolve_operator_id()?;
    let provider = get_active_provider(&state, &provider_id).await?;
    let object_store = build_object_store_for_provider(&state.config, &provider).await?;
    let destination_bucket = match payload.destination_bucket.as_deref() {
        Some(value) if !value.trim().is_empty() => {
            validate_s3_bucket_name(value, "destinationBucket")
                .map_err(map_object_store_route_error)?;
            value.to_string()
        }
        _ => provider.bucket.clone(),
    };
    let result = object_store
        .copy_object(CopyObjectRequest {
            source: DriveObjectLocator {
                bucket: provider.bucket.clone(),
                object_key: source_key,
            },
            destination: DriveObjectLocator {
                bucket: destination_bucket,
                object_key: destination_key,
            },
            metadata_directive: payload.metadata_directive,
        })
        .await
        .map_err(map_object_store_route_error)?;
    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::OBJECT_COPIED,
        &provider_id,
        &operator_id,
    )
    .await?;
    Ok(Json(ProviderObjectMutationResponse {
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

pub(crate) async fn read_storage_provider_object_content(
    State(state): State<AdminStorageState>,
    Path((provider_id, object_key)): Path<(String, String)>,
) -> Result<Json<ProviderObjectContentResponse>, (StatusCode, Json<ProblemDetail>)> {
    let provider = get_active_provider(&state, &provider_id).await?;
    let object_store = build_object_store_for_provider(&state.config, &provider).await?;
    let object_key = decode_path_object_key(&object_key)?;
    let head = object_store
        .head_object(HeadObjectRequest {
            locator: DriveObjectLocator {
                bucket: provider.bucket.clone(),
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
    Ok(Json(ProviderObjectContentResponse {
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
    payload: Result<Json<UpdateProviderObjectContentRequest>, JsonRejection>,
) -> Result<Json<ProviderObjectResponse>, (StatusCode, Json<ProblemDetail>)> {
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
    let provider = get_active_provider(&state, &provider_id).await?;
    let object_store = build_object_store_for_provider(&state.config, &provider).await?;
    let checksum = sdkwork_utils_rust::sha256_hash(&bytes);
    object_store
        .put_object(PutObjectRequest {
            locator: DriveObjectLocator {
                bucket: provider.bucket.clone(),
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
    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::OBJECT_PUT,
        &provider_id,
        &operator_id,
    )
    .await?;
    let head = object_store
        .head_object(HeadObjectRequest {
            locator: DriveObjectLocator {
                bucket: provider.bucket.clone(),
                object_key,
            },
        })
        .await
        .map_err(map_object_store_route_error)?;
    Ok(Json(ProviderObjectResponse {
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
