use crate::config::AdminStorageConfig;
use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct AdminStorageState {
    pub(crate) pool: PgPool,
    pub(crate) config: AdminStorageConfig,
}

impl AdminStorageState {
    pub(crate) fn new(pool: PgPool, config: AdminStorageConfig) -> Self {
        Self { pool, config }
    }
}
