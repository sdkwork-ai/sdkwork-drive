use crate::acl;
use crate::acl_sql;
use crate::app_context::DriveRequestContext;
use crate::dto::{
    AssetActionRequest, AssetCollectionItemResponse, AssetCollectionResponse, AssetItemResponse,
    AssetRelationResponse, CreateAssetCollectionItemRequest, CreateAssetCollectionRequest,
    CreateAssetRelationRequest, CreateAssetRequest, ListAssetCollectionsQuery, ListAssetsQuery,
    MediaResourceResponse, PageRequest, UpdateAssetRequest, ASSET_NODE_SELECT_COLUMNS,
};
use crate::error::{
    internal_problem, internal_sql_error, is_unique_constraint_error, map_service_error,
    not_found_problem, problem, ProblemDetail, SdkWorkResultCode,
};
use crate::hashing::sha256_raw_hex_separated;
use crate::ids::next_drive_id;
use crate::mappers::map_node_row;
use crate::node_repository::{find_active_node, find_node};
use crate::response::{
    no_content, success_created_resource, success_list_page_simple, success_resource,
    DriveListHttpResponse,
};
use crate::state::AppState;
use crate::validators::{normalize_optional_text, require_non_empty_text, validate_page_size_i64};
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use sdkwork_drive_contract::api::pagination_cursor::{decode_offset_cursor, encode_offset_cursor};
use sdkwork_drive_contract::drive::domain_events as drive_events;
use sdkwork_drive_contract::{
    build_drive_backed_media_resource, BuildDriveBackedMediaResourceInput, DriveNodeId,
    DriveSpaceId, DriveUri,
};
use sdkwork_drive_workspace_service::infrastructure::change_recorder::{
    self, RecordDriveChangeCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::begin_transaction_sql;
use sdkwork_drive_workspace_service::infrastructure::sql::managed_website_tree_guard::ensure_managed_website_node_mutation_allowed;
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};
use serde_json::{json, Value};
use sqlx::PgPool;
use sqlx::Row;

const PROP_DESCRIPTION: &str = "global.asset.description";
const PROP_TAGS: &str = "global.asset.tags";
const PROP_ARCHIVED: &str = "global.asset.archived";
const PROP_ARCHIVE_REASON: &str = "global.asset.archive.reason";
const PROP_VISIBILITY: &str = "global.asset.visibility";
const COLLECTION_KEY_PREFIX: &str = "global.asset.collection.";
const COLLECTION_ITEM_KEY_PREFIX: &str = "global.asset.collection.item.";
const RELATION_KEY_PREFIX: &str = "global.asset.relation.";
const CATALOG_NODE_NAME: &str = "__asset_catalog__";
const PROPERTY_VISIBILITY: &str = "private";

#[derive(Debug, Clone)]
struct AssetNodeRow {
    node: crate::dto::DriveNodeResponse,
    created_at: String,
    updated_at: String,
}

pub(crate) async fn list_assets(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<ListAssetsQuery>,
) -> Result<DriveListHttpResponse<AssetItemResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let page = parse_asset_page_request(query.page_size, query.cursor)?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    let kind_filter = normalize_optional_text(query.kind);
    let source_type_filter = normalize_optional_text(query.source_type);
    let needle = normalize_optional_text(query.q).map(|value| format!("%{value}%"));

    let pool = state.pool.clone();
    let tenant_id_for_fetch = tenant_id.clone();
    let subject_type_for_fetch = subject_type.clone();
    let subject_id_for_fetch = subject_id.clone();
    let kind_filter_for_fetch = kind_filter.clone();
    let source_type_filter_for_fetch = source_type_filter.clone();
    let needle_for_fetch = needle.clone();
    let reader_acl_predicate = acl_sql::node_reader_visible_sql("dr_drive_node", "$2", "$3");

    let (rows, next_page_token) = acl::paginate_offset_limited_items(
        page,
        move |scan_offset, batch_limit| {
            let pool = pool.clone();
            let tenant_id = tenant_id_for_fetch.clone();
            let subject_type = subject_type_for_fetch.clone();
            let subject_id = subject_id_for_fetch.clone();
            let kind_filter = kind_filter_for_fetch.clone();
            let source_type_filter = source_type_filter_for_fetch.clone();
            let needle = needle_for_fetch.clone();
            let reader_acl_predicate = reader_acl_predicate.clone();
            async move {
                let mut sql = format!(
                    "SELECT {ASSET_NODE_SELECT_COLUMNS}
                     FROM dr_drive_node
                     WHERE tenant_id=$1
                       AND node_type IN ('file', 'virtual_reference')
                       AND lifecycle_status='active'
                       AND content_state IN ('ready', 'empty')
                       AND ({reader_acl_predicate})"
                );
                let mut bind_index = 4_u8;
                if kind_filter.is_some() {
                    sql.push_str(&format!(" AND head_content_type_group=${bind_index}"));
                    bind_index += 1;
                }
                if source_type_filter.is_some() {
                    sql.push_str(&format!(" AND source=${bind_index}"));
                    bind_index += 1;
                }
                if needle.is_some() {
                    sql.push_str(&format!(" AND node_name LIKE ${bind_index}"));
                    bind_index += 1;
                }
                sql.push_str(&format!(
                    " ORDER BY updated_at DESC, id ASC
                     LIMIT ${bind_index} OFFSET ${}",
                    bind_index + 1
                ));

                let mut query = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
                    .bind(&tenant_id)
                    .bind(&subject_type)
                    .bind(&subject_id);
                if let Some(kind) = kind_filter.as_deref() {
                    query = query.bind(map_asset_kind_to_content_group(kind));
                }
                if let Some(source_type) = source_type_filter.as_deref() {
                    query = query.bind(source_type);
                }
                if let Some(needle) = needle.as_deref() {
                    query = query.bind(needle);
                }
                let rows = query
                    .bind(batch_limit as i64)
                    .bind(scan_offset)
                    .fetch_all(&pool)
                    .await
                    .map_err(internal_sql_error("list asset dr_drive_node failed"))?;
                Ok(rows)
            }
        },
        map_asset_node_row,
    )
    .await?;

    let items = rows
        .into_iter()
        .map(|row| asset_item_from_node_row(&row, None, None, None, false))
        .collect();

    Ok(success_list_page_simple(items, page, next_page_token))
}

pub(crate) async fn create_asset(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Json(payload): Json<CreateAssetRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<AssetItemResponse>>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let organization_id = ctx.organization_id.clone();

    let node = if let Some(drive_node_id) = normalize_optional_text(payload.drive_node_id) {
        let node = find_active_node(&state.pool, &tenant_id, &drive_node_id).await?;
        ensure_asset_eligible_node(&node)?;
        acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node.id, "reader").await?;
        node
    } else if let Some(virtual_reference) = payload.virtual_reference.clone() {
        create_virtual_reference_asset_node(
            &state.pool,
            CreateVirtualReferenceAssetNode {
                context: &ctx,
                tenant_id: &tenant_id,
                operator_id: &operator_id,
                virtual_reference: &virtual_reference,
                title: payload.title.as_deref(),
                scene: payload.scene.as_deref(),
                source: payload.source.as_deref(),
            },
        )
        .await?
    } else {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "driveNodeId or virtualReference is required",
            SdkWorkResultCode::ValidationError,
        ));
    };

    if let Some(title) = normalize_optional_text(payload.title) {
        update_node_name_if_needed(
            &state.pool,
            &tenant_id,
            &node.id,
            &node.space_id,
            &title,
            &operator_id,
        )
        .await?;
    }
    if let Some(scene) = normalize_optional_text(payload.scene) {
        update_node_scene(&state.pool, &tenant_id, &node.id, &scene, &operator_id).await?;
    }
    if let Some(source) = normalize_optional_text(payload.source) {
        update_node_source(&state.pool, &tenant_id, &node.id, &source, &operator_id).await?;
    }
    if let Some(description) = payload.description.as_deref() {
        upsert_asset_property(
            &state.pool,
            &tenant_id,
            &node.id,
            &node.space_id,
            PROP_DESCRIPTION,
            description,
            &operator_id,
        )
        .await?;
    }
    if let Some(tags) = payload.tags.as_ref() {
        let tags_json = serde_json::to_string(tags).map_err(|error| {
            problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                format!("tags payload is invalid: {error}"),
                SdkWorkResultCode::ValidationError,
            )
        })?;
        upsert_asset_property(
            &state.pool,
            &tenant_id,
            &node.id,
            &node.space_id,
            PROP_TAGS,
            &tags_json,
            &operator_id,
        )
        .await?;
    }

    let item = load_asset_item(
        &state.pool,
        &tenant_id,
        &node.id,
        organization_id.as_deref(),
    )
    .await?;
    Ok(success_created_resource(item))
}

pub(crate) async fn get_asset(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(asset_id): Path<String>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<AssetItemResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let node = find_node(&state.pool, &tenant_id, &asset_id).await?;
    ensure_asset_eligible_node(&node)?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &asset_id, "reader").await?;
    let item = load_asset_item(
        &state.pool,
        &tenant_id,
        &asset_id,
        ctx.organization_id.as_deref(),
    )
    .await?;
    Ok(success_resource(item))
}

pub(crate) async fn update_asset(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(asset_id): Path<String>,
    Json(payload): Json<UpdateAssetRequest>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<AssetItemResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &asset_id).await?;
    ensure_asset_eligible_node(&node)?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &asset_id, "writer").await?;

    if let Some(title) = normalize_optional_text(payload.title) {
        update_node_name_if_needed(
            &state.pool,
            &tenant_id,
            &asset_id,
            &node.space_id,
            &title,
            &operator_id,
        )
        .await?;
    }
    if let Some(scene) = normalize_optional_text(payload.scene) {
        update_node_scene(&state.pool, &tenant_id, &asset_id, &scene, &operator_id).await?;
    }
    if let Some(source) = normalize_optional_text(payload.source) {
        update_node_source(&state.pool, &tenant_id, &asset_id, &source, &operator_id).await?;
    }
    if let Some(description) = payload.description.as_deref() {
        upsert_asset_property(
            &state.pool,
            &tenant_id,
            &asset_id,
            &node.space_id,
            PROP_DESCRIPTION,
            description,
            &operator_id,
        )
        .await?;
    }
    if let Some(tags) = payload.tags.as_ref() {
        let tags_json = serde_json::to_string(tags).map_err(|error| {
            problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                format!("tags payload is invalid: {error}"),
                SdkWorkResultCode::ValidationError,
            )
        })?;
        upsert_asset_property(
            &state.pool,
            &tenant_id,
            &asset_id,
            &node.space_id,
            PROP_TAGS,
            &tags_json,
            &operator_id,
        )
        .await?;
    }
    if let Some(visibility) = normalize_optional_text(payload.visibility) {
        validate_asset_visibility(&visibility)?;
        upsert_asset_property(
            &state.pool,
            &tenant_id,
            &asset_id,
            &node.space_id,
            PROP_VISIBILITY,
            &visibility,
            &operator_id,
        )
        .await?;
    }

    let item = load_asset_item(
        &state.pool,
        &tenant_id,
        &asset_id,
        ctx.organization_id.as_deref(),
    )
    .await?;
    Ok(success_resource(item))
}

pub(crate) async fn archive_asset(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(asset_id): Path<String>,
    Json(payload): Json<AssetActionRequest>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<AssetItemResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    set_asset_archived_flag(&state, &ctx, &asset_id, true, payload.reason.as_deref()).await
}

pub(crate) async fn restore_asset(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(asset_id): Path<String>,
    Json(_payload): Json<AssetActionRequest>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<AssetItemResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    set_asset_archived_flag(&state, &ctx, &asset_id, false, None).await
}

pub(crate) async fn list_asset_collections(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<ListAssetCollectionsQuery>,
) -> Result<DriveListHttpResponse<AssetCollectionResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let user_id = ctx.user_id.clone();
    let page = parse_asset_page_request(query.page_size, query.cursor)?;
    let catalog = ensure_asset_catalog_anchor(&state.pool, &tenant_id, &user_id, &ctx).await?;

    let rows = sqlx::query(
        "SELECT property_key, property_value
         FROM dr_drive_node_property
         WHERE tenant_id=$1
           AND node_id=$2
           AND visibility=$3
           AND lifecycle_status='active'
           AND property_key LIKE $4
           AND property_key NOT LIKE $5
         ORDER BY property_key ASC
         LIMIT $6 OFFSET $7",
    )
    .bind(&tenant_id)
    .bind(&catalog.node_id)
    .bind(PROPERTY_VISIBILITY)
    .bind(format!("{COLLECTION_KEY_PREFIX}%"))
    .bind(format!("{COLLECTION_ITEM_KEY_PREFIX}%"))
    .bind(page.limit + 1)
    .bind(page.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_sql_error("list asset collections failed"))?;

    let mut items = rows
        .iter()
        .filter_map(|row| {
            let key: String = row.get("property_key");
            let value: String = row.get("property_value");
            parse_collection_from_property(&tenant_id, &user_id, &key, &value)
        })
        .collect::<Vec<_>>();
    let next_page_token = next_asset_cursor(&mut items, page);
    Ok(success_list_page_simple(items, page, next_page_token))
}

pub(crate) async fn create_asset_collection(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Json(payload): Json<CreateAssetCollectionRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<AssetCollectionResponse>>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let user_id = ctx.user_id.clone();
    let operator_id = ctx.resolve_operator_id()?;
    let title = require_non_empty_text(payload.title, "title")?;
    let catalog = ensure_asset_catalog_anchor(&state.pool, &tenant_id, &user_id, &ctx).await?;
    let collection_id = next_drive_id("acol");
    let visibility =
        normalize_optional_text(payload.visibility).unwrap_or_else(|| "private".to_string());
    validate_asset_visibility(&visibility)?;
    let collection_type =
        normalize_optional_text(payload.collection_type).unwrap_or_else(|| "manual".to_string());
    let now = current_timestamp_text();
    let organization_id = ctx.organization_id.clone();
    let body = json!({
        "id": collection_id,
        "tenantId": tenant_id,
        "organizationId": organization_id,
        "userId": user_id,
        "title": title,
        "description": payload.description,
        "collectionType": collection_type,
        "visibility": visibility,
        "lifecycleStatus": "active",
        "createdAt": now,
        "updatedAt": now,
    });
    let property_key = format!("{COLLECTION_KEY_PREFIX}{collection_id}");
    upsert_asset_property(
        &state.pool,
        &tenant_id,
        &catalog.node_id,
        &catalog.space_id,
        &property_key,
        &body.to_string(),
        &operator_id,
    )
    .await?;
    let collection =
        parse_collection_from_property(&tenant_id, &user_id, &property_key, &body.to_string())
            .ok_or_else(|| not_found_problem("collection not found"))?;
    Ok(success_created_resource(collection))
}

pub(crate) async fn add_asset_collection_item(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(collection_id): Path<String>,
    Json(payload): Json<CreateAssetCollectionItemRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<AssetCollectionItemResponse>>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let user_id = ctx.user_id.clone();
    let operator_id = ctx.resolve_operator_id()?;
    let asset_id = require_non_empty_text(payload.asset_id, "assetId")?;
    let catalog = ensure_asset_catalog_anchor(&state.pool, &tenant_id, &user_id, &ctx).await?;
    load_collection_for_user(
        &state.pool,
        &tenant_id,
        &user_id,
        &catalog.node_id,
        &collection_id,
    )
    .await?;

    let asset_node = find_active_node(&state.pool, &tenant_id, &asset_id).await?;
    ensure_asset_eligible_node(&asset_node)?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &asset_node.space_id, &asset_id, "reader").await?;

    let item_id = build_collection_item_id(&collection_id, &asset_id);
    let property_key = format!("{COLLECTION_ITEM_KEY_PREFIX}{collection_id}.{asset_id}");
    let body = json!({
        "id": item_id,
        "tenantId": tenant_id,
        "collectionId": collection_id,
        "assetId": asset_id,
        "sortOrder": payload.sort_order,
    });
    upsert_asset_property(
        &state.pool,
        &tenant_id,
        &catalog.node_id,
        &catalog.space_id,
        &property_key,
        &body.to_string(),
        &operator_id,
    )
    .await?;

    Ok(success_created_resource(AssetCollectionItemResponse {
        id: item_id,
        tenant_id,
        collection_id,
        asset_id,
        sort_order: payload.sort_order,
    }))
}

pub(crate) async fn delete_asset_collection_item(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((collection_id, item_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let user_id = ctx.user_id.clone();
    let operator_id = ctx.resolve_operator_id()?;
    let catalog = ensure_asset_catalog_anchor(&state.pool, &tenant_id, &user_id, &ctx).await?;
    load_collection_for_user(
        &state.pool,
        &tenant_id,
        &user_id,
        &catalog.node_id,
        &collection_id,
    )
    .await?;

    let property_key = find_collection_item_property_key(
        &state.pool,
        &tenant_id,
        &catalog.node_id,
        &collection_id,
        &item_id,
    )
    .await?;
    let affected = sqlx::query(
        "UPDATE dr_drive_node_property
         SET lifecycle_status='deleted', updated_by=$1, updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$2 AND node_id=$3 AND property_key=$4 AND visibility=$5 AND lifecycle_status='active'",
    )
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&catalog.node_id)
    .bind(&property_key)
    .bind(PROPERTY_VISIBILITY)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("delete asset collection item failed"))?
    .rows_affected();
    let _deleted = affected > 0;
    Ok(no_content())
}

pub(crate) async fn create_asset_relation(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(asset_id): Path<String>,
    Json(payload): Json<CreateAssetRelationRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<AssetRelationResponse>>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let relation_type = require_non_empty_text(payload.relation_type, "relationType")?;
    validate_relation_type(&relation_type)?;

    let node = find_active_node(&state.pool, &tenant_id, &asset_id).await?;
    ensure_asset_eligible_node(&node)?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &asset_id, "writer").await?;

    if let Some(related_asset_id) = normalize_optional_text(payload.related_asset_id.clone()) {
        let related = find_active_node(&state.pool, &tenant_id, &related_asset_id).await?;
        ensure_asset_eligible_node(&related)?;
        acl::ensure_ctx_node_role(
            &state.pool,
            &ctx,
            &related.space_id,
            &related_asset_id,
            "reader",
        )
        .await?;
    }

    let relation_id = next_drive_id("arel");
    let property_key = format!("{RELATION_KEY_PREFIX}{relation_id}");
    let body = json!({
        "id": relation_id,
        "tenantId": tenant_id,
        "assetId": asset_id,
        "relatedAssetId": payload.related_asset_id,
        "relationType": relation_type,
        "sourceDomain": payload.source_domain,
        "sourceResourceType": payload.source_resource_type,
        "sourceResourceId": payload.source_resource_id,
        "metadata": payload.metadata,
        "lifecycleStatus": "active",
    });
    upsert_asset_property(
        &state.pool,
        &tenant_id,
        &asset_id,
        &node.space_id,
        &property_key,
        &body.to_string(),
        &operator_id,
    )
    .await?;

    Ok(success_created_resource(
        parse_relation_from_property(&property_key, &body.to_string())
            .ok_or_else(|| not_found_problem("relation not found"))?,
    ))
}

pub(crate) async fn delete_asset_relation(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((asset_id, relation_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &asset_id).await?;
    ensure_asset_eligible_node(&node)?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &asset_id, "writer").await?;

    let property_key = format!("{RELATION_KEY_PREFIX}{relation_id}");
    let affected = sqlx::query(
        "UPDATE dr_drive_node_property
         SET lifecycle_status='deleted', updated_by=$1, updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$2 AND node_id=$3 AND property_key=$4 AND visibility=$5 AND lifecycle_status='active'",
    )
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&asset_id)
    .bind(&property_key)
    .bind(PROPERTY_VISIBILITY)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("delete asset relation failed"))?
    .rows_affected();
    let _deleted = affected > 0;
    Ok(no_content())
}

pub(crate) async fn legacy_asset_upload_route_gone() -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::GONE,
        "legacy route retired",
        "legacy asset upload endpoints are not available; use Drive uploader APIs",
        SdkWorkResultCode::Gone,
    )
}

pub(crate) async fn asset_method_not_allowed() -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::METHOD_NOT_ALLOWED,
        "method not allowed",
        "Drive assets API method is not available on this route",
        SdkWorkResultCode::MethodNotAllowed,
    )
}

#[derive(Debug, Clone)]
struct CatalogAnchor {
    node_id: String,
    space_id: String,
}

fn parse_asset_page_request(
    page_size: Option<i64>,
    cursor: Option<String>,
) -> Result<PageRequest, (StatusCode, Json<ProblemDetail>)> {
    let limit = validate_page_size_i64(
        page_size,
        crate::constants::DEFAULT_LIST_PAGE_SIZE,
        1,
        crate::constants::MAX_LIST_PAGE_SIZE,
        "page_size",
    )?;
    let offset = decode_offset_cursor(cursor.as_deref()).map_err(|_| {
        problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "cursor is invalid",
            SdkWorkResultCode::ValidationError,
        )
    })?;
    Ok(PageRequest { limit, offset })
}

fn next_asset_cursor<T>(items: &mut Vec<T>, page: PageRequest) -> Option<String> {
    if items.len() as i64 > page.limit {
        items.pop();
        encode_offset_cursor(page.offset + page.limit)
    } else {
        None
    }
}

fn map_asset_node_row(row: &sqlx::postgres::PgRow) -> AssetNodeRow {
    AssetNodeRow {
        node: map_node_row(row),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn ensure_asset_eligible_node(
    node: &crate::dto::DriveNodeResponse,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if !matches!(node.node_type.as_str(), "file" | "virtual_reference") {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "node type is not eligible for global assets",
            SdkWorkResultCode::ValidationError,
        ));
    }
    if node.lifecycle_status == "deleted" {
        return Err(not_found_problem("asset not found"));
    }
    Ok(())
}

fn map_asset_kind_to_content_group(kind: &str) -> &str {
    match kind.trim().to_ascii_lowercase().as_str() {
        "image" => "image",
        "video" => "video",
        "audio" | "voice" => "audio",
        "document" => "document",
        "model" => "binary",
        "archive" | "file" => "binary",
        _ => kind,
    }
}

fn derive_asset_kind(node: &crate::dto::DriveNodeResponse) -> String {
    if node.node_type == "virtual_reference" {
        return "other".to_string();
    }
    match node
        .content_type_group
        .as_deref()
        .unwrap_or("binary")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "image" => "image",
        "video" => "video",
        "audio" => "audio",
        "document" | "text" => "document",
        "model" => "model",
        "archive" => "file",
        "binary" => "file",
        _ => "other",
    }
    .to_string()
}

fn derive_source_type(source: Option<&str>) -> Option<String> {
    let value = source?.trim();
    if value.is_empty() {
        return None;
    }
    if value.starts_with("ai:") || value.contains("generated") {
        return Some("ai_generated".to_string());
    }
    if value.starts_with("import:") {
        return Some("imported".to_string());
    }
    if value.starts_with("system:") {
        return Some("system".to_string());
    }
    if value.starts_with("edit:") {
        return Some("edited".to_string());
    }
    Some("upload".to_string())
}

fn asset_item_from_node_row(
    row: &AssetNodeRow,
    description: Option<String>,
    tags: Option<Vec<String>>,
    visibility: Option<String>,
    archived: bool,
) -> AssetItemResponse {
    let drive_uri = DriveUri::new(
        &DriveSpaceId::new(&row.node.space_id),
        &DriveNodeId::new(&row.node.id),
    )
    .to_string();
    let resource_snapshot = build_resource_snapshot(&row.node);
    let lifecycle_status = if archived {
        "archived".to_string()
    } else {
        row.node.lifecycle_status.clone()
    };
    AssetItemResponse {
        asset_id: row.node.id.clone(),
        id: Some(row.node.id.clone()),
        tenant_id: row.node.tenant_id.clone(),
        organization_id: None,
        user_id: None,
        drive_space_id: row.node.space_id.clone(),
        drive_node_id: row.node.id.clone(),
        drive_uri,
        node_type: row.node.node_type.clone(),
        asset_kind: derive_asset_kind(&row.node),
        title: row.node.node_name.clone(),
        description,
        scene: row.node.scene.clone(),
        source: row.node.source.clone(),
        source_type: derive_source_type(row.node.source.as_deref()),
        tags,
        visibility,
        lifecycle_status,
        resource_snapshot,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
    }
}

fn build_resource_snapshot(node: &crate::dto::DriveNodeResponse) -> Option<MediaResourceResponse> {
    if node.node_type == "virtual_reference" {
        return None;
    }
    let kind = derive_asset_kind(node);
    let media = build_drive_backed_media_resource(BuildDriveBackedMediaResourceInput {
        space_id: &DriveSpaceId::new(&node.space_id),
        node_id: &DriveNodeId::new(&node.id),
        kind: &kind,
        file_name: Some(node.node_name.as_str()),
        mime_type: node.content_type.as_deref(),
        size_bytes: node.content_length,
        checksum_sha256_hex: None,
        space_type: Some(node.space_type.as_str()),
        node_version: Some(node.version.to_string().as_str()),
    });
    Some(MediaResourceResponse {
        id: media.id,
        kind: media.kind,
        source: media.source,
        uri: Some(media.uri),
        file_name: media.file_name,
        mime_type: media.mime_type,
        size_bytes: media.size_bytes,
    })
}

async fn load_asset_item(
    pool: &PgPool,
    tenant_id: &str,
    asset_id: &str,
    organization_id: Option<&str>,
) -> Result<AssetItemResponse, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {ASSET_NODE_SELECT_COLUMNS}
         FROM dr_drive_node
         WHERE tenant_id=$1 AND id=$2 AND lifecycle_status != 'deleted'",
    )))
    .bind(tenant_id)
    .bind(asset_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("load asset dr_drive_node failed"))?;
    let Some(row) = row else {
        return Err(not_found_problem("asset not found"));
    };
    let node_row = map_asset_node_row(&row);
    ensure_asset_eligible_node(&node_row.node)?;

    let description = load_asset_property_text(pool, tenant_id, asset_id, PROP_DESCRIPTION).await?;
    let tags = load_asset_property_json(pool, tenant_id, asset_id, PROP_TAGS)
        .await?
        .and_then(|value| serde_json::from_value::<Vec<String>>(value).ok());
    let visibility = load_asset_property_text(pool, tenant_id, asset_id, PROP_VISIBILITY).await?;
    let archived = load_asset_property_text(pool, tenant_id, asset_id, PROP_ARCHIVED)
        .await?
        .is_some_and(|value| value == "true");

    let mut item = asset_item_from_node_row(&node_row, description, tags, visibility, archived);
    item.organization_id = organization_id.map(str::to_string);
    if let Some(user_id) =
        load_space_owner_user_id(pool, tenant_id, &node_row.node.space_id).await?
    {
        item.user_id = Some(user_id);
    }
    Ok(item)
}

async fn load_space_owner_user_id(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
) -> Result<Option<String>, (StatusCode, Json<ProblemDetail>)> {
    let owner_subject_id: Option<String> = sqlx::query_scalar(
        "SELECT owner_subject_id
         FROM dr_drive_space
         WHERE tenant_id=$1 AND id=$2 AND owner_subject_type='user' AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("load dr_drive_space owner failed"))?;
    Ok(owner_subject_id)
}

async fn load_asset_property_text(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
    property_key: &str,
) -> Result<Option<String>, (StatusCode, Json<ProblemDetail>)> {
    let value: Option<String> = sqlx::query_scalar(
        "SELECT property_value
         FROM dr_drive_node_property
         WHERE tenant_id=$1 AND node_id=$2 AND property_key=$3 AND visibility=$4 AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(node_id)
    .bind(property_key)
    .bind(PROPERTY_VISIBILITY)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("load asset property failed"))?;
    Ok(value.filter(|value| !value.trim().is_empty()))
}

async fn load_asset_property_json(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
    property_key: &str,
) -> Result<Option<Value>, (StatusCode, Json<ProblemDetail>)> {
    let Some(raw) = load_asset_property_text(pool, tenant_id, node_id, property_key).await? else {
        return Ok(None);
    };
    serde_json::from_str(&raw).map(Some).map_err(|error| {
        problem(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error",
            format!("asset property json is invalid: {error}"),
            SdkWorkResultCode::InternalError,
        )
    })
}

async fn clear_asset_property(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
    property_key: &str,
    operator_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    sqlx::query(
        "UPDATE dr_drive_node_property
         SET lifecycle_status='deleted', updated_by=$1, updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$2 AND node_id=$3 AND property_key=$4 AND visibility=$5 AND lifecycle_status='active'",
    )
    .bind(operator_id)
    .bind(tenant_id)
    .bind(node_id)
    .bind(property_key)
    .bind(PROPERTY_VISIBILITY)
    .execute(pool)
    .await
    .map_err(internal_sql_error("clear asset property failed"))?;
    Ok(())
}

async fn upsert_asset_property(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
    space_id: &str,
    property_key: &str,
    property_value: &str,
    operator_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let property_id = build_node_property_id(tenant_id, node_id, property_key, PROPERTY_VISIBILITY);
    sqlx::query(
        "INSERT INTO dr_drive_node_property (
            id, tenant_id, node_id, property_key, property_value, visibility,
            lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $7)
         ON CONFLICT(tenant_id, node_id, property_key, visibility) DO UPDATE SET
            property_value=excluded.property_value,
            lifecycle_status='active',
            updated_by=excluded.updated_by,
            updated_at=CURRENT_TIMESTAMP,
            version=dr_drive_node_property.version + 1",
    )
    .bind(&property_id)
    .bind(tenant_id)
    .bind(node_id)
    .bind(property_key)
    .bind(property_value)
    .bind(PROPERTY_VISIBILITY)
    .bind(operator_id)
    .execute(pool)
    .await
    .map_err(internal_sql_error("upsert asset property failed"))?;
    record_change(
        pool,
        tenant_id,
        space_id,
        Some(node_id),
        drive_events::node_property::SET,
        operator_id,
    )
    .await
}

fn build_node_property_id(
    tenant_id: &str,
    node_id: &str,
    property_key: &str,
    visibility: &str,
) -> String {
    let digest = sha256_raw_hex_separated(&[
        tenant_id.trim().as_bytes(),
        node_id.trim().as_bytes(),
        visibility.trim().as_bytes(),
        property_key.trim().as_bytes(),
    ]);
    format!("p:{}", &digest[..62])
}

async fn record_change(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    node_id: Option<&str>,
    event_type: &str,
    actor_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    change_recorder::record_drive_change(
        pool,
        RecordDriveChangeCommand {
            tenant_id,
            space_id,
            node_id,
            event_type,
            actor_id,
        },
    )
    .await
    .map_err(crate::error::map_service_error)
}

async fn set_asset_archived_flag(
    state: &AppState,
    ctx: &DriveRequestContext,
    asset_id: &str,
    archived: bool,
    archive_reason: Option<&str>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<AssetItemResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let node = find_active_node(&state.pool, &tenant_id, asset_id).await?;
    ensure_asset_eligible_node(&node)?;
    acl::ensure_ctx_node_role(&state.pool, ctx, &node.space_id, asset_id, "writer").await?;

    if archived {
        upsert_asset_property(
            &state.pool,
            &tenant_id,
            asset_id,
            &node.space_id,
            PROP_ARCHIVED,
            "true",
            &operator_id,
        )
        .await?;
        if let Some(reason) = archive_reason
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            upsert_asset_property(
                &state.pool,
                &tenant_id,
                asset_id,
                &node.space_id,
                PROP_ARCHIVE_REASON,
                reason,
                &operator_id,
            )
            .await?;
        } else {
            clear_asset_property(
                &state.pool,
                &tenant_id,
                asset_id,
                PROP_ARCHIVE_REASON,
                &operator_id,
            )
            .await?;
        }
    } else {
        clear_asset_property(
            &state.pool,
            &tenant_id,
            asset_id,
            PROP_ARCHIVED,
            &operator_id,
        )
        .await?;
        clear_asset_property(
            &state.pool,
            &tenant_id,
            asset_id,
            PROP_ARCHIVE_REASON,
            &operator_id,
        )
        .await?;
        record_change(
            &state.pool,
            &tenant_id,
            &node.space_id,
            Some(asset_id),
            drive_events::node_property::DELETED,
            &operator_id,
        )
        .await?;
    }

    let item = load_asset_item(
        &state.pool,
        &tenant_id,
        asset_id,
        ctx.organization_id.as_deref(),
    )
    .await?;
    Ok(success_resource(item))
}

async fn update_node_name_if_needed(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
    space_id: &str,
    title: &str,
    operator_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    update_asset_node_metadata(
        pool,
        tenant_id,
        node_id,
        AssetNodeMetadataUpdate::Name(title),
        operator_id,
    )
    .await?;
    record_change(
        pool,
        tenant_id,
        space_id,
        Some(node_id),
        drive_events::node::UPDATED,
        operator_id,
    )
    .await
}

async fn update_node_scene(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
    scene: &str,
    operator_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let space_id = update_asset_node_metadata(
        pool,
        tenant_id,
        node_id,
        AssetNodeMetadataUpdate::Scene(scene),
        operator_id,
    )
    .await?;
    record_change(
        pool,
        tenant_id,
        &space_id,
        Some(node_id),
        drive_events::node::UPDATED,
        operator_id,
    )
    .await
}

async fn update_node_source(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
    source: &str,
    operator_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let space_id = update_asset_node_metadata(
        pool,
        tenant_id,
        node_id,
        AssetNodeMetadataUpdate::Source(source),
        operator_id,
    )
    .await?;
    record_change(
        pool,
        tenant_id,
        &space_id,
        Some(node_id),
        drive_events::node::UPDATED,
        operator_id,
    )
    .await
}

enum AssetNodeMetadataUpdate<'a> {
    Name(&'a str),
    Scene(&'a str),
    Source(&'a str),
}

async fn update_asset_node_metadata(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
    update: AssetNodeMetadataUpdate<'_>,
    operator_id: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let mut connection = pool.acquire().await.map_err(|error| {
        internal_problem(format!(
            "acquire asset node update transaction connection failed: {error}"
        ))
    })?;
    sqlx::query(begin_transaction_sql())
        .execute(&mut *connection)
        .await
        .map_err(internal_sql_error(
            "begin asset node update transaction failed",
        ))?;
    let update_result: Result<String, (StatusCode, Json<ProblemDetail>)> = async {
        ensure_managed_website_node_mutation_allowed(&mut connection, tenant_id, node_id)
            .await
            .map_err(map_service_error)?;
        let space_id: String =
            sqlx::query_scalar("SELECT space_id FROM dr_drive_node WHERE tenant_id=$1 AND id=$2")
                .bind(tenant_id)
                .bind(node_id)
                .fetch_one(&mut *connection)
                .await
                .map_err(internal_sql_error("load node space_id failed"))?;
        let affected = match update {
            AssetNodeMetadataUpdate::Name(value) => sqlx::query(
                "UPDATE dr_drive_node
                 SET node_name=$1, updated_by=$2, updated_at=CURRENT_TIMESTAMP, version=version + 1
                 WHERE tenant_id=$3 AND id=$4 AND lifecycle_status != 'deleted'",
            )
            .bind(value)
            .bind(operator_id)
            .bind(tenant_id)
            .bind(node_id)
            .execute(&mut *connection)
            .await
            .map_err(internal_sql_error("update asset title failed"))?
            .rows_affected(),
            AssetNodeMetadataUpdate::Scene(value) => sqlx::query(
                "UPDATE dr_drive_node
                 SET scene=$1, updated_by=$2, updated_at=CURRENT_TIMESTAMP, version=version + 1
                 WHERE tenant_id=$3 AND id=$4 AND lifecycle_status != 'deleted'",
            )
            .bind(value)
            .bind(operator_id)
            .bind(tenant_id)
            .bind(node_id)
            .execute(&mut *connection)
            .await
            .map_err(internal_sql_error("update asset scene failed"))?
            .rows_affected(),
            AssetNodeMetadataUpdate::Source(value) => sqlx::query(
                "UPDATE dr_drive_node
                 SET source=$1, updated_by=$2, updated_at=CURRENT_TIMESTAMP, version=version + 1
                 WHERE tenant_id=$3 AND id=$4 AND lifecycle_status != 'deleted'",
            )
            .bind(value)
            .bind(operator_id)
            .bind(tenant_id)
            .bind(node_id)
            .execute(&mut *connection)
            .await
            .map_err(internal_sql_error("update asset source failed"))?
            .rows_affected(),
        };
        if affected == 0 {
            return Err(not_found_problem("asset node not found"));
        }
        Ok(space_id)
    }
    .await;
    match update_result {
        Ok(space_id) => {
            sqlx::query("COMMIT")
                .execute(&mut *connection)
                .await
                .map_err(internal_sql_error(
                    "commit asset node update transaction failed",
                ))?;
            Ok(space_id)
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            Err(error)
        }
    }
}

struct CreateVirtualReferenceAssetNode<'a> {
    context: &'a DriveRequestContext,
    tenant_id: &'a str,
    operator_id: &'a str,
    virtual_reference: &'a Value,
    title: Option<&'a str>,
    scene: Option<&'a str>,
    source: Option<&'a str>,
}

async fn create_virtual_reference_asset_node(
    pool: &PgPool,
    request: CreateVirtualReferenceAssetNode<'_>,
) -> Result<crate::dto::DriveNodeResponse, (StatusCode, Json<ProblemDetail>)> {
    let CreateVirtualReferenceAssetNode {
        context: ctx,
        tenant_id,
        operator_id,
        virtual_reference,
        title,
        scene,
        source,
    } = request;
    let space_id = if let Some(space_id) = virtual_reference
        .get("driveSpaceId")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    {
        space_id
    } else {
        resolve_personal_space_id(pool, tenant_id, &ctx.user_id)
            .await?
            .ok_or_else(|| {
                problem(
                    StatusCode::BAD_REQUEST,
                    "validation failed",
                    "virtualReference.driveSpaceId or a personal space is required",
                    SdkWorkResultCode::ValidationError,
                )
            })?
    };

    acl::ensure_parent_writer(pool, ctx, &space_id, None).await?;

    let node_name = title
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            virtual_reference
                .get("title")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .unwrap_or("virtual-asset");
    let node_id = next_drive_id("vref");
    let scene_value = scene.or_else(|| {
        virtual_reference
            .get("sourceDomain")
            .and_then(Value::as_str)
    });
    let source_value: Option<String> = source.map(str::to_string).or_else(|| {
        virtual_reference
            .get("sourceResourceType")
            .and_then(Value::as_str)
            .map(|resource_type| format!("external:{resource_type}"))
    });

    let mut connection = pool.acquire().await.map_err(|error| {
        internal_problem(format!(
            "acquire virtual asset transaction connection failed: {error}"
        ))
    })?;
    sqlx::query(begin_transaction_sql())
        .execute(&mut *connection)
        .await
        .map_err(internal_sql_error("begin virtual asset transaction failed"))?;
    let insert_result: Result<(), (StatusCode, Json<ProblemDetail>)> = async {
        sdkwork_drive_workspace_service::infrastructure::sql::managed_website_tree_guard::ensure_managed_website_parent_mutation_allowed(
            &mut connection,
            tenant_id,
            &space_id,
            None,
        )
        .await
        .map_err(map_service_error)?;
        sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name, scene, source,
            content_state, lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, NULL, 'virtual_reference', $4, $5, $6, 'empty', 'active', 1, $7, $7)",
    )
    .bind(&node_id)
    .bind(tenant_id)
    .bind(&space_id)
    .bind(node_name)
    .bind(scene_value)
    .bind(source_value)
    .bind(operator_id)
        .execute(&mut *connection)
        .await
        .map_err(|error| {
            if is_unique_constraint_error(&error) {
                return problem(
                    StatusCode::CONFLICT,
                    "conflict",
                    "node id already exists",
                    SdkWorkResultCode::Conflict,
                );
            }
            internal_problem(format!(
                "insert virtual_reference dr_drive_node failed: {error}"
            ))
        })?;
        Ok(())
    }
    .await;
    match insert_result {
        Ok(()) => sqlx::query("COMMIT")
            .execute(&mut *connection)
            .await
            .map_err(internal_sql_error(
                "commit virtual asset transaction failed",
            ))?,
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            return Err(error);
        }
    };
    drop(connection);

    record_change(
        pool,
        tenant_id,
        &space_id,
        Some(&node_id),
        drive_events::node::CREATED,
        operator_id,
    )
    .await?;
    find_active_node(pool, tenant_id, &node_id).await
}

async fn resolve_personal_space_id(
    pool: &PgPool,
    tenant_id: &str,
    user_id: &str,
) -> Result<Option<String>, (StatusCode, Json<ProblemDetail>)> {
    sqlx::query_scalar(
        "SELECT id
         FROM dr_drive_space
         WHERE tenant_id=$1
           AND owner_subject_type='user'
           AND owner_subject_id=$2
           AND space_type='personal'
           AND lifecycle_status='active'
         ORDER BY created_at ASC
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("resolve personal dr_drive_space failed"))
}

async fn ensure_asset_catalog_anchor(
    pool: &PgPool,
    tenant_id: &str,
    user_id: &str,
    ctx: &DriveRequestContext,
) -> Result<CatalogAnchor, (StatusCode, Json<ProblemDetail>)> {
    let space_id = resolve_personal_space_id(pool, tenant_id, user_id)
        .await?
        .ok_or_else(|| {
            problem(
                StatusCode::NOT_FOUND,
                "not found",
                "personal space is required for asset collections",
                SdkWorkResultCode::NotFound,
            )
        })?;
    acl::ensure_parent_writer(pool, ctx, &space_id, None).await?;

    let catalog_node_id = format!("asset_catalog_{user_id}");
    if find_node(pool, tenant_id, &catalog_node_id).await.is_ok() {
        return Ok(CatalogAnchor {
            node_id: catalog_node_id,
            space_id,
        });
    }

    let operator_id = ctx.resolve_operator_id()?;
    if find_node(pool, tenant_id, &catalog_node_id).await.is_err() {
        let insert_result = sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
             ) VALUES ($1, $2, $3, NULL, 'virtual_reference', $4, 'empty', 'active', 1, $5, $5)",
        )
        .bind(&catalog_node_id)
        .bind(tenant_id)
        .bind(&space_id)
        .bind(CATALOG_NODE_NAME)
        .bind(&operator_id)
        .execute(pool)
        .await;
        if let Err(error) = insert_result {
            if !is_unique_constraint_error(&error) {
                return Err(crate::error::internal_problem(format!(
                    "insert asset catalog dr_drive_node failed: {error}"
                )));
            }
        } else {
            record_change(
                pool,
                tenant_id,
                &space_id,
                Some(&catalog_node_id),
                drive_events::node::CREATED,
                &operator_id,
            )
            .await?;
        }
    }
    Ok(CatalogAnchor {
        node_id: catalog_node_id,
        space_id,
    })
}

fn parse_collection_from_property(
    tenant_id: &str,
    user_id: &str,
    property_key: &str,
    property_value: &str,
) -> Option<AssetCollectionResponse> {
    if !property_key.starts_with(COLLECTION_KEY_PREFIX)
        || property_key.starts_with(COLLECTION_ITEM_KEY_PREFIX)
    {
        return None;
    }
    let payload: Value = serde_json::from_str(property_value).ok()?;
    Some(AssetCollectionResponse {
        id: payload
            .get("id")
            .and_then(Value::as_str)
            .or_else(|| property_key.strip_prefix(COLLECTION_KEY_PREFIX))
            .map(str::to_string)?,
        tenant_id: payload
            .get("tenantId")
            .and_then(Value::as_str)
            .unwrap_or(tenant_id)
            .to_string(),
        organization_id: payload
            .get("organizationId")
            .and_then(Value::as_str)
            .map(str::to_string),
        user_id: payload
            .get("userId")
            .and_then(Value::as_str)
            .unwrap_or(user_id)
            .to_string(),
        title: payload.get("title").and_then(Value::as_str)?.to_string(),
        description: payload
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_string),
        collection_type: payload
            .get("collectionType")
            .and_then(Value::as_str)
            .map(str::to_string),
        visibility: payload
            .get("visibility")
            .and_then(Value::as_str)
            .map(str::to_string),
        lifecycle_status: payload
            .get("lifecycleStatus")
            .and_then(Value::as_str)
            .unwrap_or("active")
            .to_string(),
        created_at: payload
            .get("createdAt")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        updated_at: payload
            .get("updatedAt")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
    })
}

async fn load_collection_for_user(
    pool: &PgPool,
    tenant_id: &str,
    user_id: &str,
    catalog_node_id: &str,
    collection_id: &str,
) -> Result<AssetCollectionResponse, (StatusCode, Json<ProblemDetail>)> {
    let property_key = format!("{COLLECTION_KEY_PREFIX}{collection_id}");
    let value: Option<String> = sqlx::query_scalar(
        "SELECT property_value
         FROM dr_drive_node_property
         WHERE tenant_id=$1 AND node_id=$2 AND property_key=$3 AND visibility=$4 AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(catalog_node_id)
    .bind(&property_key)
    .bind(PROPERTY_VISIBILITY)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("load asset collection failed"))?;
    let Some(value) = value else {
        return Err(not_found_problem("collection not found"));
    };
    parse_collection_from_property(tenant_id, user_id, &property_key, &value)
        .ok_or_else(|| not_found_problem("collection not found"))
}

fn parse_relation_from_property(
    property_key: &str,
    property_value: &str,
) -> Option<AssetRelationResponse> {
    if !property_key.starts_with(RELATION_KEY_PREFIX) {
        return None;
    }
    let payload: Value = serde_json::from_str(property_value).ok()?;
    Some(AssetRelationResponse {
        id: payload
            .get("id")
            .and_then(Value::as_str)
            .or_else(|| property_key.strip_prefix(RELATION_KEY_PREFIX))
            .map(str::to_string)?,
        tenant_id: payload.get("tenantId").and_then(Value::as_str)?.to_string(),
        asset_id: payload.get("assetId").and_then(Value::as_str)?.to_string(),
        related_asset_id: payload
            .get("relatedAssetId")
            .and_then(Value::as_str)
            .map(str::to_string),
        relation_type: payload
            .get("relationType")
            .and_then(Value::as_str)?
            .to_string(),
        source_domain: payload
            .get("sourceDomain")
            .and_then(Value::as_str)
            .map(str::to_string),
        source_resource_type: payload
            .get("sourceResourceType")
            .and_then(Value::as_str)
            .map(str::to_string),
        source_resource_id: payload
            .get("sourceResourceId")
            .and_then(Value::as_str)
            .map(str::to_string),
        metadata: payload.get("metadata").cloned(),
        lifecycle_status: payload
            .get("lifecycleStatus")
            .and_then(Value::as_str)
            .unwrap_or("active")
            .to_string(),
    })
}

async fn find_collection_item_property_key(
    pool: &PgPool,
    tenant_id: &str,
    catalog_node_id: &str,
    collection_id: &str,
    item_id: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let property_key_prefix = format!("{COLLECTION_ITEM_KEY_PREFIX}{collection_id}.%");
    let row = sqlx::query(
        "SELECT property_key
         FROM dr_drive_node_property
         WHERE tenant_id=$1
           AND node_id=$2
           AND visibility=$3
           AND lifecycle_status='active'
           AND property_key LIKE $4
           AND (property_value::jsonb->>'id') = $5
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(catalog_node_id)
    .bind(PROPERTY_VISIBILITY)
    .bind(&property_key_prefix)
    .bind(item_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("find asset collection item failed"))?;

    let Some(row) = row else {
        return Err(not_found_problem("collection item not found"));
    };
    Ok(row.get("property_key"))
}

fn build_collection_item_id(collection_id: &str, asset_id: &str) -> String {
    let digest = sha256_raw_hex_separated(&[collection_id.as_bytes(), asset_id.as_bytes()]);
    format!("acitem_{}", &digest[..24])
}

fn validate_asset_visibility(visibility: &str) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if matches!(visibility, "private" | "organization" | "public") {
        return Ok(());
    }
    Err(problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "visibility is invalid",
        SdkWorkResultCode::ValidationError,
    ))
}

fn validate_relation_type(relation_type: &str) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if matches!(
        relation_type,
        "derived_from"
            | "variant_of"
            | "used_by"
            | "references"
            | "collection_cover"
            | "external_ref"
    ) {
        return Ok(());
    }
    Err(problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "relationType is invalid",
        SdkWorkResultCode::ValidationError,
    ))
}

fn current_timestamp_text() -> String {
    chrono::Utc::now().to_rfc3339()
}
