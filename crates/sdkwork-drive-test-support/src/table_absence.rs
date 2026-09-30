//! Non-destructive simulation of a missing table.
//!
//! A handful of tests need to assert behaviour when a Drive table is absent —
//! the maintenance sweeps must record a *failed* job rather than panic. The
//! obvious way to write that test is `DROP TABLE`, but the integration schema is
//! shared by the whole test binary, so a dropped table strands every test that
//! runs afterwards (and forces the fixture to replay the baseline DDL).
//!
//! [`TableAbsenceGuard`] renames the table out of the way instead. The table is
//! hidden from the code under test — a rename is as effective as a drop for
//! producing `relation ... does not exist` — but its definition and rows come
//! back when the guard is dropped. Because the guard holds the rename for the
//! duration of a test, and the fixture serializes tests with an advisory lock,
//! no sibling ever observes the renamed table.
//!
//! # Restoration is explicit
//!
//! `Drop` cannot reliably run a restore: `#[tokio::test]` tears its runtime down
//! as the test body returns, and a restore task spawned onto that runtime is
//! never driven. The guard therefore restores the table **before** the runtime
//! can go away by requiring the test to call [`TableAbsenceGuard::restore`].
//!
//! To keep a forgotten `restore()` from poisoning the shared schema, [`hide`]
//! first removes any parked table left behind by an earlier failure. A parked
//! table is only ever created by this guard, so clearing it is always safe.

use sqlx::PgPool;

/// Suffix applied to a parked table name. Only this guard creates such tables.
const PARKED_SUFFIX: &str = "__parked_by_test";

/// Hides a table from the code under test until [`TableAbsenceGuard::restore`].
pub struct TableAbsenceGuard {
    pool: PgPool,
    table: String,
    parked_name: String,
}

impl TableAbsenceGuard {
    /// Renames `table` out of the way, returning a guard that can restore it.
    ///
    /// Any parked table left behind by an earlier failed run is removed first,
    /// so a stale guard cannot make this call fail. `table` is asserted to be a
    /// plain identifier before it is interpolated into DDL.
    pub async fn hide(pool: &PgPool, table: &str) -> Self {
        assert!(
            !table.is_empty()
                && table
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "unexpected table identifier {table}"
        );
        let parked_name = format!("{table}{PARKED_SUFFIX}");
        // Self-heal: a previous run may have panicked before restoring.
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "DROP TABLE IF EXISTS \"{parked_name}\""
        )))
        .execute(pool)
        .await
        .unwrap_or_else(|error| panic!("clear stale parked Drive table {parked_name}: {error}"));
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "ALTER TABLE \"{table}\" RENAME TO \"{parked_name}\""
        )))
        .execute(pool)
        .await
        .unwrap_or_else(|error| panic!("park Drive table {table} for absence test: {error}"));
        Self {
            pool: pool.clone(),
            table: table.to_string(),
            parked_name,
        }
    }

    /// Restores the table. Call this before the test body returns.
    pub async fn restore(self) {
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "ALTER TABLE \"{}\" RENAME TO \"{}\"",
            self.parked_name, self.table
        )))
        .execute(&self.pool)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "restore Drive table {} after absence test: {error}",
                self.table
            )
        });
    }
}
