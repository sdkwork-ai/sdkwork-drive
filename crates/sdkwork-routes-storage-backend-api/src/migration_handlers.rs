//! Admin endpoints for cross-provider storage migration.
//!
//! A migration moves every object that currently resolves to one storage
//! provider onto another, verifies each byte, and only then re-points the
//! registry. The endpoints here are deliberately shaped around that invariant:
//!
//! - `POST /migrations`            opens a run (validates both providers).
//! - `POST /migrations/{id}/run`   drives it forward by one bounded batch.
//! - `GET  /migrations/{id}`       reports progress.
//! - `GET  /migrations/{id}/items` lists the per-object queue.
//! - `POST /migrations/{id}/cancel` stops a run that is still resumable.
//!
//! The batch endpoint is the interesting one: it is idempotent per object and
//! safe to call repeatedly, so an operator (or a scheduler) can keep calling it
//! until `completed` flips to `true`. A run that dies halfway is simply one that
//! stopped being called — there is no separate "resume" verb because there is no
//! separate state to resume from.

use crate::app_context::DriveRequestContext;
use crate::audit::record_audit_event;
use crate::dto::{
    CancelStorageMigrationRequest, CreateStorageMigrationRequest, RunStorageMigrationRequest,
    StorageMigrationItemQuery, StorageMigrationItemResponse, StorageMigrationQuery,
    StorageMigrationResponse, StorageMigrationRunResponse,
};
use crate::error::{invalid_json_problem, map_service_error, ProblemDetail};
use crate::migration_copier::AdminStorageMigrationCopier;
use crate::response::{
    success_item, success_list_page_simple, StorageItemHttpResponse, StorageListHttpResponse,
};
use crate::state::AdminStorageState;
use crate::validators::{next_page_token, parse_offset_page};
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_workspace_service::application::storage_migration_service::{
    audit_action, CancelStorageMigrationCommand, DriveStorageMigrationEngine,
    GetStorageMigrationCommand, ListStorageMigrationItemsCommand, PlanStorageMigrationCommand,
    RunStorageMigrationCommand,
};
use sdkwork_drive_workspace_service::domain::storage_migration::{
    DriveStorageMigration, DriveStorageMigrationItem, DriveStorageMigrationItemStatus,
    DriveStorageMigrationStatus,
};
use sdkwork_drive_workspace_service::infrastructure::sql::storage_migration_store::SqlStorageMigrationStore;
use sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_store::SqlStorageProviderStore;

/// Resource type recorded on every audit row this module emits.
///
/// Deliberately not `storage_provider`: a migration is its own administrative
/// object, and an auditor filtering for "what happened to provider X" needs to
/// see the run, not a series of provider-shaped rows that differ only by id.
const MIGRATION_RESOURCE_TYPE: &str = "storage_migration";

/// Build the engine against the request-scoped admin storage state.
///
/// Each call constructs its own store, copier and provider store. They are all
/// thin handles over the same pool, so this costs a few clones rather than a
/// connection, and it keeps the handlers free of long-lived generic parameters
/// that would otherwise leak into the axum signatures.
fn build_engine<'a>(
    state: &'a AdminStorageState,
    tenant_id: &str,
) -> DriveStorageMigrationEngine<
    SqlStorageMigrationStore,
    AdminStorageMigrationCopier<'a>,
    SqlStorageProviderStore,
> {
    DriveStorageMigrationEngine::new(
        SqlStorageMigrationStore::new(state.pool.clone()),
        AdminStorageMigrationCopier::new(state, tenant_id.to_string()),
        SqlStorageProviderStore::new(state.pool.clone()),
    )
}

/// Open a migration run.
///
/// Both providers are validated here, before any object is queued: the source
/// must be readable (`active` or `disabled` — draining a retired provider is the
/// main reason to migrate) and the target must accept writes (`active`). Paying
/// that validation up front is what makes the copy loop's failure modes purely
/// per-object rather than "we moved gigabytes in the wrong direction".
pub(crate) async fn plan_storage_migration(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    payload: Result<Json<CreateStorageMigrationRequest>, JsonRejection>,
) -> Result<StorageItemHttpResponse<StorageMigrationResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;


    let migration = build_engine(&state, &tenant_id)
        .service()
        .plan_storage_migration(PlanStorageMigrationCommand {
            migration_id: payload.id,
            tenant_id: tenant_id.clone(),
            name: payload.name,
            source_provider_id: payload.source_provider_id,
            target_provider_id: payload.target_provider_id,
            target_bucket: payload.target_bucket,
            apply_binding_switch: payload.apply_binding_switch,
            operator_id: operator_id.clone(),
        })
        .await
        .map_err(map_service_error)?;

    record_audit_event(
        &state,
        audit_action::PLANNED,
        MIGRATION_RESOURCE_TYPE,
        &migration.id,
        &operator_id,
        &tenant_id,
    )
    .await?;

    Ok(success_item(map_migration(migration)))
}

/// Drive a run forward by one bounded batch.
///
/// Returns the run together with what this call did, so a caller driving the
/// loop learns whether it is finished without a second round trip. The endpoint
/// is safe to retry: objects already marked `copied` are not copied again, and a
/// concurrent caller simply loses the status transition race.
pub(crate) async fn run_storage_migration(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(migration_id): Path<String>,
    payload: Result<Json<RunStorageMigrationRequest>, JsonRejection>,
) -> Result<StorageItemHttpResponse<StorageMigrationRunResponse>, (StatusCode, Json<ProblemDetail>)>
{
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;


    let report = build_engine(&state, &tenant_id)
        .run_storage_migration(RunStorageMigrationCommand {
            migration_id: migration_id.clone(),
            tenant_id: tenant_id.clone(),
            batch_size: payload.batch_size,
            operator_id: operator_id.clone(),
        })
        .await
        .map_err(map_service_error)?;

    // One audit row per drive call rather than per object: a run of a million
    // objects would otherwise bury every other admin action in the log. The
    // per-object detail lives in the run's item queue, which is the place an
    // operator actually inspects it.
    record_audit_event(
        &state,
        run_audit_action(report.migration.status),
        MIGRATION_RESOURCE_TYPE,
        &migration_id,
        &operator_id,
        &tenant_id,
    )
    .await?;

    Ok(success_item(StorageMigrationRunResponse {
        migration: map_migration(report.migration),
        copied_this_batch: report.copied_this_batch,
        failed_this_batch: report.failed_this_batch,
        completed: report.completed,
    }))
}

/// List the migration runs of the caller's tenant.
///
/// `status` narrows the list to one lifecycle state, which is how an operator
/// finds the runs that are still moving data (`pending`, `running`) or the ones
/// that need attention (`failed`).
pub(crate) async fn list_storage_migrations(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<StorageMigrationQuery>,
) -> Result<StorageListHttpResponse<StorageMigrationResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let page = parse_offset_page(query.page_size, query.page_token)?;
    let status = normalize_optional_filter(query.status);
    if let Some(status) = status.as_deref() {
        crate::validators::validate_storage_migration_status(status)?;
    }

    let mut items = build_engine(&state, &tenant_id)
        .service()
        .list_storage_migrations(&tenant_id, status.as_deref(), page.offset, page.limit + 1)
        .await
        .map_err(map_service_error)?
        .into_iter()
        .map(map_migration)
        .collect::<Vec<_>>();
    let next_page_token = next_page_token(&mut items, page);
    Ok(success_list_page_simple(items, page, next_page_token))
}

/// Report one run's progress.
pub(crate) async fn get_storage_migration(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(migration_id): Path<String>,
) -> Result<StorageItemHttpResponse<StorageMigrationResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let migration = build_engine(&state, &tenant_id)
        .service()
        .get_storage_migration(GetStorageMigrationCommand {
            migration_id,
            tenant_id,
        })
        .await
        .map_err(map_service_error)?;
    Ok(success_item(map_migration(migration)))
}

/// List the per-object queue of a run.
///
/// `status` filters the queue, which is how an operator finds the failures a
/// retry would target. The run is resolved first by the service, so a caller
/// from another tenant gets `404` rather than an empty page.
pub(crate) async fn list_storage_migration_items(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(migration_id): Path<String>,
    Query(query): Query<StorageMigrationItemQuery>,
) -> Result<StorageListHttpResponse<StorageMigrationItemResponse>, (StatusCode, Json<ProblemDetail>)>
{
    let tenant_id = ctx.resolve_tenant_id()?;
    let page = parse_offset_page(query.page_size, query.page_token)?;
    let status = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(parse_item_status)
        .transpose()?;
    let mut items = build_engine(&state, &tenant_id)
        .service()
        .list_storage_migration_items(ListStorageMigrationItemsCommand {
            migration_id,
            tenant_id,
            status,
            offset: page.offset,
            // One extra row is read purely to decide whether a continuation
            // token is warranted; `next_page_token` discards it again.
            limit: page.limit + 1,
        })
        .await
        .map_err(map_service_error)?
        .into_iter()
        .map(map_migration_item)
        .collect::<Vec<_>>();
    let next_page_token = next_page_token(&mut items, page);
    Ok(success_list_page_simple(items, page, next_page_token))
}

/// Stop a run that has not yet reached a terminal state.
///
/// Cancelling never un-copies and never re-points: bytes already verified on the
/// target stay there, and every object keeps reading from its original provider
/// because the registry was never switched. The operation is therefore safe to
/// offer on any resumable run.
///
/// Takes no body. The operator is resolved from the verified request context —
/// it is not a client-supplied field — so the audit row cannot be filed under
/// someone else's name.
pub(crate) async fn cancel_storage_migration(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(migration_id): Path<String>,
    payload: Result<Option<Json<CancelStorageMigrationRequest>>, JsonRejection>,
) -> Result<StorageItemHttpResponse<StorageMigrationResponse>, (StatusCode, Json<ProblemDetail>)> {
    // `Option` so an entirely absent body is accepted; a body that is present
    // but malformed is still rejected, because that means the caller is
    // speaking a different version of this operation.
    if let Some(Err(rejection)) = payload.transpose() {
        return Err(invalid_json_problem(rejection));
    }
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;


    let migration = build_engine(&state, &tenant_id)
        .service()
        .cancel_storage_migration(CancelStorageMigrationCommand {
            migration_id: migration_id.clone(),
            tenant_id: tenant_id.clone(),
            operator_id: operator_id.clone(),
        })
        .await
        .map_err(map_service_error)?;

    record_audit_event(
        &state,
        audit_action::CANCELLED,
        MIGRATION_RESOURCE_TYPE,
        &migration_id,
        &operator_id,
        &tenant_id,
    )
    .await?;

    Ok(success_item(map_migration(migration)))
}

/// The audit action that describes what one drive call actually achieved.
///
/// Derived from the run's resulting status rather than from a caller-supplied
/// flag, so the log cannot claim a completion that did not happen.
fn run_audit_action(status: DriveStorageMigrationStatus) -> &'static str {
    match status {
        DriveStorageMigrationStatus::Succeeded => audit_action::COMPLETED,
        DriveStorageMigrationStatus::Failed => audit_action::FAILED,
        DriveStorageMigrationStatus::Cancelled => audit_action::CANCELLED,
        // `Pending` and `Running` both mean "more work remains"; the batch made
        // progress but did not decide the run's fate.
        DriveStorageMigrationStatus::Pending | DriveStorageMigrationStatus::Running => {
            audit_action::STARTED
        }
    }
}

fn parse_item_status(
    value: &str,
) -> Result<DriveStorageMigrationItemStatus, (StatusCode, Json<ProblemDetail>)> {
    crate::validators::validate_storage_migration_item_status(value)?;
    DriveStorageMigrationItemStatus::parse(value).map_err(map_service_error)
}

/// Collapse a blank or absent query filter to `None`.
///
/// A query string of `?status=` means "no filter", not "filter on the empty
/// string" — the latter would return an empty page for what looks like a
/// no-op request.
fn normalize_optional_filter(value: Option<String>) -> Option<String> {
    crate::validators::normalize_optional_text(value)
}

fn map_migration(migration: DriveStorageMigration) -> StorageMigrationResponse {
    let outstanding = migration.outstanding();
    let progress_ratio = migration.progress_ratio();
    StorageMigrationResponse {
        id: migration.id,
        name: migration.name,
        source_provider_id: migration.source_provider_id,
        target_provider_id: migration.target_provider_id,
        target_bucket: migration.target_bucket,
        status: migration.status.as_str().to_string(),
        apply_binding_switch: migration.apply_binding_switch,
        objects_total: migration.objects_total,
        objects_copied: migration.objects_copied,
        objects_failed: migration.objects_failed,
        objects_outstanding: outstanding,
        bytes_copied: migration.bytes_copied,
        progress_ratio,
        failure_message: migration.failure_message,
        created_by: migration.created_by,
        updated_by: migration.updated_by,
    }
}

fn map_migration_item(item: DriveStorageMigrationItem) -> StorageMigrationItemResponse {
    StorageMigrationItemResponse {
        id: item.id,
        storage_object_id: item.storage_object_id,
        source_provider_id: item.source_provider_id,
        source_bucket: item.source_bucket,
        source_object_key: item.source_object_key,
        target_provider_id: item.target_provider_id,
        target_bucket: item.target_bucket,
        target_object_key: item.target_object_key,
        content_type: item.content_type,
        content_length: item.content_length,
        checksum_sha256_hex: item.checksum_sha256_hex,
        status: item.status.as_str().to_string(),
        verified_checksum_sha256_hex: item.verified_checksum_sha256_hex,
        failure_message: item.failure_message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tenant_id is never read here, so the helper takes only what it uses.
    fn migration_with(
        status: DriveStorageMigrationStatus,
        total: i64,
        copied: i64,
        failed: i64,
    ) -> DriveStorageMigration {
        DriveStorageMigration {
            id: "mig-1".to_string(),
            tenant_id: "tenant-1".to_string(),
            source_provider_id: "provider-a".to_string(),
            target_provider_id: "provider-b".to_string(),
            name: "retire provider-a".to_string(),
            status,
            objects_total: total,
            objects_copied: copied,
            objects_failed: failed,
            bytes_copied: 10,
            target_bucket: Some("bucket-b".to_string()),
            apply_binding_switch: true,
            failure_message: None,
            created_by: "operator-1".to_string(),
            updated_by: "operator-1".to_string(),
        }
    }

    #[test]
    fn response_reports_outstanding_and_progress_from_the_derived_values() {
        let response = map_migration(migration_with(
            DriveStorageMigrationStatus::Running,
            10,
            4,
            1,
        ));
        assert_eq!(response.objects_outstanding, 5);
        assert!((response.progress_ratio - 0.5).abs() < f64::EPSILON);
        assert_eq!(response.status, "running");
    }

    #[test]
    fn run_audit_action_tracks_the_resulting_status_not_the_intent() {
        assert_eq!(
            run_audit_action(DriveStorageMigrationStatus::Succeeded),
            audit_action::COMPLETED
        );
        assert_eq!(
            run_audit_action(DriveStorageMigrationStatus::Failed),
            audit_action::FAILED
        );
        assert_eq!(
            run_audit_action(DriveStorageMigrationStatus::Running),
            audit_action::STARTED
        );
        assert_eq!(
            run_audit_action(DriveStorageMigrationStatus::Pending),
            audit_action::STARTED
        );
    }

    #[test]
    fn item_status_parse_rejects_unknown_values() {
        assert!(parse_item_status("halfway").is_err());
        assert_eq!(
            parse_item_status("copied").expect("copied parses"),
            DriveStorageMigrationItemStatus::Copied
        );
    }

    #[test]
    fn item_response_is_a_flat_projection() {
        let response = map_migration_item(DriveStorageMigrationItem {
            id: "mig-1:obj-1".to_string(),
            migration_id: "mig-1".to_string(),
            tenant_id: "tenant-1".to_string(),
            storage_object_id: "obj-1".to_string(),
            source_provider_id: "provider-a".to_string(),
            source_bucket: "bucket-a".to_string(),
            source_object_key: "keys/a.bin".to_string(),
            target_provider_id: "provider-b".to_string(),
            target_bucket: "bucket-b".to_string(),
            target_object_key: "keys/a.bin".to_string(),
            content_type: "application/octet-stream".to_string(),
            content_length: 12,
            checksum_sha256_hex: "aa".to_string(),
            status: DriveStorageMigrationItemStatus::Copied,
            verified_checksum_sha256_hex: Some("aa".to_string()),
            failure_message: None,
        });
        assert_eq!(response.status, "copied");
        assert_eq!(response.target_object_key, response.source_object_key);
    }
}
