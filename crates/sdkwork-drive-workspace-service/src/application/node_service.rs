use crate::domain::node::{DriveNode, DriveNodeType};
use crate::ports::node_store::{DriveNodeStore, NewDriveNode};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct CreateNodeCommand {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub parent_node_id: Option<String>,
    pub node_type: DriveNodeType,
    pub node_name: String,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct DriveNodeService<S>
where
    S: DriveNodeStore,
{
    store: S,
}

impl<S> DriveNodeService<S>
where
    S: DriveNodeStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn create_node(
        &self,
        command: CreateNodeCommand,
    ) -> Result<DriveNode, DriveServiceError> {
        if command.id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "node id is required".to_string(),
            ));
        }
        if command.node_name.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "node_name is required".to_string(),
            ));
        }
        if command.parent_node_id.as_deref() == Some(command.id.as_str()) {
            return Err(DriveServiceError::Validation(
                "parent_node_id cannot reference the node being created".to_string(),
            ));
        }
        let Some(space_type) = self
            .store
            .find_active_space_type(&command.tenant_id, &command.space_id)
            .await?
        else {
            return Err(DriveServiceError::NotFound("space not found".to_string()));
        };
        if command.parent_node_id.is_none()
            && space_type.accepts_only_root_folders()
            && command.node_type != DriveNodeType::Folder
        {
            return Err(DriveServiceError::Validation(format!(
                "{} root accepts only repository directories",
                space_type.display_label()
            )));
        }
        if let Some(parent_node_id) = command.parent_node_id.as_deref() {
            let Some(parent) = self
                .store
                .find_active_node_in_space(&command.tenant_id, &command.space_id, parent_node_id)
                .await?
            else {
                return Err(DriveServiceError::NotFound(
                    "parent node not found".to_string(),
                ));
            };
            if parent.node_type != DriveNodeType::Folder {
                return Err(DriveServiceError::Validation(
                    "parent_node_id must reference an active folder".to_string(),
                ));
            }
        }

        let duplicated = self
            .store
            .exists_live_name_in_parent(
                &command.tenant_id,
                &command.space_id,
                command.parent_node_id.as_deref(),
                &command.node_name,
            )
            .await?;
        if duplicated {
            return Err(DriveServiceError::Conflict(format!(
                "node name already exists in parent: {}",
                command.node_name
            )));
        }

        let new_node = NewDriveNode {
            id: command.id,
            tenant_id: command.tenant_id,
            space_id: command.space_id,
            space_type: space_type.as_str().to_string(),
            parent_node_id: command.parent_node_id,
            shortcut_target_node_id: None,
            node_type: command.node_type.as_str().to_string(),
            node_name: command.node_name,
            lifecycle_status: "active".to_string(),
            created_by: command.operator_id.clone(),
            updated_by: command.operator_id,
        };

        self.store.insert_node(&new_node).await
    }
}
