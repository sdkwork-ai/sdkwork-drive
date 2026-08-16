use crate::constants::DEFAULT_DOWNLOAD_PUBLIC_BASE_URL;
use crate::runtime_sandbox_roots::{discover_runtime_sandbox_roots, RuntimeSandboxRoot};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppState {
    pub(crate) pool: PgPool,
    pub(crate) download_public_base_url: String,
    pub(crate) runtime_sandbox_roots: Arc<[RuntimeSandboxRoot]>,
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
        Self {
            pool,
            download_public_base_url: download_public_base_url.into(),
            runtime_sandbox_roots: discover_runtime_sandbox_roots().into(),
        }
    }
}
