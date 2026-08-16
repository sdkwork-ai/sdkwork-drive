use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{PgConnection, PgPool, Row};

use crate::domain::root_scope_subscription::DriveRootScopeSubscription;
use crate::infrastructure::sql::{begin_transaction_sql, next_drive_runtime_id};
use crate::ports::root_scope_subscription_store::{
    DriveRootScopeSubscriptionStore, RegisterDriveRootScopeSubscription,
    RegisterDriveRootScopeSubscriptionResult, KNOWLEDGEBASE_RAW_CONSUMER_KIND,
};
use crate::DriveServiceError;

const SUBSCRIPTION_SELECT_COLUMNS: &str = "id, uuid, tenant_id, space_id, consumer_kind, consumer_resource_id, root_node_id, scope_status, version, CAST(created_at AS TEXT) AS created_at, CAST(updated_at AS TEXT) AS updated_at";

#[derive(Debug, Clone)]
pub struct SqlRootScopeSubscriptionStore {
    pool: PgPool,
}

impl SqlRootScopeSubscriptionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveRootScopeSubscriptionStore for SqlRootScopeSubscriptionStore {
    async fn register_knowledgebase_raw(
        &self,
        registration: &RegisterDriveRootScopeSubscription,
    ) -> Result<RegisterDriveRootScopeSubscriptionResult, DriveServiceError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire root scope subscription transaction failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "begin root scope subscription transaction failed: {error}"
                ))
            })?;

        let result = register_on_connection(&mut connection, registration).await;
        match result {
            Ok(result) => {
                sqlx::query("COMMIT")
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "commit root scope subscription transaction failed: {error}"
                        ))
                    })?;
                Ok(result)
            }
            Err(error) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                if matches!(error, DriveServiceError::Conflict(_)) {
                    if let Some(existing) = find_by_consumer_on_connection(
                        &mut connection,
                        &registration.tenant_id,
                        &registration.consumer_resource_id,
                    )
                    .await?
                    {
                        return validate_existing(existing, registration);
                    }
                }
                Err(error)
            }
        }
    }

    async fn get_by_uuid(
        &self,
        tenant_id: &str,
        subscription_uuid: &str,
    ) -> Result<DriveRootScopeSubscription, DriveServiceError> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {SUBSCRIPTION_SELECT_COLUMNS}
             FROM dr_drive_root_scope_subscription
             WHERE tenant_id=$1 AND uuid=$2",
        )))
        .bind(tenant_id)
        .bind(subscription_uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("get root scope subscription failed: {error}"))
        })?;
        row.as_ref()
            .map(map_subscription)
            .transpose()?
            .ok_or_else(|| {
                DriveServiceError::NotFound("root scope subscription not found".to_string())
            })
    }
}

async fn register_on_connection(
    connection: &mut PgConnection,
    registration: &RegisterDriveRootScopeSubscription,
) -> Result<RegisterDriveRootScopeSubscriptionResult, DriveServiceError> {
    validate_knowledgebase_raw_folder(connection, registration).await?;
    if let Some(existing) = find_by_consumer_on_connection(
        connection,
        &registration.tenant_id,
        &registration.consumer_resource_id,
    )
    .await?
    {
        return validate_existing(existing, registration);
    }

    let subscription_id = next_drive_runtime_id("root scope subscription")?.to_string();
    let subscription_uuid = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO dr_drive_root_scope_subscription (
            id, uuid, tenant_id, space_id, consumer_kind, consumer_resource_id,
            root_node_id, scope_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', 1, $8, $8)",
    )
    .bind(&subscription_id)
    .bind(&subscription_uuid)
    .bind(&registration.tenant_id)
    .bind(&registration.space_id)
    .bind(KNOWLEDGEBASE_RAW_CONSUMER_KIND)
    .bind(&registration.consumer_resource_id)
    .bind(&registration.root_node_id)
    .bind(&registration.operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        let message = error.to_string();
        if message.to_ascii_lowercase().contains("unique") {
            DriveServiceError::Conflict(
                "knowledgebase raw root scope subscription already exists".to_string(),
            )
        } else {
            DriveServiceError::Internal(format!("insert root scope subscription failed: {message}"))
        }
    })?;

    let subscription = find_by_consumer_on_connection(
        connection,
        &registration.tenant_id,
        &registration.consumer_resource_id,
    )
    .await?
    .ok_or_else(|| {
        DriveServiceError::Internal("read inserted root scope subscription failed".to_string())
    })?;
    Ok(RegisterDriveRootScopeSubscriptionResult {
        subscription,
        created: true,
    })
}

async fn validate_knowledgebase_raw_folder(
    connection: &mut PgConnection,
    registration: &RegisterDriveRootScopeSubscription,
) -> Result<(), DriveServiceError> {
    let valid_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_space space_row
         INNER JOIN dr_drive_node raw_node
            ON raw_node.tenant_id=space_row.tenant_id
           AND raw_node.space_id=space_row.id
         INNER JOIN dr_drive_node sources_node
            ON sources_node.tenant_id=raw_node.tenant_id
           AND sources_node.space_id=raw_node.space_id
           AND sources_node.id=raw_node.parent_node_id
         INNER JOIN dr_drive_node root_node
            ON root_node.tenant_id=sources_node.tenant_id
           AND root_node.space_id=sources_node.space_id
           AND root_node.id=sources_node.parent_node_id
         WHERE space_row.tenant_id=$1
           AND space_row.id=$2
           AND space_row.space_type='knowledge_base'
           AND space_row.lifecycle_status='active'
           AND raw_node.id=$3
           AND raw_node.space_type='knowledge_base'
           AND raw_node.node_type='folder'
           AND raw_node.node_name='raw'
           AND raw_node.lifecycle_status='active'
           AND sources_node.space_type='knowledge_base'
           AND sources_node.node_type='folder'
           AND sources_node.node_name='sources'
           AND sources_node.lifecycle_status='active'
           AND root_node.space_type='knowledge_base'
           AND root_node.node_type='folder'
           AND root_node.parent_node_id IS NULL
           AND root_node.lifecycle_status='active'",
    )
    .bind(&registration.tenant_id)
    .bind(&registration.space_id)
    .bind(&registration.root_node_id)
    .fetch_one(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "validate knowledgebase sources/raw folder failed: {error}"
        ))
    })?;
    if valid_count != 1 {
        return Err(DriveServiceError::Validation(
            "knowledgebase raw scope requires the exact active sources/raw folder in an active knowledge_base Space"
                .to_string(),
        ));
    }
    Ok(())
}

fn validate_existing(
    existing: DriveRootScopeSubscription,
    registration: &RegisterDriveRootScopeSubscription,
) -> Result<RegisterDriveRootScopeSubscriptionResult, DriveServiceError> {
    if existing.space_id != registration.space_id
        || existing.root_node_id != registration.root_node_id
    {
        return Err(DriveServiceError::Conflict(
            "knowledgebase raw root scope subscription cannot be retargeted".to_string(),
        ));
    }
    if existing.scope_status == "revoked" {
        return Err(DriveServiceError::Conflict(
            "revoked knowledgebase raw root scope subscription cannot be reused".to_string(),
        ));
    }
    Ok(RegisterDriveRootScopeSubscriptionResult {
        subscription: existing,
        created: false,
    })
}

async fn find_by_consumer_on_connection(
    connection: &mut PgConnection,
    tenant_id: &str,
    consumer_resource_id: &str,
) -> Result<Option<DriveRootScopeSubscription>, DriveServiceError> {
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {SUBSCRIPTION_SELECT_COLUMNS}
         FROM dr_drive_root_scope_subscription
         WHERE tenant_id=$1
           AND consumer_kind=$2
           AND consumer_resource_id=$3
         ORDER BY CASE scope_status WHEN 'active' THEN 0 WHEN 'suspended' THEN 1 ELSE 2 END,
                  created_at DESC
         LIMIT 1",
    )))
    .bind(tenant_id)
    .bind(KNOWLEDGEBASE_RAW_CONSUMER_KIND)
    .bind(consumer_resource_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "find root scope subscription by consumer failed: {error}"
        ))
    })?;
    row.as_ref().map(map_subscription).transpose()
}

fn map_subscription(row: &PgRow) -> Result<DriveRootScopeSubscription, DriveServiceError> {
    Ok(DriveRootScopeSubscription {
        id: row.get("id"),
        uuid: row.get("uuid"),
        tenant_id: row.get("tenant_id"),
        space_id: row.get("space_id"),
        consumer_kind: row.get("consumer_kind"),
        consumer_resource_id: row.get("consumer_resource_id"),
        root_node_id: row.get("root_node_id"),
        scope_status: row.get("scope_status"),
        version: row.get("version"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
