use crate::domain::change::DriveChangeRecord;
use crate::infrastructure::sql::change_feed_store::SqlChangeFeedStore;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct ListChangesCommand {
    pub tenant_id: String,
    pub space_id: String,
    pub after_sequence: i64,
    pub limit: i64,
    pub subject_type: Option<String>,
    pub subject_id: Option<String>,
    pub is_space_owner: bool,
}

#[derive(Debug, Clone)]
pub struct QueryStartPageTokenCommand {
    pub tenant_id: String,
    pub space_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SqlDriveChangeFeedService {
    store: SqlChangeFeedStore,
}

impl SqlDriveChangeFeedService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            store: SqlChangeFeedStore::new(pool),
        }
    }

    pub async fn list_changes(
        &self,
        command: ListChangesCommand,
    ) -> Result<Vec<DriveChangeRecord>, DriveServiceError> {
        if command.is_space_owner {
            self.store
                .list_changes_for_space_owner(
                    &command.tenant_id,
                    &command.space_id,
                    command.after_sequence,
                    command.limit,
                )
                .await
        } else {
            let subject_type = command.subject_type.ok_or_else(|| {
                DriveServiceError::Validation("subject_type is required".to_string())
            })?;
            let subject_id = command.subject_id.ok_or_else(|| {
                DriveServiceError::Validation("subject_id is required".to_string())
            })?;
            self.store
                .list_changes_for_reader(
                    &command.tenant_id,
                    &command.space_id,
                    command.after_sequence,
                    &subject_type,
                    &subject_id,
                    command.limit,
                )
                .await
        }
    }

    pub async fn query_start_page_token(
        &self,
        command: QueryStartPageTokenCommand,
    ) -> Result<i64, DriveServiceError> {
        self.store
            .query_start_page_token(&command.tenant_id, command.space_id.as_deref())
            .await
    }
}
