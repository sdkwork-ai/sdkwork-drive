use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};

use sdkwork_drive_config::DatabaseConfig;

const POSTGRES_CORE_SQL: &str = include_str!("postgres_core.sql");

pub async fn install_postgres_schema<'c, E>(executor: E) -> Result<(), sqlx::Error>
where
    E: Executor<'c, Database = sqlx::Postgres>,
{
    sqlx::raw_sql(POSTGRES_CORE_SQL).execute(executor).await?;
    Ok(())
}

pub fn postgres_pool_from_database_pool(
    pool: &sdkwork_database_sqlx::DatabasePool,
) -> Result<PgPool, String> {
    pool.as_postgres().cloned().ok_or_else(|| {
        "Drive authoritative server requires the process-shared PostgreSQL pool".to_string()
    })
}


/// Normalize a workspace PostgreSQL connection URL for pool construction.
///
/// Trims surrounding whitespace, forces the `postgres` scheme, and strips a
/// trailing slash so URL fragments compare and connect consistently across
/// the shared workspace database profile.
pub fn normalize_workspace_postgres_url(raw: &str) -> String {
    let trimmed = raw.trim();
    let without_trailing_slash = trimmed.strip_suffix('/').unwrap_or(trimmed);
    if let Some(rest) = without_trailing_slash.strip_prefix("postgresql://") {
        format!("postgres://{rest}")
    } else {
        without_trailing_slash.to_owned()
    }
}

/// Connect to any workspace PostgreSQL URL and install the Drive core schema.
///
/// This is the generic embedded-database sync entrypoint used by hosts that
/// collapse dependency API surfaces into a single standalone gateway: it
/// normalizes the shared workspace database URL, opens a pool, and runs the
/// Drive core DDL (idempotent `CREATE TABLE IF NOT EXISTS` statements).
pub async fn connect_any_database_and_install_schema(
    database_url: &str,
) -> Result<PgPool, sqlx::Error> {
    let normalized = normalize_workspace_postgres_url(database_url);
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&normalized)
        .await?;
    install_postgres_schema(&pool).await?;
    Ok(pool)
}

pub async fn connect_postgres_database_and_install_schema(
    config: &DatabaseConfig,
) -> Result<PgPool, sqlx::Error> {
    sdkwork_database_sqlx::enable_process_shared_database_pool();
    let host = crate::bootstrap::bootstrap_drive_database_for_config(config)
        .await
        .map_err(|error| sqlx::Error::Configuration(error.into()))?;
    postgres_pool_from_database_pool(host.pool())
        .map_err(|error| sqlx::Error::Configuration(error.into()))
}
