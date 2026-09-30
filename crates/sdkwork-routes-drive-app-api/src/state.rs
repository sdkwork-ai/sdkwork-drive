use crate::constants::DEFAULT_DOWNLOAD_PUBLIC_BASE_URL;
use crate::runtime_sandbox_roots::{discover_runtime_sandbox_roots, RuntimeSandboxRoot};
use sdkwork_drive_object_runtime::DriveObjectStoreRuntime;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
    pub download_public_base_url: String,
    pub runtime_sandbox_roots: Arc<[RuntimeSandboxRoot]>,
    /// Shared provider -> object store pipeline.
    ///
    /// The app surface holds it so upload signing, download signing, content
    /// reads, packaging, and archive extraction all resolve providers through
    /// one implementation - including the account-center credential path and the
    /// adapter cache that a per-call store construction cannot provide.
    pub object_runtime: DriveObjectStoreRuntime,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            object_runtime: DriveObjectStoreRuntime::new(pool.clone()),
            pool,
            download_public_base_url: DEFAULT_DOWNLOAD_PUBLIC_BASE_URL.to_string(),
            runtime_sandbox_roots: Arc::from([]),
        }
    }

    pub fn with_urls(pool: PgPool, download_public_base_url: impl Into<String>) -> Self {
        // When a deploy-scoped sandbox is configured, do not auto-expose the
        // whole filesystem root (`/`) as a runtime sandbox for ops browsing.
        let runtime_sandbox_roots =
            if crate::deploy_sandbox::deploy_sandbox_config_from_env().is_some() {
                Arc::from([])
            } else {
                discover_runtime_sandbox_roots().into()
            };
        Self {
            object_runtime: DriveObjectStoreRuntime::new(pool.clone()),
            pool,
            download_public_base_url: download_public_base_url.into(),
            runtime_sandbox_roots,
        }
    }
}
