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

/// Resolves the Drive application root that owns `database/`.
///
/// `SDKWORK_DRIVE_APP_ROOT` wins because packaged hosts stage the module under
/// `<install root>/database-modules/sdkwork-drive/database`; the crate's own
/// repository root is the development fallback.
pub fn resolve_app_root() -> PathBuf {
    std::env::var("SDKWORK_DRIVE_APP_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        })
}

/// Loads the canonical Drive database module from the resolved application root.
///
/// Mirrors the `database_module()` entrypoint every other `*-database-host`
/// crate exposes, so a composing host (for example the CloudRouter installer)
/// can register Drive into its own lifecycle registry and apply the exact same
/// baseline, migrations, and seed set on `install` and `upgrade` that the
/// standalone serve path applies. Registering it is what makes
/// `database/seeds/common/002_storage_provider_catalog.sql` land during an
/// explicit lifecycle command instead of only at gateway boot.
pub fn database_module() -> Result<DefaultDatabaseModule, String> {
    DefaultDatabaseModule::from_app_root(&resolve_app_root())
        .map_err(|error| format!("load drive database module failed: {error}"))
}

pub(crate) fn resolve_app_root_for_bootstrap() -> PathBuf {
    resolve_app_root()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATABASE_MANIFEST: &str = include_str!("../../../database/database.manifest.json");
    const SEED_MANIFEST: &str = include_str!("../../../database/seeds/seed.manifest.json");
    const BOOTSTRAP_SOURCE: &str = include_str!("bootstrap.rs");

    /// The module declares `seedOnBoot`, and the converge path must honor it:
    /// reference data that exists only in `database/seeds` is invisible to a
    /// deployed database unless the lifecycle actually runs the seed step.
    #[test]
    fn lifecycle_bootstrap_applies_the_declared_seed_set() {
        assert!(
            DATABASE_MANIFEST.contains("\"seedOnBoot\": true"),
            "the module manifest must keep declaring seedOnBoot, or the storage catalog never lands"
        );
        let gate = BOOTSTRAP_SOURCE
            .find("if options.seed_on_boot {")
            .expect("bootstrap must gate the seed step on the resolved lifecycle options");
        let seed_call = BOOTSTRAP_SOURCE
            .find(".seed(&options.seed_locale, &options.seed_profile)")
            .expect("bootstrap must run the seed step with the manifest/profile-resolved locale");
        assert!(
            seed_call > gate,
            "the seed call must live inside the seed_on_boot gate"
        );
    }

    /// The standard profile is what install/upgrade applies; the storage provider
    /// catalog seed has to be part of it, or `/admin/storage/providers` opens
    /// empty on a database that reports itself installed.
    #[test]
    fn standard_seed_profile_ships_the_storage_provider_catalog() {
        assert!(
            SEED_MANIFEST.contains("\"common/002_storage_provider_catalog.sql\""),
            "the standard seed profile must register the storage provider catalog seed"
        );

        let seed_path =
            resolve_app_root().join("database/seeds/common/002_storage_provider_catalog.sql");
        assert!(
            seed_path.is_file(),
            "the seed file registered in the manifest must exist at {}",
            seed_path.display()
        );
    }

    /// `database_module()` is the composing-host entrypoint; it must resolve the
    /// same module manifest the serve path uses.
    #[test]
    fn database_module_loads_the_repository_manifest() {
        let module = database_module().expect("load the Drive database module");
        assert_eq!(module.manifest().module_id, "drive");
    }
}
