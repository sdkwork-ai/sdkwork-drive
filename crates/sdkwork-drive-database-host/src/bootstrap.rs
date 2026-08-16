use std::sync::Arc;

use sdkwork_database_lifecycle::{lifecycle_options_from_env, LifecycleOrchestrator};
use sdkwork_database_spi::{DatabaseAssetProvider, DatabaseManifest, DefaultDatabaseModule};
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool};

use crate::{
    ensure_drive_database_host_installed, installed_drive_database_host,
    resolve_app_root_for_bootstrap, DriveDatabaseHost,
};

pub async fn bootstrap_drive_database(pool: DatabasePool) -> Result<DriveDatabaseHost, String> {
    if pool.as_postgres().is_none() {
        return Err("Drive authoritative server requires PostgreSQL".to_string());
    }
    let app_root = resolve_app_root_for_bootstrap();
    let module = Arc::new(
        DefaultDatabaseModule::from_app_root(&app_root)
            .map_err(|error| format!("load drive database module failed: {error}"))?,
    );
    let manifest = DatabaseManifest::from_file(module.manifest_path())
        .map_err(|error| format!("read drive database manifest failed: {error}"))?;
    let options = lifecycle_options_from_env("DRIVE", &manifest);
    let orchestrator =
        LifecycleOrchestrator::new(pool.clone(), module.clone()).with_applied_by("sdkwork-drive");

    orchestrator
        .init()
        .await
        .map_err(|error| format!("drive database init failed: {error}"))?;

    if options.auto_migrate {
        orchestrator
            .migrate()
            .await
            .map_err(|error| format!("drive database migrate failed: {error}"))?;
    }

    Ok(
        ensure_drive_database_host_installed(DriveDatabaseHost { pool, module })
            .as_ref()
            .clone(),
    )
}

pub async fn bootstrap_drive_database_from_env() -> Result<DriveDatabaseHost, String> {
    if let Some(host) = installed_drive_database_host() {
        return Ok(host.as_ref().clone());
    }

    let _ = dotenvy::dotenv();
    let drive_config = sdkwork_drive_config::DatabaseConfig::from_env()
        .map_err(|error| format!("read drive database config failed: {error}"))?;
    let config = sdkwork_database_config::DatabaseConfig {
        engine: sdkwork_database_config::DatabaseEngine::Postgres,
        url: drive_config.url().to_string(),
        max_connections: drive_config.max_connections(),
        ..Default::default()
    };
    let pool = create_pool_from_config(config)
        .await
        .map_err(|error| format!("create drive database pool failed: {error}"))?;
    bootstrap_drive_database(pool).await
}
