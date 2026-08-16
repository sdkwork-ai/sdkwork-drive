use crate::error::{map_service_error, ProblemDetail};
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_contract::drive::events::DriveNodeEligibility;
use sdkwork_drive_workspace_service::infrastructure::change_recorder::{
    self, DriveNodeLocationSnapshot, RecordDriveChangeCommand, RecordDriveNodeDeletedCommand,
    RecordDriveNodeEligibilityChangedCommand, RecordDriveNodePathChangedCommand,
};
use sqlx::{PgConnection, PgPool};

pub(crate) async fn record_change(
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
    .map_err(map_service_error)
}

pub(crate) async fn resolve_node_location_on_connection(
    connection: &mut PgConnection,
    tenant_id: &str,
    space_id: &str,
    node_id: &str,
) -> Result<DriveNodeLocationSnapshot, (StatusCode, Json<ProblemDetail>)> {
    change_recorder::resolve_drive_node_location_snapshot_on_connection(
        connection, tenant_id, space_id, node_id,
    )
    .await
    .map_err(map_service_error)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn record_node_path_changed_on_connection(
    connection: &mut PgConnection,
    tenant_id: &str,
    organization_id: Option<&str>,
    space_id: &str,
    node_id: &str,
    operation_id: &str,
    actor_id: &str,
    old_location: &DriveNodeLocationSnapshot,
    new_location: &DriveNodeLocationSnapshot,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    change_recorder::record_drive_node_path_changed_on_connection(
        connection,
        RecordDriveNodePathChangedCommand {
            tenant_id,
            organization_id,
            space_id,
            node_id,
            operation_id,
            actor_id,
            old_location,
            new_location,
        },
    )
    .await
    .map_err(map_service_error)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn record_node_eligibility_changed_on_connection(
    connection: &mut PgConnection,
    tenant_id: &str,
    organization_id: Option<&str>,
    space_id: &str,
    node_id: &str,
    operation_id: &str,
    actor_id: &str,
    old_eligibility: DriveNodeEligibility,
    new_eligibility: DriveNodeEligibility,
    reason: &str,
    location: &DriveNodeLocationSnapshot,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    change_recorder::record_drive_node_eligibility_changed_on_connection(
        connection,
        RecordDriveNodeEligibilityChangedCommand {
            tenant_id,
            organization_id,
            space_id,
            node_id,
            operation_id,
            actor_id,
            old_eligibility,
            new_eligibility,
            reason,
            location,
        },
    )
    .await
    .map_err(map_service_error)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn record_node_deleted_on_connection(
    connection: &mut PgConnection,
    tenant_id: &str,
    organization_id: Option<&str>,
    space_id: &str,
    node_id: &str,
    operation_id: &str,
    actor_id: &str,
    deletion_reason: &str,
    last_location: &DriveNodeLocationSnapshot,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    change_recorder::record_drive_node_deleted_on_connection(
        connection,
        RecordDriveNodeDeletedCommand {
            tenant_id,
            organization_id,
            space_id,
            node_id,
            operation_id,
            actor_id,
            deletion_reason,
            last_location,
        },
    )
    .await
    .map_err(map_service_error)
}

pub(crate) fn notify_committed(pool: PgPool) {
    change_recorder::notify_drive_event_committed(pool);
}
