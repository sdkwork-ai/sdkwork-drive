use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, PgPool, Row};

use crate::domain::uploader::{DriveUploadItem, DriveUploadPart};
use crate::infrastructure::sql::managed_website_tree_guard::{
    ensure_managed_website_node_mutation_allowed, ManagedWebsiteTreeSystemOverride,
    ManagedWebsiteTreeSystemOverrideReason,
};
use crate::infrastructure::sql::sql_error::is_unique_constraint_violation;
use crate::infrastructure::sql::upload_query_columns::{
    DRIVE_UPLOAD_ITEM_UI_SELECT_COLUMNS, DRIVE_UPLOAD_PART_SELECT_COLUMNS,
};
use crate::infrastructure::sql::{begin_transaction_sql, next_drive_runtime_id};
use crate::ports::permission_store::DrivePermissionStore;
use crate::ports::uploader_store::{
    CompleteDriveStoredUpload, DriveUploaderNodeRecord, DriveUploaderSpaceRecord,
    DriveUploaderStore, NewDriveUploadItem, NewDriveUploadPart, NewDriveUploaderNode,
    NewDriveUploaderSession, NewDriveUploaderSpace,
};
use crate::{drive_share_token_hash, DriveServiceError};

#[derive(Debug, Clone)]
pub struct SqlUploaderStore {
    pool: PgPool,
}

impl SqlUploaderStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveUploaderStore for SqlUploaderStore {
    async fn find_upload_space(
        &self,
        tenant_id: &str,
        owner_subject_type: &str,
        owner_subject_id: &str,
        space_type: &str,
    ) -> Result<Option<String>, DriveServiceError> {
        sqlx::query_scalar(
            "SELECT id
             FROM dr_drive_space
             WHERE tenant_id=$1
               AND owner_subject_type=$2
               AND owner_subject_id=$3
               AND space_type=$4
               AND lifecycle_status='active'",
        )
        .bind(tenant_id)
        .bind(owner_subject_type)
        .bind(owner_subject_id)
        .bind(space_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("find uploader upload space failed: {error}"))
        })
    }

    async fn find_active_space(
        &self,
        tenant_id: &str,
        space_id: &str,
    ) -> Result<Option<DriveUploaderSpaceRecord>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, owner_subject_type, owner_subject_id, space_type
             FROM dr_drive_space
             WHERE tenant_id=$1 AND id=$2 AND lifecycle_status='active'",
        )
        .bind(tenant_id)
        .bind(space_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("find uploader active space failed: {error}"))
        })?;

        Ok(row.map(|row| DriveUploaderSpaceRecord {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            owner_subject_type: row.get("owner_subject_type"),
            owner_subject_id: row.get("owner_subject_id"),
            space_type: row.get("space_type"),
        }))
    }

    async fn find_active_node(
        &self,
        tenant_id: &str,
        node_id: &str,
    ) -> Result<Option<DriveUploaderNodeRecord>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, space_id, parent_node_id, node_type
             FROM dr_drive_node
             WHERE tenant_id=$1 AND id=$2 AND lifecycle_status='active'",
        )
        .bind(tenant_id)
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("find uploader active node failed: {error}"))
        })?;

        Ok(row.map(|row| DriveUploaderNodeRecord {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            space_id: row.get("space_id"),
            parent_node_id: row.get("parent_node_id"),
            node_type: row.get("node_type"),
        }))
    }

    async fn has_writer_permission(
        &self,
        tenant_id: &str,
        node_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<bool, DriveServiceError> {
        use crate::infrastructure::sql::permission_store::SqlDrivePermissionStore;
        use crate::ports::permission_store::ResolveEffectiveNodeAccessCommand;

        let node_row = sqlx::query(
            "SELECT space_id FROM dr_drive_node \
             WHERE tenant_id=$1 AND id=$2 AND lifecycle_status='active'",
        )
        .bind(tenant_id)
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("read uploader node space failed: {error}"))
        })?;
        let Some(node_row) = node_row else {
            return Ok(false);
        };
        let space_id: String = node_row.get("space_id");
        let permission_store = SqlDrivePermissionStore::new(self.pool.clone());
        let access = permission_store
            .resolve_effective_node_access(ResolveEffectiveNodeAccessCommand {
                tenant_id: tenant_id.to_string(),
                space_id,
                node_id: node_id.to_string(),
                subject_type: subject_type.to_string(),
                subject_id: subject_id.to_string(),
            })
            .await?;
        Ok(access.allows_role("writer"))
    }

    async fn has_writer_share_token(
        &self,
        tenant_id: &str,
        node_id: &str,
        token_hash: &str,
        now_epoch_ms: i64,
    ) -> Result<bool, DriveServiceError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(1)
             FROM dr_drive_node_share_link
             WHERE tenant_id=$1
               AND node_id=$2
               AND token_hash=$3
               AND role='writer'
               AND lifecycle_status='active'
               AND (expires_at_epoch_ms IS NULL OR expires_at_epoch_ms > $4)",
        )
        .bind(tenant_id)
        .bind(node_id)
        .bind(token_hash)
        .bind(now_epoch_ms)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "check uploader writer share token failed: {error}"
            ))
        })?;
        Ok(count > 0)
    }

    async fn insert_upload_space(
        &self,
        space: &NewDriveUploaderSpace,
    ) -> Result<String, DriveServiceError> {
        let result = sqlx::query(
            "INSERT INTO dr_drive_space (
                id, tenant_id, owner_subject_type, owner_subject_id,
                space_type, display_name, lifecycle_status, version,
                created_by, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $7)",
        )
        .bind(&space.id)
        .bind(&space.tenant_id)
        .bind(&space.owner_subject_type)
        .bind(&space.owner_subject_id)
        .bind(&space.space_type)
        .bind(&space.display_name)
        .bind(&space.operator_id)
        .execute(&self.pool)
        .await;

        if let Err(error) = result {
            let message = error.to_string();
            if !is_unique_constraint_violation(&message) {
                return Err(DriveServiceError::Internal(format!(
                    "insert uploader upload space failed: {message}"
                )));
            }
        }

        self.find_upload_space(
            &space.tenant_id,
            &space.owner_subject_type,
            &space.owner_subject_id,
            &space.space_type,
        )
        .await?
        .ok_or_else(|| {
            DriveServiceError::Internal("uploader upload space was not created".to_string())
        })
    }

    async fn live_node_name_exists_in_parent(
        &self,
        tenant_id: &str,
        space_id: &str,
        parent_node_id: Option<&str>,
        node_name: &str,
    ) -> Result<bool, DriveServiceError> {
        let exists = sqlx::query_scalar::<_, i32>(
            "SELECT 1
             FROM dr_drive_node
             WHERE tenant_id=$1
               AND space_id=$2
               AND lifecycle_status = 'active'
               AND node_name = $4
               AND ((parent_node_id IS NULL AND $3 IS NULL) OR parent_node_id = $3)
             LIMIT 1",
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(parent_node_id)
        .bind(node_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("check live uploader node name failed: {error}"))
        })?;
        Ok(exists.is_some())
    }

    async fn insert_upload_node(
        &self,
        node: &NewDriveUploaderNode,
    ) -> Result<String, DriveServiceError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire uploader node transaction connection failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "begin uploader node transaction failed: {error}"
                ))
            })?;
        if let Err(error) =
            super::managed_website_tree_guard::ensure_managed_website_parent_mutation_allowed(
                &mut connection,
                &node.tenant_id,
                &node.space_id,
                node.parent_node_id.as_deref(),
            )
            .await
        {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            return Err(error);
        }
        let insert_result = sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type,
                node_name, scene, source, content_state, lifecycle_status, version,
                created_by, updated_by
            ) VALUES ($1, $2, $3, $4, 'file', $5, $6, $7, 'uploading', 'active', 1, $8, $8)",
        )
        .bind(&node.id)
        .bind(&node.tenant_id)
        .bind(&node.space_id)
        .bind(node.parent_node_id.as_deref())
        .bind(&node.node_name)
        .bind(node.scene.as_deref())
        .bind(node.source.as_deref())
        .bind(&node.operator_id)
        .execute(&mut *connection)
        .await
        .map_err(|error| {
            let message = error.to_string();
            if is_unique_constraint_violation(&message) {
                return DriveServiceError::Conflict(format!(
                    "node name already exists in parent: {}",
                    node.node_name
                ));
            }
            DriveServiceError::Internal(format!("insert uploader node failed: {message}"))
        });
        if let Err(error) = insert_result {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            return Err(error);
        }
        sqlx::query("COMMIT")
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "commit uploader node transaction failed: {error}"
                ))
            })?;

        Ok(node.id.clone())
    }

    async fn insert_upload_session(
        &self,
        session: &NewDriveUploaderSession,
    ) -> Result<String, DriveServiceError> {
        sqlx::query(
            "INSERT INTO dr_drive_upload_session (
                id, tenant_id, space_id, node_id, bucket, object_key,
                idempotency_key, storage_provider_id, storage_upload_id,
                state, expires_at_epoch_ms, version, created_by, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $1, 'created', $9, 1, $10, $10)",
        )
        .bind(&session.id)
        .bind(&session.tenant_id)
        .bind(&session.space_id)
        .bind(&session.node_id)
        .bind(&session.bucket)
        .bind(&session.object_key)
        .bind(format!("uploader-{}", session.id))
        .bind(&session.storage_provider_id)
        .bind(session.expires_at_epoch_ms)
        .bind(&session.operator_id)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("insert uploader session failed: {error}"))
        })?;

        Ok(session.id.clone())
    }

    async fn find_default_storage_provider(
        &self,
        tenant_id: &str,
    ) -> Result<Option<(String, String)>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT id, bucket
             FROM dr_drive_storage_provider
             WHERE status='active'
             ORDER BY created_at ASC
             LIMIT 1",
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("find uploader storage provider failed: {error}"))
        })?;

        Ok(row.map(|row| (row.get("id"), row.get("bucket"))))
    }

    async fn insert_upload_item(
        &self,
        item: &NewDriveUploadItem,
    ) -> Result<DriveUploadItem, DriveServiceError> {
        sqlx::query(
            "INSERT INTO dr_drive_upload_item (
                id, task_id, tenant_id, organization_id, user_id,
                actor_type, actor_id, app_id, app_resource_type, app_resource_id,
                scene, source, upload_profile_code, file_fingerprint, space_id, node_id,
                upload_session_id, storage_provider_id, storage_upload_id,
                original_file_name, file_extension, content_type, content_type_group,
                detected_content_type, content_length, checksum_sha256_hex,
                chunk_size_bytes, total_parts, status, retention_mode,
                retention_expires_at_epoch_ms, cleanup_action, hard_delete_after_epoch_ms,
                created_by, updated_by
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16,
                $17, $18, $19,
                $20, $21, $22, $23,
                $24, $25, $26,
                $27, $28, $29, $30,
                $31, $32, $33,
                $34, $35
            )",
        )
        .bind(&item.id)
        .bind(&item.task_id)
        .bind(&item.tenant_id)
        .bind(&item.organization_id)
        .bind(&item.user_id)
        .bind(&item.actor_type)
        .bind(&item.actor_id)
        .bind(&item.app_id)
        .bind(&item.app_resource_type)
        .bind(&item.app_resource_id)
        .bind(&item.scene)
        .bind(&item.source)
        .bind(&item.upload_profile_code)
        .bind(&item.file_fingerprint)
        .bind(&item.space_id)
        .bind(&item.node_id)
        .bind(&item.upload_session_id)
        .bind(&item.storage_provider_id)
        .bind(&item.storage_upload_id)
        .bind(&item.original_file_name)
        .bind(&item.file_extension)
        .bind(&item.content_type)
        .bind(&item.content_type_group)
        .bind(&item.detected_content_type)
        .bind(item.content_length)
        .bind(&item.checksum_sha256_hex)
        .bind(item.chunk_size_bytes)
        .bind(item.total_parts)
        .bind(&item.status)
        .bind(&item.retention_mode)
        .bind(item.retention_expires_at_epoch_ms)
        .bind(&item.cleanup_action)
        .bind(item.hard_delete_after_epoch_ms)
        .bind(&item.created_by)
        .bind(&item.updated_by)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("insert dr_drive_upload_item failed: {error}"))
        })?;

        self.find_upload_item_by_task(&item.tenant_id, &item.task_id)
            .await?
            .ok_or_else(|| {
                DriveServiceError::Internal("inserted upload item not found".to_string())
            })
    }

    async fn find_upload_item_by_task(
        &self,
        tenant_id: &str,
        task_id: &str,
    ) -> Result<Option<DriveUploadItem>, DriveServiceError> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {DRIVE_UPLOAD_ITEM_UI_SELECT_COLUMNS},
                    us.bucket AS object_bucket,
                    us.object_key AS object_key
             FROM dr_drive_upload_item ui
             LEFT JOIN dr_drive_upload_session us
               ON us.tenant_id=ui.tenant_id
              AND us.id=ui.upload_session_id
             WHERE ui.tenant_id=$1 AND ui.task_id=$2",
        )))
        .bind(tenant_id)
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("find dr_drive_upload_item failed: {error}"))
        })?;

        row.as_ref().map(map_row_to_upload_item).transpose()
    }

    async fn record_uploaded_part(
        &self,
        part: &NewDriveUploadPart,
    ) -> Result<DriveUploadPart, DriveServiceError> {
        let existing = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {DRIVE_UPLOAD_PART_SELECT_COLUMNS}
             FROM dr_drive_upload_part
             WHERE tenant_id=$1 AND upload_item_id=$2 AND part_no=$3",
        )))
        .bind(&part.tenant_id)
        .bind(&part.upload_item_id)
        .bind(part.part_no)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("find dr_drive_upload_part failed: {error}"))
        })?;
        if let Some(row) = existing {
            let existing = map_row_to_upload_part(&row)?;
            if existing.etag != part.etag || existing.size_bytes != part.size_bytes {
                return Err(DriveServiceError::Conflict(
                    "uploaded part already exists with different metadata".to_string(),
                ));
            }
            return Ok(existing);
        }

        sqlx::query(
            "INSERT INTO dr_drive_upload_part (
                id, tenant_id, upload_item_id, upload_session_id, part_no,
                offset_bytes, size_bytes, etag, checksum_sha256_hex,
                status, retry_count, uploaded_at_epoch_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'uploaded', 0, $10)",
        )
        .bind(&part.id)
        .bind(&part.tenant_id)
        .bind(&part.upload_item_id)
        .bind(&part.upload_session_id)
        .bind(part.part_no)
        .bind(part.offset_bytes)
        .bind(part.size_bytes)
        .bind(&part.etag)
        .bind(&part.checksum_sha256_hex)
        .bind(part.uploaded_at_epoch_ms)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("insert dr_drive_upload_part failed: {error}"))
        })?;

        sqlx::query(
            "UPDATE dr_drive_upload_item
             SET uploaded_parts_count=uploaded_parts_count + 1,
                 uploaded_bytes=uploaded_bytes + $1,
                 updated_at=CURRENT_TIMESTAMP
             WHERE tenant_id=$2 AND id=$3",
        )
        .bind(part.size_bytes)
        .bind(&part.tenant_id)
        .bind(&part.upload_item_id)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "update dr_drive_upload_item counters failed: {error}"
            ))
        })?;

        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {DRIVE_UPLOAD_PART_SELECT_COLUMNS}
             FROM dr_drive_upload_part
             WHERE tenant_id=$1 AND upload_item_id=$2 AND part_no=$3",
        )))
        .bind(&part.tenant_id)
        .bind(&part.upload_item_id)
        .bind(part.part_no)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("read dr_drive_upload_part failed: {error}"))
        })?;
        map_row_to_upload_part(&row)
    }

    async fn complete_stored_upload(
        &self,
        completion: &CompleteDriveStoredUpload,
    ) -> Result<DriveUploadItem, DriveServiceError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire uploader completion transaction connection failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "begin uploader completion transaction failed: {error}"
                ))
            })?;

        let result = complete_stored_upload_in_transaction(&mut connection, completion).await;
        match result {
            Ok(item) => {
                sqlx::query("COMMIT")
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "commit uploader completion transaction failed: {error}"
                        ))
                    })?;
                sdkwork_drive_observability::metrics::record_outbox_pending();
                crate::infrastructure::outbox_dispatch::trigger_immediate_outbox_dispatch(
                    self.pool.clone(),
                );
                Ok(item)
            }
            Err(error) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                Err(error)
            }
        }
    }

    async fn quarantine_blocked_upload_content(
        &self,
        tenant_id: &str,
        upload_item_id: &str,
        operator_id: &str,
    ) -> Result<(), DriveServiceError> {
        use crate::infrastructure::change_recorder::{
            record_drive_change_on_connection, RecordDriveChangeCommand,
        };
        use sdkwork_drive_contract::drive::domain_events as drive_events;

        let row = sqlx::query(
            "SELECT ui.space_id, ui.node_id, ui.organization_id, ui.user_id,
                    ui.content_type, ui.content_type_group, ui.content_length,
                    ui.checksum_sha256_hex, ui.upload_session_id,
                    us.bucket AS object_bucket, us.object_key AS object_key
             FROM dr_drive_upload_item ui
             LEFT JOIN dr_drive_upload_session us
               ON us.tenant_id = ui.tenant_id
              AND us.id = ui.upload_session_id
             WHERE ui.tenant_id = $1 AND ui.id = $2",
        )
        .bind(tenant_id)
        .bind(upload_item_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "find dr_drive_upload_item for quarantine failed: {error}"
            ))
        })?
        .ok_or_else(|| {
            DriveServiceError::NotFound(format!("upload item not found: {upload_item_id}"))
        })?;

        let space_id: String = row.get("space_id");
        let node_id: String = row.get("node_id");
        let upload_session_id: Option<String> = row.get("upload_session_id");

        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire quarantine transaction connection failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!("begin quarantine transaction failed: {error}"))
            })?;

        let transaction_result: Result<(), DriveServiceError> = async {
            let system_override = match ensure_managed_website_node_mutation_allowed(
                &mut connection,
                tenant_id,
                &node_id,
            )
            .await
            {
                Ok(()) => None,
                Err(DriveServiceError::Conflict(_)) => {
                    Some(ManagedWebsiteTreeSystemOverride::authorize(
                        ManagedWebsiteTreeSystemOverrideReason::ContentPolicyQuarantine,
                        operator_id,
                    )?)
                }
                Err(error) => return Err(error),
            };

            sqlx::query(
                "UPDATE dr_drive_node
                 SET lifecycle_status='trashed',
                     version=version + 1,
                     updated_by=$1,
                     updated_at=CURRENT_TIMESTAMP
                 WHERE tenant_id=$2
                   AND id=$3
                   AND lifecycle_status='active'",
            )
            .bind(operator_id)
            .bind(tenant_id)
            .bind(&node_id)
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "quarantine blocked upload node failed: {error}"
                ))
            })?;

            sqlx::query(
                "UPDATE dr_drive_upload_item
                 SET status='failed',
                     cleanup_status='failed',
                     updated_by=$1,
                     updated_at=CURRENT_TIMESTAMP
                 WHERE tenant_id=$2 AND id=$3",
            )
            .bind(operator_id)
            .bind(tenant_id)
            .bind(upload_item_id)
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "mark quarantined dr_drive_upload_item failed: {error}"
                ))
            })?;

            if let Some(session_id) = upload_session_id.as_deref() {
                sqlx::query(
                    "UPDATE dr_drive_upload_session
                     SET state='aborted',
                         updated_by=$1,
                         updated_at=CURRENT_TIMESTAMP,
                         version=version + 1
                     WHERE tenant_id=$2
                       AND id=$3
                       AND state IN ('created', 'uploading', 'completing')",
                )
                .bind(operator_id)
                .bind(tenant_id)
                .bind(session_id)
                .execute(&mut *connection)
                .await
                .map_err(|error| {
                    DriveServiceError::Internal(format!(
                        "abort quarantined dr_drive_upload_session failed: {error}"
                    ))
                })?;
            }

            let sensitive_operation_id = format!("fso-quarantine-{upload_item_id}");
            sqlx::query(
                "INSERT INTO dr_drive_file_sensitive_operation (
                    id, tenant_id, organization_id, user_id, space_id, node_id,
                    storage_object_id, upload_item_id, operation_type, operation_reason,
                    content_type, content_type_group, content_length, checksum_sha256_hex,
                    object_bucket, object_key, before_lifecycle_status, after_lifecycle_status,
                    operator_id, maintenance_job_id, request_id, trace_id, object_delete_status
                ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    NULL, $7, 'soft_delete', 'system',
                    $8, $9, $10, $11,
                    $12, $13, 'active', 'trashed',
                    $14, NULL, NULL, NULL, 'not_required'
                )",
            )
            .bind(sensitive_operation_id)
            .bind(tenant_id)
            .bind(row.get::<Option<String>, _>("organization_id"))
            .bind(row.get::<Option<String>, _>("user_id"))
            .bind(&space_id)
            .bind(&node_id)
            .bind(upload_item_id)
            .bind(row.get::<String, _>("content_type"))
            .bind(row.get::<String, _>("content_type_group"))
            .bind(row.get::<i64, _>("content_length"))
            .bind(row.get::<Option<String>, _>("checksum_sha256_hex"))
            .bind(row.get::<Option<String>, _>("object_bucket"))
            .bind(row.get::<Option<String>, _>("object_key"))
            .bind(operator_id)
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "insert quarantine dr_drive_file_sensitive_operation failed: {error}"
                ))
            })?;

            if let Some(system_override) = system_override {
                system_override
                    .record_on_connection(&mut connection, tenant_id, "drive_node", &node_id)
                    .await?;
            }

            record_drive_change_on_connection(
                &mut connection,
                RecordDriveChangeCommand {
                    tenant_id,
                    space_id: &space_id,
                    node_id: Some(&node_id),
                    event_type: drive_events::object::QUARANTINED,
                    actor_id: operator_id,
                },
            )
            .await?;

            Ok(())
        }
        .await;

        match transaction_result {
            Ok(()) => {
                sqlx::query("COMMIT")
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "commit quarantine transaction failed: {error}"
                        ))
                    })?;
                sdkwork_drive_observability::metrics::record_outbox_pending();
                crate::infrastructure::outbox_dispatch::trigger_immediate_outbox_dispatch(
                    self.pool.clone(),
                );
                Ok(())
            }
            Err(error) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                Err(error)
            }
        }
    }
}

struct StoredUploadCompletionTarget {
    item: DriveUploadItem,
    session_state: String,
    node_content_state: String,
}

struct StoredUploadObject {
    id: String,
    version_no: i64,
    content_type: String,
    content_length: i64,
    checksum_sha256_hex: String,
}

async fn complete_stored_upload_in_transaction(
    connection: &mut PgConnection,
    completion: &CompleteDriveStoredUpload,
) -> Result<DriveUploadItem, DriveServiceError> {
    let target = find_stored_upload_completion_target(connection, completion).await?;
    let bucket =
        target.item.object_bucket.as_deref().ok_or_else(|| {
            DriveServiceError::Internal("upload item is missing bucket".to_string())
        })?;
    let object_key = target.item.object_key.as_deref().ok_or_else(|| {
        DriveServiceError::Internal("upload item is missing object key".to_string())
    })?;
    let storage_provider_id = target.item.storage_provider_id.as_deref().ok_or_else(|| {
        DriveServiceError::Internal("upload item is missing storage provider".to_string())
    })?;
    if target.item.content_type != completion.content_type {
        return Err(DriveServiceError::Conflict(
            "stored upload content_type does not match prepared upload item".to_string(),
        ));
    }
    if target.item.content_length != completion.content_length {
        return Err(DriveServiceError::Conflict(
            "stored upload content_length does not match prepared upload item".to_string(),
        ));
    }
    if completion.uploaded_parts_count > target.item.total_parts {
        return Err(DriveServiceError::Validation(
            "uploaded_parts_count must not exceed total_parts".to_string(),
        ));
    }

    if target.item.status == "completed" || target.session_state == "completed" {
        let stored_object = find_active_stored_upload_object(
            connection,
            &completion.tenant_id,
            &target.item.node_id,
            bucket,
            object_key,
        )
        .await?
        .ok_or_else(|| {
            DriveServiceError::Internal(
                "completed upload item is missing active storage object".to_string(),
            )
        })?;
        ensure_stored_object_matches_completion(&stored_object, completion)?;
        return read_upload_item_by_id(connection, &completion.tenant_id, &target.item.id).await;
    }

    super::managed_website_tree_guard::ensure_managed_website_node_mutation_allowed(
        connection,
        &completion.tenant_id,
        &target.item.node_id,
    )
    .await?;

    if !matches!(
        target.item.status.as_str(),
        "prepared" | "uploading" | "paused" | "completing"
    ) {
        return Err(DriveServiceError::Conflict(format!(
            "upload item cannot be completed from {} status",
            target.item.status
        )));
    }
    if !matches!(
        target.session_state.as_str(),
        "created" | "uploading" | "completing"
    ) {
        return Err(DriveServiceError::Conflict(format!(
            "upload session cannot be completed from {} state",
            target.session_state
        )));
    }

    let storage_object = match find_active_stored_upload_object(
        connection,
        &completion.tenant_id,
        &target.item.node_id,
        bucket,
        object_key,
    )
    .await?
    {
        Some(stored_object) => {
            ensure_stored_object_matches_completion(&stored_object, completion)?;
            stored_object
        }
        None => {
            insert_stored_upload_object(
                connection,
                completion,
                &target.item,
                storage_provider_id,
                bucket,
                object_key,
            )
            .await?
        }
    };

    sqlx::query(
        "UPDATE dr_drive_upload_item
         SET status='completed',
             checksum_sha256_hex=$1,
             uploaded_bytes=$2,
             uploaded_parts_count=$3,
             updated_by=$4,
             updated_at=CURRENT_TIMESTAMP
         WHERE tenant_id=$5
           AND id=$6
           AND upload_session_id=$7",
    )
    .bind(&completion.checksum_sha256_hex)
    .bind(completion.content_length)
    .bind(completion.uploaded_parts_count)
    .bind(&completion.operator_id)
    .bind(&completion.tenant_id)
    .bind(&target.item.id)
    .bind(&completion.upload_session_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("complete dr_drive_upload_item failed: {error}"))
    })?;

    sqlx::query(
        "UPDATE dr_drive_upload_session
         SET state='completed',
             updated_by=$1,
             updated_at=CURRENT_TIMESTAMP,
             version=version + 1
         WHERE tenant_id=$2 AND id=$3",
    )
    .bind(&completion.operator_id)
    .bind(&completion.tenant_id)
    .bind(&completion.upload_session_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("complete dr_drive_upload_session failed: {error}"))
    })?;

    super::node_head_metadata::apply_file_node_head_snapshot_in_transaction(
        connection,
        &completion.tenant_id,
        &target.item.node_id,
        &completion.operator_id,
        &super::node_head_metadata::FileNodeHeadSnapshot {
            file_extension: target.item.file_extension.clone(),
            content_type: storage_object.content_type.clone(),
            content_type_group: target.item.content_type_group.clone(),
            content_length: storage_object.content_length,
            version_no: storage_object.version_no,
            checksum_sha256_hex: storage_object.checksum_sha256_hex.clone(),
        },
    )
    .await?;

    insert_upload_completed_sensitive_operation(
        connection,
        completion,
        &target,
        &storage_object,
        bucket,
        object_key,
    )
    .await?;
    let node_version_id =
        insert_stored_upload_node_version(connection, &target, &storage_object, completion).await?;
    crate::infrastructure::change_recorder::record_drive_node_version_committed_on_connection(
        connection,
        crate::infrastructure::change_recorder::RecordDriveNodeVersionCommittedCommand {
            tenant_id: &completion.tenant_id,
            organization_id: target.item.organization_id.as_deref(),
            space_id: &target.item.space_id,
            node_id: &target.item.node_id,
            node_version_id: &node_version_id,
            version_no: storage_object.version_no,
            operation_id: &target.item.id,
            content_type: &storage_object.content_type,
            content_length: storage_object.content_length,
            checksum_sha256_hex: &storage_object.checksum_sha256_hex,
            actor_id: &completion.operator_id,
        },
    )
    .await?;

    read_upload_item_by_id(connection, &completion.tenant_id, &target.item.id).await
}

async fn insert_stored_upload_node_version(
    connection: &mut PgConnection,
    target: &StoredUploadCompletionTarget,
    storage_object: &StoredUploadObject,
    completion: &CompleteDriveStoredUpload,
) -> Result<String, DriveServiceError> {
    let node_version_id = next_drive_runtime_id("Drive node version")?.to_string();
    sqlx::query(
        "INSERT INTO dr_drive_node_version (
            id, tenant_id, space_id, node_id, version_no, storage_object_id,
            content_type, content_length, checksum_sha256_hex, version_kind,
            version_label, change_source, change_summary, restored_from_version_id,
            app_id, app_resource_type, app_resource_id, scene, source,
            lifecycle_status, created_by, updated_by
         ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, 'auto',
            NULL, 'uploader', NULL, NULL, $10, $11, $12, $13, $14,
            'active', $15, $15
         )",
    )
    .bind(&node_version_id)
    .bind(&completion.tenant_id)
    .bind(&target.item.space_id)
    .bind(&target.item.node_id)
    .bind(storage_object.version_no)
    .bind(&storage_object.id)
    .bind(&storage_object.content_type)
    .bind(storage_object.content_length)
    .bind(&storage_object.checksum_sha256_hex)
    .bind(&target.item.app_id)
    .bind(&target.item.app_resource_type)
    .bind(&target.item.app_resource_id)
    .bind(target.item.scene.as_deref())
    .bind(target.item.source.as_deref())
    .bind(&completion.operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "insert uploaded dr_drive_node_version failed: {error}"
        ))
    })?;
    Ok(node_version_id)
}

async fn find_stored_upload_completion_target(
    connection: &mut PgConnection,
    completion: &CompleteDriveStoredUpload,
) -> Result<StoredUploadCompletionTarget, DriveServiceError> {
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {DRIVE_UPLOAD_ITEM_UI_SELECT_COLUMNS},
                us.bucket AS object_bucket,
                us.object_key AS object_key,
                us.state AS upload_session_state,
                n.content_state AS node_content_state
         FROM dr_drive_upload_item ui
         INNER JOIN dr_drive_upload_session us
            ON us.tenant_id=ui.tenant_id
           AND us.id=ui.upload_session_id
         INNER JOIN dr_drive_node n
            ON n.tenant_id=ui.tenant_id
           AND n.id=ui.node_id
           AND n.lifecycle_status != 'deleted'
         WHERE ui.tenant_id=$1
           AND ui.id=$2
           AND ui.upload_session_id=$3",
    )))
    .bind(&completion.tenant_id)
    .bind(&completion.upload_item_id)
    .bind(&completion.upload_session_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "find stored upload completion target failed: {error}"
        ))
    })?;
    let Some(row) = row else {
        return Err(DriveServiceError::NotFound(
            "upload item or upload session not found".to_string(),
        ));
    };
    Ok(StoredUploadCompletionTarget {
        item: map_row_to_upload_item(&row)?,
        session_state: row.get("upload_session_state"),
        node_content_state: row.get("node_content_state"),
    })
}

async fn read_upload_item_by_id(
    connection: &mut PgConnection,
    tenant_id: &str,
    upload_item_id: &str,
) -> Result<DriveUploadItem, DriveServiceError> {
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {DRIVE_UPLOAD_ITEM_UI_SELECT_COLUMNS},
                us.bucket AS object_bucket,
                us.object_key AS object_key
         FROM dr_drive_upload_item ui
         LEFT JOIN dr_drive_upload_session us
            ON us.tenant_id=ui.tenant_id
           AND us.id=ui.upload_session_id
         WHERE ui.tenant_id=$1 AND ui.id=$2",
    )))
    .bind(tenant_id)
    .bind(upload_item_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("read completed upload item failed: {error}"))
    })?;
    row.as_ref()
        .map(map_row_to_upload_item)
        .transpose()?
        .ok_or_else(|| DriveServiceError::NotFound("upload item not found".to_string()))
}

async fn find_active_stored_upload_object(
    connection: &mut PgConnection,
    tenant_id: &str,
    node_id: &str,
    bucket: &str,
    object_key: &str,
) -> Result<Option<StoredUploadObject>, DriveServiceError> {
    let row = sqlx::query(
        "SELECT id, version_no, content_type, content_length, checksum_sha256_hex
         FROM dr_drive_storage_object
         WHERE tenant_id=$1
           AND node_id=$2
           AND bucket=$3
           AND object_key=$4
           AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(node_id)
    .bind(bucket)
    .bind(object_key)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("find active stored upload object failed: {error}"))
    })?;
    Ok(row.map(|row| StoredUploadObject {
        id: row.get("id"),
        version_no: row.get("version_no"),
        content_type: row.get("content_type"),
        content_length: row.get("content_length"),
        checksum_sha256_hex: row.get("checksum_sha256_hex"),
    }))
}

async fn insert_stored_upload_object(
    connection: &mut PgConnection,
    completion: &CompleteDriveStoredUpload,
    item: &DriveUploadItem,
    storage_provider_id: &str,
    bucket: &str,
    object_key: &str,
) -> Result<StoredUploadObject, DriveServiceError> {
    let next_version_no: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version_no), 0) + 1
         FROM dr_drive_storage_object
         WHERE tenant_id=$1 AND node_id=$2",
    )
    .bind(&completion.tenant_id)
    .bind(&item.node_id)
    .fetch_one(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "allocate stored upload object version failed: {error}"
        ))
    })?;
    let storage_object_id = format!("{}-v{}", completion.upload_session_id, next_version_no);
    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            scene, source, content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'active', $13, $13)",
    )
    .bind(&storage_object_id)
    .bind(&completion.tenant_id)
    .bind(&item.node_id)
    .bind(next_version_no)
    .bind(storage_provider_id)
    .bind(bucket)
    .bind(object_key)
    .bind(&item.scene)
    .bind(&item.source)
    .bind(&completion.content_type)
    .bind(completion.content_length)
    .bind(&completion.checksum_sha256_hex)
    .bind(&completion.operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("insert stored upload object failed: {error}"))
    })?;
    Ok(StoredUploadObject {
        id: storage_object_id,
        version_no: next_version_no,
        content_type: completion.content_type.clone(),
        content_length: completion.content_length,
        checksum_sha256_hex: completion.checksum_sha256_hex.clone(),
    })
}

fn ensure_stored_object_matches_completion(
    object: &StoredUploadObject,
    completion: &CompleteDriveStoredUpload,
) -> Result<(), DriveServiceError> {
    if object.content_type != completion.content_type
        || object.content_length != completion.content_length
        || object.checksum_sha256_hex != completion.checksum_sha256_hex
    {
        return Err(DriveServiceError::Conflict(
            "stored upload was already completed with different content metadata".to_string(),
        ));
    }
    Ok(())
}

async fn insert_upload_completed_sensitive_operation(
    connection: &mut PgConnection,
    completion: &CompleteDriveStoredUpload,
    target: &StoredUploadCompletionTarget,
    storage_object: &StoredUploadObject,
    bucket: &str,
    object_key: &str,
) -> Result<(), DriveServiceError> {
    sqlx::query(
        "INSERT INTO dr_drive_file_sensitive_operation (
            id, tenant_id, organization_id, user_id, space_id, node_id,
            storage_object_id, upload_item_id, operation_type, operation_reason,
            content_type, content_type_group, content_length, checksum_sha256_hex,
            object_bucket, object_key, before_lifecycle_status, after_lifecycle_status,
            operator_id, maintenance_job_id, request_id, trace_id, object_delete_status
         ) VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, 'upload_completed', 'user_request',
            $9, $10, $11, $12,
            $13, $14, $15, 'active',
            $16, NULL, NULL, NULL, 'not_required'
         )
         ON CONFLICT(id) DO NOTHING",
    )
    .bind(sensitive_operation_id(
        &completion.tenant_id,
        "upload_completed",
        &target.item.id,
        &storage_object.id,
    ))
    .bind(&completion.tenant_id)
    .bind(&target.item.organization_id)
    .bind(&target.item.user_id)
    .bind(&target.item.space_id)
    .bind(&target.item.node_id)
    .bind(&storage_object.id)
    .bind(&target.item.id)
    .bind(&completion.content_type)
    .bind(&target.item.content_type_group)
    .bind(completion.content_length)
    .bind(&completion.checksum_sha256_hex)
    .bind(bucket)
    .bind(object_key)
    .bind(&target.node_content_state)
    .bind(&completion.operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "insert stored upload completion operation failed: {error}"
        ))
    })?;
    Ok(())
}

fn sensitive_operation_id(
    tenant_id: &str,
    operation_type: &str,
    upload_item_id: &str,
    storage_object_id: &str,
) -> String {
    let raw = format!("{tenant_id}:{operation_type}:{upload_item_id}:{storage_object_id}");
    drive_share_token_hash(&raw)
        .strip_prefix("sha256:")
        .expect("drive_share_token_hash should include prefix")
        .to_string()
}

fn map_row_to_upload_item(row: &PgRow) -> Result<DriveUploadItem, DriveServiceError> {
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

fn map_row_to_upload_part(row: &PgRow) -> Result<DriveUploadPart, DriveServiceError> {
    Ok(DriveUploadPart {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        upload_item_id: row.get("upload_item_id"),
        upload_session_id: row.get("upload_session_id"),
        part_no: i64::from(row.get::<i32, _>("part_no")),
        offset_bytes: row.get("offset_bytes"),
        size_bytes: row.get("size_bytes"),
        etag: row.get("etag"),
        checksum_sha256_hex: row.get("checksum_sha256_hex"),
        status: row.get("status"),
        retry_count: i64::from(row.get::<i32, _>("retry_count")),
        uploaded_at_epoch_ms: row.get("uploaded_at_epoch_ms"),
    })
}
