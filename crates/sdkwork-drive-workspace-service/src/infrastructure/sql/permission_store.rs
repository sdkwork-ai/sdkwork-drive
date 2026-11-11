use async_trait::async_trait;
use sdkwork_drive_contract::api::pagination_cursor::{decode_offset_cursor, encode_offset_cursor};
use sdkwork_utils_rust::{DEFAULT_LIST_PAGE_SIZE, MAX_LIST_PAGE_SIZE};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

/// Upper bound on one batch ACL resolution.
///
/// The batch exists to remove an N+1 from material-save paths, which resolve a
/// handful to a few hundred nodes. A caller that resolves thousands of nodes in
/// one call is not saving materials; it is enumerating the space and should page.
/// The bound also keeps the recursive CTE's row count predictable.
pub const MAX_NODE_ACCESS_BATCH: usize = 1_000;

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

    /// Resolve effective node access for many nodes in one set-based query.
    ///
    /// The result for each requested node is the same decision
    /// [`DrivePermissionStore::resolve_effective_node_access`] makes for that
    /// node: space owner wins over any grant, otherwise the nearest ancestor
    /// (including the node itself) with an active grant for the subject decides,
    /// otherwise `none`. Both use the same predicates - active space owner,
    /// `dr_drive_node_permission` filtered on `lifecycle_status = 'active'`,
    /// and a recursive walk over `dr_drive_node.parent_node_id` that skips
    /// `deleted` nodes.
    ///
    /// # Contract
    ///
    /// * `tenant_id` scopes every row read; a node of another tenant is never
    ///   returned.
    /// * `node_ids` must contain at most [`MAX_NODE_ACCESS_BATCH`] entries. Ids
    ///   are deduplicated.
    /// * Only nodes that exist, are not `deleted`, and whose ancestry is
    ///   resolvable are placed in the map. An id absent from the result has no
    ///   access answer and callers must treat it as no access - the single-node
    ///   API reports `NotFound` for the same condition.
    ///
    /// # Errors
    ///
    /// `Validation` when the batch is empty of usable ids or exceeds the bound,
    /// and `Internal` when the query fails.
    pub async fn resolve_node_access_batch(
        &self,
        tenant_id: &str,
        node_ids: &[String],
        subject_type: &str,
        subject_id: &str,
    ) -> Result<HashMap<String, DriveEffectiveNodeAccess>, DriveServiceError> {
        if node_ids.len() > MAX_NODE_ACCESS_BATCH {
            return Err(DriveServiceError::Validation(format!(
                "node_ids must contain at most {MAX_NODE_ACCESS_BATCH} items"
            )));
        }
        let mut access_by_node = HashMap::with_capacity(node_ids.len());
        if node_ids.is_empty() {
            return Ok(access_by_node);
        }

        // One recursive walk over all requested nodes at once. `depth` orders the
        // ancestors so the nearest grant wins, exactly like the per-node loop's
        // node -> root iteration.
        let rows = sqlx::query(
            "WITH RECURSIVE requested(node_id) AS (
                SELECT DISTINCT CAST(requested_id AS TEXT)
                FROM unnest($2::text[]) AS requested_id
             ),
             ancestry(root_id, node_id, depth) AS (
                SELECT requested.node_id, requested.node_id, 0
                FROM requested
                INNER JOIN dr_drive_node node
                    ON node.tenant_id = $1
                   AND node.id = requested.node_id
                   AND node.lifecycle_status != 'deleted'
                UNION ALL
                SELECT ancestry.root_id, CAST(parent.id AS TEXT), ancestry.depth + 1
                FROM ancestry
                INNER JOIN dr_drive_node current_node
                    ON current_node.tenant_id = $1
                   AND current_node.id = ancestry.node_id
                   AND current_node.lifecycle_status != 'deleted'
                INNER JOIN dr_drive_node parent
                    ON parent.tenant_id = $1
                   AND parent.id = current_node.parent_node_id
                   AND parent.lifecycle_status != 'deleted'
                WHERE ancestry.depth < 256
             ),
             nearest_permission AS (
                SELECT DISTINCT ON (ancestry.root_id)
                       ancestry.root_id,
                       ancestry.node_id AS matched_node_id,
                       ancestry.depth,
                       permission.id AS permission_id,
                       permission.role
                FROM ancestry
                INNER JOIN dr_drive_node_permission permission
                    ON permission.tenant_id = $1
                   AND permission.node_id = ancestry.node_id
                   AND permission.subject_type = $3
                   AND permission.subject_id = $4
                   AND permission.lifecycle_status = 'active'
                ORDER BY ancestry.root_id, ancestry.depth ASC
             )
             SELECT requested.node_id AS requested_node_id,
                    (owner.space_id IS NOT NULL) AS is_space_owner,
                    nearest_permission.matched_node_id AS matched_node_id,
                    nearest_permission.permission_id AS permission_id,
                    nearest_permission.role AS role
             FROM requested
             INNER JOIN dr_drive_node node
                 ON node.tenant_id = $1
                AND node.id = requested.node_id
                AND node.lifecycle_status != 'deleted'
             LEFT JOIN LATERAL (
                 SELECT space.id AS space_id
                 FROM dr_drive_space space
                 WHERE space.tenant_id = $1
                   AND space.id = node.space_id
                   AND space.lifecycle_status = 'active'
                   AND space.owner_subject_type = $3
                   AND space.owner_subject_id = $4
             ) owner ON TRUE
             LEFT JOIN nearest_permission
                 ON nearest_permission.root_id = requested.node_id",
        )
        .bind(tenant_id)
        .bind(node_ids.to_vec())
        .bind(subject_type)
        .bind(subject_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "resolve batch dr_drive_node_permission access failed: {error}"
            ))
        })?;

        for row in rows {
            let requested_node_id: String = row.get("requested_node_id");
            let is_space_owner: bool = row.get("is_space_owner");
            let matched_node_id: Option<String> = row.get("matched_node_id");
            let permission_id: Option<String> = row.get("permission_id");
            let role: Option<String> = row.get("role");

            let access = if is_space_owner {
                DriveEffectiveNodeAccess {
                    role: "owner".to_string(),
                    source: "space_owner".to_string(),
                    permission_id: None,
                    inherited: false,
                    inherited_from_node_id: None,
                }
            } else if let Some(role) = role {
                let inherited = matched_node_id
                    .as_deref()
                    .is_some_and(|matched| matched != requested_node_id);
                DriveEffectiveNodeAccess {
                    role,
                    source: "permission".to_string(),
                    permission_id,
                    inherited,
                    inherited_from_node_id: inherited.then(|| matched_node_id).flatten(),
                }
            } else {
                DriveEffectiveNodeAccess {
                    role: "none".to_string(),
                    source: "none".to_string(),
                    permission_id: None,
                    inherited: false,
                    inherited_from_node_id: None,
                }
            };
            access_by_node.insert(requested_node_id, access);
        }

        Ok(access_by_node)
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
