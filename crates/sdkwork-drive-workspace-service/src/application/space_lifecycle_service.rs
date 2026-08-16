use sdkwork_drive_contract::drive::domain_events as drive_events;

use crate::application::permission_service::SqlDrivePermissionService;
use crate::domain::space::DriveSpace;
use crate::infrastructure::change_recorder::{
    record_drive_change, record_drive_change_on_connection, RecordDriveChangeCommand,
};
use crate::infrastructure::outbox_dispatch::trigger_immediate_outbox_dispatch;
use crate::infrastructure::sql::begin_transaction_sql;
use crate::infrastructure::sql::space_lifecycle_store::SqlSpaceLifecycleStore;
use crate::infrastructure::sql::space_store::SqlSpaceStore;
use crate::ports::permission_store::GrantDriveNodePermissionCommand;
use crate::ports::space_store::DriveSpaceStore;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct BootstrapTeamSpaceCreatorAccessCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub creator_user_id: String,
    pub display_name: String,
    pub root_folder_id: String,
}

#[derive(Debug, Clone)]
pub struct RetireSpaceContentsCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteSpaceWithContentsCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteSpaceWithContentsResult {
    pub space: DriveSpace,
    pub deleted_node_count: i64,
}

#[derive(Debug, Clone)]
pub struct SqlDriveSpaceLifecycleService {
    lifecycle_store: SqlSpaceLifecycleStore,
    permission_service: SqlDrivePermissionService,
    pool: sqlx::PgPool,
}

impl SqlDriveSpaceLifecycleService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            lifecycle_store: SqlSpaceLifecycleStore::new(pool.clone()),
            permission_service: SqlDrivePermissionService::new(pool.clone()),
            pool,
        }
    }

    pub async fn bootstrap_team_space_creator_access(
        &self,
        command: BootstrapTeamSpaceCreatorAccessCommand,
    ) -> Result<(), DriveServiceError> {
        let existing_root = self
            .lifecycle_store
            .find_active_root_folder_id(&command.tenant_id, &command.space_id)
            .await?;

        let root_folder_id = if let Some(root_folder_id) = existing_root {
            root_folder_id
        } else {
            let root_name = command.display_name.trim();
            let root_name = if root_name.is_empty() {
                "Shared space"
            } else {
                root_name
            };
            self.lifecycle_store
                .insert_team_space_root_folder(
                    &command.root_folder_id,
                    &command.tenant_id,
                    &command.space_id,
                    root_name,
                    &command.creator_user_id,
                )
                .await?;
            record_drive_change(
                &self.pool,
                RecordDriveChangeCommand {
                    tenant_id: &command.tenant_id,
                    space_id: &command.space_id,
                    node_id: Some(&command.root_folder_id),
                    event_type: drive_events::node::CREATED,
                    actor_id: &command.creator_user_id,
                },
            )
            .await?;
            command.root_folder_id
        };

        self.permission_service
            .grant_node_permission(GrantDriveNodePermissionCommand {
                tenant_id: command.tenant_id,
                node_id: root_folder_id,
                subject_type: "user".to_string(),
                subject_id: command.creator_user_id.clone(),
                role: "owner".to_string(),
                operator_id: command.creator_user_id,
            })
            .await?;

        Ok(())
    }

    pub async fn retire_space_contents(
        &self,
        command: RetireSpaceContentsCommand,
    ) -> Result<i64, DriveServiceError> {
        self.lifecycle_store
            .retire_space_contents(&command.tenant_id, &command.space_id, &command.operator_id)
            .await
    }

    pub async fn delete_space_with_contents(
        &self,
        command: DeleteSpaceWithContentsCommand,
    ) -> Result<DeleteSpaceWithContentsResult, DriveServiceError> {
        let tenant_id = require_non_empty(command.tenant_id, "tenant_id")?;
        let space_id = require_non_empty(command.space_id, "space_id")?;
        let operator_id = require_non_empty(command.operator_id, "operator_id")?;

        let space_store = SqlSpaceStore::new(self.pool.clone());
        let existing = space_store.get_space(&tenant_id, &space_id).await?;
        if existing.space_type.is_non_deletable() {
            return Err(DriveServiceError::Validation(format!(
                "{} cannot be deleted",
                existing.space_type.display_label()
            )));
        }

        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire delete space with contents connection failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "begin delete space with contents transaction failed: {error}"
                ))
            })?;

        let transaction_result = async {
            let space = SqlSpaceStore::delete_space_on_connection(
                &mut connection,
                &tenant_id,
                &space_id,
                &operator_id,
            )
            .await?;
            let deleted_node_count = SqlSpaceLifecycleStore::retire_space_contents_on_connection(
                &mut connection,
                &tenant_id,
                &space_id,
                &operator_id,
            )
            .await?;
            record_drive_change_on_connection(
                &mut connection,
                RecordDriveChangeCommand {
                    tenant_id: &tenant_id,
                    space_id: &space_id,
                    node_id: None,
                    event_type: drive_events::space::DELETED,
                    actor_id: &operator_id,
                },
            )
            .await?;
            Ok((space, deleted_node_count))
        }
        .await;

        match transaction_result {
            Ok((space, deleted_node_count)) => {
                sqlx::query("COMMIT")
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "commit delete space with contents transaction failed: {error}"
                        ))
                    })?;
                sdkwork_drive_observability::metrics::record_outbox_pending();
                trigger_immediate_outbox_dispatch(self.pool.clone());
                Ok(DeleteSpaceWithContentsResult {
                    space,
                    deleted_node_count,
                })
            }
            Err(error) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                Err(error)
            }
        }
    }
}

fn require_non_empty(value: String, field_name: &str) -> Result<String, DriveServiceError> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} is required"
        )));
    }
    Ok(trimmed)
}
