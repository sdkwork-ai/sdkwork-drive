use crate::acl;
use crate::app_context::DriveRequestContext;
use crate::dto::*;
use crate::error::{
    internal_problem, internal_sql_error, map_service_error, not_found_problem, problem,
    service_error_kind, ProblemDetail, SdkWorkResultCode,
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
use sdkwork_drive_workspace_service::application::upload_service::{
    CreateUploadSessionCommand, DriveUploadService,
};
use sdkwork_drive_workspace_service::domain::uploader::content_type_group_for;
use sdkwork_drive_workspace_service::infrastructure::sql::node_head_metadata::{
    apply_file_node_head_snapshot_in_transaction, FileNodeHeadSnapshot,
};
use sdkwork_drive_workspace_service::infrastructure::sql::quota_store::SqlQuotaStore;
use sdkwork_drive_workspace_service::infrastructure::sql::upload_session_store::SqlUploadSessionStore;

use crate::response::{
    success_created_command_data, success_created_resource, success_envelope, success_resource,
};
use crate::route_change::record_change;
use crate::upload_support::*;
use sdkwork_drive_workspace_service::infrastructure::outbox_dispatch::trigger_immediate_outbox_dispatch;
use sdkwork_drive_workspace_service::infrastructure::sql::begin_transaction_sql;
use sdkwork_drive_workspace_service::infrastructure::sql::managed_website_tree_guard::ensure_managed_website_node_mutation_allowed;
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
    let _client_requested_object_key = payload.object_key.as_deref();
    let session_id = payload.session_id.trim();
    let tenant_id = ctx.resolve_tenant_id()?;
    let space_id = payload.space_id.trim();
    let node_id = payload.node_id.trim();
    let idempotency_key = payload.idempotency_key.trim();
    let operator_id = ctx.resolve_operator_id()?;
    if idempotency_key.is_empty() {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "idempotencyKey is required",
            SdkWorkResultCode::ValidationError,
        ));
    }
    validate_optional_future_epoch_ms(Some(payload.expires_at_epoch_ms), "expiresAtEpochMs")?;
    let node = find_active_node(&state.pool, &tenant_id, node_id).await?;
    if node.space_id != space_id {
        return Err(not_found_problem("node not found"));
    }
    if node.node_type != "file" {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "nodeId must reference an active file",
            SdkWorkResultCode::ValidationError,
        ));
    }
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, node_id, "writer").await?;
    if let Some(existing) = find_upload_session_by_idempotency(
        &state.pool,
        &tenant_id,
        space_id,
        node_id,
        idempotency_key,
    )
    .await?
    {
        return Ok(success_created_resource(CreateUploadSessionResponse::from(
            existing,
        )));
    }
    ensure_upload_session_id_available(&state.pool, session_id).await?;
    let target_version_no =
        next_storage_object_version_no(&state.pool, &tenant_id, node_id).await?;
    let storage_target = resolve_storage_target(
        &state.pool,
        &tenant_id,
        space_id,
        payload.bucket.as_deref(),
        node_id,
        session_id,
        target_version_no,
    )
    .await?;
    let created_storage_upload = initiate_storage_multipart_upload(&state, &storage_target).await?;
    let service = DriveUploadService::new(SqlUploadSessionStore::new(state.pool.clone()));
    let created_result = service
        .create_upload_session(CreateUploadSessionCommand {
            session_id: session_id.to_string(),
            tenant_id,
            space_id: space_id.to_string(),
            node_id: node_id.to_string(),
            bucket: storage_target.bucket,
            object_key: storage_target.object_key,
            storage_provider_id: storage_target.provider_id,
            storage_upload_id: Some(created_storage_upload.upload_id),
            idempotency_key: idempotency_key.to_string(),
            operator_id: operator_id.clone(),
            expires_at_epoch_ms: payload.expires_at_epoch_ms,
        })
        .await;
    let created = match created_result {
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
    let latency_ms = elapsed_ms(started);
    let session_state = upload_session_state_as_str(&created.state);
    sdkwork_drive_observability::observe_route!(
        event = events::APP_UPLOAD_SESSIONS_CREATE,
        result = "ok",
        latency_ms = latency_ms,
        state = session_state,
        expires_at_epoch_ms = created.expires_at_epoch_ms,
        version = created.version
    );

    record_change(
        &state.pool,
        &created.tenant_id,
        &created.space_id,
        Some(&created.node_id),
        drive_events::upload_session::CREATED,
        &operator_id,
    )
    .await?;

    Ok(success_created_resource(CreateUploadSessionResponse {
        id: created.id,
        tenant_id: created.tenant_id,
        space_id: created.space_id,
        node_id: created.node_id,
        bucket: created.bucket,
        object_key: created.object_key,
        idempotency_key: created.idempotency_key,
        storage_provider_id: created.storage_provider_id,
        storage_upload_id: created.storage_upload_id,
        state: session_state.to_string(),
        expires_at_epoch_ms: created.expires_at_epoch_ms,
        version: created.version,
    }))
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
    let upload_session = find_upload_session(&state.pool, &tenant_id, &upload_session_id).await?;
    let node = find_active_node(&state.pool, &tenant_id, &upload_session.node_id).await?;
    acl::ensure_ctx_node_role(
        &state.pool,
        &ctx,
        &node.space_id,
        &upload_session.node_id,
        "reader",
    )
    .await?;
    Ok(success_resource(UploadSessionMutationResponse::from(
        upload_session,
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

    update_upload_session_state(
        &state.pool,
        &tenant_id,
        &upload_session_id,
        "uploading",
        &operator_id,
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
    validate_completed_multipart_parts(&payload.parts)?;
    let content_type = validate_content_type(&payload.content_type, "contentType")?;
    let checksum_sha256_hex =
        validate_sha256_checksum(&payload.checksum_sha256_hex, "checksumSha256Hex")?;
    let content_length = payload.content_length.into_i64();
    if content_length < 0 {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "contentLength must be greater than or equal to 0",
            SdkWorkResultCode::ValidationError,
        ));
    }

    let operator_id = ctx.resolve_operator_id()?;
    let upload_session = find_upload_session(&state.pool, &tenant_id, &upload_session_id).await?;
    validate_mutable_upload_session(&upload_session)?;
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
    let node_content_state_before_completion =
        read_node_content_state(&state.pool, &tenant_id, &upload_session.node_id).await?;
    let uploader_upload_item =
        find_uploader_upload_item_by_session(&state.pool, &tenant_id, &upload_session_id).await?;
    let completed_object_plan =
        plan_completed_storage_object_insert(&state.pool, &tenant_id, &upload_session).await?;
    claim_upload_session_completion(&state.pool, &upload_session, &operator_id).await?;
    if let Err(error) =
        complete_storage_multipart_upload(&state, &upload_session, &payload.parts).await
    {
        let _ = update_upload_session_state(
            &state.pool,
            &tenant_id,
            &upload_session_id,
            "uploading",
            &operator_id,
        )
        .await;
        return Err(error);
    }

    let head_snapshot = if let Some(upload_item) = uploader_upload_item.as_ref() {
        FileNodeHeadSnapshot {
            file_extension: upload_item.file_extension.clone(),
            content_type: content_type.to_string(),
            content_type_group: upload_item.content_type_group.clone(),
            content_length,
            version_no: completed_object_plan.version_no,
            checksum_sha256_hex: checksum_sha256_hex.to_string(),
        }
    } else {
        FileNodeHeadSnapshot {
            file_extension: None,
            content_type: content_type.to_string(),
            content_type_group: content_type_group_for(content_type).to_string(),
            content_length,
            version_no: completed_object_plan.version_no,
            checksum_sha256_hex: checksum_sha256_hex.to_string(),
        }
    };

    let mut connection = state.pool.acquire().await.map_err(|error| {
        internal_problem(format!(
            "acquire upload completion transaction connection failed: {error}"
        ))
    })?;
    sqlx::query(begin_transaction_sql())
        .execute(&mut *connection)
        .await
        .map_err(internal_sql_error(
            "begin upload completion transaction failed",
        ))?;

    let tx_result: Result<(), (StatusCode, Json<ProblemDetail>)> = async {
        ensure_managed_website_node_mutation_allowed(
            &mut connection,
            &tenant_id,
            &upload_session.node_id,
        )
        .await
        .map_err(map_service_error)?;
        sqlx::query(
            "INSERT INTO dr_drive_storage_object (
                id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
                scene, source, content_type, content_length, checksum_sha256_hex, lifecycle_status,
                created_by, updated_by
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'active', $13, $13)",
        )
        .bind(&completed_object_plan.id)
        .bind(&tenant_id)
        .bind(&upload_session.node_id)
        .bind(completed_object_plan.version_no)
        .bind(&upload_session.storage_provider_id)
        .bind(&upload_session.bucket)
        .bind(&upload_session.object_key)
        .bind(
            uploader_upload_item
                .as_ref()
                .and_then(|item| item.scene.as_deref()),
        )
        .bind(
            uploader_upload_item
                .as_ref()
                .and_then(|item| item.source.as_deref()),
        )
        .bind(content_type)
        .bind(content_length)
        .bind(checksum_sha256_hex)
        .bind(&operator_id)
        .execute(&mut *connection)
        .await
        .map_err(internal_sql_error("insert dr_drive_storage_object failed"))?;

        let node_version_id = insert_node_version_for_storage_object(
            &mut *connection,
            NodeVersionStorageMetadata {
                tenant_id: &tenant_id,
                space_id: &node.space_id,
                node_id: &upload_session.node_id,
                version_no: completed_object_plan.version_no,
                storage_object_id: &completed_object_plan.id,
                content_type,
                content_length,
                checksum_sha256_hex,
            },
            uploader_upload_item.as_ref(),
            &operator_id,
        )
        .await?;

        if let Some(upload_item) = uploader_upload_item.as_ref() {
            complete_uploader_upload_item(
                &mut connection,
                CompletedUploaderUploadItem {
                    tenant_id: &tenant_id,
                    upload_item,
                    storage_object: &completed_object_plan,
                    upload_session: &upload_session,
                    content_type,
                    content_length,
                    checksum_sha256_hex,
                    uploaded_parts_count: payload.parts.len() as i64,
                    operator_id: &operator_id,
                    before_content_state: &node_content_state_before_completion,
                },
            )
            .await?;
        }

        update_upload_session_state(
            &mut *connection,
            &tenant_id,
            &upload_session_id,
            "completed",
            &operator_id,
        )
        .await?;

        apply_file_node_head_snapshot_in_transaction(
            &mut connection,
            &tenant_id,
            &upload_session.node_id,
            &operator_id,
            &head_snapshot,
        )
        .await
        .map_err(map_service_error)?;

        sdkwork_drive_workspace_service::infrastructure::change_recorder::record_drive_node_version_committed_on_connection(
            &mut connection,
            sdkwork_drive_workspace_service::infrastructure::change_recorder::RecordDriveNodeVersionCommittedCommand {
                tenant_id: &tenant_id,
                organization_id: uploader_upload_item
                    .as_ref()
                    .and_then(|item| item.organization_id.as_deref()),
                space_id: &node.space_id,
                node_id: &upload_session.node_id,
                node_version_id: &node_version_id,
                version_no: completed_object_plan.version_no,
                operation_id: uploader_upload_item
                    .as_ref()
                    .map(|item| item.id.as_str())
                    .unwrap_or(upload_session_id.as_str()),
                content_type,
                content_length,
                checksum_sha256_hex,
                actor_id: &operator_id,
            },
        )
        .await
        .map_err(map_service_error)?;

        Ok(())
    }
    .await;

    match tx_result {
        Ok(()) => {
            sqlx::query("COMMIT")
                .execute(&mut *connection)
                .await
                .map_err(internal_sql_error(
                    "commit upload completion transaction failed",
                ))?;
            sdkwork_drive_observability::metrics::record_outbox_pending();
            trigger_immediate_outbox_dispatch(state.pool.clone());
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            drop(connection);
            recover_upload_completion_after_db_failure(
                &state,
                &state.pool,
                &tenant_id,
                &upload_session,
                &operator_id,
            )
            .await;
            return Err(error);
        }
    }
    drop(connection);

    let updated = find_upload_session(&state.pool, &tenant_id, &upload_session_id).await?;
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
    let upload_session = find_upload_session(&state.pool, &tenant_id, &upload_session_id).await?;
    validate_mutable_upload_session(&upload_session)?;
    let node = find_active_node(&state.pool, &tenant_id, &upload_session.node_id).await?;
    acl::ensure_ctx_node_role(
        &state.pool,
        &ctx,
        &node.space_id,
        &upload_session.node_id,
        "writer",
    )
    .await?;
    abort_storage_multipart_upload(&state, &upload_session).await?;
    update_upload_session_state(
        &state.pool,
        &tenant_id,
        &upload_session_id,
        "aborted",
        &operator_id,
    )
    .await?;
    record_change(
        &state.pool,
        &tenant_id,
        &node.space_id,
        Some(&upload_session.node_id),
        drive_events::upload_session::ABORTED,
        &operator_id,
    )
    .await?;
    let updated = find_upload_session(&state.pool, &tenant_id, &upload_session_id).await?;
    Ok(success_resource(UploadSessionMutationResponse::from(
        updated,
    )))
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
