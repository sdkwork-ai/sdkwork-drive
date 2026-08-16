//! SDKWork Drive database pool bootstrap via `sdkwork-database`.

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool, PoolError};
use sdkwork_drive_config::DatabaseConfig as DriveDatabaseConfig;

pub use sdkwork_drive_database_host::{
    bootstrap_drive_database, bootstrap_drive_database_from_env, DriveDatabaseHost,
};

pub async fn bootstrap_drive_database_for_config(
    config: &DriveDatabaseConfig,
) -> Result<DriveDatabaseHost, String> {
    if let Some(host) = sdkwork_drive_database_host::installed_drive_database_host() {
        return Ok(host.as_ref().clone());
    }

    let sdk_config = drive_database_config_to_sdkwork(config)?;
    let pool = create_pool_from_config(sdk_config)
        .await
        .map_err(|error| error.to_string())?;
    bootstrap_drive_database(pool).await
}

pub async fn connect_drive_database_pool_from_env() -> Result<DatabasePool, PoolError> {
    let config = DriveDatabaseConfig::from_env()
        .map_err(|error| PoolError::DatabaseConfig(error.to_string()))?;
    let sdk_config =
        drive_database_config_to_sdkwork(&config).map_err(PoolError::DatabaseConfig)?;
    create_pool_from_config(sdk_config).await
}

fn drive_database_config_to_sdkwork(
    config: &DriveDatabaseConfig,
) -> Result<DatabaseConfig, String> {
    Ok(DatabaseConfig {
        engine: DatabaseEngine::Postgres,
        url: config.url().to_string(),
        max_connections: config.max_connections(),
        ..Default::default()
    })
}
