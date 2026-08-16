use std::future::Future;
use std::pin::Pin;
use std::time::Duration as StdDuration;

use async_trait::async_trait;
use chrono::{Duration, SecondsFormat};
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, PgPool, Row};

use crate::domain::website_root::{
    DriveWebsiteContentMode, DriveWebsiteRoot, DriveWebsiteSourceRootMode,
};
use crate::domain::website_sync::{
    validate_website_sync_tree, DriveWebsiteGeneration, DriveWebsiteSync, DriveWebsiteSyncStatus,
    DriveWebsiteSyncTreeEntry, MAX_WEBSITE_SYNC_NODES,
};
use crate::infrastructure::change_recorder::{
    notify_drive_event_committed, record_drive_website_root_generation_changed_on_connection,
    RecordDriveWebsiteRootGenerationChangedCommand,
};
use crate::infrastructure::sql::{begin_transaction_sql, next_drive_runtime_id};
use crate::ports::website_sync_store::{
    AbortDriveWebsiteSync, ActivateDriveWebsiteGeneration, ActivateValidatedWebsiteSync,
    CreateDriveWebsiteSync, CreateDriveWebsiteSyncResult, DriveWebsiteGenerationActivation,
    DriveWebsiteSyncActivation, DriveWebsiteSyncStore, DriveWebsiteSyncValidation,
    MarkDriveWebsiteSyncFailed, ValidateDriveWebsiteSync,
};
use crate::DriveServiceError;

const WEBSITE_SYNC_SELECT_COLUMNS: &str = "sync.id, sync.tenant_id, sync.website_root_id, root.uuid AS website_root_uuid, sync.space_id, sync.idempotency_key, sync.expected_root_version, sync.expected_generation, sync.staging_node_id, sync.manifest_sha256, sync.manifest_file_count, sync.manifest_total_bytes, sync.uploaded_file_count, sync.uploaded_total_bytes, sync.sync_status, CAST(sync.expires_at AS TEXT) AS expires_at, CAST(sync.validated_at AS TEXT) AS validated_at, CAST(sync.activated_at AS TEXT) AS activated_at, CAST(sync.completed_at AS TEXT) AS completed_at, sync.error_code, sync.error_summary, sync.version, CAST(sync.created_at AS TEXT) AS created_at, CAST(sync.updated_at AS TEXT) AS updated_at";
const WEBSITE_ROOT_SELECT_COLUMNS: &str = "root.id, root.uuid, root.tenant_id, root.space_id, root.root_key, root.display_name, root.source_root_mode, root.selected_folder_node_id, root.content_mode, root.active_node_id, root.active_generation, root.root_status, root.version, CAST(root.created_at AS TEXT) AS created_at, CAST(root.updated_at AS TEXT) AS updated_at";
const MAX_SERIALIZABLE_TRANSACTION_ATTEMPTS: usize = 4;
const RETRYABLE_TRANSACTION_ERROR_PREFIX: &str = "[retryable-database-transaction] ";
const WEBSITE_SYNC_VALIDATION_LEASE_MINUTES: i64 = 5;

type WebsiteSyncTransactionFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, DriveServiceError>> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct SqlWebsiteSyncStore {
    pool: PgPool,
}

impl SqlWebsiteSyncStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveWebsiteSyncStore for SqlWebsiteSyncStore {
    async fn create_or_get(
        &self,
        command: &CreateDriveWebsiteSync,
    ) -> Result<CreateDriveWebsiteSyncResult, DriveServiceError> {
        run_serializable_transaction(&self.pool, "create WebsiteSync", |connection| {
            let command = command.clone();
            Box::pin(async move { create_sync_on_connection(connection, &command).await })
        })
        .await
    }

    async fn get(
        &self,
        tenant_id: &str,
        website_root_uuid: &str,
        sync_id: &str,
    ) -> Result<DriveWebsiteSync, DriveServiceError> {
        get_sync_from_executor(&self.pool, tenant_id, website_root_uuid, sync_id).await
    }

    async fn begin_validation(
        &self,
        command: &ValidateDriveWebsiteSync,
    ) -> Result<DriveWebsiteSyncValidation, DriveServiceError> {
        run_serializable_transaction(&self.pool, "begin WebsiteSync validation", |connection| {
            let command = command.clone();
            Box::pin(async move { begin_validation_on_connection(connection, &command).await })
        })
        .await
    }

    async fn list_staging_tree(
        &self,
        tenant_id: &str,
        website_root_uuid: &str,
        sync_id: &str,
    ) -> Result<Vec<DriveWebsiteSyncTreeEntry>, DriveServiceError> {
        let sync = self.get(tenant_id, website_root_uuid, sync_id).await?;
        list_staging_tree_on_connection(&self.pool, &sync).await
    }

    async fn activate_validated(
        &self,
        command: &ActivateValidatedWebsiteSync,
    ) -> Result<DriveWebsiteSyncActivation, DriveServiceError> {
        let activation =
            run_serializable_transaction(&self.pool, "activate WebsiteSync", |connection| {
                let command = command.clone();
                Box::pin(
                    async move { activate_validated_on_connection(connection, &command).await },
                )
            })
            .await?;
        notify_drive_event_committed(self.pool.clone());
        Ok(activation)
    }

    async fn mark_failed(
        &self,
        command: &MarkDriveWebsiteSyncFailed,
    ) -> Result<(), DriveServiceError> {
        let updated = sqlx::query(
            "UPDATE dr_drive_website_sync
             SET sync_status='failed', error_code=$1, error_summary=$2,
                 lease_owner=NULL, lease_token=NULL, lease_expires_at=NULL,
                 updated_by=$3, updated_at=CURRENT_TIMESTAMP, version=version + 1
             WHERE tenant_id=$4
               AND id=$5
               AND version=$6
               AND lease_token=$7
               AND sync_status='validating'
               AND website_root_id=(
                 SELECT id FROM dr_drive_website_root
                 WHERE tenant_id=$4 AND uuid=$8
               )",
        )
        .bind(&command.error_code)
        .bind(&command.error_summary)
        .bind(&command.operator_id)
        .bind(&command.tenant_id)
        .bind(&command.sync_id)
        .bind(command.expected_sync_version)
        .bind(&command.lease_token)
        .bind(&command.website_root_uuid)
        .execute(&self.pool)
        .await
        .map_err(|error| internal("mark WebsiteSync failed", error))?;
        if updated.rows_affected() == 0 {
            return Err(DriveServiceError::Conflict(
                "WebsiteSync failure state fence changed".to_string(),
            ));
        }
        Ok(())
    }

    async fn abort(
        &self,
        command: &AbortDriveWebsiteSync,
    ) -> Result<DriveWebsiteSync, DriveServiceError> {
        run_serializable_transaction(&self.pool, "abort WebsiteSync", |connection| {
            let command = command.clone();
            Box::pin(async move { abort_on_connection(connection, &command).await })
        })
        .await
    }

    async fn activate_generation(
        &self,
        command: &ActivateDriveWebsiteGeneration,
    ) -> Result<DriveWebsiteGenerationActivation, DriveServiceError> {
        let activation = run_serializable_transaction(
            &self.pool,
            "activate retained WebsiteRoot generation",
            |connection| {
                let command = command.clone();
                Box::pin(
                    async move { activate_generation_on_connection(connection, &command).await },
                )
            },
        )
        .await?;
        notify_drive_event_committed(self.pool.clone());
        Ok(activation)
    }
}

async fn create_sync_on_connection(
    connection: &mut PgConnection,
    command: &CreateDriveWebsiteSync,
) -> Result<CreateDriveWebsiteSyncResult, DriveServiceError> {
    if let Some(existing) =
        find_sync_by_idempotency(connection, &command.tenant_id, &command.idempotency_key).await?
    {
        if sync_matches_create(&existing, command) {
            return Ok(CreateDriveWebsiteSyncResult {
                sync: existing,
                created: false,
            });
        }
        return Err(DriveServiceError::Conflict(
            "Idempotency-Key is already bound to a different WebsiteSync request".to_string(),
        ));
    }

    lock_website_root(connection, &command.tenant_id, &command.website_root_uuid).await?;
    let root =
        get_root_on_connection(connection, &command.tenant_id, &command.website_root_uuid).await?;
    ensure_root_can_sync(
        &root,
        command.expected_root_version,
        command.expected_generation,
    )?;
    ensure_tenant_sync_quota(connection, &command.tenant_id, command.manifest_total_bytes).await?;

    let anchor_node_id: String = match root.source_root_mode {
        DriveWebsiteSourceRootMode::Folder => {
            root.selected_folder_node_id.clone().ok_or_else(|| {
                DriveServiceError::Internal(
                    "FOLDER WebsiteRoot is missing selected folder".to_string(),
                )
            })?
        }
        DriveWebsiteSourceRootMode::SpaceRoot => sqlx::query_scalar(
            "SELECT root_node_id
             FROM dr_drive_website_root_generation
             WHERE tenant_id=$1 AND website_root_id=$2 AND generation_no=1",
        )
        .bind(&command.tenant_id)
        .bind(&root.id)
        .fetch_optional(&mut *connection)
        .await
        .map_err(|error| internal("resolve WebsiteSync source anchor", error))?
        .ok_or_else(|| {
            DriveServiceError::Internal("WebsiteRoot initial generation is missing".to_string())
        })?,
    };
    ensure_active_folder(
        connection,
        &command.tenant_id,
        &root.space_id,
        &anchor_node_id,
    )
    .await?;
    let staging_parent_id = ensure_staging_parent(
        connection,
        &command.tenant_id,
        &root.space_id,
        &anchor_node_id,
        &command.operator_id,
    )
    .await?;
    let sync_id = uuid::Uuid::new_v4().to_string();
    let staging_node_id = next_drive_runtime_id("WebsiteSync staging node")?.to_string();
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, space_type, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, 'website', $4, 'folder', $5,
                   'ready', 'active', 1, $6, $6)",
    )
    .bind(&staging_node_id)
    .bind(&command.tenant_id)
    .bind(&root.space_id)
    .bind(&staging_parent_id)
    .bind(format!("sync-{sync_id}"))
    .bind(&command.operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("insert WebsiteSync staging node", error))?;
    let expires_at_parameter = instant_parameter("$12");
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "INSERT INTO dr_drive_website_sync (
            id, tenant_id, website_root_id, space_id, idempotency_key,
            expected_root_version, expected_generation, staging_node_id,
            manifest_sha256, manifest_file_count, manifest_total_bytes,
            uploaded_file_count, uploaded_total_bytes, sync_status, expires_at,
            version, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                   0, 0, 'created', {expires_at_parameter}, 1, $13, $13)"
    )))
    .bind(&sync_id)
    .bind(&command.tenant_id)
    .bind(&root.id)
    .bind(&root.space_id)
    .bind(&command.idempotency_key)
    .bind(command.expected_root_version)
    .bind(command.expected_generation)
    .bind(&staging_node_id)
    .bind(&command.manifest_sha256)
    .bind(command.manifest_file_count)
    .bind(command.manifest_total_bytes)
    .bind(&command.expires_at)
    .bind(&command.operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("insert WebsiteSync", error))?;
    insert_audit(
        connection,
        &command.tenant_id,
        "drive.website_sync.created",
        "website_sync",
        &sync_id,
        &command.operator_id,
        Some(&sync_id),
    )
    .await?;
    Ok(CreateDriveWebsiteSyncResult {
        sync: get_sync_on_connection(
            connection,
            &command.tenant_id,
            &command.website_root_uuid,
            &sync_id,
        )
        .await?,
        created: true,
    })
}

async fn begin_validation_on_connection(
    connection: &mut PgConnection,
    command: &ValidateDriveWebsiteSync,
) -> Result<DriveWebsiteSyncValidation, DriveServiceError> {
    let sync = get_sync_on_connection(
        connection,
        &command.tenant_id,
        &command.website_root_uuid,
        &command.sync_id,
    )
    .await?;
    if sync.status == DriveWebsiteSyncStatus::Completed {
        return Ok(DriveWebsiteSyncValidation {
            sync,
            lease_token: None,
        });
    }
    if sync.version != command.expected_sync_version {
        return Err(DriveServiceError::Conflict(
            "WebsiteSync version changed".to_string(),
        ));
    }
    let recover_expired_lease = sync.status == DriveWebsiteSyncStatus::Validating;
    if !recover_expired_lease
        && !matches!(
            sync.status,
            DriveWebsiteSyncStatus::Created
                | DriveWebsiteSyncStatus::Uploading
                | DriveWebsiteSyncStatus::Ready
        )
    {
        return Err(DriveServiceError::Conflict(
            "WebsiteSync is not finalizable".to_string(),
        ));
    }
    let lease_token = uuid::Uuid::new_v4().to_string();
    let lease_expires_at = (chrono::Utc::now()
        + Duration::minutes(WEBSITE_SYNC_VALIDATION_LEASE_MINUTES))
    .to_rfc3339_opts(SecondsFormat::Millis, true);
    let lease_expires_at_parameter = instant_parameter("$3");
    let updated = sqlx::query(sqlx::AssertSqlSafe(format!(
        "UPDATE dr_drive_website_sync
         SET sync_status='validating', validated_at=CURRENT_TIMESTAMP,
             lease_owner=$1, lease_token=$2, lease_expires_at={lease_expires_at_parameter},
             error_code=NULL, error_summary=NULL, updated_by=$4,
             updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$5 AND id=$6 AND version=$7
           AND (
             sync_status IN ('created', 'uploading', 'ready')
             OR (sync_status='validating' AND lease_expires_at <= CURRENT_TIMESTAMP)
           )
           AND expires_at > CURRENT_TIMESTAMP",
    )))
    .bind(&command.operator_id)
    .bind(&lease_token)
    .bind(&lease_expires_at)
    .bind(&command.operator_id)
    .bind(&command.tenant_id)
    .bind(&command.sync_id)
    .bind(command.expected_sync_version)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("begin WebsiteSync validation", error))?;
    if updated.rows_affected() != 1 {
        return Err(DriveServiceError::Conflict(
            "WebsiteSync expired or changed before validation".to_string(),
        ));
    }
    insert_audit(
        connection,
        &command.tenant_id,
        if recover_expired_lease {
            "drive.website_sync.validation_lease_recovered"
        } else {
            "drive.website_sync.validation_lease_acquired"
        },
        "website_sync",
        &command.sync_id,
        &command.operator_id,
        Some(&command.sync_id),
    )
    .await?;
    Ok(DriveWebsiteSyncValidation {
        sync: get_sync_on_connection(
            connection,
            &command.tenant_id,
            &command.website_root_uuid,
            &command.sync_id,
        )
        .await?,
        lease_token: Some(lease_token),
    })
}

async fn activate_validated_on_connection(
    connection: &mut PgConnection,
    command: &ActivateValidatedWebsiteSync,
) -> Result<DriveWebsiteSyncActivation, DriveServiceError> {
    lock_website_root(connection, &command.tenant_id, &command.website_root_uuid).await?;
    lock_website_sync(
        connection,
        &command.tenant_id,
        &command.website_root_uuid,
        &command.sync_id,
    )
    .await?;
    let root =
        get_root_on_connection(connection, &command.tenant_id, &command.website_root_uuid).await?;
    let sync = get_sync_on_connection(
        connection,
        &command.tenant_id,
        &command.website_root_uuid,
        &command.sync_id,
    )
    .await?;
    if sync.status == DriveWebsiteSyncStatus::Completed {
        return Ok(DriveWebsiteSyncActivation {
            sync,
            website_root: root,
        });
    }
    let lease_token = command.lease_token.as_deref().ok_or_else(|| {
        DriveServiceError::Conflict("WebsiteSync validation lease is missing".to_string())
    })?;
    if sync.status != DriveWebsiteSyncStatus::Validating
        || sync.version != command.expected_sync_version
    {
        return Err(DriveServiceError::Conflict(
            "WebsiteSync validation fence changed".to_string(),
        ));
    }
    ensure_validation_lease_current(connection, &sync, lease_token).await?;
    ensure_root_can_sync(&root, sync.expected_root_version, sync.expected_generation)?;
    let entries = list_staging_tree_on_connection(&mut *connection, &sync).await?;
    let current_manifest = validate_website_sync_tree(&entries)?;
    let declared_manifest = crate::domain::website_sync::DriveWebsiteManifestSummary {
        sha256: sync.manifest_sha256.clone(),
        file_count: sync.manifest_file_count,
        total_bytes: sync.manifest_total_bytes,
    };
    if current_manifest != command.observed_manifest || current_manifest != declared_manifest {
        return Err(DriveServiceError::Validation(
            "WEBSITE_SYNC_MANIFEST_MISMATCH".to_string(),
        ));
    }
    ensure_activation_eligibility(connection, &sync).await?;
    ensure_tenant_sync_quota(connection, &command.tenant_id, 0).await?;

    let previous_root_node_id = root.active_node_id.clone();
    let next_generation = root.active_generation + 1;
    let retention_until =
        (chrono::Utc::now() + Duration::days(30)).to_rfc3339_opts(SecondsFormat::Millis, true);
    let retention_until_parameter = instant_parameter("$1");
    let retained = sqlx::query(sqlx::AssertSqlSafe(format!(
        "UPDATE dr_drive_website_root_generation
         SET generation_status='retained', retention_until={retention_until_parameter}
         WHERE tenant_id=$2 AND website_root_id=$3 AND generation_status='current'"
    )))
    .bind(&retention_until)
    .bind(&command.tenant_id)
    .bind(&root.id)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("retain previous WebsiteRoot generation", error))?;
    if retained.rows_affected() != 1 {
        return Err(DriveServiceError::Conflict(
            "WebsiteRoot current generation changed".to_string(),
        ));
    }
    let generation_id = next_drive_runtime_id("WebsiteRoot generation")?.to_string();
    sqlx::query(
        "INSERT INTO dr_drive_website_root_generation (
            id, tenant_id, website_root_id, generation_no, root_node_id,
            source_sync_id, manifest_sha256, file_count, total_bytes,
            generation_status, activated_by
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'current', $10)",
    )
    .bind(&generation_id)
    .bind(&command.tenant_id)
    .bind(&root.id)
    .bind(next_generation)
    .bind(&sync.staging_node_id)
    .bind(&sync.id)
    .bind(&current_manifest.sha256)
    .bind(current_manifest.file_count)
    .bind(current_manifest.total_bytes)
    .bind(&command.operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("insert active WebsiteRoot generation", error))?;
    expire_excess_retained_generations(
        connection,
        &command.tenant_id,
        &root.id,
        &root.space_id,
        &root.uuid,
        &command.operator_id,
    )
    .await?;
    let switched = sqlx::query(
        "UPDATE dr_drive_website_root
         SET active_node_id=$1, active_generation=$2, content_mode='atomic_generation',
             last_switch_at=CURRENT_TIMESTAMP, last_switch_by=$3,
             updated_by=$3, updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$4 AND id=$5 AND uuid=$6 AND version=$7 AND active_generation=$8",
    )
    .bind(&sync.staging_node_id)
    .bind(next_generation)
    .bind(&command.operator_id)
    .bind(&command.tenant_id)
    .bind(&root.id)
    .bind(&command.website_root_uuid)
    .bind(sync.expected_root_version)
    .bind(sync.expected_generation)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("switch WebsiteRoot generation", error))?;
    if switched.rows_affected() != 1 {
        return Err(DriveServiceError::Conflict(
            "WebsiteRoot changed during generation switch".to_string(),
        ));
    }
    let completed = sqlx::query(
        "UPDATE dr_drive_website_sync
         SET uploaded_file_count=$1, uploaded_total_bytes=$2,
             sync_status='completed', activated_at=CURRENT_TIMESTAMP,
             completed_at=CURRENT_TIMESTAMP, error_code=NULL, error_summary=NULL,
             lease_owner=NULL, lease_token=NULL, lease_expires_at=NULL,
             updated_by=$3, updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$4 AND id=$5 AND website_root_id=$6
           AND version=$7 AND sync_status='validating'
           AND lease_token=$8 AND lease_expires_at > CURRENT_TIMESTAMP
           AND expires_at > CURRENT_TIMESTAMP",
    )
    .bind(current_manifest.file_count)
    .bind(current_manifest.total_bytes)
    .bind(&command.operator_id)
    .bind(&command.tenant_id)
    .bind(&sync.id)
    .bind(&root.id)
    .bind(command.expected_sync_version)
    .bind(lease_token)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("complete WebsiteSync", error))?;
    if completed.rows_affected() != 1 {
        return Err(DriveServiceError::Conflict(
            "WebsiteSync changed during generation switch".to_string(),
        ));
    }
    insert_audit(
        connection,
        &command.tenant_id,
        "drive.website_root.generation_activated",
        "website_root",
        &root.uuid,
        &command.operator_id,
        Some(&sync.id),
    )
    .await?;
    record_drive_website_root_generation_changed_on_connection(
        connection,
        RecordDriveWebsiteRootGenerationChangedCommand {
            tenant_id: &command.tenant_id,
            organization_id: None,
            space_id: &root.space_id,
            website_root_uuid: &root.uuid,
            operation_id: &sync.id,
            previous_root_node_id: &previous_root_node_id,
            root_node_id: &sync.staging_node_id,
            previous_generation: root.active_generation,
            generation: next_generation,
            manifest_sha256: Some(&current_manifest.sha256),
            file_count: current_manifest.file_count,
            total_bytes: current_manifest.total_bytes,
            change_reason: "SYNC_ACTIVATED",
            actor_id: &command.operator_id,
        },
    )
    .await?;
    Ok(DriveWebsiteSyncActivation {
        sync: get_sync_on_connection(
            connection,
            &command.tenant_id,
            &command.website_root_uuid,
            &command.sync_id,
        )
        .await?,
        website_root: get_root_on_connection(
            connection,
            &command.tenant_id,
            &command.website_root_uuid,
        )
        .await?,
    })
}

async fn abort_on_connection(
    connection: &mut PgConnection,
    command: &AbortDriveWebsiteSync,
) -> Result<DriveWebsiteSync, DriveServiceError> {
    let sync = get_sync_on_connection(
        connection,
        &command.tenant_id,
        &command.website_root_uuid,
        &command.sync_id,
    )
    .await?;
    if sync.status == DriveWebsiteSyncStatus::Aborted {
        return Ok(sync);
    }
    if sync.version != command.expected_sync_version
        || !matches!(
            sync.status,
            DriveWebsiteSyncStatus::Created
                | DriveWebsiteSyncStatus::Uploading
                | DriveWebsiteSyncStatus::Ready
        )
    {
        return Err(DriveServiceError::Conflict(
            "WebsiteSync cannot be aborted in its current state".to_string(),
        ));
    }
    let updated = sqlx::query(
        "UPDATE dr_drive_website_sync
         SET sync_status='aborted', updated_by=$1,
             updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$2 AND id=$3 AND version=$4
           AND sync_status IN ('created', 'uploading', 'ready')",
    )
    .bind(&command.operator_id)
    .bind(&command.tenant_id)
    .bind(&command.sync_id)
    .bind(command.expected_sync_version)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("abort WebsiteSync", error))?;
    if updated.rows_affected() != 1 {
        return Err(DriveServiceError::Conflict(
            "WebsiteSync changed before abort".to_string(),
        ));
    }
    insert_audit(
        connection,
        &command.tenant_id,
        "drive.website_sync.aborted",
        "website_sync",
        &command.sync_id,
        &command.operator_id,
        Some(&command.sync_id),
    )
    .await?;
    get_sync_on_connection(
        connection,
        &command.tenant_id,
        &command.website_root_uuid,
        &command.sync_id,
    )
    .await
}

async fn activate_generation_on_connection(
    connection: &mut PgConnection,
    command: &ActivateDriveWebsiteGeneration,
) -> Result<DriveWebsiteGenerationActivation, DriveServiceError> {
    lock_website_root(connection, &command.tenant_id, &command.website_root_uuid).await?;
    let root =
        get_root_on_connection(connection, &command.tenant_id, &command.website_root_uuid).await?;
    ensure_root_can_sync(
        &root,
        command.expected_root_version,
        command.expected_generation,
    )?;
    if command.target_generation == root.active_generation {
        return Err(DriveServiceError::Conflict(
            "target generation is already active".to_string(),
        ));
    }
    let target_row = sqlx::query(
        "SELECT generation_no, root_node_id, manifest_sha256, file_count, total_bytes,
                generation_status, CAST(activated_at AS TEXT) AS activated_at
         FROM dr_drive_website_root_generation
         WHERE tenant_id=$1 AND website_root_id=$2 AND generation_no=$3
           AND generation_status='retained'",
    )
    .bind(&command.tenant_id)
    .bind(&root.id)
    .bind(command.target_generation)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| internal("read retained WebsiteRoot generation", error))?
    .ok_or_else(|| {
        DriveServiceError::NotFound("retained WebsiteRoot generation not found".to_string())
    })?;
    let source_generation = map_generation(&target_row);
    ensure_active_folder(
        connection,
        &command.tenant_id,
        &root.space_id,
        &source_generation.root_node_id,
    )
    .await?;
    let next_generation = root.active_generation + 1;
    let previous_root_node_id = root.active_node_id.clone();
    let retention_until =
        (chrono::Utc::now() + Duration::days(30)).to_rfc3339_opts(SecondsFormat::Millis, true);
    let retention_until_parameter = instant_parameter("$1");
    let retained = sqlx::query(sqlx::AssertSqlSafe(format!(
        "UPDATE dr_drive_website_root_generation
         SET generation_status='retained', retention_until={retention_until_parameter}
         WHERE tenant_id=$2 AND website_root_id=$3 AND generation_status='current'"
    )))
    .bind(&retention_until)
    .bind(&command.tenant_id)
    .bind(&root.id)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("retain current WebsiteRoot generation for rollback", error))?;
    if retained.rows_affected() != 1 {
        return Err(DriveServiceError::Conflict(
            "WebsiteRoot current generation changed".to_string(),
        ));
    }
    let generation_id = next_drive_runtime_id("WebsiteRoot rollback generation")?.to_string();
    sqlx::query(
        "INSERT INTO dr_drive_website_root_generation (
            id, tenant_id, website_root_id, generation_no, root_node_id,
            source_sync_id, manifest_sha256, file_count, total_bytes,
            generation_status, activated_by
         ) VALUES ($1, $2, $3, $4, $5, NULL, $6, $7, $8, 'current', $9)",
    )
    .bind(&generation_id)
    .bind(&command.tenant_id)
    .bind(&root.id)
    .bind(next_generation)
    .bind(&source_generation.root_node_id)
    .bind(source_generation.manifest_sha256.as_deref())
    .bind(source_generation.file_count)
    .bind(source_generation.total_bytes)
    .bind(&command.operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("insert WebsiteRoot rollback generation", error))?;
    expire_excess_retained_generations(
        connection,
        &command.tenant_id,
        &root.id,
        &root.space_id,
        &root.uuid,
        &command.operator_id,
    )
    .await?;
    let switched = sqlx::query(
        "UPDATE dr_drive_website_root
         SET active_node_id=$1, active_generation=$2, content_mode='atomic_generation',
             last_switch_at=CURRENT_TIMESTAMP, last_switch_by=$3,
             updated_by=$3, updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$4 AND id=$5 AND uuid=$6 AND version=$7 AND active_generation=$8",
    )
    .bind(&source_generation.root_node_id)
    .bind(next_generation)
    .bind(&command.operator_id)
    .bind(&command.tenant_id)
    .bind(&root.id)
    .bind(&command.website_root_uuid)
    .bind(command.expected_root_version)
    .bind(command.expected_generation)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("activate retained WebsiteRoot generation", error))?;
    if switched.rows_affected() != 1 {
        return Err(DriveServiceError::Conflict(
            "WebsiteRoot changed during rollback".to_string(),
        ));
    }
    let operation_id = format!("rollback:{}:{}", root.uuid, next_generation);
    insert_audit(
        connection,
        &command.tenant_id,
        "drive.website_root.generation_rolled_back",
        "website_root",
        &root.uuid,
        &command.operator_id,
        Some(&operation_id),
    )
    .await?;
    record_drive_website_root_generation_changed_on_connection(
        connection,
        RecordDriveWebsiteRootGenerationChangedCommand {
            tenant_id: &command.tenant_id,
            organization_id: None,
            space_id: &root.space_id,
            website_root_uuid: &root.uuid,
            operation_id: &operation_id,
            previous_root_node_id: &previous_root_node_id,
            root_node_id: &source_generation.root_node_id,
            previous_generation: root.active_generation,
            generation: next_generation,
            manifest_sha256: source_generation.manifest_sha256.as_deref(),
            file_count: source_generation.file_count,
            total_bytes: source_generation.total_bytes,
            change_reason: "ROLLBACK_ACTIVATED",
            actor_id: &command.operator_id,
        },
    )
    .await?;
    Ok(DriveWebsiteGenerationActivation {
        source_generation,
        website_root: get_root_on_connection(
            connection,
            &command.tenant_id,
            &command.website_root_uuid,
        )
        .await?,
    })
}

fn ensure_root_can_sync(
    root: &DriveWebsiteRoot,
    expected_root_version: i64,
    expected_generation: i64,
) -> Result<(), DriveServiceError> {
    if root.root_status != "active" {
        return Err(DriveServiceError::Conflict(
            "WebsiteRoot is not active".to_string(),
        ));
    }
    if root.version != expected_root_version || root.active_generation != expected_generation {
        return Err(DriveServiceError::Conflict(
            "WebsiteRoot version or generation changed".to_string(),
        ));
    }
    Ok(())
}

async fn lock_website_root(
    connection: &mut PgConnection,
    tenant_id: &str,
    website_root_uuid: &str,
) -> Result<(), DriveServiceError> {
    let locked = sqlx::query(
        "UPDATE dr_drive_website_root
         SET updated_at=updated_at
         WHERE tenant_id=$1 AND uuid=$2",
    )
    .bind(tenant_id)
    .bind(website_root_uuid)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("lock WebsiteRoot", error))?;
    if locked.rows_affected() != 1 {
        return Err(DriveServiceError::NotFound(
            "WebsiteRoot not found".to_string(),
        ));
    }
    Ok(())
}

async fn lock_website_sync(
    connection: &mut PgConnection,
    tenant_id: &str,
    website_root_uuid: &str,
    sync_id: &str,
) -> Result<(), DriveServiceError> {
    let locked = sqlx::query(
        "UPDATE dr_drive_website_sync
         SET updated_at=updated_at
         WHERE tenant_id=$1 AND id=$2
           AND website_root_id=(
             SELECT id FROM dr_drive_website_root
             WHERE tenant_id=$1 AND uuid=$3
           )",
    )
    .bind(tenant_id)
    .bind(sync_id)
    .bind(website_root_uuid)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("lock WebsiteSync", error))?;
    if locked.rows_affected() != 1 {
        return Err(DriveServiceError::NotFound(
            "WebsiteSync not found".to_string(),
        ));
    }
    Ok(())
}

async fn ensure_validation_lease_current(
    connection: &mut PgConnection,
    sync: &DriveWebsiteSync,
    lease_token: &str,
) -> Result<(), DriveServiceError> {
    let current: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_website_sync
         WHERE tenant_id=$1 AND id=$2 AND website_root_id=$3
           AND sync_status='validating' AND version=$4
           AND lease_token=$5 AND lease_expires_at > CURRENT_TIMESTAMP
           AND expires_at > CURRENT_TIMESTAMP",
    )
    .bind(&sync.tenant_id)
    .bind(&sync.id)
    .bind(&sync.website_root_id)
    .bind(sync.version)
    .bind(lease_token)
    .fetch_one(&mut *connection)
    .await
    .map_err(|error| internal("validate WebsiteSync lease", error))?;
    if current != 1 {
        return Err(DriveServiceError::Conflict(
            "WebsiteSync validation lease expired or changed".to_string(),
        ));
    }
    Ok(())
}

async fn expire_excess_retained_generations(
    connection: &mut PgConnection,
    tenant_id: &str,
    website_root_id: &str,
    space_id: &str,
    website_root_uuid: &str,
    operator_id: &str,
) -> Result<(), DriveServiceError> {
    let retained_generation_count: i64 = sqlx::query_scalar(
        "SELECT retained_generation_count
         FROM dr_drive_space_website_profile
         WHERE tenant_id=$1 AND space_id=$2 AND profile_status='active'",
    )
    .bind(tenant_id)
    .bind(space_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| internal("read Website Space retained generation policy", error))?
    .ok_or_else(|| {
        DriveServiceError::Conflict("Website Space publishing profile is not active".to_string())
    })?;
    let expired = sqlx::query(
        "UPDATE dr_drive_website_root_generation
         SET generation_status='expired'
         WHERE tenant_id=$1 AND website_root_id=$2 AND generation_status='retained'
           AND (
             SELECT COUNT(1)
             FROM dr_drive_website_root_generation newer
             WHERE newer.tenant_id=$1
               AND newer.website_root_id=$2
               AND newer.generation_status='retained'
               AND newer.generation_no > dr_drive_website_root_generation.generation_no
           ) >= $3",
    )
    .bind(tenant_id)
    .bind(website_root_id)
    .bind(retained_generation_count)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("expire excess retained WebsiteRoot generations", error))?
    .rows_affected();
    if expired == 0 {
        return Ok(());
    }
    insert_audit(
        connection,
        tenant_id,
        "drive.website_root.generations_expired_by_retention_count",
        "website_root",
        website_root_uuid,
        operator_id,
        None,
    )
    .await?;
    sdkwork_drive_observability::metrics::record_website_generations_expired(expired);
    Ok(())
}

async fn ensure_staging_parent(
    connection: &mut PgConnection,
    tenant_id: &str,
    space_id: &str,
    anchor_node_id: &str,
    operator_id: &str,
) -> Result<String, DriveServiceError> {
    let existing = sqlx::query(
        "SELECT id, node_type
         FROM dr_drive_node
         WHERE tenant_id=$1 AND space_id=$2 AND parent_node_id=$3
           AND lower(node_name)='.staging' AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(space_id)
    .bind(anchor_node_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| internal("find WebsiteSync staging parent", error))?;
    if let Some(row) = existing {
        if row.get::<String, _>("node_type") != "folder" {
            return Err(DriveServiceError::Conflict(
                "reserved .staging node is not a folder".to_string(),
            ));
        }
        return Ok(row.get("id"));
    }
    let staging_parent_id = next_drive_runtime_id("WebsiteSync staging parent")?.to_string();
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, space_type, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, 'website', $4, 'folder', '.staging',
                   'ready', 'active', 1, $5, $5)",
    )
    .bind(&staging_parent_id)
    .bind(tenant_id)
    .bind(space_id)
    .bind(anchor_node_id)
    .bind(operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("insert WebsiteSync staging parent", error))?;
    Ok(staging_parent_id)
}

async fn ensure_active_folder(
    connection: &mut PgConnection,
    tenant_id: &str,
    space_id: &str,
    node_id: &str,
) -> Result<(), DriveServiceError> {
    let found: Option<i64> = sqlx::query_scalar(
        "SELECT 1
         FROM dr_drive_node
         WHERE tenant_id=$1 AND space_id=$2 AND id=$3
           AND space_type='website' AND node_type='folder'
           AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(space_id)
    .bind(node_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| internal("validate WebsiteSync folder", error))?;
    if found.is_none() {
        return Err(DriveServiceError::Validation(
            "WebsiteSync root folder is not active".to_string(),
        ));
    }
    Ok(())
}

async fn ensure_activation_eligibility(
    connection: &mut PgConnection,
    sync: &DriveWebsiteSync,
) -> Result<(), DriveServiceError> {
    let invalid_files: i64 = sqlx::query_scalar(
        "WITH RECURSIVE tree(id, depth) AS (
           SELECT id, 0 FROM dr_drive_node WHERE tenant_id=$1 AND id=$2
           UNION ALL
           SELECT child.id, tree.depth + 1
           FROM dr_drive_node child
           INNER JOIN tree ON child.parent_node_id=tree.id
           WHERE child.tenant_id=$1 AND child.space_id=$3
             AND child.lifecycle_status='active' AND tree.depth < 64
         )
         SELECT COUNT(1)
         FROM dr_drive_node node
         INNER JOIN tree ON tree.id=node.id
         WHERE node.tenant_id=$1 AND node.node_type='file'
           AND (
             node.lifecycle_status != 'active'
             OR node.content_state != 'ready'
             OR NOT EXISTS (
               SELECT 1
               FROM dr_drive_storage_object object
               INNER JOIN dr_drive_storage_provider provider
                 ON provider.id=object.storage_provider_id
               WHERE object.tenant_id=$1
                 AND object.node_id=node.id
                 AND object.version_no=node.head_version_no
                 AND object.content_length=node.head_content_length
                 AND object.checksum_sha256_hex=node.head_checksum_sha256_hex
                 AND object.lifecycle_status='active'
                 AND provider.status='active'
             )
             OR EXISTS (
               SELECT 1
               FROM dr_drive_upload_item upload
               WHERE upload.id=(
                 SELECT latest.id
                 FROM dr_drive_upload_item latest
                 WHERE latest.tenant_id=$1 AND latest.node_id=node.id
                 ORDER BY latest.updated_at DESC, latest.id DESC
                 LIMIT 1
               )
                 AND (
                   upload.status != 'completed'
                   OR upload.cleanup_status != 'active'
                   OR upload.post_process_status NOT IN ('not_required', 'completed')
                 )
             )
           )",
    )
    .bind(&sync.tenant_id)
    .bind(&sync.staging_node_id)
    .bind(&sync.space_id)
    .fetch_one(&mut *connection)
    .await
    .map_err(|error| internal("validate WebsiteSync activation eligibility", error))?;
    if invalid_files != 0 {
        return Err(DriveServiceError::Validation(
            "WEBSITE_SYNC_CONTENT_NOT_ELIGIBLE".to_string(),
        ));
    }
    Ok(())
}

async fn ensure_tenant_sync_quota(
    connection: &mut PgConnection,
    tenant_id: &str,
    additional_reserved_bytes: i64,
) -> Result<(), DriveServiceError> {
    let locked = sqlx::query(
        "UPDATE dr_drive_tenant_quota
         SET updated_at=updated_at
         WHERE tenant_id=$1",
    )
    .bind(tenant_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("lock tenant quota for WebsiteSync", error))?;
    if locked.rows_affected() == 0 {
        return Ok(());
    }
    let max_bytes: Option<i64> =
        sqlx::query_scalar("SELECT max_bytes FROM dr_drive_tenant_quota WHERE tenant_id=$1")
            .bind(tenant_id)
            .fetch_one(&mut *connection)
            .await
            .map_err(|error| internal("read tenant quota for WebsiteSync", error))?;
    let Some(max_bytes) = max_bytes else {
        return Ok(());
    };
    let row = sqlx::query(
        "WITH RECURSIVE staging_tree(sync_id, node_id, depth) AS (
           SELECT sync.id, sync.staging_node_id, 0
           FROM dr_drive_website_sync sync
           WHERE sync.tenant_id=$1
             AND sync.sync_status IN ('created', 'uploading', 'ready', 'validating')
           UNION ALL
           SELECT tree.sync_id, child.id, tree.depth + 1
           FROM dr_drive_node child
           INNER JOIN staging_tree tree ON child.parent_node_id=tree.node_id
           WHERE child.tenant_id=$1 AND tree.depth < 64
         )
         SELECT
           CAST(COALESCE((
             SELECT SUM(content_length) FROM dr_drive_storage_object
             WHERE tenant_id=$1 AND lifecycle_status='active'
           ), 0) AS BIGINT) AS stored_bytes,
           CAST(COALESCE((
             SELECT SUM(object.content_length)
             FROM dr_drive_storage_object object
             INNER JOIN staging_tree tree ON tree.node_id=object.node_id
             WHERE object.tenant_id=$1 AND object.lifecycle_status='active'
           ), 0) AS BIGINT) AS staging_bytes,
           CAST(COALESCE((
             SELECT SUM(manifest_total_bytes)
             FROM dr_drive_website_sync
             WHERE tenant_id=$1
               AND sync_status IN ('created', 'uploading', 'ready', 'validating')
           ), 0) AS BIGINT) AS reserved_bytes",
    )
    .bind(tenant_id)
    .fetch_one(&mut *connection)
    .await
    .map_err(|error| internal("calculate WebsiteSync quota reservation", error))?;
    let stored_bytes: i64 = row.get("stored_bytes");
    let staging_bytes: i64 = row.get("staging_bytes");
    let reserved_bytes: i64 = row.get("reserved_bytes");
    let projected = stored_bytes
        .checked_sub(staging_bytes)
        .and_then(|value| value.checked_add(reserved_bytes))
        .and_then(|value| value.checked_add(additional_reserved_bytes))
        .ok_or_else(|| DriveServiceError::Validation("WEBSITE_SYNC_QUOTA_OVERFLOW".to_string()))?;
    if projected > max_bytes {
        return Err(DriveServiceError::Conflict(
            "WEBSITE_SYNC_QUOTA_EXCEEDED".to_string(),
        ));
    }
    Ok(())
}

async fn list_staging_tree_on_connection<'e, E>(
    executor: E,
    sync: &DriveWebsiteSync,
) -> Result<Vec<DriveWebsiteSyncTreeEntry>, DriveServiceError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let rows = sqlx::query(
        "WITH RECURSIVE tree(
            id, parent_node_id, node_name, relative_path, depth, node_type,
            content_state, head_content_length, head_checksum_sha256_hex,
            shortcut_target_node_id
         ) AS (
            SELECT id, parent_node_id, node_name, CAST(node_name AS TEXT), 1, node_type,
                   content_state, head_content_length, head_checksum_sha256_hex,
                   shortcut_target_node_id
            FROM dr_drive_node
            WHERE tenant_id=$1 AND space_id=$2 AND parent_node_id=$3
              AND lifecycle_status='active'
            UNION ALL
            SELECT child.id, child.parent_node_id, child.node_name,
                   CAST(tree.relative_path || '/' || child.node_name AS TEXT),
                   tree.depth + 1, child.node_type, child.content_state,
                   child.head_content_length, child.head_checksum_sha256_hex,
                   child.shortcut_target_node_id
            FROM dr_drive_node child
            INNER JOIN tree ON child.parent_node_id=tree.id
            WHERE child.tenant_id=$1 AND child.space_id=$2
              AND child.lifecycle_status='active' AND tree.depth <= 64
         )
         SELECT relative_path, depth, node_type, content_state,
                head_content_length, head_checksum_sha256_hex,
                shortcut_target_node_id
         FROM tree
         ORDER BY relative_path ASC
         LIMIT $4",
    )
    .bind(&sync.tenant_id)
    .bind(&sync.space_id)
    .bind(&sync.staging_node_id)
    .bind((MAX_WEBSITE_SYNC_NODES + 1) as i64)
    .fetch_all(executor)
    .await
    .map_err(|error| internal("enumerate WebsiteSync staging tree", error))?;
    Ok(rows
        .iter()
        .map(|row| DriveWebsiteSyncTreeEntry {
            relative_path: row.get("relative_path"),
            depth: row.get("depth"),
            node_type: row.get("node_type"),
            content_state: row.get("content_state"),
            content_length: row.get("head_content_length"),
            checksum_sha256_hex: row.get("head_checksum_sha256_hex"),
            shortcut_target_node_id: row.get("shortcut_target_node_id"),
        })
        .collect())
}

async fn get_sync_from_executor<'e, E>(
    executor: E,
    tenant_id: &str,
    website_root_uuid: &str,
    sync_id: &str,
) -> Result<DriveWebsiteSync, DriveServiceError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {WEBSITE_SYNC_SELECT_COLUMNS}
         FROM dr_drive_website_sync sync
         INNER JOIN dr_drive_website_root root ON root.id=sync.website_root_id
         WHERE sync.tenant_id=$1 AND root.tenant_id=$1 AND root.uuid=$2 AND sync.id=$3"
    )))
    .bind(tenant_id)
    .bind(website_root_uuid)
    .bind(sync_id)
    .fetch_optional(executor)
    .await
    .map_err(|error| internal("get WebsiteSync", error))?;
    row.as_ref()
        .map(map_sync)
        .transpose()?
        .ok_or_else(|| DriveServiceError::NotFound("WebsiteSync not found".to_string()))
}

async fn get_sync_on_connection(
    connection: &mut PgConnection,
    tenant_id: &str,
    website_root_uuid: &str,
    sync_id: &str,
) -> Result<DriveWebsiteSync, DriveServiceError> {
    get_sync_from_executor(&mut *connection, tenant_id, website_root_uuid, sync_id).await
}

async fn find_sync_by_idempotency(
    connection: &mut PgConnection,
    tenant_id: &str,
    idempotency_key: &str,
) -> Result<Option<DriveWebsiteSync>, DriveServiceError> {
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {WEBSITE_SYNC_SELECT_COLUMNS}
         FROM dr_drive_website_sync sync
         INNER JOIN dr_drive_website_root root ON root.id=sync.website_root_id
         WHERE sync.tenant_id=$1 AND sync.idempotency_key=$2"
    )))
    .bind(tenant_id)
    .bind(idempotency_key)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| internal("find WebsiteSync idempotency key", error))?;
    row.as_ref().map(map_sync).transpose()
}

fn sync_matches_create(sync: &DriveWebsiteSync, command: &CreateDriveWebsiteSync) -> bool {
    sync.website_root_uuid == command.website_root_uuid
        && sync.expected_root_version == command.expected_root_version
        && sync.expected_generation == command.expected_generation
        && sync.manifest_sha256 == command.manifest_sha256
        && sync.manifest_file_count == command.manifest_file_count
        && sync.manifest_total_bytes == command.manifest_total_bytes
        && sync.expires_at == command.expires_at
}

async fn get_root_on_connection(
    connection: &mut PgConnection,
    tenant_id: &str,
    website_root_uuid: &str,
) -> Result<DriveWebsiteRoot, DriveServiceError> {
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {WEBSITE_ROOT_SELECT_COLUMNS}
         FROM dr_drive_website_root root
         INNER JOIN dr_drive_space space
           ON space.id=root.space_id AND space.tenant_id=root.tenant_id
         WHERE root.tenant_id=$1 AND root.uuid=$2
           AND space.space_type='website' AND space.lifecycle_status='active'"
    )))
    .bind(tenant_id)
    .bind(website_root_uuid)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| internal("get WebsiteRoot for sync", error))?;
    row.as_ref()
        .map(map_root)
        .transpose()?
        .ok_or_else(|| DriveServiceError::NotFound("active WebsiteRoot not found".to_string()))
}

fn map_sync(row: &PgRow) -> Result<DriveWebsiteSync, DriveServiceError> {
    let status_raw: String = row.get("sync_status");
    let status = DriveWebsiteSyncStatus::try_from_str(&status_raw).ok_or_else(|| {
        DriveServiceError::Internal(format!("unknown WebsiteSync status: {status_raw}"))
    })?;
    Ok(DriveWebsiteSync {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        website_root_id: row.get("website_root_id"),
        website_root_uuid: row.get("website_root_uuid"),
        space_id: row.get("space_id"),
        idempotency_key: row.get("idempotency_key"),
        expected_root_version: row.get("expected_root_version"),
        expected_generation: row.get("expected_generation"),
        staging_node_id: row.get("staging_node_id"),
        manifest_sha256: row.get("manifest_sha256"),
        manifest_file_count: row.get("manifest_file_count"),
        manifest_total_bytes: row.get("manifest_total_bytes"),
        uploaded_file_count: row.get("uploaded_file_count"),
        uploaded_total_bytes: row.get("uploaded_total_bytes"),
        status,
        expires_at: row.get("expires_at"),
        validated_at: row.get("validated_at"),
        activated_at: row.get("activated_at"),
        completed_at: row.get("completed_at"),
        error_code: row.get("error_code"),
        error_summary: row.get("error_summary"),
        version: row.get("version"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn map_root(row: &PgRow) -> Result<DriveWebsiteRoot, DriveServiceError> {
    let source_root_mode_raw: String = row.get("source_root_mode");
    let content_mode_raw: String = row.get("content_mode");
    Ok(DriveWebsiteRoot {
        id: row.get("id"),
        uuid: row.get("uuid"),
        tenant_id: row.get("tenant_id"),
        space_id: row.get("space_id"),
        root_key: row.get("root_key"),
        display_name: row.get("display_name"),
        source_root_mode: DriveWebsiteSourceRootMode::try_from_str(&source_root_mode_raw)
            .ok_or_else(|| {
                DriveServiceError::Internal(format!(
                    "unknown WebsiteRoot source_root_mode: {source_root_mode_raw}"
                ))
            })?,
        selected_folder_node_id: row.get("selected_folder_node_id"),
        content_mode: DriveWebsiteContentMode::try_from_str(&content_mode_raw).ok_or_else(
            || {
                DriveServiceError::Internal(format!(
                    "unknown WebsiteRoot content_mode: {content_mode_raw}"
                ))
            },
        )?,
        active_node_id: row.get("active_node_id"),
        active_generation: row.get("active_generation"),
        root_status: row.get("root_status"),
        version: row.get("version"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn map_generation(row: &PgRow) -> DriveWebsiteGeneration {
    DriveWebsiteGeneration {
        generation_no: row.get("generation_no"),
        root_node_id: row.get("root_node_id"),
        manifest_sha256: row.get("manifest_sha256"),
        file_count: row.get("file_count"),
        total_bytes: row.get("total_bytes"),
        generation_status: row.get("generation_status"),
        activated_at: row.get("activated_at"),
    }
}

async fn insert_audit(
    connection: &mut PgConnection,
    tenant_id: &str,
    action: &str,
    resource_type: &str,
    resource_id: &str,
    operator_id: &str,
    request_id: Option<&str>,
) -> Result<(), DriveServiceError> {
    let audit_id = next_drive_runtime_id("WebsiteSync audit event")?;
    sqlx::query(
        "INSERT INTO dr_drive_audit_event (
            id, tenant_id, action, resource_type, resource_id,
            operator_id, request_id, trace_id
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, NULL)",
    )
    .bind(audit_id)
    .bind(tenant_id)
    .bind(action)
    .bind(resource_type)
    .bind(resource_id)
    .bind(operator_id)
    .bind(request_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| internal("insert WebsiteSync audit event", error))?;
    Ok(())
}

async fn acquire(
    pool: &PgPool,
) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, DriveServiceError> {
    pool.acquire()
        .await
        .map_err(|error| internal("acquire WebsiteSync connection", error))
}

async fn run_serializable_transaction<T, F>(
    pool: &PgPool,
    operation: &str,
    mut action: F,
) -> Result<T, DriveServiceError>
where
    T: Send,
    F: for<'a> FnMut(&'a mut PgConnection) -> WebsiteSyncTransactionFuture<'a, T>,
{
    for attempt in 1..=MAX_SERIALIZABLE_TRANSACTION_ATTEMPTS {
        let mut connection = acquire(pool).await?;
        begin_serializable_write(&mut connection).await?;
        let result = action(&mut connection).await;
        let result = match result {
            Ok(value) => commit(&mut connection, operation).await.map(|()| value),
            Err(error) => Err(error),
        };
        match result {
            Ok(value) => return Ok(value),
            Err(error) => {
                rollback(&mut connection).await;
                if !is_retryable_transaction_error(&error)
                    || attempt == MAX_SERIALIZABLE_TRANSACTION_ATTEMPTS
                {
                    if is_retryable_transaction_error(&error) {
                        sdkwork_drive_observability::metrics::
                            record_website_sync_transaction_retry_exhausted();
                    }
                    return Err(error);
                }
                sdkwork_drive_observability::metrics::record_website_sync_transaction_retry();
                tokio::time::sleep(serializable_retry_delay(attempt)).await;
            }
        }
    }
    unreachable!("WebsiteSync transaction retry loop always returns")
}

async fn begin_serializable_write(connection: &mut PgConnection) -> Result<(), DriveServiceError> {
    sqlx::query(begin_transaction_sql())
        .execute(&mut *connection)
        .await
        .map_err(|error| internal("begin WebsiteSync transaction", error))?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
        .execute(&mut *connection)
        .await
        .map_err(|error| internal("set WebsiteSync serializable isolation", error))?;
    Ok(())
}

async fn commit(connection: &mut PgConnection, operation: &str) -> Result<(), DriveServiceError> {
    sqlx::query("COMMIT")
        .execute(&mut *connection)
        .await
        .map_err(|error| internal(&format!("commit {operation}"), error))?;
    Ok(())
}

async fn rollback(connection: &mut PgConnection) {
    let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
}

fn instant_parameter(parameter: &str) -> String {
    format!("CAST({parameter} AS TIMESTAMPTZ)")
}

fn serializable_retry_delay(attempt: usize) -> StdDuration {
    let exponent = attempt.saturating_sub(1).min(3) as u32;
    let base_millis = 10_u64 << exponent;
    let jitter_millis = (uuid::Uuid::new_v4().as_u128() as u64) % (base_millis + 1);
    StdDuration::from_millis(base_millis + jitter_millis)
}

fn is_retryable_transaction_error(error: &DriveServiceError) -> bool {
    matches!(
        error,
        DriveServiceError::Internal(detail)
            if detail.starts_with(RETRYABLE_TRANSACTION_ERROR_PREFIX)
    )
}

fn internal(operation: &str, error: sqlx::Error) -> DriveServiceError {
    let retryable = matches!(
        &error,
        sqlx::Error::Database(database_error)
            if database_error.code().as_deref() == Some("40001")
    );
    let prefix = if retryable {
        RETRYABLE_TRANSACTION_ERROR_PREFIX
    } else {
        ""
    };
    DriveServiceError::Internal(format!("{prefix}{operation} failed: {error}"))
}
