use crate::app_context::DriveRequestContext;
use crate::dto::{FlexibleI64, PrepareUploaderUploadRequest};
use crate::error::{problem, validation_problem, ProblemDetail, SdkWorkResultCode};
use crate::time::current_epoch_ms;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_uploader_service::service::{
    PrepareUploaderUploadCommand, UploaderActor, UploaderRetention, UploaderTarget,
};
use sdkwork_drive_workspace_service::domain::uploader::DriveUploadItem;
use sdkwork_web_core::WebAuthLevel;
use sqlx::Row;

pub(crate) fn prepare_uploader_command(
    payload: PrepareUploaderUploadRequest,
    ctx: &DriveRequestContext,
    tenant_id: String,
    operator_id: String,
) -> Result<PrepareUploaderUploadCommand, (StatusCode, Json<ProblemDetail>)> {
    let app_id = ctx.resolve_app_id()?;
    let actor = resolve_uploader_actor(ctx)?;
    let target = match payload.space_id {
        Some(space_id) if !space_id.trim().is_empty() => UploaderTarget::Space {
            space_id: space_id.trim().to_string(),
            parent_node_id: payload.parent_node_id,
            share_token: payload.share_token,
        },
        _ => UploaderTarget::AutoUploadSpace {
            parent_node_id: payload.parent_node_id,
        },
    };
    let retention = match payload.retention {
        Some(retention) if retention.mode.trim() == "temporary" => UploaderRetention::Temporary {
            ttl_seconds: retention
                .ttl_seconds
                .map(FlexibleI64::into_i64)
                .unwrap_or(86_400),
            cleanup_action: retention
                .cleanup_action
                .unwrap_or_else(|| "soft_delete".to_string()),
            hard_delete_after_seconds: retention
                .hard_delete_after_seconds
                .map(FlexibleI64::into_i64),
        },
        Some(retention) if retention.mode.trim() == "long_term" => UploaderRetention::LongTerm,
        Some(_) => {
            return Err(validation_problem(
                "retention.mode must be temporary or long_term",
            ));
        }
        None => UploaderRetention::LongTerm,
    };

    Ok(PrepareUploaderUploadCommand {
        id: payload.id,
        task_id: payload.task_id,
        tenant_id,
        organization_id: ctx.organization_id.clone(),
        actor,
        app_id,
        app_resource_type: payload.app_resource_type,
        app_resource_id: payload.app_resource_id,
        scene: payload.scene,
        source: payload.source,
        upload_profile_code: payload
            .upload_profile_code
            .unwrap_or_else(|| "generic".to_string()),
        file_fingerprint: payload.file_fingerprint,
        original_file_name: payload.original_file_name,
        content_type: payload.content_type,
        content_length: payload.content_length.into_i64(),
        chunk_size_bytes: payload.chunk_size_bytes.into_i64(),
        target,
        retention,
        operator_id,
        now_epoch_ms: payload
            .now_epoch_ms
            .map(FlexibleI64::into_i64)
            .unwrap_or_else(current_epoch_ms),
    })
}

fn resolve_uploader_actor(
    ctx: &DriveRequestContext,
) -> Result<UploaderActor, (StatusCode, Json<ProblemDetail>)> {
    ctx.require_verified_context()?;
    match (ctx.subject_type.as_str(), &ctx.auth_level) {
        ("user", WebAuthLevel::Anonymous) => Ok(UploaderActor::Anonymous {
            anonymous_id: require_uploader_actor_id(&ctx.actor_id)?,
        }),
        ("user", WebAuthLevel::Password | WebAuthLevel::Mfa) => Ok(UploaderActor::User {
            user_id: require_uploader_actor_id(&ctx.user_id)?,
        }),
        ("service" | "system", WebAuthLevel::System) => Ok(UploaderActor::System {
            operator_id: require_uploader_actor_id(&ctx.actor_id)?,
        }),
        _ => Err(problem(
            StatusCode::FORBIDDEN,
            "forbidden",
            "verified principal is not permitted to prepare Drive App API uploads",
            SdkWorkResultCode::PermissionRequired,
        )),
    }
}

fn require_uploader_actor_id(actor_id: &str) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let actor_id = actor_id.trim();
    if !actor_id.is_empty() {
        return Ok(actor_id.to_owned());
    }
    Err(problem(
        StatusCode::UNAUTHORIZED,
        "unauthorized",
        "verified uploader actor identity is required",
        SdkWorkResultCode::AuthenticationRequired,
    ))
}

pub(crate) fn map_uploader_upload_item_row(
    row: &sqlx::postgres::PgRow,
) -> Result<DriveUploadItem, (StatusCode, Json<ProblemDetail>)> {
    Ok(DriveUploadItem {
        id: row.get("id"),
        task_id: row.get("task_id"),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        user_id: row.get("user_id"),
        actor_type: row.get("actor_type"),
        actor_id: row.get("actor_id"),
        app_id: row.get("app_id"),
        app_resource_type: row.get("app_resource_type"),
        app_resource_id: row.get("app_resource_id"),
        scene: row.get("scene"),
        source: row.get("source"),
        upload_profile_code: row.get("upload_profile_code"),
        file_fingerprint: row.get("file_fingerprint"),
        space_id: row.get("space_id"),
        node_id: row.get("node_id"),
        upload_session_id: row.get("upload_session_id"),
        storage_provider_id: row.get("storage_provider_id"),
        storage_upload_id: row.get("storage_upload_id"),
        object_bucket: row.try_get("object_bucket").ok(),
        object_key: row.try_get("object_key").ok(),
        original_file_name: row.get("original_file_name"),
        file_extension: row.get("file_extension"),
        content_type: row.get("content_type"),
        content_type_group: row.get("content_type_group"),
        detected_content_type: row.get("detected_content_type"),
        content_length: row.get("content_length"),
        checksum_sha256_hex: row.get("checksum_sha256_hex"),
        chunk_size_bytes: row.get("chunk_size_bytes"),
        total_parts: i64::from(row.get::<i32, _>("total_parts")),
        uploaded_parts_count: i64::from(row.get::<i32, _>("uploaded_parts_count")),
        uploaded_bytes: row.get("uploaded_bytes"),
        status: row.get("status"),
        retention_mode: row.get("retention_mode"),
        retention_expires_at_epoch_ms: row.get("retention_expires_at_epoch_ms"),
        cleanup_action: row.get("cleanup_action"),
        hard_delete_after_epoch_ms: row.get("hard_delete_after_epoch_ms"),
        cleanup_status: row.get("cleanup_status"),
        post_process_status: row.get("post_process_status"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(subject_type: &str, auth_level: WebAuthLevel) -> DriveRequestContext {
        DriveRequestContext {
            tenant_id: "tenant-001".to_owned(),
            user_id: "user-001".to_owned(),
            organization_id: None,
            app_id: Some("drive-public".to_owned()),
            actor_id: "verified-actor-001".to_owned(),
            subject_type: subject_type.to_owned(),
            subject_id: "user-001".to_owned(),
            auth_level,
            request_id: "request-001".to_owned(),
            trace_id: "trace-001".to_owned(),
            from_token: true,
        }
    }

    #[test]
    fn anonymous_auth_level_uses_verified_actor_identity() {
        let actor = resolve_uploader_actor(&context("user", WebAuthLevel::Anonymous))
            .expect("anonymous actor");

        assert!(matches!(
            actor,
            UploaderActor::Anonymous { anonymous_id }
                if anonymous_id == "verified-actor-001"
        ));
    }

    #[test]
    fn password_and_mfa_users_keep_user_semantics() {
        for auth_level in [WebAuthLevel::Password, WebAuthLevel::Mfa] {
            let actor = resolve_uploader_actor(&context("user", auth_level)).expect("user actor");
            assert!(matches!(
                actor,
                UploaderActor::User { user_id } if user_id == "user-001"
            ));
        }
    }

    #[test]
    fn service_and_system_principals_keep_system_semantics() {
        for subject_type in ["service", "system"] {
            let actor = resolve_uploader_actor(&context(subject_type, WebAuthLevel::System))
                .expect("system actor");
            assert!(matches!(
                actor,
                UploaderActor::System { operator_id }
                    if operator_id == "verified-actor-001"
            ));
        }
    }

    #[test]
    fn api_keys_and_inconsistent_principals_fail_closed() {
        for context in [
            context("api_key", WebAuthLevel::ApiKey),
            context("user", WebAuthLevel::System),
            context("service", WebAuthLevel::Password),
        ] {
            let error = resolve_uploader_actor(&context).expect_err("principal must be rejected");
            assert_eq!(error.0, StatusCode::FORBIDDEN);
        }
    }
}
