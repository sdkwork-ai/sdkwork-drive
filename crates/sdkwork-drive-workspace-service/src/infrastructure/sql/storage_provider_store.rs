use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::PgPool;
use sqlx::Postgres;
use sqlx::Row;

use crate::domain::storage_provider::{DriveStorageProvider, DriveStorageProviderKind};
use crate::infrastructure::sql::sql_error::is_unique_constraint_violation;
use crate::ports::storage_provider_store::{
    DriveStorageProviderLookup, DriveStorageProviderStore, NewDriveStorageProvider,
    UpdateDriveStorageProvider,
};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlStorageProviderStore {
    pool: PgPool,
}

impl SqlStorageProviderStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveStorageProviderStore for SqlStorageProviderStore {
    async fn insert_storage_provider(
        &self,
        new_provider: &NewDriveStorageProvider,
    ) -> Result<DriveStorageProvider, DriveServiceError> {
        let result = sqlx::query(
            "INSERT INTO dr_drive_storage_provider (
                id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
                strict_tls, credential_ref, provider_account_id, server_side_encryption_mode,
                default_storage_class, status, version, created_by, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, 1, $15, $16)",
        )
        .bind(&new_provider.id)
        .bind(&new_provider.tenant_id)
        .bind(&new_provider.provider_kind)
        .bind(&new_provider.name)
        .bind(&new_provider.endpoint_url)
        .bind(&new_provider.region)
        .bind(&new_provider.bucket)
        .bind(new_provider.path_style)
        .bind(new_provider.strict_tls)
        .bind(&new_provider.credential_ref)
        .bind(&new_provider.provider_account_id)
        .bind(&new_provider.server_side_encryption_mode)
        .bind(&new_provider.default_storage_class)
        .bind(&new_provider.status)
        .bind(&new_provider.created_by)
        .bind(&new_provider.updated_by)
        .execute(&self.pool)
        .await;

        if let Err(error) = result {
            let message = error.to_string();
            if is_unique_constraint_violation(&message) {
                return Err(DriveServiceError::Conflict(
                    "storage provider already exists".to_string(),
                ));
            }
            return Err(DriveServiceError::Internal(format!(
                "insert dr_drive_storage_provider failed: {message}"
            )));
        }

        let row = sqlx::query(
            "SELECT id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
                    strict_tls, credential_ref, provider_account_id, server_side_encryption_mode,
                    default_storage_class, status, version
             FROM dr_drive_storage_provider
             WHERE id=$1",
        )
        .bind(&new_provider.id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "read inserted dr_drive_storage_provider failed: {error}"
            ))
        })?;

        map_row_to_storage_provider(&row)
    }

    async fn list_storage_providers(
        &self,
        tenant_id: &str,
        provider_kind: Option<&str>,
        status: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DriveStorageProvider>, DriveServiceError> {
        // Both filters are pushed into SQL rather than applied to a fetched page.
        // Filtering the kind *after* the cursor window is what made the console's
        // provider switch answer "no rows" for a kind whose row simply sat on the
        // next page (`PAGINATION_SPEC.md` §2.2 rule 4).
        //
        // The statement is assembled with `QueryBuilder` so the optional
        // predicates stay numbered bind parameters: no caller value is ever
        // interpolated into the SQL text, only the literal fragments below.
        let mut builder = sqlx::QueryBuilder::<Postgres>::new(
            "SELECT id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
                    strict_tls, credential_ref, provider_account_id, server_side_encryption_mode,
                    default_storage_class, status, version
             FROM dr_drive_storage_provider",
        );
        builder.push(" WHERE ");
        {
            let mut predicates = builder.separated(" AND ");
            // Tenant scope is unconditional: the optional kind/status filters
            // only ever narrow the requesting tenant's own rows.
            predicates
                .push("tenant_id = ")
                .push_bind_unseparated(tenant_id);
            if let Some(kind) = provider_kind {
                // `custom` is a family, not one stored value: the console offers
                // a single "custom" option while the column persists
                // `custom:<vendor>`. `starts_with` keeps this an exact prefix
                // test — a `LIKE` pattern would read the `_` in every catalogued
                // kind as a wildcard and match unrelated custom keys.
                let custom_prefix = format!("{kind}:");
                predicates
                    .push("(provider_kind = ")
                    .push_bind_unseparated(kind)
                    .push_unseparated(" OR starts_with(provider_kind, ")
                    .push_bind_unseparated(custom_prefix)
                    .push_unseparated("))");
            }
            if let Some(status_value) = status {
                predicates
                    .push("status = ")
                    .push_bind_unseparated(status_value);
            }
        }
        builder
            .push(" ORDER BY id ASC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);

        let rows = builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "list dr_drive_storage_provider failed: {error}"
                ))
            })?;

        rows.iter().map(map_row_to_storage_provider).collect()
    }

    async fn find_storage_provider(
        &self,
        provider_id: &str,
    ) -> Result<Option<DriveStorageProvider>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
                    strict_tls, credential_ref, provider_account_id, server_side_encryption_mode,
                    default_storage_class, status, version
             FROM dr_drive_storage_provider
             WHERE id=$1",
        )
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("find dr_drive_storage_provider failed: {error}"))
        })?;

        let Some(row) = row else {
            return Ok(None);
        };
        map_row_to_storage_provider(&row).map(Some)
    }

    async fn update_storage_provider(
        &self,
        provider_id: &str,
        patch: &UpdateDriveStorageProvider,
    ) -> Result<DriveStorageProvider, DriveServiceError> {
        let updated = sqlx::query(
            "UPDATE dr_drive_storage_provider
             SET name=$2,
                 endpoint_url=$3,
                 region=$4,
                 bucket=$5,
                 path_style=$6,
                 strict_tls=$7,
                 credential_ref=$8,
                 provider_account_id=$9,
                 server_side_encryption_mode=$10,
                 default_storage_class=$11,
                 status=$12,
                 version=version + 1,
                 updated_by=$13,
                 updated_at=CURRENT_TIMESTAMP
             WHERE id=$1",
        )
        .bind(provider_id)
        .bind(&patch.name)
        .bind(&patch.endpoint_url)
        .bind(&patch.region)
        .bind(&patch.bucket)
        .bind(patch.path_style)
        .bind(patch.strict_tls)
        .bind(&patch.credential_ref)
        .bind(&patch.provider_account_id)
        .bind(&patch.server_side_encryption_mode)
        .bind(&patch.default_storage_class)
        .bind(&patch.status)
        .bind(&patch.updated_by)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("update dr_drive_storage_provider failed: {error}"))
        })?;

        if updated.rows_affected() == 0 {
            return Err(DriveServiceError::NotFound(
                "storage provider not found".to_string(),
            ));
        }

        let row = sqlx::query(
            "SELECT id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
                    strict_tls, credential_ref, provider_account_id, server_side_encryption_mode,
                    default_storage_class, status, version
             FROM dr_drive_storage_provider
             WHERE id=$1",
        )
        .bind(provider_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "read updated dr_drive_storage_provider failed: {error}"
            ))
        })?;

        map_row_to_storage_provider(&row)
    }

    async fn has_active_storage_provider_bindings(
        &self,
        provider_id: &str,
    ) -> Result<bool, DriveServiceError> {
        // `EXISTS` returns a real `boolean`, unlike `SELECT 1`, whose literal is
        // `int4` and therefore cannot be decoded as `i64`.
        let found: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                 SELECT 1
                 FROM dr_drive_storage_provider_binding
                 WHERE provider_id=$1 AND lifecycle_status='active'
             )",
        )
        .bind(provider_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "check active dr_drive_storage_provider_binding failed: {error}"
            ))
        })?;
        Ok(found)
    }

    async fn count_active_storage_provider_objects(
        &self,
        provider_id: &str,
    ) -> Result<i64, DriveServiceError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM dr_drive_storage_object
             WHERE storage_provider_id=$1 AND lifecycle_status='active'",
        )
        .bind(provider_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "count active dr_drive_storage_object rows failed: {error}"
            ))
        })?;
        Ok(count)
    }

    async fn find_active_storage_provider_by_bucket(
        &self,
        tenant_id: &str,
        bucket: &str,
    ) -> Result<DriveStorageProviderLookup, DriveServiceError> {
        // Fetch every candidate instead of `LIMIT 1`: the extra rows are what
        // make ambiguity detectable. A tenant has a handful of providers, so
        // the additional row cost is not worth trading for a silent wrong
        // answer.
        let rows = sqlx::query(
            "SELECT id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
                    strict_tls, credential_ref, provider_account_id, server_side_encryption_mode,
                    default_storage_class, status, version
             FROM dr_drive_storage_provider
             WHERE tenant_id=$1 AND status='active' AND bucket=$2
             ORDER BY id ASC",
        )
        .bind(tenant_id)
        .bind(bucket)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "resolve active dr_drive_storage_provider by bucket failed: {error}"
            ))
        })?;

        let mut providers = rows
            .iter()
            .map(map_row_to_storage_provider)
            .collect::<Result<Vec<_>, _>>()?;
        match providers.len() {
            0 => Ok(DriveStorageProviderLookup::Missing),
            1 => Ok(DriveStorageProviderLookup::Found(
                providers.pop().expect("one provider"),
            )),
            _ => Ok(DriveStorageProviderLookup::Ambiguous(
                providers.into_iter().map(|provider| provider.id).collect(),
            )),
        }
    }

    async fn delete_storage_provider(&self, provider_id: &str) -> Result<bool, DriveServiceError> {
        let deleted = sqlx::query(
            "UPDATE dr_drive_storage_provider
             SET status='deleted',
                 version=version + 1,
                 updated_at=CURRENT_TIMESTAMP
             WHERE id=$1 AND status != 'deleted'",
        )
        .bind(provider_id)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("delete dr_drive_storage_provider failed: {error}"))
        })?;
        Ok(deleted.rows_affected() > 0)
    }
}

fn map_row_to_storage_provider(row: &PgRow) -> Result<DriveStorageProvider, DriveServiceError> {
    let provider_kind_raw: String = row.get("provider_kind");
    let provider_kind =
        DriveStorageProviderKind::try_from_str(&provider_kind_raw).ok_or_else(|| {
            DriveServiceError::Internal(format!(
                "storage provider kind is invalid: {provider_kind_raw}"
            ))
        })?;
    let status: String = row.get("status");
    if status.trim().is_empty() {
        return Err(DriveServiceError::Internal(
            "storage provider status is empty".to_string(),
        ));
    }

    Ok(DriveStorageProvider {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        provider_kind,
        name: row.get("name"),
        endpoint_url: row.get("endpoint_url"),
        region: row.get("region"),
        bucket: row.get("bucket"),
        path_style: get_bool(row, "path_style")?,
        strict_tls: get_bool(row, "strict_tls")?,
        credential_ref: row.get("credential_ref"),
        provider_account_id: row.get("provider_account_id"),
        server_side_encryption_mode: row.get("server_side_encryption_mode"),
        default_storage_class: row.get("default_storage_class"),
        status,
        version: row.get("version"),
    })
}

fn get_bool(row: &PgRow, column: &str) -> Result<bool, DriveServiceError> {
    row.try_get::<bool, _>(column)
        .or_else(|_| row.try_get::<i64, _>(column).map(|value| value != 0))
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "decode dr_drive_storage_provider.{column} as bool failed: {error}"
            ))
        })
}
