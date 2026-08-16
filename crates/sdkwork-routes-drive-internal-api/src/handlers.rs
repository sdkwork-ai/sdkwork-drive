use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use sdkwork_drive_workspace_service::application::resource_resolution_service::{
    DriveResourceResolutionService, ResolveDriveResourceCommand,
};
use sdkwork_drive_workspace_service::application::root_scope_event_delivery_service::{
    DriveRootScopeEventDeliveryService, EnsureRootScopeEventDeliveryCommand,
};
use sdkwork_drive_workspace_service::application::root_scope_subscription_service::{
    DriveRootScopeSubscriptionService, EnsureKnowledgebaseRawScopeCommand,
    GetRootScopeSubscriptionCommand, SqlDriveKnowledgebaseRawScopeService,
};
use sdkwork_drive_workspace_service::application::website_root_event_delivery_service::{
    DriveWebsiteRootEventDeliveryService, EnsureWebsiteRootEventDeliveryCommand,
};
use sdkwork_drive_workspace_service::application::website_root_service::{
    DriveWebsiteRootService, GetWebsiteRootCommand,
};
use sdkwork_drive_workspace_service::domain::resource_resolution::{
    DriveResourceScopeKind, ResolvedDriveResource,
};
use sdkwork_drive_workspace_service::domain::root_scope_subscription::DriveRootScopeSubscription;
use sdkwork_drive_workspace_service::infrastructure::sql::resource_resolution_store::SqlResourceResolutionStore;
use sdkwork_drive_workspace_service::infrastructure::sql::root_scope_subscription_store::SqlRootScopeSubscriptionStore;
use sdkwork_drive_workspace_service::infrastructure::sql::website_root_store::SqlWebsiteRootStore;
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};
use sdkwork_web_core::RequireInternalApi;

use crate::dto::{
    CreateRootScopeSubscriptionRequest, DriveResourceResolutionResponse,
    EnsureRootScopeEventDeliveryRequest, EnsureWebsiteRootEventDeliveryRequest,
    ResolveDriveResourceRequest, RootScopeEventDeliveryResponse, RootScopeSubscriptionResponse,
    WebsiteRootEventDeliveryResponse, WebsiteRootResponse,
};
use crate::error::{
    invalid_parameter, map_service_error, missing_internal_principal, RouteProblem,
};
use crate::response::{resource, ResourceResponse};
use crate::state::InternalApiState;

pub async fn create_root_scope_subscription(
    State(state): State<InternalApiState>,
    RequireInternalApi(context): RequireInternalApi,
    Json(payload): Json<CreateRootScopeSubscriptionRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<RootScopeSubscriptionResponse>>>,
    ),
    RouteProblem,
> {
    let principal = context
        .principal
        .as_ref()
        .ok_or_else(missing_internal_principal)?;
    let result = SqlDriveKnowledgebaseRawScopeService::new(state.pool.clone())
        .ensure_knowledgebase_raw_scope(EnsureKnowledgebaseRawScopeCommand {
            tenant_id: principal.tenant_id().to_string(),
            space_id: payload.space_id,
            knowledge_base_id: payload.knowledge_base_id,
            operator_id: principal.user_id().to_string(),
        })
        .await
        .map_err(map_service_error)?;
    let status = if result.created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, resource(map_subscription(result.subscription))))
}

pub async fn retrieve_root_scope_subscription(
    State(state): State<InternalApiState>,
    RequireInternalApi(context): RequireInternalApi,
    Path(subscription_uuid): Path<String>,
) -> Result<ResourceResponse<RootScopeSubscriptionResponse>, RouteProblem> {
    validate_uuid(&subscription_uuid, "subscriptionUuid")?;
    let principal = context
        .principal
        .as_ref()
        .ok_or_else(missing_internal_principal)?;
    let subscription = DriveRootScopeSubscriptionService::new(SqlRootScopeSubscriptionStore::new(
        state.pool.clone(),
    ))
    .get_subscription(GetRootScopeSubscriptionCommand {
        tenant_id: principal.tenant_id().to_string(),
        subscription_uuid,
    })
    .await
    .map_err(map_service_error)?;
    Ok(resource(map_subscription(subscription)))
}

pub async fn retrieve_website_root(
    State(state): State<InternalApiState>,
    RequireInternalApi(context): RequireInternalApi,
    Path(website_root_uuid): Path<String>,
) -> Result<ResourceResponse<WebsiteRootResponse>, RouteProblem> {
    validate_uuid(&website_root_uuid, "websiteRootUuid")?;
    let principal = context
        .principal
        .as_ref()
        .ok_or_else(missing_internal_principal)?;
    let website_root = DriveWebsiteRootService::new(SqlWebsiteRootStore::new(state.pool.clone()))
        .get_root(GetWebsiteRootCommand {
            tenant_id: principal.tenant_id().to_string(),
            root_uuid: website_root_uuid,
        })
        .await
        .map_err(map_service_error)?;
    Ok(resource(WebsiteRootResponse {
        uuid: website_root.uuid,
        space_id: website_root.space_id,
        source_root_mode: website_root.source_root_mode.as_str().to_ascii_uppercase(),
        content_mode: website_root.content_mode.as_str().to_ascii_uppercase(),
        active_generation: website_root.active_generation.to_string(),
        root_status: website_root.root_status.to_ascii_uppercase(),
        capabilities: vec![
            "STATIC_CONTENT".to_string(),
            "BYTE_RANGE".to_string(),
            "CONDITIONAL_REQUESTS".to_string(),
        ],
        version: website_root.version.to_string(),
        updated_at: normalize_timestamp(&website_root.updated_at),
    }))
}

pub async fn ensure_root_scope_event_delivery(
    State(state): State<InternalApiState>,
    RequireInternalApi(context): RequireInternalApi,
    Path(subscription_uuid): Path<String>,
    Json(payload): Json<EnsureRootScopeEventDeliveryRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<RootScopeEventDeliveryResponse>>>,
    ),
    RouteProblem,
> {
    validate_uuid(&subscription_uuid, "subscriptionUuid")?;
    let principal = context
        .principal
        .as_ref()
        .ok_or_else(missing_internal_principal)?;
    let expected_caller_app_id = std::env::var("SDKWORK_DRIVE_ROOT_SCOPE_EVENT_CALLER_APP_ID")
        .unwrap_or_else(|_| "knowledgebase".to_string());
    if principal.app_id() != expected_caller_app_id {
        return Err(crate::error::forbidden_internal_caller());
    }
    let expiration_epoch_ms =
        parse_positive_i64(&payload.expiration_epoch_ms, "expirationEpochMs")?;
    let result = DriveRootScopeEventDeliveryService::new(
        sdkwork_drive_workspace_service::infrastructure::sql::provider_event_delivery_store::SqlProviderEventDeliveryStore::new(
            state.pool.clone(),
        ),
    )
    .ensure_delivery(EnsureRootScopeEventDeliveryCommand {
        tenant_id: principal.tenant_id().to_string(),
        subscription_uuid,
        address: payload.address,
        verification_token: payload.verification_token,
        expiration_epoch_ms,
        operator_id: principal.user_id().to_string(),
    })
    .await
    .map_err(map_service_error)?;
    let status = if result.created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((
        status,
        resource(RootScopeEventDeliveryResponse {
            channel_id: result.delivery.channel_id,
            subscription_uuid: result.delivery.provider_resource_uuid,
            address: result.delivery.address,
            expiration_epoch_ms: result.delivery.expiration_epoch_ms.to_string(),
            lifecycle_status: result.delivery.lifecycle_status.to_ascii_uppercase(),
            version: result.delivery.version.to_string(),
            created_at: normalize_timestamp(&result.delivery.created_at),
            updated_at: normalize_timestamp(&result.delivery.updated_at),
        }),
    ))
}

pub async fn ensure_website_root_event_delivery(
    State(state): State<InternalApiState>,
    RequireInternalApi(context): RequireInternalApi,
    Path((website_root_uuid, channel_id)): Path<(String, String)>,
    Json(payload): Json<EnsureWebsiteRootEventDeliveryRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<WebsiteRootEventDeliveryResponse>>>,
    ),
    RouteProblem,
> {
    validate_uuid(&website_root_uuid, "websiteRootUuid")?;
    let principal = context
        .principal
        .as_ref()
        .ok_or_else(missing_internal_principal)?;
    let expected_caller_app_id = std::env::var("SDKWORK_DRIVE_WEBSITE_EVENT_CALLER_APP_ID")
        .unwrap_or_else(|_| "sdkwork-deployments".to_string());
    if principal.app_id() != expected_caller_app_id {
        return Err(crate::error::forbidden_internal_caller());
    }
    let expiration_epoch_ms =
        parse_positive_i64(&payload.expiration_epoch_ms, "expirationEpochMs")?;
    let result = DriveWebsiteRootEventDeliveryService::new(
        sdkwork_drive_workspace_service::infrastructure::sql::provider_event_delivery_store::SqlProviderEventDeliveryStore::new(
            state.pool.clone(),
        ),
    )
    .ensure_delivery(EnsureWebsiteRootEventDeliveryCommand {
        tenant_id: principal.tenant_id().to_string(),
        website_root_uuid,
        channel_id,
        address: payload.address,
        verification_token: payload.verification_token,
        expiration_epoch_ms,
        operator_id: principal.user_id().to_string(),
    })
    .await
    .map_err(map_service_error)?;
    let status = if result.created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((
        status,
        resource(WebsiteRootEventDeliveryResponse {
            channel_id: result.delivery.channel_id,
            website_root_uuid: result.delivery.provider_resource_uuid,
            address: result.delivery.address,
            expiration_epoch_ms: result.delivery.expiration_epoch_ms.to_string(),
            lifecycle_status: result.delivery.lifecycle_status.to_ascii_uppercase(),
            version: result.delivery.version.to_string(),
            created_at: normalize_timestamp(&result.delivery.created_at),
            updated_at: normalize_timestamp(&result.delivery.updated_at),
        }),
    ))
}

pub async fn resolve_drive_resource(
    State(state): State<InternalApiState>,
    RequireInternalApi(context): RequireInternalApi,
    Json(payload): Json<ResolveDriveResourceRequest>,
) -> Result<ResourceResponse<DriveResourceResolutionResponse>, RouteProblem> {
    validate_uuid(&payload.scope_uuid, "scopeUuid")?;
    let principal = context
        .principal
        .as_ref()
        .ok_or_else(missing_internal_principal)?;
    let resolved = resolve_resource(
        &state,
        principal.tenant_id(),
        &payload.scope_type,
        payload.scope_uuid,
        payload.relative_path,
        payload.pinned_generation,
        payload.pinned_node_version_id,
    )
    .await?;
    Ok(resource(map_resolution(&resolved)?))
}

pub(crate) async fn resolve_resource(
    state: &InternalApiState,
    tenant_id: &str,
    scope_type: &str,
    scope_uuid: String,
    relative_path: String,
    pinned_generation: Option<String>,
    pinned_node_version_id: Option<String>,
) -> Result<ResolvedDriveResource, RouteProblem> {
    let scope_kind = parse_scope_kind(scope_type)?;
    let pinned_generation = pinned_generation
        .map(|value| parse_positive_i64(&value, "pinnedGeneration"))
        .transpose()?;
    DriveResourceResolutionService::new(SqlResourceResolutionStore::new(state.pool.clone()))
        .resolve(ResolveDriveResourceCommand {
            tenant_id: tenant_id.to_string(),
            scope_kind,
            scope_uuid,
            relative_path,
            pinned_generation,
            pinned_node_version_id,
        })
        .await
        .map_err(map_service_error)
}

pub(crate) fn parse_scope_kind(value: &str) -> Result<DriveResourceScopeKind, RouteProblem> {
    match value {
        "WEBSITE_ROOT" => Ok(DriveResourceScopeKind::WebsiteRoot),
        "ROOT_SCOPE_SUBSCRIPTION" => Ok(DriveResourceScopeKind::RootScopeSubscription),
        _ => Err(invalid_parameter(
            "scopeType must be WEBSITE_ROOT or ROOT_SCOPE_SUBSCRIPTION",
        )),
    }
}

fn parse_positive_i64(value: &str, field_name: &str) -> Result<i64, RouteProblem> {
    let parsed = value
        .parse::<i64>()
        .map_err(|_| invalid_parameter(format!("{field_name} must be a positive int64 string")))?;
    if parsed < 1 || parsed.to_string() != value {
        return Err(invalid_parameter(format!(
            "{field_name} must be a canonical positive int64 string"
        )));
    }
    Ok(parsed)
}

fn validate_uuid(value: &str, field_name: &str) -> Result<(), RouteProblem> {
    uuid::Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| invalid_parameter(format!("{field_name} must be a UUID")))
}

fn map_subscription(subscription: DriveRootScopeSubscription) -> RootScopeSubscriptionResponse {
    RootScopeSubscriptionResponse {
        uuid: subscription.uuid,
        space_id: subscription.space_id,
        consumer_kind: subscription.consumer_kind.to_ascii_uppercase(),
        consumer_resource_id: subscription.consumer_resource_id,
        root_node_id: subscription.root_node_id,
        scope_status: subscription.scope_status.to_ascii_uppercase(),
        version: subscription.version.to_string(),
        created_at: normalize_timestamp(&subscription.created_at),
        updated_at: normalize_timestamp(&subscription.updated_at),
    }
}

pub(crate) fn map_resolution(
    resource: &ResolvedDriveResource,
) -> Result<DriveResourceResolutionResponse, RouteProblem> {
    if !resource.checksum_sha256_hex.starts_with("sha256:") {
        return Err(map_service_error(
            sdkwork_drive_workspace_service::DriveServiceError::Internal(
                "resolved Drive resource checksum is invalid".to_string(),
            ),
        ));
    }
    Ok(DriveResourceResolutionResponse {
        scope_type: resource.scope_kind.as_str().to_string(),
        scope_uuid: resource.scope_uuid.clone(),
        scope_generation: resource.scope_generation.to_string(),
        normalized_relative_path: resource.relative_path.clone(),
        resource_type: resource.resource_type.clone(),
        node_id: resource.node_id.clone(),
        logical_node_version_id: resource.node_version_id.clone(),
        version_no: resource.version_no.to_string(),
        checksum_sha256_hex: resource.checksum_sha256_hex.clone(),
        etag: format!("\"{}\"", resource.checksum_sha256_hex),
        content_type: resource.content_type.clone(),
        content_length: resource.content_length.to_string(),
        last_modified: normalize_timestamp(&resource.last_modified),
        scope_status: resource.scope_status.clone(),
        node_status: resource.node_status.clone(),
        eligibility: resource.eligibility.clone(),
    })
}

pub(crate) fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.with_timezone(&Utc))
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f")
                .ok()
                .map(|value| value.and_utc())
        })
}

fn normalize_timestamp(value: &str) -> String {
    parse_timestamp(value)
        .map(|value| value.to_rfc3339_opts(SecondsFormat::Millis, true))
        .unwrap_or_else(|| value.to_string())
}
