use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct OpenState {
    pub(crate) pool: PgPool,
}

impl OpenState {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
