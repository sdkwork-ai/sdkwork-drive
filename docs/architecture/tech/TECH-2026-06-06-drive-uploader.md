> Owner: SDKWork maintainers

# Drive Uploader Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build Drive Uploader end to end across the workspace Rust service, App API route crate, App SDK `client.uploader.*`, resumable upload metadata, retention cleanup metadata, and PC integration.

**Architecture:** `sdkwork-drive-workspace-service` owns the uploader business core and persistence. `sdkwork-routes-drive-app-api` exposes that core as `/app/v3/api/drive/uploader/*`. `sdkwork-drive-app-sdk` exposes high-level `client.uploader.*` methods backed by generated App API operations and composed TypeScript orchestration.

**Tech Stack:** Rust, sqlx, SQLite/Postgres schema, Axum App API, OpenAPI, SDKWork generated SDKs, TypeScript composed SDK layer, Vite React PC app.

---

### Task 1: Standard And Schema Contracts

**Files:**
- Create: `docs/drive-uploader-standard.md`
- Create: `docs/architecture/tech/TECH-2026-06-06-drive-uploader.md`
- Modify: `crates/sdkwork-drive-workspace-service/tests/sqlite_schema_contract.rs`
- Modify: `crates/sdkwork-drive-workspace-service/tests/postgres_schema_contract.rs`
- Modify: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/sqlite_core.sql`
- Modify: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/postgres_core.sql`

- [ ] Add schema tests requiring `dr_drive_upload_item`, `dr_drive_upload_part`, and `dr_drive_file_sensitive_operation`.
- [ ] Run the schema test and verify it fails before implementation.
- [ ] Add the three tables and indexes to SQLite and Postgres schema files.
- [ ] Run schema tests and verify they pass.

### Task 2: Product Uploader Domain And Store

**Files:**
- Create: `crates/sdkwork-drive-workspace-service/src/domain/uploader.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/ports/uploader_store.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/uploader_store.rs`
- Modify: `crates/sdkwork-drive-workspace-service/src/domain/mod.rs`
- Modify: `crates/sdkwork-drive-workspace-service/src/ports/mod.rs`
- Modify: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/mod.rs`
- Test: `crates/sdkwork-drive-workspace-service/tests/uploader_service.rs`

- [ ] Add failing tests for creating an upload item and recording uploaded parts idempotently.
- [ ] Implement domain types for profile, actor, retention, task, and part.
- [ ] Implement SQL store insert/get/list operations for upload items and parts.
- [ ] Verify product uploader store tests pass.

### Task 3: Product Uploader Service

**Files:**
- Create: `crates/sdkwork-drive-workspace-service/src/application/uploader_service.rs`
- Modify: `crates/sdkwork-drive-workspace-service/src/application/mod.rs`
- Modify: `crates/sdkwork-drive-workspace-service/src/lib.rs`
- Test: `crates/sdkwork-drive-workspace-service/tests/uploader_service.rs`

- [ ] Add failing tests proving anonymous upload resolves `app_upload` ownership metadata and video helper selects the `video` profile.
- [ ] Add failing tests proving temporary retention sets expiration and cleanup action.
- [ ] Implement `DriveUploaderService` command validation and task creation.
- [ ] Add public `sdkwork_drive_workspace_service::uploader` re-export.
- [ ] Verify uploader service tests pass.

### Task 4: Maintenance Cleanup Metadata

**Files:**
- Modify: `crates/sdkwork-drive-workspace-service/src/ports/maintenance_store.rs`
- Modify: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/maintenance_store.rs`
- Modify: `crates/sdkwork-drive-workspace-service/src/application/maintenance_service.rs`
- Modify: `crates/sdkwork-drive-workspace-service/tests/maintenance_service.rs`

- [ ] Add failing tests for `expired_upload_content_sweep` soft-delete metadata and sensitive file operation creation.
- [ ] Extend maintenance job type validation.
- [ ] Implement expired upload content sweep.
- [ ] Verify maintenance tests pass.

### Task 5: App API Uploader Routes

**Files:**
- Modify: `crates/sdkwork-routes-drive-app-api/src/lib.rs`
- Modify: `crates/sdkwork-routes-drive-app-api/tests/drive_routes.rs`
- Modify: `crates/sdkwork-routes-drive-app-api/tests/command_routes.rs`
- Modify: `generated/openapi/drive-app-api.openapi.json`

- [ ] Add route contract tests for uploader prepare, resume, part uploaded, complete, abort, get, list, and profiles.
- [ ] Implement App API DTOs and route handlers that delegate to `DriveUploaderService`.
- [ ] Export OpenAPI operations under `uploader.*`.
- [ ] Verify App API tests pass.

### Task 6: App SDK `client.uploader.*`

**Files:**
- Modify: `sdks/sdkwork-drive-app-sdk/sdkwork-drive-app-sdk-typescript/composed/operations.ts`
- Create: `sdks/sdkwork-drive-app-sdk/sdkwork-drive-app-sdk-typescript/composed/uploader/types.ts`
- Create: `sdks/sdkwork-drive-app-sdk/sdkwork-drive-app-sdk-typescript/composed/uploader/uploaderClient.ts`
- Create: `sdks/sdkwork-drive-app-sdk/sdkwork-drive-app-sdk-typescript/composed/uploader/uploadPlanner.ts`
- Create: `sdks/sdkwork-drive-app-sdk/sdkwork-drive-app-sdk-typescript/composed/uploader/uploadStateStore.ts`
- Create: `sdks/sdkwork-drive-app-sdk/sdkwork-drive-app-sdk-typescript/composed/uploader/index.ts`
- Test: `sdks/sdkwork-drive-app-sdk/tests/sdk-family-smoke.test.mjs`

- [ ] Add failing tests proving `client.uploader.upload`, `uploadVideo`, and `uploadImage` are exposed in the composed SDK layer.
- [ ] Implement high-level TypeScript uploader methods.
- [ ] Add resumable task state interfaces and an in-memory state store.
- [ ] Verify SDK smoke tests pass.

### Task 7: PC Integration

**Files:**
- Modify: `apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-core/src/services/driveFileService.ts`
- Modify: `apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-core/src/sdk/driveAppSdkClient.ts`
- Modify: `apps/sdkwork-drive-pc/packages/sdkwork-drive-types/src/transferJobs.ts`
- Modify: `apps/sdkwork-drive-pc/packages/sdkwork-drive-file/src/components/FileBrowser.tsx`
- Modify: `apps/sdkwork-drive-pc/src/__tests__/desktop-architecture.contract.test.ts`

- [ ] Add failing PC contract tests proving uploads use the SDK uploader boundary.
- [ ] Refactor PC upload service to delegate to `client.uploader.upload`.
- [ ] Preserve transfer queue progress and cancellation behavior.
- [ ] Verify PC tests, typecheck, and build pass.

