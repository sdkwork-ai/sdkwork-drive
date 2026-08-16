use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_sqlx::{create_pool_from_config, enable_process_shared_database_pool};
use sqlx::pool::PoolConnection;
use sqlx::{PgPool, Postgres, Row};

const DRIVE_TEST_ADVISORY_LOCK_KEY: i64 = 0x4452_4956_4554_4553;
const LAZY_POSTGRES_TEST_URL: &str = "postgres://sdkwork:sdkwork@127.0.0.1:5432/sdkwork_ai_test";

pub struct PostgresTestDatabaseGuard {
    lock_connection: Option<PoolConnection<Postgres>>,
}

/// Builds a non-connecting PostgreSQL pool for router tests that exit before data access.
pub fn lazy_postgres_test_pool() -> PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(LAZY_POSTGRES_TEST_URL)
        .expect("create lazy PostgreSQL test pool")
}

impl Drop for PostgresTestDatabaseGuard {
    fn drop(&mut self) {
        if let Some(connection) = self.lock_connection.take() {
            drop(connection.detach());
        }
    }
}

/// Creates a serialized PostgreSQL test fixture from the standard workspace URL.
///
/// Tests skip only when SDKWORK_DATABASE_URL is absent. When the variable is set,
/// connection, lifecycle, or cleanup failures fail the test immediately.
pub async fn postgres_test_database() -> Option<(PgPool, PostgresTestDatabaseGuard)> {
    let database_url = match std::env::var("SDKWORK_DATABASE_URL") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            eprintln!("skip PostgreSQL integration test: SDKWORK_DATABASE_URL is not set");
            return None;
        }
    };

    enable_process_shared_database_pool();
    let process_pool = create_pool_from_config(DatabaseConfig {
        engine: DatabaseEngine::Postgres,
        url: database_url,
        max_connections: 8,
        min_connections: 1,
        ..DatabaseConfig::default()
    })
    .await
    .expect("create process-shared PostgreSQL test pool");
    let pool = process_pool
        .as_postgres()
        .cloned()
        .expect("Drive server tests require PostgreSQL");
    let mut lock_connection = pool
        .acquire()
        .await
        .expect("acquire PostgreSQL test lock connection");
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(DRIVE_TEST_ADVISORY_LOCK_KEY)
        .execute(&mut *lock_connection)
        .await
        .expect("acquire Drive PostgreSQL test advisory lock");

    sdkwork_drive_database_host::bootstrap_drive_database(process_pool)
        .await
        .expect("bootstrap Drive PostgreSQL test schema");
    truncate_drive_tables(&mut lock_connection).await;

    Some((
        pool,
        PostgresTestDatabaseGuard {
            lock_connection: Some(lock_connection),
        },
    ))
}

async fn truncate_drive_tables(connection: &mut PoolConnection<Postgres>) {
    let rows = sqlx::query(
        "SELECT tablename
         FROM pg_tables
         WHERE schemaname = current_schema()
           AND tablename LIKE 'dr_drive_%'
         ORDER BY tablename",
    )
    .fetch_all(&mut **connection)
    .await
    .expect("list Drive PostgreSQL test tables");
    let table_names = rows
        .into_iter()
        .map(|row| row.get::<String, _>("tablename"))
        .collect::<Vec<_>>();
    if table_names.is_empty() {
        return;
    }

    let identifiers = table_names
        .iter()
        .map(|table_name| {
            assert!(
                table_name
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
                "unexpected Drive table identifier {table_name}"
            );
            format!("\"{table_name}\"")
        })
        .collect::<Vec<_>>()
        .join(", ");
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "TRUNCATE TABLE {identifiers} RESTART IDENTITY CASCADE"
    )))
    .execute(&mut **connection)
    .await
    .expect("truncate Drive PostgreSQL test tables");
}
