# SDKWork Drive Backend Implementation Plan

> **Historical document.** Implementation completed. Active references: [TECH_ARCHITECTURE.md](../../architecture/tech/TECH_ARCHITECTURE.md), [FULL_CODE_REVIEW_REPORT.md](../../reviews/FULL_CODE_REVIEW_REPORT.md), and shipped crates under `crates/sdkwork-routes-*`.

> Owner: SDKWork maintainers

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a contract-first, TDD-first Rust backend for `sdkwork-drive`, including workspace scaffolding, schema registry, OpenAPI contracts, generated SDK skeletons, S3-compatible storage abstraction, database model, and core app/backend APIs.

**Architecture:** Use a modular monolith aligned with `sdkwork-cloudrouter-router` conventions. Keep clean boundaries between `domain`, `application`, `ports`, and `infrastructure`, expose app and backend routers as reusable components, and enforce storage provider isolation through `DriveObjectStore`.

**Tech Stack:** Rust 2021, `axum`, `tokio`, `sqlx` (PostgreSQL + SQLite), Redis, OpenAPI 3.1.x, SDKWork SDK generator (`--standard-profile sdkwork-v3`), S3-compatible object storage (AWS S3/MinIO first).

---

## Scope And Assumptions

- This plan implements backend only.
- Frontend and `apps/*` runtime product screens are out of scope.
- The design spec source is `docs/architecture/tech/TECH-2026-06-01-sdkwork-drive-backend-design.md`.
- Current workspace is not a git repo yet. Commit steps are still included; they execute after `git init` or after moving into a tracked workspace.

## File Structure Plan

### Workspace Root

- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `README.md`
- Create: `docs/schema-registry/tables/`
- Create: `generated/openapi/`
- Create: `generated/schema/`
- Create: `generated/sdk/`
- Create: `sdks/sdkwork-drive-app-sdk/`
- Create: `sdks/sdkwork-drive-backend-sdk/`
- Create: `tools/`
- Create: `tests/`

### Crates

- Create: `crates/sdkwork-drive-contract/`
- Create: `crates/sdkwork-drive-contract/`
- Create: `crates/sdkwork-drive-config/`
- Create: `crates/sdkwork-drive-http/`
- Create: `crates/sdkwork-drive-security/`
- Create: `crates/sdkwork-drive-observability/`
- Create: `crates/sdkwork-drive-storage-contract/`
- Create: `crates/sdkwork-drive-storage-local/`
- Create: `crates/sdkwork-drive-storage-s3/`
- Create: `crates/sdkwork-drive-test-support/`

### Services

- Create: `crates/sdkwork-drive-workspace-service/`
- Create: `crates/sdkwork-routes-drive-app-api/`
- Create: `crates/sdkwork-routes-drive-backend-api/`
- Create: `crates/sdkwork-drive-install-worker/`

### Specs And Contracts

- Create: `docs/schema-registry/tables/001-drive-core.yaml`
- Create: `docs/schema-registry/tables/002-drive-special-spaces.yaml`
- Create: `docs/schema-registry/tables/003-drive-storage.yaml`
- Create: `docs/schema-registry/tables/004-drive-security-audit.yaml`
- Create: `generated/openapi/drive-app-api.openapi.json`
- Create: `generated/openapi/drive-backend-api.openapi.json`

---

### Task 1: Bootstrap Rust Workspace Skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `README.md`
- Create: `tests/workspace_smoke.rs`

- [ ] **Step 1: Write the failing smoke test**

```rust
#[test]
fn workspace_declares_expected_members() {
    let manifest = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml must exist");
    assert!(manifest.contains("crates/sdkwork-drive-contract"));
    assert!(manifest.contains("crates/sdkwork-drive-workspace-service"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test workspace_smoke`
Expected: FAIL because root manifest and members are not created yet.

- [ ] **Step 3: Add root manifests and workspace members**

```toml
[workspace]
members = [
  "crates/sdkwork-drive-contract",
  "crates/sdkwork-drive-contract",
  "crates/sdkwork-drive-config",
  "crates/sdkwork-drive-http",
  "crates/sdkwork-drive-security",
  "crates/sdkwork-drive-observability",
  "crates/sdkwork-drive-storage-contract",
  "crates/sdkwork-drive-storage-local",
  "crates/sdkwork-drive-storage-s3",
  "crates/sdkwork-drive-test-support",
  "crates/sdkwork-drive-workspace-service",
  "crates/sdkwork-routes-drive-app-api",
  "crates/sdkwork-routes-drive-backend-api",
  "crates/sdkwork-drive-install-worker",
]
resolver = "2"
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test workspace_smoke`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml rust-toolchain.toml README.md tests/workspace_smoke.rs
git commit -m "chore: bootstrap sdkwork-drive workspace skeleton"
```

---

### Task 2: Build Schema Registry Skeleton (TDD First)

**Files:**
- Create: `docs/schema-registry/tables/001-drive-core.yaml`
- Create: `docs/schema-registry/tables/002-drive-special-spaces.yaml`
- Create: `docs/schema-registry/tables/003-drive-storage.yaml`
- Create: `docs/schema-registry/tables/004-drive-security-audit.yaml`
- Create: `tests/schema_registry_smoke.rs`

- [ ] **Step 1: Write the failing schema registry test**

```rust
#[test]
fn schema_registry_includes_special_space_profiles() {
    let doc = std::fs::read_to_string("docs/schema-registry/tables/002-drive-special-spaces.yaml")
        .expect("special spaces schema file missing");
    assert!(doc.contains("dr_drive_space_knowledge_profile"));
    assert!(doc.contains("dr_drive_space_ai_generation_profile"));
    assert!(doc.contains("dr_drive_space_app_upload_profile"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test schema_registry_smoke`
Expected: FAIL because schema registry files do not exist.

- [ ] **Step 3: Add schema registry table contract YAML files**

Use stable table metadata sections:

```yaml
table: dr_drive_space
domain: drive
owner: sdkwork-drive
columns:
  - name: space_type
    type: varchar(32)
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test schema_registry_smoke`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add docs/schema-registry/tables tests/schema_registry_smoke.rs
git commit -m "spec: add drive schema registry baseline"
```

---

### Task 3: Author OpenAPI Skeletons For App And Backend APIs

**Files:**
- Create: `generated/openapi/drive-app-api.openapi.json`
- Create: `generated/openapi/drive-backend-api.openapi.json`
- Create: `tests/openapi_contract_smoke.rs`

- [ ] **Step 1: Write the failing OpenAPI contract test**

```rust
#[test]
fn openapi_paths_follow_sdkwork_v3_prefixes() {
    let app = std::fs::read_to_string("generated/openapi/drive-app-api.openapi.json").unwrap();
    let backend = std::fs::read_to_string("generated/openapi/drive-backend-api.openapi.json").unwrap();
    assert!(app.contains("/app/v3/api/drive/spaces"));
    assert!(backend.contains("/backend/v3/api/drive/quotas"));
    assert!(backend.contains("/backend/v3/api/drive/audit_events"));
    assert!(app.contains("\"operationId\": \"spaces.list\""));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test openapi_contract_smoke`
Expected: FAIL because OpenAPI files are missing.

- [ ] **Step 3: Add minimal valid OpenAPI JSON documents**

Include:
- `openapi: 3.1.0` or `3.1.2` compatible toolchain output.
- app and backend route prefixes.
- dotted resource operationIds.
- problem detail schema.
- auth security scheme placeholders.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test openapi_contract_smoke`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add generated/openapi tests/openapi_contract_smoke.rs
git commit -m "spec: add drive app/backend openapi skeletons"
```

---

### Task 4: Add SDK Family Skeleton And Generator Entrypoints

**Files:**
- Create: `sdks/sdkwork-drive-app-sdk/sdk-manifest.json`
- Create: `sdks/sdkwork-drive-backend-sdk/sdk-manifest.json`
- Create: `sdks/sdkwork-drive-app-sdk/bin/generate-sdk.mjs`
- Create: `sdks/sdkwork-drive-backend-sdk/bin/generate-sdk.mjs`
- Create: `sdks/sdkwork-drive-app-sdk/tests/sdk-family-smoke.test.mjs`
- Create: `sdks/sdkwork-drive-backend-sdk/tests/sdk-family-smoke.test.mjs`
- Create: `tests/sdk_manifest_smoke.rs`

- [ ] **Step 1: Write failing SDK manifest test**

```rust
#[test]
fn sdk_assemblies_use_sdkwork_v3_profile() {
    let app = std::fs::read_to_string("sdks/sdkwork-drive-app-sdk/bin/generate-sdk.mjs").unwrap();
    let backend = std::fs::read_to_string("sdks/sdkwork-drive-backend-sdk/bin/generate-sdk.mjs").unwrap();
    assert!(app.contains("--standard-profile"));
    assert!(backend.contains("sdkwork-v3"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test sdk_manifest_smoke`
Expected: FAIL because SDK generator files are missing.

- [ ] **Step 3: Add SDK family layout and scripts**

Script requirements:
- consume `generated/openapi/*.json`
- call `sdkwork-sdk-generator`
- include `--standard-profile sdkwork-v3`
- generate app and backend TypeScript first
- keep extension points for Rust/Java/Python/Go

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test sdk_manifest_smoke`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add sdks tests/sdk_manifest_smoke.rs
git commit -m "build: scaffold drive sdk generation families"
```

---

### Task 5: Implement `sdkwork-drive-storage-contract` With TDD

**Files:**
- Create: `crates/sdkwork-drive-storage-contract/Cargo.toml`
- Create: `crates/sdkwork-drive-storage-contract/src/lib.rs`
- Create: `crates/sdkwork-drive-storage-contract/src/types.rs`
- Create: `crates/sdkwork-drive-storage-contract/tests/object_store_contract.rs`

- [ ] **Step 1: Write failing trait-level tests**

```rust
use sdkwork_drive_storage_contract::DriveStorageProviderKind;

#[test]
fn provider_kind_includes_s3_compatible() {
    assert_eq!(DriveStorageProviderKind::S3Compatible.as_str(), "s3_compatible");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-drive-storage-contract`
Expected: FAIL because crate and types are not implemented.

- [ ] **Step 3: Add `DriveObjectStore` trait and capability model**

Include:
- multipart upload methods
- presigned upload/download methods
- range stream method
- provider capability flags
- stable error enum with no provider SDK leakage

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-drive-storage-contract`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-drive-storage-contract
git commit -m "feat: add drive object storage contract crate"
```

---

### Task 6: Implement Local Object Store Adapter

**Files:**
- Create: `crates/sdkwork-drive-storage-local/Cargo.toml`
- Create: `crates/sdkwork-drive-storage-local/src/lib.rs`
- Create: `crates/sdkwork-drive-storage-local/src/local_store.rs`
- Create: `crates/sdkwork-drive-storage-local/tests/local_store.rs`

- [ ] **Step 1: Write failing adapter tests**

```rust
#[tokio::test]
async fn local_store_supports_put_head_delete_roundtrip() {
    // create temp store, put object, head object, delete object
    // assert consistent metadata and absence after delete
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-drive-storage-local`
Expected: FAIL because adapter is missing.

- [ ] **Step 3: Implement minimal local adapter to satisfy contract**

Implementation scope:
- filesystem-backed object store
- metadata sidecar for checksum and content-type
- streaming read support
- delete and head behavior

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-drive-storage-local`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-drive-storage-local
git commit -m "feat: implement local object store adapter"
```

---

### Task 7: Implement S3-Compatible Adapter (`sdkwork-drive-storage-s3`)

**Files:**
- Create: `crates/sdkwork-drive-storage-s3/Cargo.toml`
- Create: `crates/sdkwork-drive-storage-s3/src/lib.rs`
- Create: `crates/sdkwork-drive-storage-s3/src/s3_store.rs`
- Create: `crates/sdkwork-drive-storage-s3/src/config.rs`
- Create: `crates/sdkwork-drive-storage-s3/tests/s3_contract.rs`
- Create: `docker-compose.minio-test.yml`

- [ ] **Step 1: Write failing contract tests against MinIO**

```rust
#[tokio::test]
async fn s3_store_supports_multipart_and_presign() {
    // create multipart, presign part, complete upload, presign download, verify stream
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-drive-storage-s3 -- --ignored`
Expected: FAIL until adapter and test setup exist.

- [ ] **Step 3: Implement S3 adapter with compact abstraction boundary**

Requirements:
- endpoint/region/bucket/path-style config
- optional TLS strict mode
- multipart lifecycle
- presigned URL support
- head/delete/range
- stable mapping from SDK errors to contract errors

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test -p sdkwork-drive-storage-s3`
Expected: PASS for unit tests.

Run: `cargo test -p sdkwork-drive-storage-s3 -- --ignored`
Expected: PASS for integration tests when MinIO is up.

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-drive-storage-s3 docker-compose.minio-test.yml
git commit -m "feat: add s3-compatible object store adapter"
```

---

### Task 8: Create `sdkwork-drive-workspace-service` Domain And Port Skeleton

**Files:**
- Create: `crates/sdkwork-drive-workspace-service/Cargo.toml`
- Create: `crates/sdkwork-drive-workspace-service/src/lib.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/domain/mod.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/domain/space.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/domain/node.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/domain/upload.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/ports/mod.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/application/mod.rs`
- Create: `crates/sdkwork-drive-workspace-service/tests/domain_space_rules.rs`

- [ ] **Step 1: Write failing domain tests for space types**

```rust
#[test]
fn space_type_supports_special_spaces() {
    use sdkwork_drive_workspace_service::domain::space::DriveSpaceType;
    assert_eq!(DriveSpaceType::KnowledgeBase.as_str(), "knowledge_base");
    assert_eq!(DriveSpaceType::AiGenerated.as_str(), "ai_generated");
    assert_eq!(DriveSpaceType::GitRepository.as_str(), "git_repository");
    assert_eq!(DriveSpaceType::Deployment.as_str(), "deployment");
    assert_eq!(DriveSpaceType::AppUpload.as_str(), "app_upload");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-drive-workspace-service domain_space_rules`
Expected: FAIL because domain model is not implemented.

- [ ] **Step 3: Implement minimal domain and ports**

Add:
- `DriveSpaceType`
- `DriveNodeType`
- upload session state enum
- core error types
- initial store port traits

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-drive-workspace-service domain_space_rules`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-drive-workspace-service
git commit -m "feat: scaffold drive product domain and ports"
```

---

### Task 9: Implement SQL Installer And Core Tables

**Files:**
- Create: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/mod.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/installer.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/sqlite_core.sql`
- Create: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/postgres_core.sql`
- Create: `crates/sdkwork-drive-workspace-service/tests/sqlite_schema_contract.rs`
- Create: `crates/sdkwork-drive-workspace-service/tests/postgres_schema_contract.rs`

- [ ] **Step 1: Write failing SQL contract tests**

```rust
#[test]
fn sqlite_installer_creates_special_space_profile_tables() {
    // run installer on sqlite and assert table existence:
    // dr_drive_space_knowledge_profile, dr_drive_space_ai_generation_profile, dr_drive_space_app_upload_profile
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-drive-workspace-service sqlite_schema_contract`
Expected: FAIL because installer and tables are not present.

- [ ] **Step 3: Implement installer and DDL scripts**

Requirements:
- create all phase-1 core tables
- include required indexes and unique constraints
- include tenant-first indexing for list-heavy queries
- include id/version/audit lifecycle fields

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test -p sdkwork-drive-workspace-service sqlite_schema_contract`
Expected: PASS.

Run: `cargo test -p sdkwork-drive-workspace-service postgres_schema_contract`
Expected: PASS when postgres test DB is configured.

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-drive-workspace-service/src/infrastructure/sql crates/sdkwork-drive-workspace-service/tests
git commit -m "feat: add drive sql installer and core schema contracts"
```

---

### Task 10: Implement Space And Node Stores + Services

**Files:**
- Create: `crates/sdkwork-drive-workspace-service/src/ports/space_store.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/ports/node_store.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/space_store.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/node_store.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/application/space_service.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/application/node_service.rs`
- Create: `crates/sdkwork-drive-workspace-service/tests/space_service.rs`
- Create: `crates/sdkwork-drive-workspace-service/tests/node_service.rs`

- [ ] **Step 1: Write failing service tests**

```rust
#[tokio::test]
async fn create_space_supports_knowledge_ai_upload_types() {}

#[tokio::test]
async fn create_folder_enforces_live_name_uniqueness_per_parent() {}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-drive-workspace-service space_service node_service`
Expected: FAIL until stores and services exist.

- [ ] **Step 3: Implement SQL stores and application services**

Include:
- create/list/retrieve spaces
- create/list/retrieve nodes
- special-space profile linkage
- parent scope uniqueness checks
- optimistic version checks

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test -p sdkwork-drive-workspace-service space_service node_service`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-drive-workspace-service/src crates/sdkwork-drive-workspace-service/tests
git commit -m "feat: implement drive space and node services"
```

---

### Task 11: Implement Upload Session, Version, And Download Services

**Files:**
- Create: `crates/sdkwork-drive-workspace-service/src/ports/upload_session_store.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/ports/storage_object_store.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/application/upload_service.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/application/download_service.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/upload_session_store.rs`
- Create: `crates/sdkwork-drive-workspace-service/src/infrastructure/sql/storage_object_store.rs`
- Create: `crates/sdkwork-drive-workspace-service/tests/upload_service.rs`
- Create: `crates/sdkwork-drive-workspace-service/tests/download_service.rs`

- [ ] **Step 1: Write failing tests for idempotency and presign**

```rust
#[tokio::test]
async fn create_upload_session_is_idempotent_for_same_key() {}

#[tokio::test]
async fn download_url_is_short_lived_and_hides_object_key() {}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-drive-workspace-service upload_service download_service`
Expected: FAIL.

- [ ] **Step 3: Implement upload/download flows**

Include:
- session creation with quota reservation
- multipart part tracking
- completion with checksum validation
- version and storage object creation
- short-lived presigned download response
- audit event emission points

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test -p sdkwork-drive-workspace-service upload_service download_service`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-drive-workspace-service/src crates/sdkwork-drive-workspace-service/tests
git commit -m "feat: implement upload and download services"
```

---

### Task 12: Implement App API Service And Contract Tests

**Files:**
- Create: `crates/sdkwork-routes-drive-app-api/Cargo.toml`
- Create: `crates/sdkwork-routes-drive-app-api/src/main.rs`
- Create: `crates/sdkwork-routes-drive-app-api/src/lib.rs`
- Create: `crates/sdkwork-routes-drive-app-api/tests/drive_routes.rs`
- Create: `crates/sdkwork-routes-drive-app-api/tests/contract_routes.rs`

- [ ] **Step 1: Write failing app route tests**

```rust
#[tokio::test]
async fn app_router_exposes_dr_drive_space_and_upload_routes() {
    // assert /app/v3/api/drive/spaces and /app/v3/api/drive/upload_sessions exist
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-routes-drive-app-api`
Expected: FAIL because router is not implemented.

- [ ] **Step 3: Implement app router with workspace service injection**

Include:
- health route
- drive routes for spaces/nodes/upload/download/versions/permissions/share/search/changes
- auth context middleware hooks
- problem detail mapper

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test -p sdkwork-routes-drive-app-api`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-routes-drive-app-api
git commit -m "feat: add drive app api router and contract tests"
```

---

### Task 13: Implement Backend API Service And Contract Tests

**Files:**
- Create: `crates/sdkwork-routes-drive-backend-api/Cargo.toml`
- Create: `crates/sdkwork-routes-drive-backend-api/src/main.rs`
- Create: `crates/sdkwork-routes-drive-backend-api/src/lib.rs`
- Create: `crates/sdkwork-routes-drive-backend-api/tests/backend_routes.rs`
- Create: `crates/sdkwork-routes-drive-backend-api/tests/storage_provider_routes.rs`

- [ ] **Step 1: Write failing backend route tests**

```rust
#[tokio::test]
async fn backend_router_exposes_storage_provider_and_quota_routes() {}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-routes-drive-backend-api`
Expected: FAIL because backend router is not implemented.

- [ ] **Step 3: Implement backend router**

Include:
- storage provider CRUD and test route
- space administration routes
- audit and quota routes
- maintenance sweep routes
- admin auth context hooks

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test -p sdkwork-routes-drive-backend-api`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-routes-drive-backend-api
git commit -m "feat: add drive backend api router and admin contracts"
```

---

### Task 14: SDK Generation Pipeline And Verification Gates

**Files:**
- Create: `tools/drive_openapi_export.mjs`
- Create: `tools/drive_sdk_generate.mjs`
- Create: `tools/drive_schema_quality_gate.mjs`
- Create: `tests/sdk_generation_smoke.rs`
- Modify: `README.md`

- [ ] **Step 1: Write failing generation smoke test**

```rust
#[test]
fn sdk_generation_scripts_reference_drive_openapi_inputs() {
    let script = std::fs::read_to_string("tools/drive_sdk_generate.mjs").unwrap();
    assert!(script.contains("generated/openapi/drive-app-api.openapi.json"));
    assert!(script.contains("generated/openapi/drive-backend-api.openapi.json"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test sdk_generation_smoke`
Expected: FAIL because scripts are missing.

- [ ] **Step 3: Implement generation and verification scripts**

Include:
- OpenAPI export from service routers/contracts
- app/backend SDK generation calls
- schema quality checks
- reproducible output locations

- [ ] **Step 4: Run verification commands**

Run: `cargo test --workspace`
Expected: PASS.

Run: `cargo fmt --all -- --check`
Expected: PASS.

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS.

Run: `node tools/drive_sdk_generate.mjs --check`
Expected: PASS with generated artifacts in expected folders.

- [ ] **Step 5: Commit**

```bash
git add tools tests/sdk_generation_smoke.rs README.md
git commit -m "build: add drive openapi and sdk generation verification pipeline"
```

---

## Global Verification Checklist

- [ ] `cargo test --workspace` passes.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] SQLite schema tests pass.
- [ ] PostgreSQL schema contract tests pass when DB URL is provided.
- [ ] S3-compatible integration tests pass against MinIO when enabled.
- [ ] OpenAPI files validate.
- [ ] App and backend SDK generation succeeds with `sdkwork-v3` profile.
- [ ] Generated SDKs expose resource-style methods from dotted operationIds.
- [ ] No API path violates `/app/v3/api` and `/backend/v3/api`.
- [ ] No sensitive fields leak in logs or responses.

## Execution Notes

- Implement in strict TDD order: test fail -> minimal code -> test pass -> refactor -> verify.
- Keep each commit single-purpose and reviewable.
- Do not hand-edit generated SDK outputs except when refreshing generated artifacts through scripts.
- Keep storage provider details inside adapter crates, never in application service logic.
- Keep `knowledge_base`, `ai_generated`, `app`, and `app_upload` as first-class spaces end-to-end across schema, service, API, and SDK.

