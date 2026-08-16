use async_trait::async_trait;
use sdkwork_drive_contract::api::pagination_cursor::{decode_offset_cursor, encode_offset_cursor};
use sdkwork_utils_rust::{DEFAULT_LIST_PAGE_SIZE, MAX_LIST_PAGE_SIZE};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::ports::permission_store::{
    CheckDriveNodePermissionCommand, DriveEffectiveNodeAccess, DriveNodePermissionCheck,
    DriveNodePermissionGrant, DriveNodePermissionList, DrivePermissionStore,
    GrantDriveNodePermissionCommand, ListDriveNodePermissionsCommand,
    ResolveEffectiveNodeAccessCommand, RevokeDriveNodePermissionCommand,
};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlDrivePermissionStore {
    pool: PgPool,
}

impl SqlDrivePermissionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn find_permission(
        &self,
        tenant_id: &str,
        node_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<Option<(String, String)>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT id, role FROM dr_drive_node_permission \
             WHERE tenant_id = $1 AND node_id = $2 AND subject_type = $3 AND subject_id = $4 AND lifecycle_status = 'active'",
        )
        .bind(tenant_id)
        .bind(node_id)
        .bind(subject_type)
        .bind(subject_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("read dr_drive_node_permission failed: {error}"))
        })?;

        Ok(row.map(|row| (row.get("id"), row.get("role"))))
    }

    async fn collect_node_path_ids(
        &self,
        tenant_id: &str,
        node_id: &str,
    ) -> Result<Vec<String>, DriveServiceError> {
        let mut path = Vec::new();
        let mut current = Some(node_id.to_string());
        let mut depth = 0usize;
        while let Some(id) = current {
            depth += 1;
            if depth > 256 {
                return Err(DriveServiceError::Internal(
                    "node path exceeded maximum depth".to_string(),
                ));
            }
            let row = sqlx::query(
                "SELECT id, parent_node_id FROM dr_drive_node \
                 WHERE tenant_id = $1 AND id = $2 AND lifecycle_status != 'deleted'",
            )
            .bind(tenant_id)
            .bind(&id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!("read dr_drive_node path failed: {error}"))
            })?;
            let Some(row) = row else {
                return Err(DriveServiceError::NotFound(format!(
                    "drive node {node_id} not found or inactive"
                )));
            };
            let parent_node_id: Option<String> = row.get("parent_node_id");
            path.push(id);
            current = parent_node_id;
        }
        Ok(path)
    }
}

#[async_trait]
impl DrivePermissionStore for SqlDrivePermissionStore {
    async fn resolve_space_permission_anchor_node(
        &self,
        tenant_id: &str,
        space_id: &str,
    ) -> Result<String, DriveServiceError> {
        let space_exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(1) FROM dr_drive_space WHERE tenant_id = $1 AND id = $2 AND lifecycle_status = 'active'",
        )
        .bind(tenant_id)
        .bind(space_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("read dr_drive_space failed: {error}"))
        })?;

        if space_exists == 0 {
            return Err(DriveServiceError::NotFound(format!(
                "drive space {space_id} not found or inactive"
            )));
        }

        let node_id = sqlx::query_scalar::<_, String>(
            "SELECT id FROM dr_drive_node \
             WHERE tenant_id = $1 AND space_id = $2 AND parent_node_id IS NULL AND node_type = 'folder' AND lifecycle_status = 'active' \
             ORDER BY created_at ASC, id ASC LIMIT 1",
        )
        .bind(tenant_id)
        .bind(space_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| DriveServiceError::Internal(format!("read dr_drive_node failed: {error}")))?;

        if let Some(node_id) = node_id {
            return Ok(node_id);
        }

        let fallback_node_id = sqlx::query_scalar::<_, String>(
            "SELECT id FROM dr_drive_node \
             WHERE tenant_id = $1 AND space_id = $2 AND parent_node_id IS NULL AND lifecycle_status = 'active' \
             ORDER BY created_at ASC, id ASC LIMIT 1",
        )
        .bind(tenant_id)
        .bind(space_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| DriveServiceError::Internal(format!("read dr_drive_node failed: {error}")))?;

        fallback_node_id.ok_or_else(|| {
            DriveServiceError::NotFound(format!(
                "no root folder node found for drive space {space_id}"
            ))
        })
    }

    async fn grant_node_permission(
        &self,
        command: GrantDriveNodePermissionCommand,
    ) -> Result<DriveNodePermissionGrant, DriveServiceError> {
        if let Some((existing_id, existing_role)) = self
            .find_permission(
                &command.tenant_id,
                &command.node_id,
                &command.subject_type,
                &command.subject_id,
            )
            .await?
        {
            if existing_role == command.role {
                return Ok(DriveNodePermissionGrant {
                    id: existing_id,
                    node_id: command.node_id,
                    subject_type: command.subject_type,
                    subject_id: command.subject_id,
                    role: command.role,
                });
            }

            sqlx::query(
                "UPDATE dr_drive_node_permission SET role = $1, version = version + 1, updated_by = $2, updated_at = CURRENT_TIMESTAMP \
                 WHERE id = $3 AND lifecycle_status = 'active'",
            )
            .bind(&command.role)
            .bind(&command.operator_id)
            .bind(&existing_id)
            .execute(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!("update dr_drive_node_permission failed: {error}"))
            })?;

            return Ok(DriveNodePermissionGrant {
                id: existing_id,
                node_id: command.node_id,
                subject_type: command.subject_type,
                subject_id: command.subject_id,
                role: command.role,
            });
        }

        let node_exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(1) FROM dr_drive_node WHERE tenant_id = $1 AND id = $2 AND lifecycle_status != 'deleted'",
        )
        .bind(&command.tenant_id)
        .bind(&command.node_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("verify dr_drive_node for permission grant failed: {error}"))
        })?;
        if node_exists == 0 {
            return Err(DriveServiceError::NotFound(format!(
                "node {} not found for tenant {}",
                command.node_id, command.tenant_id
            )));
        }

        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO dr_drive_node_permission \
             (id, tenant_id, node_id, subject_type, subject_id, role, inherited, lifecycle_status, version, created_by, updated_by) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', 1, $8, $8)",
        )
        .bind(&id)
        .bind(&command.tenant_id)
        .bind(&command.node_id)
        .bind(&command.subject_type)
        .bind(&command.subject_id)
        .bind(&command.role)
        .bind(false)
        .bind(&command.operator_id)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            let message = error.to_string();
            if message.contains("UNIQUE") || message.contains("unique") {
                DriveServiceError::Conflict(format!(
                    "permission already exists for {}/{} on node {}",
                    command.subject_type, command.subject_id, command.node_id
                ))
            } else {
                DriveServiceError::Internal(format!("insert dr_drive_node_permission failed: {message}"))
            }
        })?;

        Ok(DriveNodePermissionGrant {
            id,
            node_id: command.node_id,
            subject_type: command.subject_type,
            subject_id: command.subject_id,
            role: command.role,
        })
    }

    async fn revoke_node_permission(
        &self,
        command: RevokeDriveNodePermissionCommand,
    ) -> Result<(), DriveServiceError> {
        let result = sqlx::query(
            "UPDATE dr_drive_node_permission SET lifecycle_status = 'deleted', updated_by = $1, updated_at = CURRENT_TIMESTAMP \
             WHERE tenant_id = $2 AND node_id = $3 AND subject_type = $4 AND subject_id = $5 AND lifecycle_status = 'active'",
        )
        .bind(&command.operator_id)
        .bind(&command.tenant_id)
        .bind(&command.node_id)
        .bind(&command.subject_type)
        .bind(&command.subject_id)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("delete dr_drive_node_permission failed: {error}"))
        })?;

        if result.rows_affected() == 0 {
            return Err(DriveServiceError::NotFound(format!(
                "no active permission for {}/{} on node {}",
                command.subject_type, command.subject_id, command.node_id
            )));
        }

        Ok(())
    }

    async fn list_node_permissions(
        &self,
        command: ListDriveNodePermissionsCommand,
    ) -> Result<DriveNodePermissionList, DriveServiceError> {
        let page_size = command
            .page_size
            .map(|value| i64::from(value.min(MAX_LIST_PAGE_SIZE as u32)))
            .unwrap_or(i64::from(DEFAULT_LIST_PAGE_SIZE));
        let offset = decode_offset_cursor(command.page_token.as_deref())
            .map_err(|_| DriveServiceError::Validation("page_token is invalid".to_string()))?;
        let rows = sqlx::query(
            "SELECT id, node_id, subject_type, subject_id, role \
             FROM dr_drive_node_permission \
             WHERE tenant_id = $1 AND node_id = $2 AND lifecycle_status = 'active' \
             ORDER BY created_at \
             LIMIT $3 OFFSET $4",
        )
        .bind(&command.tenant_id)
        .bind(&command.node_id)
        .bind(page_size + 1)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("list dr_drive_node_permission failed: {error}"))
        })?;

        let has_more = rows.len() > page_size as usize;
        let items = rows
            .into_iter()
            .take(page_size as usize)
            .map(|row| DriveNodePermissionGrant {
                id: row.get("id"),
                node_id: row.get("node_id"),
                subject_type: row.get("subject_type"),
                subject_id: row.get("subject_id"),
                role: row.get("role"),
            })
            .collect::<Vec<_>>();

        let next_page_token = if has_more {
            encode_offset_cursor(offset + page_size)
        } else {
            None
        };

        Ok(DriveNodePermissionList {
            items,
            next_page_token,
        })
    }

    async fn check_node_permission(
        &self,
        command: CheckDriveNodePermissionCommand,
    ) -> Result<DriveNodePermissionCheck, DriveServiceError> {
        let role = sqlx::query_scalar::<_, String>(
            "SELECT role FROM dr_drive_node_permission \
             WHERE tenant_id = $1 AND node_id = $2 AND subject_type = $3 AND subject_id = $4 AND lifecycle_status = 'active'",
        )
        .bind(&command.tenant_id)
        .bind(&command.node_id)
        .bind(&command.subject_type)
        .bind(&command.subject_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("read dr_drive_node_permission failed: {error}"))
        })?;

        Ok(match role {
            Some(role) => DriveNodePermissionCheck {
                allowed: crate::domain::permission_role::drive_role_satisfies(
                    &role,
                    &command.required_role,
                ),
                effective_role: Some(role),
            },
            None => DriveNodePermissionCheck {
                allowed: false,
                effective_role: None,
            },
        })
    }

    async fn resolve_effective_node_access(
        &self,
        command: ResolveEffectiveNodeAccessCommand,
    ) -> Result<DriveEffectiveNodeAccess, DriveServiceError> {
        if self
            .is_space_owner(
                &command.tenant_id,
                &command.space_id,
                &command.subject_type,
                &command.subject_id,
            )
            .await?
        {
            return Ok(DriveEffectiveNodeAccess {
                role: "owner".to_string(),
                source: "space_owner".to_string(),
                permission_id: None,
                inherited: false,
                inherited_from_node_id: None,
            });
        }

        let path_ids = self
            .collect_node_path_ids(&command.tenant_id, &command.node_id)
            .await?;
        for path_node_id in path_ids.iter() {
            if let Some((permission_id, role)) = self
                .find_permission(
                    &command.tenant_id,
                    path_node_id,
                    &command.subject_type,
                    &command.subject_id,
                )
                .await?
            {
                let inherited = path_node_id != &command.node_id;
                return Ok(DriveEffectiveNodeAccess {
                    role,
                    source: "permission".to_string(),
                    permission_id: Some(permission_id),
                    inherited,
                    inherited_from_node_id: inherited.then(|| path_node_id.clone()),
                });
            }
        }

        Ok(DriveEffectiveNodeAccess {
            role: "none".to_string(),
            source: "none".to_string(),
            permission_id: None,
            inherited: false,
            inherited_from_node_id: None,
        })
    }

    async fn is_space_owner(
        &self,
        tenant_id: &str,
        space_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<bool, DriveServiceError> {
        let row = sqlx::query(
            "SELECT owner_subject_type, owner_subject_id \
             FROM dr_drive_space \
             WHERE tenant_id = $1 AND id = $2 AND lifecycle_status = 'active'",
        )
        .bind(tenant_id)
        .bind(space_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("read dr_drive_space owner failed: {error}"))
        })?;
        Ok(row.is_some_and(|row| {
            let owner_subject_type: String = row.get("owner_subject_type");
            let owner_subject_id: String = row.get("owner_subject_id");
            owner_subject_type == subject_type && owner_subject_id == subject_id
        }))
    }
}
