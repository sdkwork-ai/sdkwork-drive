use crate::acl;
use crate::app_context::DriveRequestContext;
use crate::dto::*;
use crate::error::{
    internal_problem, map_service_error, not_found_problem, problem, service_error_kind,
    ProblemDetail, SdkWorkResultCode,
};
use crate::node_repository::find_active_node;
use crate::object_store::{AppDownloadSigner, UploadPartSignCommand};
use crate::state::AppState;
use crate::storage_keys::*;
use crate::time::current_epoch_ms;
use crate::uploader::prepare_uploader_command;
use crate::validators::*;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use sdkwork_drive_contract::drive::domain_events as drive_events;
use sdkwork_drive_observability::{elapsed_ms, events, start_timer};
use sdkwork_drive_uploader_service::service::SqlUploaderStore;
use sdkwork_drive_uploader_service::service::{
    DriveUploaderService, MarkUploaderPartUploadedCommand, UploaderTarget,
};
use sdkwork_drive_workspace_service::application::quota_enforcement::ensure_tenant_can_allocate_bytes;
use sdkwork_drive_workspace_service::application::upload_session_rules::{
    validate_completed_multipart_parts, validate_content_type, validate_sha256_checksum,
};
use sdkwork_drive_workspace_service::application::upload_session_service::{
    CancelDriveUploadSessionCommand, CompleteDriveUploadSessionCommand, DriveUploadSessionService,
    PrepareDriveUploadSessionCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::quota_store::SqlQuotaStore;
use sdkwork_drive_workspace_service::ports::upload_multipart_storage::DriveMultipartPart;

use crate::response::{
    success_created_command_data, success_created_resource, success_envelope, success_resource,
};
use crate::route_change::record_change;
use crate::upload_support::*;
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};

pub(crate) async fn create_upload_session(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Json(payload): Json<CreateUploadSessionRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<CreateUploadSessionResponse>>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let started = start_timer();
    // The client may send an `objectKey` hint, but the key is always derived from
    // the tenant/space/node/version so it cannot point outside the session scope.
    let _client_requested_object_key = payload.object_key.as_deref();
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    if payload.idempotency_key.trim().is_empty() {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "idempotencyKey is required",
            SdkWorkResultCode::ValidationError,
        ));
    }
    validate_optional_future_epoch_ms(Some(payload.expires_at_epoch_ms), "expiresAtEpochMs")?;

    // Node/space validation, `writer` authorization, idempotency, storage-target
    // resolution, multipart creation, version allocation and the
    // `upload_session.created` change all live in the workspace service. This
    // handler only parses, delegates and maps.
    let service = upload_session_service(&state);
    let created = match service
        .prepare(PrepareDriveUploadSessionCommand {
            session_id: payload.session_id.trim().to_string(),
            tenant_id,
            space_id: payload.space_id.trim().to_string(),
            node_id: payload.node_id.trim().to_string(),
            requested_storage_provider_id: payload.storage_provider_id,
            requested_bucket: payload.bucket,
            idempotency_key: payload.idempotency_key.trim().to_string(),
            operator_id,
            subject_type,
            subject_id,
            expires_at_epoch_ms: payload.expires_at_epoch_ms,
        })
        .await
    {
        Ok(created) => created,
        Err(error) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_UPLOAD_SESSIONS_CREATE,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = service_error_kind(&error)
            );
            return Err(map_service_error(error));
        }
    };
    sdkwork_drive_observability::observe_route!(
        event = events::APP_UPLOAD_SESSIONS_CREATE,
        result = "ok",
        latency_ms = elapsed_ms(started),
        state = created.state.as_str(),
        expires_at_epoch_ms = created.expires_at_epoch_ms,
        version = created.version
    );

    Ok(success_created_resource(CreateUploadSessionResponse::from(
        created,
    )))
}
pub(crate) async fn get_upload_session(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(upload_session_id): Path<String>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<UploadSessionMutationResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    let session = upload_session_service(&state)
        .retrieve(&tenant_id, &upload_session_id, &subject_type, &subject_id)
        .await
        .map_err(map_service_error)?;
    Ok(success_resource(UploadSessionMutationResponse::from(
        session,
    )))
}
pub(crate) async fn presign_upload_part(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((upload_session_id, part_no)): Path<(String, u16)>,
    Json(payload): Json<PresignUploadPartRequest>,
) -> Result<Json<SdkWorkApiResponse<PresignedUploadPartResponse>>, (StatusCode, Json<ProblemDetail>)>
{
    let tenant_id = ctx.resolve_tenant_id()?;
    if part_no == 0 || part_no > 10_000 {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "partNo must be between 1 and 10000",
            SdkWorkResultCode::ValidationError,
        ));
    }
    let upload_session = find_upload_session(&state.pool, &tenant_id, &upload_session_id).await?;
    validate_mutable_upload_session(&upload_session)?;
    ensure_upload_session_not_expired(&upload_session, current_epoch_ms())?;
    if let Some(upload_id) = payload.upload_id.as_deref() {
        let upload_id = upload_id.trim();
        if upload_id.is_empty() {
            return Err(problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                "uploadId must not be empty",
                SdkWorkResultCode::ValidationError,
            ));
        }
        if upload_id != upload_session.storage_upload_id {
            return Err(problem(
                StatusCode::CONFLICT,
                "conflict",
                "uploadId does not match upload session storage upload id",
                SdkWorkResultCode::Conflict,
            ));
        }
    }
    let node = find_active_node(&state.pool, &tenant_id, &upload_session.node_id).await?;
    acl::ensure_ctx_node_role(
        &state.pool,
        &ctx,
        &node.space_id,
        &upload_session.node_id,
        "writer",
    )
    .await?;
    let upload_id = upload_session.storage_upload_id.clone();
    let requested_ttl_seconds = validate_requested_ttl_seconds(
        payload.requested_ttl_seconds,
        120,
        30,
        300,
        "requestedTtlSeconds",
    )?;
    let expires_at_epoch_ms = current_epoch_ms() + i64::from(requested_ttl_seconds) * 1000;
    let operator_id = ctx.resolve_operator_id()?;
    let signed = AppDownloadSigner::new(state.pool.clone())
        .sign_upload_part(UploadPartSignCommand {
            storage_provider_id: upload_session.storage_provider_id.clone(),
            bucket: upload_session.bucket.clone(),
            object_key: upload_session.object_key.clone(),
            upload_id: upload_id.clone(),
            part_no,
            expires_at_epoch_ms,
        })
        .await
        .map_err(map_service_error)?;

    mark_upload_session_uploading(
        &state.pool,
        &tenant_id,
        &upload_session_id,
        &operator_id,
        current_epoch_ms(),
    )
    .await?;

    Ok(success_envelope(PresignedUploadPartResponse {
        upload_url: signed.raw_url,
        expires_at_epoch_ms: signed.expires_at_epoch_ms,
        method: signed.method,
        headers: signed.headers,
        part_no,
        upload_id,
    }))
}
pub(crate) async fn complete_upload_session(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(upload_session_id): Path<String>,
    Json(payload): Json<CompleteUploadSessionRequest>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<UploadSessionMutationResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    let parts = payload
        .parts
        .iter()
        .map(|part| DriveMultipartPart {
            part_no: part.part_no,
            etag: part.etag.clone(),
        })
        .collect::<Vec<_>>();
    // Boundary validation keeps the exact problem-detail wording the app API has
    // always returned; the service re-validates the same rules with its own field
    // names for in-process callers.
    validate_completed_multipart_parts(&parts).map_err(map_service_error)?;
    validate_content_type(&payload.content_type, "contentType").map_err(map_service_error)?;
    validate_sha256_checksum(&payload.checksum_sha256_hex, "checksumSha256Hex")
        .map_err(map_service_error)?;
    let content_length = payload.content_length.into_i64();
    if content_length < 0 {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "contentLength must be greater than or equal to 0",
            SdkWorkResultCode::ValidationError,
        ));
    }

    let updated = upload_session_service(&state)
        .complete(CompleteDriveUploadSessionCommand {
            tenant_id,
            upload_session_id,
            upload_id: payload.upload_id,
            content_type: payload.content_type,
            content_length,
            checksum_sha256_hex: payload.checksum_sha256_hex,
            parts,
            operator_id,
            subject_type,
            subject_id,
        })
        .await
        .map_err(map_service_error)?;
    Ok(success_resource(UploadSessionMutationResponse::from(
        updated,
    )))
}
pub(crate) async fn abort_upload_session(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(upload_session_id): Path<String>,
    Json(_payload): Json<NodeCommandRequest>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<UploadSessionMutationResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    let updated = upload_session_service(&state)
        .cancel(CancelDriveUploadSessionCommand {
            tenant_id,
            upload_session_id,
            operator_id,
            subject_type,
            subject_id,
        })
        .await
        .map_err(map_service_error)?;
    Ok(success_resource(UploadSessionMutationResponse::from(
        updated,
    )))
}

/// Build the in-process upload-session service over the host pool.
///
/// One constructor keeps the storage adapter (which owns the object-store
/// runtime) wired the same way for every handler.
fn upload_session_service(state: &AppState) -> DriveUploadSessionService<AppUploadMultipartStorage> {
    DriveUploadSessionService::new(state.pool.clone(), AppUploadMultipartStorage::new(state.clone()))
}
pub(crate) async fn prepare_uploader_upload(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Json(payload): Json<PrepareUploaderUploadRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<PrepareUploaderUploadResponse>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let started = start_timer();
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;

    let command = prepare_uploader_command(payload, &ctx, tenant_id, operator_id.clone())?;
    if let UploaderTarget::Space {
        space_id,
        parent_node_id,
        share_token,
    } = &command.target
    {
        if share_token.is_none() {
            acl::ensure_parent_writer(&state.pool, &ctx, space_id, parent_node_id.as_deref())
                .await?;
        }
    }
    ensure_tenant_can_allocate_bytes(
        &SqlQuotaStore::new(state.pool.clone()),
        &command.tenant_id,
        command.content_length,
    )
    .await
    .map_err(map_service_error)?;
    let service = DriveUploaderService::new(SqlUploaderStore::new(state.pool.clone()));
    let upload_item = service
        .prepare_upload(command)
        .await
        .map_err(map_service_error)?;
    let upload_session_id = upload_item
        .upload_session_id
        .as_deref()
        .ok_or_else(|| internal_problem("uploader upload session id is missing"))?;
    let mut upload_session =
        find_upload_session(&state.pool, &upload_item.tenant_id, upload_session_id).await?;

    if upload_session.storage_upload_id == upload_session.id {
        let target_version_no = next_storage_object_version_no(
            &state.pool,
            &upload_item.tenant_id,
            &upload_item.node_id,
        )
        .await?;
        let mut storage_target = resolve_storage_target(
            &state.pool,
            &upload_item.tenant_id,
            &upload_item.space_id,
            None,
            Some(&upload_session.bucket),
            &upload_item.node_id,
            &upload_item.id,
            target_version_no,
        )
        .await?;
        if let Some(object_key) = uploader_final_object_key(
            upload_session.object_key.as_str(),
            storage_target.object_key.as_str(),
        )? {
            storage_target.object_key = object_key;
        }
        let created_storage_upload =
            initiate_storage_multipart_upload(&state, &storage_target).await?;
        update_uploader_storage_target(
            &state.pool,
            &upload_item.tenant_id,
            &upload_item.id,
            upload_session_id,
            &storage_target,
            &created_storage_upload.upload_id,
        )
        .await?;
        upload_session =
            find_upload_session(&state.pool, &upload_item.tenant_id, upload_session_id).await?;
    }

    let upload_item =
        find_uploader_upload_item(&state.pool, &upload_item.tenant_id, &upload_item.id)
            .await?
            .ok_or_else(|| internal_problem("uploader upload item was not found after prepare"))?;

    record_change(
        &state.pool,
        &upload_item.tenant_id,
        &upload_item.space_id,
        Some(&upload_item.node_id),
        drive_events::uploader::UPLOAD_PREPARED,
        &operator_id,
    )
    .await?;

    sdkwork_drive_observability::observe_route!(
        event = events::APP_UPLOADER_UPLOADS_PREPARE,
        result = "ok",
        latency_ms = elapsed_ms(started),
        upload_item_id = upload_item.id.as_str(),
        space_id = upload_item.space_id.as_str()
    );

    Ok(success_created_command_data(
        PrepareUploaderUploadResponse {
            upload_item: UploaderUploadItemResponse::from(upload_item),
            upload_session: UploadSessionMutationResponse::from(upload_session),
        },
    ))
}
pub(crate) async fn mark_uploader_part_uploaded(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((upload_item_id, part_no)): Path<(String, i64)>,
    Json(payload): Json<MarkUploaderPartUploadedRequest>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<UploaderUploadPartResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let started = start_timer();
    let tenant_id = ctx.resolve_tenant_id()?;
    let upload_item = find_uploader_upload_item(&state.pool, &tenant_id, &upload_item_id)
        .await?
        .ok_or_else(|| not_found_problem("upload item not found"))?;
    acl::ensure_ctx_node_role(
        &state.pool,
        &ctx,
        &upload_item.space_id,
        &upload_item.node_id,
        "writer",
    )
    .await?;
    let upload_session_id = require_non_empty_text(payload.upload_session_id, "uploadSessionId")?;
    let service = DriveUploaderService::new(SqlUploaderStore::new(state.pool.clone()));
    let part = service
        .mark_part_uploaded(MarkUploaderPartUploadedCommand {
            id: format!("upload-part-{upload_item_id}-{part_no}"),
            tenant_id,
            upload_item_id,
            upload_session_id,
            part_no,
            offset_bytes: payload.offset_bytes.into_i64(),
            size_bytes: payload.size_bytes.into_i64(),
            etag: payload.etag,
            checksum_sha256_hex: payload.checksum_sha256_hex,
            uploaded_at_epoch_ms: payload
                .uploaded_at_epoch_ms
                .map(FlexibleI64::into_i64)
                .unwrap_or_else(current_epoch_ms),
        })
        .await
        .map_err(map_service_error)?;
    sdkwork_drive_observability::metrics::record_uploader_part_uploaded();
    sdkwork_drive_observability::observe_route!(
        event = events::APP_UPLOADER_PART_MARK_UPLOADED,
        result = "ok",
        latency_ms = elapsed_ms(started),
        upload_item_id = upload_item.id.as_str(),
        part_no = part_no
    );
    Ok(success_resource(UploaderUploadPartResponse::from(part)))
}
