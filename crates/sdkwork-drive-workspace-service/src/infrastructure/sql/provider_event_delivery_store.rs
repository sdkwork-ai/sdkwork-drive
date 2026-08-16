use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, PgPool, Row};

use crate::domain::provider_event_delivery::DriveProviderEventDelivery;
use crate::infrastructure::sql::begin_transaction_sql;
use crate::ports::provider_event_delivery_store::{
    DriveProviderEventDeliveryStore, DriveProviderEventResourceKind,
    EnsureDriveProviderEventDelivery, EnsureDriveProviderEventDeliveryResult,
};
use crate::DriveServiceError;

const DELIVERY_SELECT_COLUMNS: &str = "id, address, expiration_epoch_ms, lifecycle_status, version, CAST(created_at AS TEXT) AS created_at, CAST(updated_at AS TEXT) AS updated_at";

#[derive(Debug, Clone)]
pub struct SqlProviderEventDeliveryStore {
    pool: PgPool,
}

impl SqlProviderEventDeliveryStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveProviderEventDeliveryStore for SqlProviderEventDeliveryStore {
    async fn ensure_delivery(
        &self,
        command: &EnsureDriveProviderEventDelivery,
    ) -> Result<EnsureDriveProviderEventDeliveryResult, DriveServiceError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire provider event delivery transaction failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "begin provider event delivery transaction failed: {error}"
                ))
            })?;
        let result = ensure_on_connection(&mut connection, command).await;
        match result {
            Ok(result) => {
                sqlx::query("COMMIT")
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "commit provider event delivery transaction failed: {error}"
                        ))
                    })?;
                Ok(result)
            }
            Err(error) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                Err(error)
            }
        }
    }
}

async fn ensure_on_connection(
    connection: &mut PgConnection,
    command: &EnsureDriveProviderEventDelivery,
) -> Result<EnsureDriveProviderEventDeliveryResult, DriveServiceError> {
    let space_id = resolve_active_provider_resource(connection, command).await?;
    let existing = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {DELIVERY_SELECT_COLUMNS}, space_id, node_id, resource_type, resource_id, token_hash
         FROM dr_drive_watch_channel
         WHERE tenant_id=$1 AND id=$2",
    )))
    .bind(&command.tenant_id)
    .bind(&command.channel_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "find provider event delivery channel failed: {error}"
        ))
    })?;

    if let Some(existing) = existing {
        validate_existing_channel(&existing, &space_id, command)?;
        let unchanged = existing.get::<String, _>("address") == command.address
            && existing.get::<Option<String>, _>("token_hash").as_deref()
                == Some(command.signing_key_sha256.as_str())
            && existing.get::<i64, _>("expiration_epoch_ms") == command.expiration_epoch_ms
            && existing.get::<String, _>("lifecycle_status") == "active"
            && existing.get::<Option<String>, _>("resource_id").as_deref()
                == Some(command.provider_resource_uuid.as_str());
        if unchanged {
            return Ok(EnsureDriveProviderEventDeliveryResult {
                delivery: map_delivery(&existing, &command.provider_resource_uuid),
                created: false,
            });
        }
        sqlx::query(
            "UPDATE dr_drive_watch_channel
             SET resource_id=$1, address=$2, token_hash=$3, expiration_epoch_ms=$4,
                 lifecycle_status='active', updated_by=$5,
                 updated_at=CURRENT_TIMESTAMP, version=version + 1
             WHERE tenant_id=$6 AND id=$7",
        )
        .bind(&command.provider_resource_uuid)
        .bind(&command.address)
        .bind(&command.signing_key_sha256)
        .bind(command.expiration_epoch_ms)
        .bind(&command.operator_id)
        .bind(&command.tenant_id)
        .bind(&command.channel_id)
        .execute(&mut *connection)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "update provider event delivery channel failed: {error}"
            ))
        })?;
        return read_delivery(connection, command, false).await;
    }

    sqlx::query(
        "INSERT INTO dr_drive_watch_channel (
            id, tenant_id, space_id, node_id, resource_type, resource_id,
            channel_type, address, token_hash, expiration_epoch_ms,
            lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, NULL, 'changes', $4, 'web_hook', $5, $6, $7,
                   'active', 1, $8, $8)",
    )
    .bind(&command.channel_id)
    .bind(&command.tenant_id)
    .bind(&space_id)
    .bind(&command.provider_resource_uuid)
    .bind(&command.address)
    .bind(&command.signing_key_sha256)
    .bind(command.expiration_epoch_ms)
    .bind(&command.operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "insert provider event delivery channel failed: {error}"
        ))
    })?;
    read_delivery(connection, command, true).await
}

async fn resolve_active_provider_resource(
    connection: &mut PgConnection,
    command: &EnsureDriveProviderEventDelivery,
) -> Result<String, DriveServiceError> {
    let (table, status_column, required_status, extra_predicate) = match command.resource_kind {
        DriveProviderEventResourceKind::KnowledgebaseRawScope => (
            "dr_drive_root_scope_subscription",
            "scope_status",
            "active",
            " AND consumer_kind='knowledgebase_raw'",
        ),
        DriveProviderEventResourceKind::WebsiteRoot => {
            ("dr_drive_website_root", "root_status", "active", "")
        }
    };
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT space_id, {status_column} AS resource_status
         FROM {table}
         WHERE tenant_id=$1 AND uuid=$2{extra_predicate}",
    )))
    .bind(&command.tenant_id)
    .bind(&command.provider_resource_uuid)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "find provider resource for event delivery failed: {error}"
        ))
    })?
    .ok_or_else(|| DriveServiceError::NotFound("provider resource not found".to_string()))?;
    if row.get::<String, _>("resource_status") != required_status {
        return Err(DriveServiceError::Conflict(
            "provider resource is not active".to_string(),
        ));
    }
    Ok(row.get("space_id"))
}

fn validate_existing_channel(
    row: &PgRow,
    space_id: &str,
    command: &EnsureDriveProviderEventDelivery,
) -> Result<(), DriveServiceError> {
    let resource_id: Option<String> = row.get("resource_id");
    let legacy_knowledgebase_scope = matches!(
        command.resource_kind,
        DriveProviderEventResourceKind::KnowledgebaseRawScope
    ) && resource_id.as_deref() == Some(space_id);
    if row.get::<Option<String>, _>("space_id").as_deref() != Some(space_id)
        || row.get::<Option<String>, _>("node_id").is_some()
        || row.get::<String, _>("resource_type") != "changes"
        || (resource_id.as_deref() != Some(command.provider_resource_uuid.as_str())
            && !legacy_knowledgebase_scope)
    {
        return Err(DriveServiceError::Conflict(
            "provider event delivery channel cannot be retargeted".to_string(),
        ));
    }
    Ok(())
}

async fn read_delivery(
    connection: &mut PgConnection,
    command: &EnsureDriveProviderEventDelivery,
    created: bool,
) -> Result<EnsureDriveProviderEventDeliveryResult, DriveServiceError> {
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {DELIVERY_SELECT_COLUMNS}
         FROM dr_drive_watch_channel WHERE tenant_id=$1 AND id=$2",
    )))
    .bind(&command.tenant_id)
    .bind(&command.channel_id)
    .fetch_one(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "read provider event delivery channel failed: {error}"
        ))
    })?;
    Ok(EnsureDriveProviderEventDeliveryResult {
        delivery: map_delivery(&row, &command.provider_resource_uuid),
        created,
    })
}

fn map_delivery(row: &PgRow, provider_resource_uuid: &str) -> DriveProviderEventDelivery {
    DriveProviderEventDelivery {
        channel_id: row.get("id"),
        provider_resource_uuid: provider_resource_uuid.to_string(),
        address: row.get("address"),
        expiration_epoch_ms: row.get("expiration_epoch_ms"),
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
