//! PostgreSQL integration-test fixture.
//!
//! # Why every fixture owns its own pool
//!
//! sqlx binds a pool to the Tokio runtime that built it: the returning half of
//! `PoolConnection::drop` releases the pool's semaphore permit from a task
//! spawned on `tokio::runtime::Handle::current()` (sqlx `pool/connection.rs` →
//! `crate::rt::spawn`). `#[tokio::test]` gives **every** test its own
//! current-thread runtime, so a pool shared between tests cannot work: the
//! moment test A's runtime shuts down, any connection it still had checked out
//! can never be returned, its permit is lost forever, and test B blocks until
//! its `acquire_timeout` expires and fails with `PoolTimedOut`.
//!
//! The fixture therefore binds a pool to each test's own runtime. The pool is
//! closed by [`PostgresTestDatabaseGuard`] while that runtime is still running,
//! so the close event is actually driven and no connection outlives its runtime.
//!
//! # Reference data
//!
//! `dr_drive_storage_provider_kind` rows come from the baseline DDL, not from the
//! application, so the per-test truncate preserves them; only the operator-owned
//! `enabled` flag is reset, which is the part a test may mutate.
//!
//! # Self-healing against destructive tests
//!
//! A few tests simulate infrastructure failures by `DROP TABLE`-ing a Drive
//! table. Because the schema is shared, that would strand every later test in
//! the binary with a missing table. The fixture therefore *verifies* that the
//! baseline's table set is complete before each test and re-applies the baseline
//! DDL when anything is missing. The baseline is written entirely with guarded
//! `IF NOT EXISTS` blocks, so replaying it is idempotent and never disturbs
//! surviving tables or their rows.

use std::path::PathBuf;
use std::time::Duration;

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_drive_workspace_service::application::storage_provider_kind_service::STORAGE_PROVIDER_KIND_TABLE;
use sqlx::{Connection, PgPool, Row};

/// Placeholder URL used by [`lazy_postgres_test_pool`] when the operator has not
/// configured a real test database.
///
/// It deliberately points at a database that does not exist, so a route test that
/// accidentally performs data access fails loudly instead of silently passing.
const LAZY_POSTGRES_TEST_URL: &str = "postgres://sdkwork:sdkwork@127.0.0.1:5432/sdkwork_ai_test";

/// Advisory-lock key that serializes Drive integration tests, one at a time.
///
/// Spells "DRIVETES" in ASCII hex, so it is unlikely to collide with anything
/// else on a shared test cluster.
const TEST_ADVISORY_LOCK_KEY: i64 = 0x4452_4956_4554_4553;

/// Per-fixture connection ceiling.
///
/// One pool exists per test, so keep this small: the harness runs tests
/// concurrently, and the server's `max_connections` (200 by default) has to hold
/// every pool at once. A single Drive service test needs only a handful.
const TEST_POOL_MAX_CONNECTIONS: u32 = 5;

/// Runs the schema bootstrap exactly once per process, after repairing any gap.
///
/// The repair runs first: the bootstrap's drift pass fails closed on a missing
/// table, so a schema damaged by an earlier destructive test has to be healed
/// *before* the bootstrap can succeed. The bootstrap itself is idempotent but not
/// free — it runs DDL and a full information-schema drift pass — so the
/// `OnceLock` receipt keeps late fixtures from repeating it.
async fn bootstrap_schema_once(pool: &PgPool, database_url: &str) {
    use std::sync::OnceLock;
    static BOOTSTRAP_ONCE: OnceLock<()> = OnceLock::new();
    static BOOTSTRAP_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    ensure_baseline_schema(pool).await;
    if BOOTSTRAP_ONCE.get().is_some() {
        return;
    }
    let _guard = BOOTSTRAP_MUTEX.lock().await;
    if BOOTSTRAP_ONCE.get().is_some() {
        return;
    }
    // Re-check under the lock: a sibling may have dropped a table between our
    // repair and acquiring it.
    ensure_baseline_schema(pool).await;
    let database_pool = bootstrap_pool(pool, database_url);
    sdkwork_drive_database_host::bootstrap_drive_database(database_pool)
        .await
        .expect("bootstrap Drive PostgreSQL test schema");
    let _ = BOOTSTRAP_ONCE.set(());
}

/// Wraps the fixture's `PgPool` in the framework's [`DatabasePool`] for bootstrap.
///
/// This reuses the fixture's pool — connections and all — rather than opening a
/// second one. The wrapper is deliberately **not** process-shared: the registry
/// is left disabled, because a process-shared pool is reused across tests and
/// therefore across runtimes, which is exactly the failure this module exists to
/// prevent.
fn bootstrap_pool(pool: &PgPool, database_url: &str) -> sdkwork_database_sqlx::DatabasePool {
    sdkwork_database_sqlx::DatabasePool::Postgres(
        pool.clone(),
        sdkwork_database_sqlx::PoolContext {
            config: DatabaseConfig {
                engine: DatabaseEngine::Postgres,
                url: database_url.to_string(),
                max_connections: TEST_POOL_MAX_CONNECTIONS,
                min_connections: 0,
                ..DatabaseConfig::default()
            },
        },
    )
}

/// Guards a single test's fixture: holds the test's serialization lock and closes
/// its pool.
///
/// `lock_connection` carries a `pg_advisory_lock` taken for the whole test, so
/// no sibling can truncate the tables this test is reading. It is a *dedicated*
/// connection, never part of the test's pool: a pooled connection would let two
/// tests share one session, and `pg_advisory_lock` is re-entrant per session, so
/// both would "hold" the same lock.
///
/// The pool is closed here, inside the test's own runtime, so sqlx can drive the
/// close event; a pool merely dropped instead would leak its connections.
pub struct PostgresTestDatabaseGuard {
    pool: PgPool,
    lock_connection: Option<sqlx::postgres::PgConnection>,
}

impl Drop for PostgresTestDatabaseGuard {
    fn drop(&mut self) {
        // Closing the session releases the advisory lock.
        drop(self.lock_connection.take());

        let pool = self.pool.clone();
        // `close()` drives its close event from the runtime that created the
        // pool, which is the caller's runtime and still alive here. Hand the
        // close to a detached task so we neither block the runtime nor drop the
        // pool without closing it.
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::spawn(async move {
                let _ = tokio::time::timeout(Duration::from_secs(5), pool.close()).await;
            });
        }
    }
}

/// Builds a non-connecting PostgreSQL pool for router tests that exit before data access.
///
/// The pool targets `SDKWORK_DATABASE_URL` when the operator has configured one,
/// so a `/readyz` probe driven by the same pool reaches the real test database
/// and reports `200` rather than a spurious `503`. When the variable is absent
/// the placeholder URL is used: the pool then stays unconnected (its first
/// checkout is what would fail), which is exactly what route-shape tests need.
pub fn lazy_postgres_test_pool() -> PgPool {
    let database_url = std::env::var("SDKWORK_DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| LAZY_POSTGRES_TEST_URL.to_string());
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(&database_url)
        .expect("create lazy PostgreSQL test pool")
}

/// Creates a PostgreSQL test fixture bound to the caller's runtime.
///
/// Tests skip only when `SDKWORK_DATABASE_URL` is absent. When the variable is
/// set, connection, lifecycle or cleanup failures fail the test immediately.
///
/// The pool belongs to the calling test alone; see the module docs for why.
///
/// Keep `SDKWORK_DATABASE_AUTO_MIGRATE=1` set: without it the bootstrap only runs
/// `init` and the schema may lag the migrations.
pub async fn postgres_test_database() -> Option<(PgPool, PostgresTestDatabaseGuard)> {
    let database_url = match std::env::var("SDKWORK_DATABASE_URL") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            eprintln!("skip PostgreSQL integration test: SDKWORK_DATABASE_URL is not set");
            return None;
        }
    };

    // Serialize the whole test, not just setup: a sibling that truncated after
    // this fixture returned would wipe the rows this test is about to assert on.
    // The lock lives on its own connection so its session cannot be shared.
    let mut lock_connection = sqlx::postgres::PgConnection::connect(&database_url)
        .await
        .expect("connect Drive test lock connection");
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(TEST_ADVISORY_LOCK_KEY)
        .execute(&mut lock_connection)
        .await
        .expect("acquire Drive test advisory lock");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(TEST_POOL_MAX_CONNECTIONS)
        .connect(&database_url)
        .await
        .expect("create PostgreSQL test pool");

    bootstrap_schema_once(&pool, &database_url).await;
    truncate_drive_tables(&pool).await;
    reset_storage_provider_kind_state(&pool).await;

    Some((
        pool.clone(),
        PostgresTestDatabaseGuard {
            pool,
            lock_connection: Some(lock_connection),
        },
    ))
}

/// Removes all Drive *test* state, keeping catalogued reference data.
///
/// `dr_drive_storage_provider_kind` is deliberately preserved: production seeds
/// it from the baseline DDL (`0001_drive_baseline.sql`) and migration `0008`, and
/// the application never recreates it on demand. Truncating it leaves the catalog
/// empty, which makes `ensure_storage_provider_kind_available` reject every
/// built-in kind with `NotFound` — surfacing as a bogus 404 from
/// `POST /storage/providers` and breaking unrelated route tests.
///
/// The `TRUNCATE` runs inside an explicit transaction that is committed before
/// the connection returns to the pool. `TRUNCATE` takes `ACCESS EXCLUSIVE` on
/// every listed table, and without an explicit `COMMIT` sqlx would leave the
/// connection idle *inside* that transaction, holding those locks until the
/// server-side idle-in-transaction timeout.
async fn truncate_drive_tables(pool: &PgPool) {
    let rows = sqlx::query(
        "SELECT tablename
         FROM pg_tables
         WHERE schemaname = current_schema()
           AND tablename LIKE 'dr_drive_%'
           AND tablename <> $1
         ORDER BY tablename",
    )
    .bind(STORAGE_PROVIDER_KIND_TABLE)
    .fetch_all(pool)
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
    let mut transaction = pool
        .begin()
        .await
        .expect("begin Drive PostgreSQL truncate transaction");
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "TRUNCATE TABLE {identifiers} RESTART IDENTITY CASCADE"
    )))
    .execute(&mut *transaction)
    .await
    .expect("truncate Drive PostgreSQL test tables");
    transaction
        .commit()
        .await
        .expect("commit Drive PostgreSQL truncate transaction");
}

/// Re-applies the baseline DDL when any of its tables is missing.
///
/// Tests that simulate a missing table (`DROP TABLE dr_drive_storage_object`)
/// leave the shared schema permanently broken for their siblings. Rather than
/// duplicate table definitions in test code — which would drift from the
/// contract — the fixture checks for gaps and replays the authoritative baseline.
/// The baseline is idempotent by construction, so this is safe on a healthy
/// schema and repairing on a damaged one.
async fn ensure_baseline_schema(pool: &PgPool) {
    let baseline_tables = baseline_table_names();
    if baseline_tables.is_empty() {
        return;
    }
    let existing: Vec<String> =
        sqlx::query_scalar("SELECT tablename FROM pg_tables WHERE schemaname = current_schema()")
            .fetch_all(pool)
            .await
            .expect("list baseline Drive tables");
    if baseline_tables
        .iter()
        .all(|name| existing.iter().any(|row| row == name))
    {
        return;
    }

    let path = baseline_ddl_path().expect("resolve Drive baseline DDL path");
    let ddl = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read Drive baseline DDL {}: {error}", path.display()));
    sqlx::raw_sql(sqlx::AssertSqlSafe(ddl))
        .execute(pool)
        .await
        .expect("re-apply Drive baseline DDL after a destructive test");
}

/// The set of Drive tables the baseline creates.
fn baseline_table_names() -> Vec<String> {
    let Some(path) = baseline_ddl_path() else {
        return Vec::new();
    };
    let Ok(ddl) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    ddl.lines()
        .filter_map(|line| {
            let rest = line
                .trim_start()
                .strip_prefix("CREATE TABLE IF NOT EXISTS ")?;
            let name = rest.split(['(', ' ']).next()?.trim().to_string();
            (!name.is_empty()).then_some(name)
        })
        .collect()
}

/// Resolves `<app_root>/database/ddl/baseline/postgres/0001_drive_baseline.sql`.
///
/// Mirrors the host crate's own app-root resolution so tests and the application
/// agree on where the schema lives.
fn baseline_ddl_path() -> Option<PathBuf> {
    let app_root = std::env::var("SDKWORK_DRIVE_APP_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let path = app_root.join("database/ddl/baseline/postgres/0001_drive_baseline.sql");
    path.exists().then_some(path)
}

/// Restores the baseline `enabled` default of the provider-kind catalog.
///
/// The catalog *rows* are reference data seeded by the baseline DDL and must
/// survive the truncate. The `enabled` flag, however, is operator-mutable state:
/// a test that disables a kind would otherwise leak that choice into every test
/// that runs afterwards, because the catalog is not truncated. The baseline
/// seeds every kind as `enabled = TRUE` (`0001_drive_baseline.sql`), so
/// restoring that default is the per-test isolation the harness owes its tests.
async fn reset_storage_provider_kind_state(pool: &PgPool) {
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "UPDATE \"{STORAGE_PROVIDER_KIND_TABLE}\" SET enabled = TRUE WHERE enabled IS DISTINCT FROM TRUE"
    )))
    .execute(pool)
    .await
    .expect("reset Drive storage provider kind state");
}
