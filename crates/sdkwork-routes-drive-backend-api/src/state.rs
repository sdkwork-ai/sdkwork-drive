use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct BackendState {
    pub(crate) pool: PgPool,
}

impl BackendState {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
