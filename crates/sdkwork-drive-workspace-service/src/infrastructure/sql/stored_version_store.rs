use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::application::upload_session_rules::{
    validate_content_type, validate_object_key, validate_sha256_checksum,
};
use crate::domain::node_version::{
    CreateDriveNodeVersionCommand, DriveNodeVersion, DriveNodeVersionChangeSource,
    DriveNodeVersionKind,
};
use crate::domain::uploader::content_type_group_for;
use crate::infrastructure::change_recorder::{
    record_drive_node_version_committed_on_connection, RecordDriveNodeVersionCommittedCommand,
};
use crate::infrastructure::sql::begin_transaction_sql;
use crate::infrastructure::sql::managed_website_tree_guard::ensure_managed_website_node_mutation_allowed;
use crate::infrastructure::sql::node_head_metadata::{
    apply_file_node_head_snapshot_in_transaction, file_extension_from_name, FileNodeHeadSnapshot,
};
use crate::infrastructure::sql::node_version_store::{
    create_node_version_in_transaction, find_node_version_by_storage_object,
};
use crate::infrastructure::sql::sql_error::is_unique_constraint_violation;
use crate::infrastructure::sql::storage_object_store::next_storage_object_version_no_on_connection;
use crate::infrastructure::sql::uploader_store::{
    ensure_stored_object_matches, find_active_stored_upload_object,
};
use crate::ports::stored_version_store::{
    CompleteStoredVersionUploadCommand, DriveStoredVersionStore,
};
use crate::DriveServiceError;

/// PostgreSQL repository for stored-version completion.
///
/// The write shape mirrors upload-session completion exactly - storage object,
/// node version, node head snapshot and the `node.version_committed` event in one
/// transaction - because the two paths publish the same kind of fact. It reuses
/// the node-version repository insert instead of carrying a second column map.
#[derive(Debug, Clone)]
pub struct SqlStoredVersionStore {
    pool: PgPool,
}

impl SqlStoredVersionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveStoredVersionStore for SqlStoredVersionStore {
    async fn complete_stored_version(
        &self,
        command: &CompleteStoredVersionUploadCommand,
    ) -> Result<DriveNodeVersion, DriveServiceError> {
        let tenant_id = require_text(&command.tenant_id, "tenant_id")?;
        let space_id = require_text(&command.space_id, "space_id")?;
        let node_id = require_text(&command.node_id, "node_id")?;
        let storage_provider_id =
            require_text(&command.storage_provider_id, "storage_provider_id")?;
        let bucket = require_text(&command.bucket, "bucket")?;
        let object_key = validate_object_key(&command.object_key, "object_key")?;
        let content_type = validate_content_type(&command.content_type, "content_type")?;
        let checksum_sha256_hex =
            validate_sha256_checksum(&command.checksum_sha256_hex, "checksum_sha256_hex")?;
        if command.content_length < 0 {
            return Err(DriveServiceError::Validation(
                "content_length must be greater than or equal to 0".to_string(),
            ));
        }
        let idempotency_key = require_text(&command.idempotency_key, "idempotency_key")?;
        let operator_id = require_text(&command.operator_id, "operator_id")?;

        let node = self
            .load_active_file_node(&tenant_id, &node_id)
            .await
            .ok_or_else(|| DriveServiceError::NotFound("node not found".to_string()))?;
        if node.space_id != space_id {
            return Err(DriveServiceError::NotFound("node not found".to_string()));
        }

        if let Some(version) = self
            .replay_existing_version(
                &tenant_id,
                &node_id,
                &bucket,
                &object_key,
                content_type,
                command.content_length,
                checksum_sha256_hex,
            )
            .await?
        {
            return Ok(version);
        }

        let write_result = self
            .insert_version(
                &tenant_id,
                &space_id,
                &node_id,
                &node,
                &storage_provider_id,
                &bucket,
                &object_key,
                content_type,
                command.content_length,
                checksum_sha256_hex,
                &idempotency_key,
                &operator_id,
            )
            .await;

        match write_result {
            Ok(version) => Ok(version),
            Err(error) => {
                // A concurrent identical completion may have won the unique
                // locator/version index between our replay check and our insert.
                // If its object now matches this request, answer with its version
                // instead of failing an idempotent retry.
                if matches!(error, DriveServiceError::Conflict(_)) {
                    if let Some(version) = self
                        .replay_existing_version(
                            &tenant_id,
                            &node_id,
                            &bucket,
                            &object_key,
                            content_type,
                            command.content_length,
                            checksum_sha256_hex,
                        )
                        .await?
                    {
                        return Ok(version);
                    }
                }
                Err(error)
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ActiveFileNode {
    space_id: String,
    node_name: String,
    file_extension: Option<String>,
}

impl SqlStoredVersionStore {
    async fn load_active_file_node(
        &self,
        tenant_id: &str,
        node_id: &str,
    ) -> Option<ActiveFileNode> {
        let row = sqlx::query(
            "SELECT node.space_id, node.node_name, node.file_extension
             FROM dr_drive_node node
             INNER JOIN dr_drive_space space
                ON space.tenant_id = node.tenant_id
               AND space.id = node.space_id
               AND space.lifecycle_status = 'active'
             WHERE node.tenant_id=$1
               AND node.id=$2
               AND node.lifecycle_status='active'
               AND node.node_type='file'",
        )
        .bind(tenant_id)
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()?;
        Some(ActiveFileNode {
            space_id: row.get("space_id"),
            node_name: row.get("node_name"),
            file_extension: row.get("file_extension"),
        })
    }

    /// Return the version a previous identical completion published, if any.
    #[allow(clippy::too_many_arguments)]
    async fn replay_existing_version(
        &self,
        tenant_id: &str,
        node_id: &str,
        bucket: &str,
        object_key: &str,
        content_type: &str,
        content_length: i64,
        checksum_sha256_hex: &str,
    ) -> Result<Option<DriveNodeVersion>, DriveServiceError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire stored version replay connection failed: {error}"
            ))
        })?;
        let Some(stored_object) = find_active_stored_upload_object(
            &mut connection,
            tenant_id,
            node_id,
            bucket,
            object_key,
        )
        .await?
        else {
            return Ok(None);
        };
        ensure_stored_object_matches(
            &stored_object,
            content_type,
            content_length,
            checksum_sha256_hex,
        )?;
        let version = find_node_version_by_storage_object(
            &mut connection,
            tenant_id,
            node_id,
            &stored_object.id,
        )
        .await?
        .ok_or_else(|| {
            DriveServiceError::Conflict(
                "stored object is already active for this node without a node version".to_string(),
            )
        })?;
        Ok(Some(version))
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_version(
        &self,
        tenant_id: &str,
        space_id: &str,
        node_id: &str,
        node: &ActiveFileNode,
        storage_provider_id: &str,
        bucket: &str,
        object_key: &str,
        content_type: &str,
        content_length: i64,
        checksum_sha256_hex: &str,
        idempotency_key: &str,
        operator_id: &str,
    ) -> Result<DriveNodeVersion, DriveServiceError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire stored version completion transaction connection failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "begin stored version completion transaction failed: {error}"
                ))
            })?;

        let result: Result<DriveNodeVersion, DriveServiceError> = async {
            ensure_managed_website_node_mutation_allowed(&mut connection, tenant_id, node_id)
                .await?;
            let version_no = next_storage_object_version_no_on_connection(
                &mut connection,
                tenant_id,
                node_id,
            )
            .await?;
            let storage_object_id = format!(
                "obj_{}",
                crate::infrastructure::sql::next_drive_runtime_id("Drive storage object")?
            );
            sqlx::query(
                "INSERT INTO dr_drive_storage_object (
                    id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
                    scene, source, content_type, content_length, checksum_sha256_hex, lifecycle_status,
                    created_by, updated_by
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, NULL, $8, $9, $10, 'active', $11, $11)",
            )
            .bind(&storage_object_id)
            .bind(tenant_id)
            .bind(node_id)
            .bind(version_no)
            .bind(storage_provider_id)
            .bind(bucket)
            .bind(object_key)
            .bind(content_type)
            .bind(content_length)
            .bind(checksum_sha256_hex)
            .bind(operator_id)
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                let message = error.to_string();
                if is_unique_constraint_violation(&message) {
                    DriveServiceError::Conflict(
                        "stored object locator or version number was taken by a concurrent completion"
                            .to_string(),
                    )
                } else {
                    DriveServiceError::Internal(format!(
                        "insert stored dr_drive_storage_object failed: {message}"
                    ))
                }
            })?;

            let version = create_node_version_in_transaction(
                &mut connection,
                &CreateDriveNodeVersionCommand {
                    id: format!(
                        "ver_{}",
                        crate::infrastructure::sql::next_drive_runtime_id("Drive node version")?
                    ),
                    tenant_id: tenant_id.to_string(),
                    space_id: space_id.to_string(),
                    node_id: node_id.to_string(),
                    version_no,
                    storage_object_id: Some(storage_object_id),
                    content_type: content_type.to_string(),
                    content_length,
                    checksum_sha256_hex: checksum_sha256_hex.to_string(),
                    version_kind: DriveNodeVersionKind::Auto,
                    version_label: None,
                    change_source: DriveNodeVersionChangeSource::AppApi,
                    change_summary: None,
                    restored_from_version_id: None,
                    app_id: None,
                    app_resource_type: None,
                    app_resource_id: None,
                    scene: None,
                    source: None,
                    operator_id: operator_id.to_string(),
                },
            )
            .await?;

            apply_file_node_head_snapshot_in_transaction(
                &mut connection,
                tenant_id,
                node_id,
                operator_id,
                &FileNodeHeadSnapshot {
                    file_extension: node.head_file_extension(),
                    content_type: content_type.to_string(),
                    content_type_group: content_type_group_for(content_type).to_string(),
                    content_length,
                    version_no,
                    checksum_sha256_hex: checksum_sha256_hex.to_string(),
                },
            )
            .await?;

            record_drive_node_version_committed_on_connection(
                &mut connection,
                RecordDriveNodeVersionCommittedCommand {
                    tenant_id,
                    organization_id: None,
                    space_id,
                    node_id,
                    node_version_id: &version.id,
                    version_no,
                    operation_id: idempotency_key,
                    content_type,
                    content_length,
                    checksum_sha256_hex,
                    actor_id: operator_id,
                },
            )
            .await?;

            Ok(version)
        }
        .await;

        match result {
            Ok(version) => {
                sqlx::query("COMMIT")
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "commit stored version completion transaction failed: {error}"
                        ))
                    })?;
                drop(connection);
                sdkwork_drive_observability::metrics::record_outbox_pending();
                crate::infrastructure::outbox_dispatch::trigger_immediate_outbox_dispatch(
                    self.pool.clone(),
                );
                Ok(version)
            }
            Err(error) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                Err(error)
            }
        }
    }
}

impl ActiveFileNode {
    /// The extension the head snapshot should carry.
    ///
    /// Preserves the node's existing extension; derives one from the node name
    /// only when the row has none, so a page-body rewrite cannot silently strip
    /// the `.json` extension the node was created with.
    fn head_file_extension(&self) -> Option<String> {
        self.file_extension
            .clone()
            .or_else(|| file_extension_from_name(&self.node_name))
    }
}

fn require_text(value: &str, field_name: &str) -> Result<String, DriveServiceError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} is required"
        )));
    }
    Ok(trimmed.to_string())
}
