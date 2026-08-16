use crate::domain::root_scope_subscription::DriveRootScopeSubscription;
use crate::domain::space::DriveSpaceType;
use crate::infrastructure::sql::root_scope_subscription_store::SqlRootScopeSubscriptionStore;
use crate::ports::root_scope_subscription_store::{
    DriveRootScopeSubscriptionStore, RegisterDriveRootScopeSubscription,
    RegisterDriveRootScopeSubscriptionResult,
};
use crate::DriveServiceError;
use sqlx::PgPool;

use super::space_service::{GetSpaceCommand, SqlDriveSpaceService};
use super::workspace_service::{
    DriveWorkspaceNodeKind, EnsureDriveWorkspaceNode, EnsureDriveWorkspaceNodesCommand,
    ResolveDriveWorkspacePathCommand, SqlDriveWorkspaceService,
};

const KNOWLEDGEBASE_SPACE_ROOT_PATH: &str = "root";
const KNOWLEDGEBASE_RAW_PARENT_PATH: &str = "root/sources";
const KNOWLEDGEBASE_RAW_ROOT_PATH: &str = "root/sources/raw";

#[derive(Debug, Clone)]
pub struct RegisterKnowledgebaseRawScopeCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub knowledge_base_id: String,
    pub raw_folder_node_id: String,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct GetRootScopeSubscriptionCommand {
    pub tenant_id: String,
    pub subscription_uuid: String,
}

#[derive(Debug, Clone)]
pub struct EnsureKnowledgebaseRawScopeCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub knowledge_base_id: String,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct SqlDriveKnowledgebaseRawScopeService {
    pool: PgPool,
}

impl SqlDriveKnowledgebaseRawScopeService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn ensure_knowledgebase_raw_scope(
        &self,
        command: EnsureKnowledgebaseRawScopeCommand,
    ) -> Result<RegisterDriveRootScopeSubscriptionResult, DriveServiceError> {
        let tenant_id = require_text(command.tenant_id, "tenant_id", 64)?;
        let space_id = require_text(command.space_id, "space_id", 64)?;
        let knowledge_base_id = require_text(command.knowledge_base_id, "knowledge_base_id", 128)?;
        let operator_id = require_text(command.operator_id, "operator_id", 128)?;
        let space = SqlDriveSpaceService::new(self.pool.clone())
            .get_space(GetSpaceCommand {
                tenant_id: tenant_id.clone(),
                space_id: space_id.clone(),
            })
            .await?;
        if space.space_type != DriveSpaceType::KnowledgeBase {
            return Err(DriveServiceError::Validation(
                "knowledgebase raw scope requires a knowledge_base Space".to_string(),
            ));
        }

        let workspace = SqlDriveWorkspaceService::new(self.pool.clone());
        workspace
            .ensure_nodes(EnsureDriveWorkspaceNodesCommand {
                tenant_id: tenant_id.clone(),
                space_id: space_id.clone(),
                operator_id: operator_id.clone(),
                nodes: vec![
                    EnsureDriveWorkspaceNode::folder(KNOWLEDGEBASE_SPACE_ROOT_PATH),
                    EnsureDriveWorkspaceNode::folder(KNOWLEDGEBASE_RAW_PARENT_PATH),
                    EnsureDriveWorkspaceNode::folder(KNOWLEDGEBASE_RAW_ROOT_PATH),
                ],
            })
            .await?;
        let raw_folder = workspace
            .resolve_path(ResolveDriveWorkspacePathCommand {
                tenant_id: tenant_id.clone(),
                space_id: space_id.clone(),
                logical_path: KNOWLEDGEBASE_RAW_ROOT_PATH.to_string(),
            })
            .await?
            .ok_or_else(|| {
                DriveServiceError::NotFound(
                    "canonical root/sources/raw folder is missing".to_string(),
                )
            })?;
        if raw_folder.kind != DriveWorkspaceNodeKind::Folder
            || raw_folder.path != KNOWLEDGEBASE_RAW_ROOT_PATH
        {
            return Err(DriveServiceError::Conflict(
                "canonical root/sources/raw path is not a folder".to_string(),
            ));
        }

        DriveRootScopeSubscriptionService::new(SqlRootScopeSubscriptionStore::new(
            self.pool.clone(),
        ))
        .register_knowledgebase_raw(RegisterKnowledgebaseRawScopeCommand {
            tenant_id,
            space_id,
            knowledge_base_id,
            raw_folder_node_id: raw_folder.id,
            operator_id,
        })
        .await
    }
}

#[derive(Debug, Clone)]
pub struct DriveRootScopeSubscriptionService<S>
where
    S: DriveRootScopeSubscriptionStore,
{
    store: S,
}

impl<S> DriveRootScopeSubscriptionService<S>
where
    S: DriveRootScopeSubscriptionStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn register_knowledgebase_raw(
        &self,
        command: RegisterKnowledgebaseRawScopeCommand,
    ) -> Result<RegisterDriveRootScopeSubscriptionResult, DriveServiceError> {
        self.store
            .register_knowledgebase_raw(&RegisterDriveRootScopeSubscription {
                tenant_id: require_text(command.tenant_id, "tenant_id", 64)?,
                space_id: require_text(command.space_id, "space_id", 64)?,
                consumer_resource_id: require_text(
                    command.knowledge_base_id,
                    "knowledge_base_id",
                    128,
                )?,
                root_node_id: require_text(command.raw_folder_node_id, "raw_folder_node_id", 64)?,
                operator_id: require_text(command.operator_id, "operator_id", 128)?,
            })
            .await
    }

    pub async fn get_subscription(
        &self,
        command: GetRootScopeSubscriptionCommand,
    ) -> Result<DriveRootScopeSubscription, DriveServiceError> {
        self.store
            .get_by_uuid(
                &require_text(command.tenant_id, "tenant_id", 64)?,
                &require_text(command.subscription_uuid, "subscription_uuid", 64)?,
            )
            .await
    }
}

pub type SqlDriveRootScopeSubscriptionService =
    DriveRootScopeSubscriptionService<SqlRootScopeSubscriptionStore>;

pub(super) fn require_text(
    value: String,
    field_name: &str,
    max_length: usize,
) -> Result<String, DriveServiceError> {
    let value = value.trim().to_string();
    if value.is_empty() || value.len() > max_length {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} must contain between 1 and {max_length} characters"
        )));
    }
    Ok(value)
}
