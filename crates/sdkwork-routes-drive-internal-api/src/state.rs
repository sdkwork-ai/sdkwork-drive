use sdkwork_drive_object_runtime::DriveObjectStoreRuntime;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct InternalApiState {
    pub pool: PgPool,
    pub object_runtime: DriveObjectStoreRuntime,
}

impl InternalApiState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            object_runtime: DriveObjectStoreRuntime::new(pool.clone()),
            pool,
        }
    }
}
