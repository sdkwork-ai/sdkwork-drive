use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use sdkwork_database_spi::DefaultDatabaseModule;
use sdkwork_database_sqlx::DatabasePool;

mod bootstrap;

pub use bootstrap::{bootstrap_drive_database, bootstrap_drive_database_from_env};

static DRIVE_DATABASE_HOST: OnceLock<Arc<DriveDatabaseHost>> = OnceLock::new();

#[derive(Clone)]
pub struct DriveDatabaseHost {
    pool: DatabasePool,
    module: Arc<DefaultDatabaseModule>,
}

impl DriveDatabaseHost {
    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    pub fn module(&self) -> Arc<DefaultDatabaseModule> {
        self.module.clone()
    }
}

pub fn installed_drive_database_host() -> Option<Arc<DriveDatabaseHost>> {
    DRIVE_DATABASE_HOST.get().cloned()
}

pub fn install_drive_database_host(host: DriveDatabaseHost) -> Result<(), String> {
    DRIVE_DATABASE_HOST
        .set(Arc::new(host))
        .map_err(|_| "Drive database host is already installed for this process".to_owned())
}

pub fn ensure_drive_database_host_installed(host: DriveDatabaseHost) -> Arc<DriveDatabaseHost> {
    DRIVE_DATABASE_HOST.get_or_init(|| Arc::new(host)).clone()
}

fn resolve_app_root() -> PathBuf {
    std::env::var("SDKWORK_DRIVE_APP_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        })
}

pub(crate) fn resolve_app_root_for_bootstrap() -> PathBuf {
    resolve_app_root()
}
