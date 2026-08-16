> Owner: SDKWork maintainers

# SDKWork Drive Database Architecture

SDKWork Drive uses PostgreSQL for authoritative server persistence.

- PostgreSQL is the server, Docker, Kubernetes, and production target.
- SQLite is limited to client-local data and test fixtures; it is not a Drive gateway/server profile.
- Schema Registry, PostgreSQL DDL, SQLite DDL, OpenAPI, and SDK contracts must stay synchronized.
- Health and dry-run output may expose only safe database facts: configured flag, engine, and max connection count. It must not expose URLs, usernames, passwords, hosts, file paths, or query strings.

## Runtime Configuration

Root package scripts define the local development modes:

- `pnpm dev --database postgres` resolves PostgreSQL configuration.
- `pnpm topology:plan` renders the same launch plan without starting services.

Public application-server development starts the app, backend, open, and admin storage API surfaces with PostgreSQL. SQLite remains available to internal persistence tests and migration compatibility checks, but is not exposed as an application-server development profile.

The Rust configuration boundary is `sdkwork-drive-config::DatabaseConfig`. It accepts:

- Explicit override: `SDKWORK_DATABASE_URL`.
- Runtime TOML file: `SDKWORK_DRIVE_CONFIG_FILE=./etc/drive.database.example.toml`, reading the `[database]` section.
- PostgreSQL structured fields: `SDKWORK_DATABASE_ENGINE=postgresql`, host, port, database name, username, password, `SDKWORK_DATABASE_SSL_MODE`, and max connections.
- Client-local SQLite adapters use `SDKWORK_DATABASE_ENGINE=sqlite` with `SDKWORK_DATABASE_FILE`; Drive server entrypoints reject that profile.

Configuration precedence is explicit URL, runtime TOML, then structured environment fields. `SDKWORK_DATABASE_URL` remains an explicit operator override and wins over TOML and structured fields.

PostgreSQL TOML supports either `url` or structured fields. Structured PostgreSQL config must provide `host`, `database`, `username`, and one of `password` or `password_file`; `password_file` is resolved relative to the TOML file location. Structured values are percent-encoded before building the PostgreSQL URL so service accounts such as `sdkwork` and secret values containing `/`, `+`, spaces, or `@` remain valid connection strings.

Default pool sizing is engine-specific: PostgreSQL defaults to 10 connections, SQLite defaults to 1 connection. SQLite local mode should not be widened unless the caller has a concrete lock-contention plan.

## Runtime Boundary

The workspace service DDL has complete PostgreSQL and SQLite installers:

- `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/postgres_core.sql`
- `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/sqlite_core.sql`

The runtime pool boundary is `sqlx::AnyPool`. App, backend, open, and admin storage API startup all call `build_router_with_database_config`, which connects through `sdkwork-drive-workspace-service::infrastructure::sql::connect_any_database_and_install_schema` and installs the schema selected by `DatabaseConfig::engine()`.

Runtime SQL must use PostgreSQL-compatible `$1`, `$2`, ... bind placeholders. SQLite accepts the same `$NNN` bind names, so handlers and workspace-service stores must not introduce SQLite-only `?1` placeholders. SQLite boolean columns are decoded through explicit helpers because `sqlx::Any` returns SQLite booleans as integer values while PostgreSQL returns native booleans.

Supported runtime database engines are PostgreSQL and SQLite only. Enabling `sqlx::AnyPool` may pull SQLx MySQL internals into Cargo dependency resolution, but Drive configuration rejects MySQL URLs and Drive does not expose MySQL as a supported database backend.

Direct SQL in request handlers is still allowed for narrow API-specific read models, but shared business persistence must be implemented through workspace-service store ports. New SQL must be validated against both PostgreSQL and SQLite semantics before being exposed through `pnpm dev`.

## Lifecycle And Migrations

`database/database.manifest.json` declares both `postgres` and `sqlite` engines. Governed assets live under:

- `database/ddl/baseline/{engine}/0001_drive_baseline.sql` — greenfield snapshot
- `database/migrations/{engine}/0002_*` — outbox pending-dispatch partial index
- `database/migrations/{engine}/0003_*` — tenant quota policy table

PostgreSQL production upgrades run through `pnpm db:migrate` (`sdkwork-database-cli`). SQLite local mode installs `sqlite_core.sql` at runtime and applies incremental boot upgrades for legacy files when required.

Domain outbox claim uses `FOR UPDATE SKIP LOCKED` on PostgreSQL and a single-writer-safe claim on SQLite. Multi-instance cloud deployments must run outbox dispatch from `sdkwork-drive-install-worker` with `SDKWORK_DRIVE_DOMAIN_OUTBOX_EMBEDDED_DISPATCH=false` on API pods.

## Schema Rules

- Every installed table and index must be listed in `docs/schema-registry/tables/*.yaml`.
- SQLite and PostgreSQL DDL must expose the same table and index set.
- Core identity fields use `varchar(64)` logical IDs in the Drive runtime.
- Object bytes live in configured storage providers; database metadata remains the source of truth for Drive nodes, versions, upload sessions, package downloads, permissions, audit, and maintenance history.
- New schema changes must update both DDL files, schema registry, API contracts if exposed, and contract tests in the same change.

