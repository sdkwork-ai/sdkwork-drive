> Owner: SDKWork maintainers

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Drive-owned logical node versions and version policy tables so Notes can reference Drive versions without duplicating revision storage.

**Architecture:** Keep `dr_drive_storage_object` as the concrete byte fact and add `dr_drive_node_version` as the stable logical version fact. Existing App API version routes remain compatible and are backed by logical versions when available, with storage-object fallback during migration.

**Tech Stack:** Rust 2021, SQLx Any/Postgres/SQLite, Axum route handlers, SDKWork Drive schema registry YAML.

---

### Task 1: Schema Contract

**Files:**
- Modify: `docs/schema-registry/tables/003-drive-storage.yaml`
- Modify: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/sqlite_core.sql`
- Modify: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/postgres_core.sql`
- Modify: `crates/sdkwork-drive-workspace-service/tests/sqlite_schema_contract.rs`
- Modify: `crates/sdkwork-drive-workspace-service/tests/postgres_schema_contract.rs`

- [ ] Add failing SQLite schema tests for `dr_drive_node_version`, `dr_drive_space_version_policy`, and `dr_drive_node_version_policy`.
- [ ] Add failing PostgreSQL schema contract checks for the same tables and index names.
- [ ] Add DDL and schema-registry entries for the three tables.
- [ ] Run `cargo test -p sdkwork-drive-workspace-service sqlite_schema_contract`.

### Task 2: Product Version Store

**Files:**
- Create: `crates/sdkwork-drive-workspace-service/src/domain/node_version.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/ports/node_version_store.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/node_version_store.rs`
- Modify: `crates/sdkwork-drive-workspace-service/src/domain/mod.rs`
- Modify: `crates/sdkwork-drive-workspace-service/src/ports/mod.rs`
- Modify: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/mod.rs`
- Test: `crates/sdkwork-drive-workspace-service/tests/node_version_store.rs`

- [ ] Write failing tests for inserting a logical node version from a storage object and listing it by node.
- [ ] Implement the domain model, store trait, and SQL store.
- [ ] Keep errors mapped through `DriveServiceError`.
- [ ] Run `cargo test -p sdkwork-drive-workspace-service node_version_store`.

### Task 3: App API Route Compatibility

**Files:**
- Modify: `crates/sdkwork-routes-drive-app-api/src/dto.rs`
- Modify: `crates/sdkwork-routes-drive-app-api/src/mappers.rs`
- Modify: `crates/sdkwork-routes-drive-app-api/src/routes.rs`
- Test: `crates/sdkwork-routes-drive-app-api/tests/drive_routes.rs` or `command_routes.rs`

- [ ] Add failing route tests proving version ids come from `dr_drive_node_version` when present.
- [ ] Insert logical versions during upload completion and archive extraction flows.
- [ ] Update list/get/delete/restore version handlers to read logical versions first and keep storage-object fallback.
- [ ] Run `cargo test -p sdkwork-routes-drive-app-api version`.

### Task 4: Verification And Notes Handoff

**Files:**
- Review: `generated/openapi/drive-app-api.openapi.json`
- Review: `sdkwork-notes/docs/schema-registry/tables/001-notes-core.yaml`
- Review: `sdkwork-notes/generated/openapi/notes-app-api.openapi.json`

- [ ] Confirm no generated SDK transport output was hand-edited.
- [ ] Confirm Notes contract references Drive version ids instead of a local revision table.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run the narrowest passing Rust tests, then `cargo test --workspace` if time and dependencies allow.

