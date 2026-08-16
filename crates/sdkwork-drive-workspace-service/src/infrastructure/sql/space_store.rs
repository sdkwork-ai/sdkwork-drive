use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::PgConnection;
use sqlx::PgPool;
use sqlx::Row;

use crate::domain::space::{DriveSpace, DriveSpaceType};
use crate::infrastructure::sql::begin_transaction_sql;
use crate::infrastructure::sql::sql_error::is_unique_constraint_violation;
use crate::ports::space_store::{DriveSpaceStore, ListAccessibleSpacesQuery, NewDriveSpace};
use crate::DriveServiceError;

const SPACE_SELECT_COLUMNS: &str = "id, tenant_id, owner_subject_type, owner_subject_id, display_name, presentation_icon, presentation_color, description, space_type, lifecycle_status, version, created_by";

#[derive(Debug, Clone)]
pub struct SqlSpaceStore {
    pool: PgPool,
}

impl SqlSpaceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn delete_space_on_connection(
        connection: &mut PgConnection,
        tenant_id: &str,
        space_id: &str,
        operator_id: &str,
    ) -> Result<DriveSpace, DriveServiceError> {
        let affected = sqlx::query(
            "UPDATE dr_drive_space
             SET lifecycle_status='deleted', updated_by=$1, updated_at=CURRENT_TIMESTAMP, version=version + 1
             WHERE tenant_id=$2 AND id=$3 AND lifecycle_status='active'",
        )
        .bind(operator_id)
        .bind(tenant_id)
        .bind(space_id)
        .execute(&mut *connection)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("delete dr_drive_space failed: {error}"))
        })?
        .rows_affected();
        if affected == 0 {
            return Err(DriveServiceError::NotFound("space not found".to_string()));
        }

        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {SPACE_SELECT_COLUMNS}
             FROM dr_drive_space
             WHERE tenant_id=$1 AND id=$2",
        )))
        .bind(tenant_id)
        .bind(space_id)
        .fetch_one(&mut *connection)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("read deleted dr_drive_space failed: {error}"))
        })?;
        map_row_to_space(&row)
    }
}

#[async_trait]
impl DriveSpaceStore for SqlSpaceStore {
    async fn insert_space(
        &self,
        new_space: &NewDriveSpace,
    ) -> Result<DriveSpace, DriveServiceError> {
        insert_space_on_executor(&self.pool, new_space).await?;
        read_inserted_space(&self.pool, &new_space.id).await
    }

    async fn insert_website_space(
        &self,
        new_space: &NewDriveSpace,
    ) -> Result<DriveSpace, DriveServiceError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire website space transaction connection failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "begin website space transaction failed: {error}"
                ))
            })?;

        let result: Result<(), DriveServiceError> = async {
            insert_space_on_executor(&mut *connection, new_space).await?;
            super::website_space_store::provision_default_website_root_on_connection(
                &mut connection,
                new_space,
            )
            .await
        }
        .await;

        match result {
            Ok(()) => {
                sqlx::query("COMMIT")
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "commit website space transaction failed: {error}"
                        ))
                    })?;
                drop(connection);
                read_inserted_space(&self.pool, &new_space.id).await
            }
            Err(error) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                Err(error)
            }
        }
    }

    async fn list_spaces(
        &self,
        tenant_id: &str,
        owner_subject_type: Option<&str>,
        owner_subject_id: Option<&str>,
        space_type: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DriveSpace>, DriveServiceError> {
        let rows = match (owner_subject_type, owner_subject_id, space_type) {
            (Some(owner_type), Some(owner_id), Some(space_type)) => sqlx::query(
                sqlx::AssertSqlSafe(format!(
                    "SELECT {SPACE_SELECT_COLUMNS}
                    FROM dr_drive_space
                    WHERE tenant_id=$1
                      AND owner_subject_type=$2
                      AND owner_subject_id=$3
                      AND space_type=$4
                      AND lifecycle_status='active'
                    ORDER BY id ASC
                    LIMIT $5 OFFSET $6",
                )),
            )
            .bind(tenant_id)
            .bind(owner_type)
            .bind(owner_id)
            .bind(space_type)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!("list dr_drive_space by owner failed: {error}"))
            })?,
            (Some(owner_type), Some(owner_id), None) => sqlx::query(sqlx::AssertSqlSafe(format!(
                "SELECT {SPACE_SELECT_COLUMNS}
                    FROM dr_drive_space
                    WHERE tenant_id=$1
                      AND owner_subject_type=$2
                      AND owner_subject_id=$3
                      AND lifecycle_status='active'
                    ORDER BY id ASC
                    LIMIT $4 OFFSET $5",
            )))
            .bind(tenant_id)
            .bind(owner_type)
            .bind(owner_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!("list dr_drive_space by owner failed: {error}"))
            })?,
            (None, None, Some(space_type)) => sqlx::query(sqlx::AssertSqlSafe(format!(
                "SELECT {SPACE_SELECT_COLUMNS}
                    FROM dr_drive_space
                    WHERE tenant_id=$1
                      AND space_type=$2
                      AND lifecycle_status='active'
                    ORDER BY id ASC
                    LIMIT $3 OFFSET $4",
            )))
            .bind(tenant_id)
            .bind(space_type)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!("list dr_drive_space by type failed: {error}"))
            })?,
            _ => sqlx::query(sqlx::AssertSqlSafe(format!(
                "SELECT {SPACE_SELECT_COLUMNS}
                    FROM dr_drive_space
                    WHERE tenant_id=$1
                      AND lifecycle_status='active'
                    ORDER BY id ASC
                    LIMIT $2 OFFSET $3",
            )))
            .bind(tenant_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "list dr_drive_space by tenant failed: {error}"
                ))
            })?,
        };

        rows.iter().map(map_row_to_space).collect()
    }

    async fn list_accessible_spaces(
        &self,
        query: ListAccessibleSpacesQuery<'_>,
    ) -> Result<Vec<DriveSpace>, DriveServiceError> {
        use crate::infrastructure::sql::acl_predicate::space_accessible_to_subject_sql;

        let space_accessible_predicate =
            space_accessible_to_subject_sql("dr_drive_space", "$2", "$3");
        let rows = match (
            query.owner_subject_type,
            query.owner_subject_id,
            query.space_type,
        ) {
            (Some(owner_type), Some(owner_id), Some(space_type)) => {
                sqlx::query(sqlx::AssertSqlSafe(format!(
                    "SELECT {SPACE_SELECT_COLUMNS}
                 FROM dr_drive_space
                 WHERE tenant_id=$1
                   AND owner_subject_type=$4
                   AND owner_subject_id=$5
                   AND space_type=$6
                   AND lifecycle_status='active'
                   AND ({space_accessible_predicate})
                 ORDER BY id ASC
                 LIMIT $7 OFFSET $8",
                )))
                .bind(query.tenant_id)
                .bind(query.viewer_subject_type)
                .bind(query.viewer_subject_id)
                .bind(owner_type)
                .bind(owner_id)
                .bind(space_type)
                .bind(query.limit)
                .bind(query.offset)
                .fetch_all(&self.pool)
                .await
                .map_err(|error| {
                    DriveServiceError::Internal(format!(
                        "list accessible dr_drive_space by owner failed: {error}"
                    ))
                })?
            }
            (Some(owner_type), Some(owner_id), None) => sqlx::query(sqlx::AssertSqlSafe(format!(
                "SELECT {SPACE_SELECT_COLUMNS}
                 FROM dr_drive_space
                 WHERE tenant_id=$1
                   AND owner_subject_type=$4
                   AND owner_subject_id=$5
                   AND lifecycle_status='active'
                   AND ({space_accessible_predicate})
                 ORDER BY id ASC
                 LIMIT $6 OFFSET $7",
            )))
            .bind(query.tenant_id)
            .bind(query.viewer_subject_type)
            .bind(query.viewer_subject_id)
            .bind(owner_type)
            .bind(owner_id)
            .bind(query.limit)
            .bind(query.offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "list accessible dr_drive_space by owner failed: {error}"
                ))
            })?,
            (None, None, Some(space_type)) => sqlx::query(sqlx::AssertSqlSafe(format!(
                "SELECT {SPACE_SELECT_COLUMNS}
                 FROM dr_drive_space
                 WHERE tenant_id=$1
                   AND space_type=$4
                   AND lifecycle_status='active'
                   AND ({space_accessible_predicate})
                 ORDER BY id ASC
                 LIMIT $5 OFFSET $6",
            )))
            .bind(query.tenant_id)
            .bind(query.viewer_subject_type)
            .bind(query.viewer_subject_id)
            .bind(space_type)
            .bind(query.limit)
            .bind(query.offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "list accessible dr_drive_space by type failed: {error}"
                ))
            })?,
            _ => sqlx::query(sqlx::AssertSqlSafe(format!(
                "SELECT {SPACE_SELECT_COLUMNS}
                 FROM dr_drive_space
                 WHERE tenant_id=$1
                   AND lifecycle_status='active'
                   AND ({space_accessible_predicate})
                 ORDER BY id ASC
                 LIMIT $4 OFFSET $5",
            )))
            .bind(query.tenant_id)
            .bind(query.viewer_subject_type)
            .bind(query.viewer_subject_id)
            .bind(query.limit)
            .bind(query.offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "list accessible dr_drive_space by tenant failed: {error}"
                ))
            })?,
        };

        rows.iter().map(map_row_to_space).collect()
    }

    async fn get_space(
        &self,
        tenant_id: &str,
        space_id: &str,
    ) -> Result<DriveSpace, DriveServiceError> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {SPACE_SELECT_COLUMNS}
             FROM dr_drive_space
             WHERE tenant_id=$1 AND id=$2 AND lifecycle_status='active'",
        )))
        .bind(tenant_id)
        .bind(space_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("get dr_drive_space failed: {error}"))
        })?;
        let Some(row) = row else {
            return Err(DriveServiceError::NotFound("space not found".to_string()));
        };
        map_row_to_space(&row)
    }

    #[allow(clippy::too_many_arguments)]
    async fn update_space(
        &self,
        tenant_id: &str,
        space_id: &str,
        display_name: Option<&str>,
        presentation_icon: Option<&str>,
        presentation_color: Option<&str>,
        description: Option<&str>,
        operator_id: &str,
    ) -> Result<DriveSpace, DriveServiceError> {
        let existing = self.get_space(tenant_id, space_id).await?;
        let next_display_name = display_name.unwrap_or(existing.display_name.as_str());
        let next_presentation_icon = presentation_icon
            .map(str::to_string)
            .or(existing.presentation_icon);
        let next_presentation_color = presentation_color
            .map(str::to_string)
            .or(existing.presentation_color);
        let next_description = description.map(str::to_string).or(existing.description);

        let affected = sqlx::query(
            "UPDATE dr_drive_space
             SET display_name=$1,
                 presentation_icon=$2,
                 presentation_color=$3,
                 description=$4,
                 updated_by=$5,
                 updated_at=CURRENT_TIMESTAMP,
                 version=version + 1
             WHERE tenant_id=$6 AND id=$7 AND lifecycle_status='active'",
        )
        .bind(next_display_name)
        .bind(&next_presentation_icon)
        .bind(&next_presentation_color)
        .bind(&next_description)
        .bind(operator_id)
        .bind(tenant_id)
        .bind(space_id)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("update dr_drive_space failed: {error}"))
        })?
        .rows_affected();
        if affected == 0 {
            return Err(DriveServiceError::NotFound("space not found".to_string()));
        }
        self.get_space(tenant_id, space_id).await
    }

    async fn delete_space(
        &self,
        tenant_id: &str,
        space_id: &str,
        operator_id: &str,
    ) -> Result<DriveSpace, DriveServiceError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire delete dr_drive_space connection failed: {error}"
            ))
        })?;
        Self::delete_space_on_connection(&mut connection, tenant_id, space_id, operator_id).await
    }
}

async fn insert_space_on_executor<'e, E>(
    executor: E,
    new_space: &NewDriveSpace,
) -> Result<(), DriveServiceError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let result = sqlx::query(
        "INSERT INTO dr_drive_space (
                id, tenant_id, owner_subject_type, owner_subject_id,
                space_type, display_name, presentation_icon, presentation_color, description,
                lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 1, $11, $12)",
    )
    .bind(&new_space.id)
    .bind(&new_space.tenant_id)
    .bind(&new_space.owner_subject_type)
    .bind(&new_space.owner_subject_id)
    .bind(&new_space.space_type)
    .bind(&new_space.display_name)
    .bind(&new_space.presentation_icon)
    .bind(&new_space.presentation_color)
    .bind(&new_space.description)
    .bind(&new_space.lifecycle_status)
    .bind(&new_space.created_by)
    .bind(&new_space.updated_by)
    .execute(executor)
    .await;

    if let Err(error) = result {
        let message = error.to_string();
        if is_unique_constraint_violation(&message) {
            return Err(DriveServiceError::Conflict(
                "space already exists for tenant/owner/type".to_string(),
            ));
        }
        return Err(DriveServiceError::Internal(format!(
            "insert dr_drive_space failed: {message}"
        )));
    }

    Ok(())
}

async fn read_inserted_space<'e, E>(
    executor: E,
    space_id: &str,
) -> Result<DriveSpace, DriveServiceError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {SPACE_SELECT_COLUMNS}
         FROM dr_drive_space
         WHERE id=$1",
    )))
    .bind(space_id)
    .fetch_one(executor)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("read inserted dr_drive_space failed: {error}"))
    })?;
    map_row_to_space(&row)
}

fn map_row_to_space(row: &PgRow) -> Result<DriveSpace, DriveServiceError> {
    let space_type_raw: String = row.get("space_type");
    let space_type = DriveSpaceType::try_from_str(&space_type_raw).ok_or_else(|| {
        DriveServiceError::Internal(format!("unknown space_type in database: {space_type_raw}"))
    })?;

    Ok(DriveSpace {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        owner_subject_type: row.get("owner_subject_type"),
        owner_subject_id: row.get("owner_subject_id"),
        display_name: row.get("display_name"),
        presentation_icon: row.get("presentation_icon"),
        presentation_color: row.get("presentation_color"),
        description: row.get("description"),
        space_type,
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
        created_by: row.get("created_by"),
    })
}
