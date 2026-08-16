use sqlx::PgPool;

/// Schema migration status.
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    pub current_version: i64,
    pub target_version: i64,
    pub applied_migrations: Vec<String>,
    pub pending_migrations: Vec<String>,
}

/// Run database schema migrations.
///
/// This function applies pending migrations to bring the database
/// schema up to date with the current application version.
pub async fn run_migrations(pool: &PgPool) -> Result<MigrationStatus, sqlx::Error> {
    // Check current schema version
    let current_version = get_current_schema_version(pool).await?;

    // Get list of pending migrations
    let pending = get_pending_migrations(current_version);

    // Apply each pending migration
    let mut applied = Vec::new();
    for migration in &pending {
        apply_migration(pool, migration).await?;
        applied.push(migration.clone());
    }

    let target_version = current_version + applied.len() as i64;

    Ok(MigrationStatus {
        current_version,
        target_version,
        applied_migrations: applied,
        pending_migrations: Vec::new(),
    })
}

/// Get the current schema version from the database.
async fn get_current_schema_version(pool: &PgPool) -> Result<i64, sqlx::Error> {
    // Try to read from schema_version table
    let result = sqlx::query_scalar::<_, i64>(
        "SELECT version FROM _drive_schema_version ORDER BY version DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.unwrap_or(0))
}

/// Get list of pending migrations based on current version.
fn get_pending_migrations(current_version: i64) -> Vec<String> {
    let all_migrations = vec![
        "001_create_drive_space_table",
        "002_create_drive_node_table",
        "003_create_drive_storage_provider_table",
        "004_create_drive_upload_session_table",
        "005_create_drive_upload_part_table",
        "006_create_drive_download_grant_table",
        "007_create_drive_quota_usage_table",
        "008_create_drive_audit_event_table",
    ];

    all_migrations
        .into_iter()
        .skip(current_version as usize)
        .map(|s| s.to_string())
        .collect()
}

/// Apply a single migration.
async fn apply_migration(pool: &PgPool, migration: &str) -> Result<(), sqlx::Error> {
    tracing::info!("Applying migration: {}", migration);

    // Execute migration SQL based on migration name
    match migration {
        "001_create_drive_space_table" => {
            sqlx::query(CREATE_SPACE_TABLE_SQL).execute(pool).await?;
        }
        "002_create_drive_node_table" => {
            sqlx::query(CREATE_NODE_TABLE_SQL).execute(pool).await?;
        }
        "003_create_drive_storage_provider_table" => {
            sqlx::query(CREATE_STORAGE_PROVIDER_TABLE_SQL)
                .execute(pool)
                .await?;
        }
        "004_create_drive_upload_session_table" => {
            sqlx::query(CREATE_UPLOAD_SESSION_TABLE_SQL)
                .execute(pool)
                .await?;
        }
        "005_create_drive_upload_part_table" => {
            sqlx::query(CREATE_UPLOAD_PART_TABLE_SQL)
                .execute(pool)
                .await?;
        }
        "006_create_drive_download_grant_table" => {
            sqlx::query(CREATE_DOWNLOAD_GRANT_TABLE_SQL)
                .execute(pool)
                .await?;
        }
        "007_create_drive_quota_usage_table" => {
            sqlx::query(CREATE_QUOTA_USAGE_TABLE_SQL)
                .execute(pool)
                .await?;
        }
        "008_create_drive_audit_event_table" => {
            sqlx::query(CREATE_AUDIT_EVENT_TABLE_SQL)
                .execute(pool)
                .await?;
        }
        _ => {
            tracing::warn!("Unknown migration: {}", migration);
        }
    }

    // Update schema version
    sqlx::query(
        "INSERT INTO _drive_schema_version (version, name, applied_at) VALUES ($1, $2, CURRENT_TIMESTAMP)"
    )
    .bind(migration.len() as i64)
    .bind(migration)
    .execute(pool)
    .await?;

    Ok(())
}

// Migration SQL constants
const CREATE_SPACE_TABLE_SQL: &str = "
CREATE TABLE IF NOT EXISTS drive_space (
    id VARCHAR(128) PRIMARY KEY,
    tenant_id VARCHAR(128) NOT NULL,
    owner_type VARCHAR(64) NOT NULL,
    owner_id VARCHAR(128) NOT NULL,
    space_type VARCHAR(64) NOT NULL,
    name VARCHAR(512) NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    created_at_ms BIGINT NOT NULL,
    updated_at_ms BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_drive_space_tenant ON drive_space(tenant_id);
";

const CREATE_NODE_TABLE_SQL: &str = "
CREATE TABLE IF NOT EXISTS drive_node (
    id VARCHAR(128) PRIMARY KEY,
    space_id VARCHAR(128) NOT NULL,
    parent_id VARCHAR(128),
    node_type VARCHAR(64) NOT NULL,
    name VARCHAR(512) NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    content_state VARCHAR(64) NOT NULL DEFAULT 'active',
    created_at_ms BIGINT NOT NULL,
    updated_at_ms BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_drive_node_space ON drive_node(space_id);
CREATE INDEX IF NOT EXISTS idx_drive_node_parent ON drive_node(parent_id);
";

const CREATE_STORAGE_PROVIDER_TABLE_SQL: &str = "
CREATE TABLE IF NOT EXISTS drive_storage_provider (
    id VARCHAR(128) PRIMARY KEY,
    provider_kind VARCHAR(64) NOT NULL,
    name VARCHAR(256) NOT NULL,
    endpoint_url VARCHAR(1024) NOT NULL,
    region VARCHAR(128),
    bucket VARCHAR(256) NOT NULL,
    path_style BOOLEAN NOT NULL DEFAULT FALSE,
    credential_ref VARCHAR(512),
    status VARCHAR(64) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 1,
    created_at_ms BIGINT NOT NULL,
    updated_at_ms BIGINT NOT NULL
);
";

const CREATE_UPLOAD_SESSION_TABLE_SQL: &str = "
CREATE TABLE IF NOT EXISTS drive_upload_session (
    id VARCHAR(128) PRIMARY KEY,
    space_id VARCHAR(128) NOT NULL,
    node_id VARCHAR(128) NOT NULL,
    idempotency_key VARCHAR(256),
    state VARCHAR(64) NOT NULL DEFAULT 'created',
    expires_at_ms BIGINT NOT NULL,
    created_at_ms BIGINT NOT NULL,
    updated_at_ms BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_drive_upload_session_space ON drive_upload_session(space_id);
CREATE INDEX IF NOT EXISTS idx_drive_upload_session_idempotency ON drive_upload_session(idempotency_key);
";

const CREATE_UPLOAD_PART_TABLE_SQL: &str = "
CREATE TABLE IF NOT EXISTS drive_upload_part (
    id VARCHAR(128) PRIMARY KEY,
    session_id VARCHAR(128) NOT NULL,
    part_number INTEGER NOT NULL,
    etag VARCHAR(256),
    size_bytes BIGINT NOT NULL,
    uploaded BOOLEAN NOT NULL DEFAULT FALSE,
    created_at_ms BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_drive_upload_part_session ON drive_upload_part(session_id);
";

const CREATE_DOWNLOAD_GRANT_TABLE_SQL: &str = "
CREATE TABLE IF NOT EXISTS drive_download_grant (
    id VARCHAR(128) PRIMARY KEY,
    tenant_id VARCHAR(128) NOT NULL,
    node_id VARCHAR(128) NOT NULL,
    expires_at_ms BIGINT NOT NULL,
    created_at_ms BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_drive_download_grant_tenant ON drive_download_grant(tenant_id);
";

const CREATE_QUOTA_USAGE_TABLE_SQL: &str = "
CREATE TABLE IF NOT EXISTS drive_quota_usage (
    id VARCHAR(128) PRIMARY KEY,
    tenant_id VARCHAR(128) NOT NULL,
    space_id VARCHAR(128),
    used_bytes BIGINT NOT NULL DEFAULT 0,
    file_count BIGINT NOT NULL DEFAULT 0,
    updated_at_ms BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_drive_quota_usage_tenant ON drive_quota_usage(tenant_id);
";

const CREATE_AUDIT_EVENT_TABLE_SQL: &str = "
CREATE TABLE IF NOT EXISTS drive_audit_event (
    id VARCHAR(128) PRIMARY KEY,
    tenant_id VARCHAR(128) NOT NULL,
    operator_id VARCHAR(128) NOT NULL,
    action VARCHAR(128) NOT NULL,
    resource_type VARCHAR(128) NOT NULL,
    resource_id VARCHAR(128) NOT NULL,
    details TEXT,
    created_at_ms BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_drive_audit_event_tenant ON drive_audit_event(tenant_id);
CREATE INDEX IF NOT EXISTS idx_drive_audit_event_created ON drive_audit_event(created_at_ms);
";
