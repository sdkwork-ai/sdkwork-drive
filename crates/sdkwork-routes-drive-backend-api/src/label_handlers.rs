use crate::audit::record_label_audit;
use crate::dto::{CreateLabelRequest, LabelListQuery, LabelResponse, UpdateLabelRequest};
use crate::error::{internal_sql_error, invalid_json_problem, not_found_problem, ProblemDetail};
use crate::mappers::map_label_row;
use crate::response::{no_content, success_list_page_simple, DriveListHttpResponse};
use crate::state::BackendState;
use crate::tenant_context::authenticated_tenant_id;
use crate::validators::{
    next_page_token, normalize_optional_text, parse_offset_page, require_non_empty_text,
    validate_label_color, validate_label_key, validate_lifecycle_status,
};
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use sdkwork_drive_contract::drive::domain_events::admin_audit;
use sdkwork_drive_security::DriveAppContext;

pub(crate) async fn list_labels(
    State(state): State<BackendState>,
    Extension(app_context): Extension<DriveAppContext>,
    Query(query): Query<LabelListQuery>,
) -> Result<DriveListHttpResponse<LabelResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = authenticated_tenant_id(&app_context);
    let page = parse_offset_page(query.page_size, query.page_token)?;
    let lifecycle_status =
        normalize_optional_text(query.lifecycle_status).unwrap_or_else(|| "active".to_string());
    validate_lifecycle_status(&lifecycle_status)?;

    let rows = sqlx::query(
        "SELECT id, tenant_id, label_key, display_name, color, description,
                lifecycle_status, version
         FROM dr_drive_label
         WHERE tenant_id=$1 AND lifecycle_status=$2
         ORDER BY label_key ASC
         LIMIT $3 OFFSET $4",
    )
    .bind(&tenant_id)
    .bind(&lifecycle_status)
    .bind(page.limit + 1)
    .bind(page.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_sql_error("list dr_drive_label failed"))?;
    let mut items = rows.iter().map(map_label_row).collect::<Vec<_>>();
    let next_page_token = next_page_token(&mut items, page);
    Ok(success_list_page_simple(items, page, next_page_token))
}

pub(crate) async fn get_label(
    State(state): State<BackendState>,
    Extension(app_context): Extension<DriveAppContext>,
    Path(label_id): Path<String>,
) -> Result<Json<LabelResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = authenticated_tenant_id(&app_context);
    Ok(Json(find_label(&state, &tenant_id, &label_id).await?))
}

pub(crate) async fn create_label(
    State(state): State<BackendState>,
    Extension(app_context): Extension<DriveAppContext>,
    payload: Result<Json<CreateLabelRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<LabelResponse>), (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let id = require_non_empty_text(payload.id, "id")?;
    let tenant_id = authenticated_tenant_id(&app_context);
    let label_key = validate_label_key(&payload.label_key)?.to_string();
    let display_name = require_non_empty_text(payload.display_name, "displayName")?;
    let color = match normalize_optional_text(payload.color) {
        Some(color) => Some(validate_label_color(&color)?.to_string()),
        None => None,
    };
    let description = normalize_optional_text(payload.description);
    let operator_id = app_context.actor_id.clone();

    sqlx::query(
        "INSERT INTO dr_drive_label (
            id, tenant_id, label_key, display_name, color, description,
            lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $7)",
    )
    .bind(&id)
    .bind(&tenant_id)
    .bind(&label_key)
    .bind(&display_name)
    .bind(&color)
    .bind(&description)
    .bind(&operator_id)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("insert dr_drive_label failed"))?;

    let created = find_label(&state, &tenant_id, &id).await?;
    record_label_audit(
        &state,
        &tenant_id,
        admin_audit::label::CREATED,
        &created.id,
        &operator_id,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(created)))
}

pub(crate) async fn update_label(
    State(state): State<BackendState>,
    Extension(app_context): Extension<DriveAppContext>,
    Path(label_id): Path<String>,
    payload: Result<Json<UpdateLabelRequest>, JsonRejection>,
) -> Result<Json<LabelResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let tenant_id = authenticated_tenant_id(&app_context);
    let operator_id = app_context.actor_id.clone();
    let current = find_label(&state, &tenant_id, &label_id).await?;
    let display_name = payload
        .display_name
        .map(|value| require_non_empty_text(value, "displayName"))
        .transpose()?
        .unwrap_or(current.display_name);
    let color = match payload.color {
        Some(value) => Some(validate_label_color(&value)?.to_string()),
        None => current.color,
    };
    let description = payload.description.or(current.description);

    sqlx::query(
        "UPDATE dr_drive_label
         SET display_name=$1,
             color=$2,
             description=$3,
             version=version + 1,
             updated_by=$4,
             updated_at=CURRENT_TIMESTAMP
         WHERE tenant_id=$5 AND id=$6 AND lifecycle_status='active'",
    )
    .bind(&display_name)
    .bind(&color)
    .bind(&description)
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&label_id)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("update dr_drive_label failed"))?;

    let updated = find_label(&state, &tenant_id, &label_id).await?;
    record_label_audit(
        &state,
        &tenant_id,
        admin_audit::label::UPDATED,
        &updated.id,
        &operator_id,
    )
    .await?;
    Ok(Json(updated))
}

pub(crate) async fn delete_label(
    State(state): State<BackendState>,
    Extension(app_context): Extension<DriveAppContext>,
    Path(label_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = authenticated_tenant_id(&app_context);
    let operator_id = app_context.actor_id.clone();
    find_label(&state, &tenant_id, &label_id).await?;

    sqlx::query(
        "UPDATE dr_drive_label
         SET lifecycle_status='deleted',
             version=version + 1,
             updated_by=$1,
             updated_at=CURRENT_TIMESTAMP
         WHERE tenant_id=$2 AND id=$3 AND lifecycle_status='active'",
    )
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&label_id)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("delete dr_drive_label failed"))?;

    record_label_audit(
        &state,
        &tenant_id,
        admin_audit::label::DELETED,
        &label_id,
        &operator_id,
    )
    .await?;
    Ok(no_content())
}

async fn find_label(
    state: &BackendState,
    tenant_id: &str,
    label_id: &str,
) -> Result<LabelResponse, (StatusCode, Json<ProblemDetail>)> {
    let rows = sqlx::query(
        "SELECT id, tenant_id, label_key, display_name, color, description,
                lifecycle_status, version
         FROM dr_drive_label
         WHERE tenant_id=$1 AND id=$2 AND lifecycle_status='active'
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(label_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_sql_error("find dr_drive_label failed"))?;
    let Some(row) = rows.first() else {
        return Err(not_found_problem("label not found"));
    };
    Ok(map_label_row(row))
}
