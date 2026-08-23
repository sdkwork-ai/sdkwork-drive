use crate::constants::DEFAULT_DOWNLOAD_PUBLIC_BASE_URL;
use crate::runtime_sandbox_roots::{discover_runtime_sandbox_roots, RuntimeSandboxRoot};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
    pub download_public_base_url: String,
    pub runtime_sandbox_roots: Arc<[RuntimeSandboxRoot]>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            download_public_base_url: DEFAULT_DOWNLOAD_PUBLIC_BASE_URL.to_string(),
            runtime_sandbox_roots: Arc::from([]),
        }
    }

    pub fn with_urls(pool: PgPool, download_public_base_url: impl Into<String>) -> Self {
        // When a deploy-scoped sandbox is configured, do not auto-expose the
        // whole filesystem root (`/`) as a runtime sandbox for ops browsing.
        let runtime_sandbox_roots = if crate::deploy_sandbox::deploy_sandbox_config_from_env().is_some()
        {
            Arc::from([])
        } else {
            discover_runtime_sandbox_roots().into()
        };
        Self {
            pool,
            download_public_base_url: download_public_base_url.into(),
            runtime_sandbox_roots,
        }
    }
}
