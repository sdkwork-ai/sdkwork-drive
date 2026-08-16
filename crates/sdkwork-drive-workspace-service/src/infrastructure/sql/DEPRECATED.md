# Deprecated runtime SQL entrypoints

`postgres_core.sql` remains as the PostgreSQL schema contract used by focused installer tests.

Production bootstrap MUST use `sdkwork-drive-database-host` via `bootstrap_drive_database()` instead of calling `install_postgres_schema()` directly.

Canonical baseline: `database/ddl/baseline/postgres/0001_drive_baseline.sql`.
