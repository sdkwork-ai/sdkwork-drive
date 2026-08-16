use crate::app_context::DriveRequestContext;
use crate::audit::record_storage_provider_audit;
use crate::dto::{
    ListProviderBucketsQuery, ProviderBucketListItemResponse, ProviderBucketMutationResponse,
    ProviderBucketResponse,
};
use crate::error::{map_object_store_route_error, ProblemDetail};
use crate::object_store::build_full_s3_object_store_for_provider;
use crate::provider_lookup::get_active_provider;
use crate::response::{no_content, success_list_page_simple, StorageListHttpResponse};
use crate::state::AdminStorageState;
use crate::validators::{next_page_token, parse_offset_page};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_contract::drive::domain_events::admin_audit;
use sdkwork_drive_storage_contract::{
    CreateBucketRequest, DeleteBucketRequest, DriveObjectStore, HeadBucketRequest,
    ListBucketsRequest,
};

/// S3 ListBuckets returns the full account inventory in one call.
/// Bounded L3 admin exception (PAGINATION_SPEC §2.3): inventory capped at
/// `MAX_ADMIN_BUCKET_LIST_ITEMS`, then offset-paginated for the HTTP response.
const MAX_ADMIN_BUCKET_LIST_ITEMS: usize = 200;

pub(crate) async fn head_storage_provider_bucket(
    State(state): State<AdminStorageState>,
    Path(provider_id): Path<String>,
) -> Result<Json<ProviderBucketResponse>, (StatusCode, Json<ProblemDetail>)> {
    let provider = get_active_provider(&state, &provider_id).await?;
    let object_store = build_full_s3_object_store_for_provider(&provider).await?;
    let result = object_store
        .head_bucket(HeadBucketRequest {
            bucket: provider.bucket.clone(),
        })
        .await
        .map_err(map_object_store_route_error)?;
    Ok(Json(ProviderBucketResponse {
        provider_id,
        bucket: result.bucket,
        exists: result.exists,
    }))
}

pub(crate) async fn list_storage_provider_buckets(
    State(state): State<AdminStorageState>,
    Path(provider_id): Path<String>,
    Query(query): Query<ListProviderBucketsQuery>,
) -> Result<
    StorageListHttpResponse<ProviderBucketListItemResponse>,
    (StatusCode, Json<ProblemDetail>),
> {
    let page = parse_offset_page(query.page_size, query.page_token)?;
    let provider = get_active_provider(&state, &provider_id).await?;
    let configured_bucket = provider.bucket.clone();
    let object_store = build_full_s3_object_store_for_provider(&provider).await?;
    let result = object_store
        .list_buckets(ListBucketsRequest)
        .await
        .map_err(map_object_store_route_error)?;
    if result.items.len() > MAX_ADMIN_BUCKET_LIST_ITEMS {
        return Err(crate::error::problem(
            StatusCode::PAYLOAD_TOO_LARGE,
            "bucket inventory too large",
            format!(
                "account exposes more than {MAX_ADMIN_BUCKET_LIST_ITEMS} buckets; narrow credentials or contact platform support"
            ),
            sdkwork_drive_http::api_problem::SdkWorkResultCode::ValidationError,
        ));
    }
    let mut mapped_items: Vec<ProviderBucketListItemResponse> = result
        .items
        .into_iter()
        .map(|item| ProviderBucketListItemResponse {
            configured: item.bucket == configured_bucket,
            bucket: item.bucket,
            creation_date_epoch_ms: item.creation_date_epoch_ms,
        })
        .collect();
    mapped_items.sort_by(|left, right| left.bucket.cmp(&right.bucket));
    let start = page.offset as usize;
    let take = (page.limit as usize).saturating_add(1);
    let end = (start + take).min(mapped_items.len());
    let mut items = if start >= mapped_items.len() {
        Vec::new()
    } else {
        mapped_items[start..end].to_vec()
    };
    let next_page_token = next_page_token(&mut items, page);
    Ok(success_list_page_simple(items, page, next_page_token))
}

pub(crate) async fn create_storage_provider_bucket(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_id): Path<String>,
) -> Result<Json<ProviderBucketMutationResponse>, (StatusCode, Json<ProblemDetail>)> {
    let operator_id = ctx.resolve_operator_id()?;
    let provider = get_active_provider(&state, &provider_id).await?;
    let object_store = build_full_s3_object_store_for_provider(&provider).await?;
    let result = object_store
        .create_bucket(CreateBucketRequest {
            bucket: provider.bucket.clone(),
        })
        .await
        .map_err(map_object_store_route_error)?;
    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::BUCKET_CREATED,
        &provider_id,
        &operator_id,
    )
    .await?;
    Ok(Json(ProviderBucketMutationResponse {
        provider_id,
        bucket: result.bucket,
        changed: result.created,
    }))
}

pub(crate) async fn delete_storage_provider_bucket(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(provider_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let operator_id = ctx.resolve_operator_id()?;
    let provider = get_active_provider(&state, &provider_id).await?;
    let object_store = build_full_s3_object_store_for_provider(&provider).await?;
    let result = object_store
        .delete_bucket(DeleteBucketRequest {
            bucket: provider.bucket.clone(),
        })
        .await
        .map_err(map_object_store_route_error)?;
    record_storage_provider_audit(
        &state,
        admin_audit::storage_provider::BUCKET_DELETED,
        &provider_id,
        &operator_id,
    )
    .await?;
    let _deleted = result.deleted;
    Ok(no_content())
}
