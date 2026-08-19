use sqlx::PgPool;

#[derive(Clone)]
pub struct GitState {
    pub pool: PgPool,
}

impl GitState {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
