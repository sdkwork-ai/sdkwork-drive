# sdkwork-drive
repository-kind: application

Rust backend workspace for SDKWork Drive.

This repository is scaffolded for contract-first and TDD-first delivery.

## Backend Scope

- Rust modular backend services and reusable crates.
- App API, backend API, open API, and admin storage API OpenAPI contracts.
- SDK family generation skeleton (`sdks/sdkwork-drive-sdk`, `sdks/sdkwork-drive-app-sdk`, `sdks/sdkwork-drive-backend-sdk`, `sdks/sdkwork-drive-admin-storage-sdk`).
- S3-compatible object-storage abstraction with extension-ready provider boundary.
- PostgreSQL-first database configuration with SQLite local/private mode.

## SDKWork Standard Project Root

This repository is a SDKWork standard project root governed by
`../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`.

- `apis/` contains Drive-owned API contract sources and materialized OpenAPI inputs.
- `apps/` contains runnable Drive application roots such as the PC/Tauri app.
- `crates/` contains Rust service crates, route crates, workers, host/server crates, and reusable Rust libraries.
- `database/` contains database lifecycle assets: schema registry, DDL baselines, migrations, drift policy, seeds, and fixtures.
- `sdks/` contains SDK family workspaces and generated SDK output.
- `jobs/` is reserved for independently packaged Drive workers and scheduled jobs.
- `tools/` contains deterministic developer, validation, and generation tools.
- `plugins/` is reserved for Drive application/runtime plugin source.
- `examples/` is reserved for runnable examples and SDK/API usage samples.
- `etc/` contains safe checked-in source config templates and topology profiles.
- `deployments/` contains deployment descriptors.
- `scripts/` contains thin command entrypoints.
- `docs/` contains architecture notes, runbooks, and standards docs.
- `tests/` is reserved for cross-package tests and shared fixtures.
- `.sdkwork/` contains repository/application development metadata, local skills, and local agent plugins.

`apis/` and `sdks/` are separate boundaries: API contracts and materialized
OpenAPI inputs live under `apis/`; SDK family metadata, generation manifests,
generated language workspaces, and generated transport output live under
`sdks/`.

## Database Development Modes

```bash
pnpm dev --database postgres          # PostgreSQL profile (default)
pnpm topology:plan                    # Render the local launch plan without starting services
```

Database policy and current runtime boundaries are documented in
[docs/architecture/tech/TECH-database-architecture.md](docs/architecture/tech/TECH-database-architecture.md).

The local backend launch plan starts the four runtime API services together:

- App API route crate: `sdkwork-routes-drive-app-api` on `127.0.0.1:18080`.
- Backend API route crate: `sdkwork-routes-drive-backend-api` on `127.0.0.1:18081`.
- Open API route crate: `sdkwork-routes-drive-open-api` on `127.0.0.1:18082`.
- Storage backend route crate: `sdkwork-routes-storage-backend-api` on `127.0.0.1:18083`.

Runtime services also accept `SDKWORK_DRIVE_CONFIG_FILE` pointing at a TOML file
with a `[database]` section, for example `etc/drive.database.example.toml`.
`SDKWORK_DATABASE_URL` remains the highest-priority override.

## IAM Login Integration

Drive does not implement application-local login, refresh, logout, or current-session
routes. Login/session UX and transport are owned by SDKWork appbase; Drive
consumes the resulting SDKWork dual-token session. AppContext is derived server-side from
verified token claims, not from client projection headers.
Drive-specific standalone and embeddable IAM boundaries are documented in
[docs/architecture/tech/TECH-drive-iam-integration-standard.md](docs/architecture/tech/TECH-drive-iam-integration-standard.md).

Protected app, backend, and admin storage routes require:

- `Authorization: Bearer <authToken>`
- `Access-Token: <accessToken>`

Tenant, user, actor, and permission context are derived from verified dual-token claims at the
service boundary. Clients must not send `x-sdkwork-tenant-id`, `x-sdkwork-user-id`,
`x-sdkwork-actor-id`, `x-sdkwork-actor-kind`, or other AppContext projection headers.

Open API share-link routes and admin storage `/healthz` remain explicitly public and do not require
IAM credentials.

When Drive is embedded into another application that already owns IAM, the host
must consume Drive packages in host-managed mode and must not mount the
standalone Drive app shell. The host owns `/auth/*`, refresh, logout, and appbase
runtime; Drive only consumes the host-provided SDKWork session projection.

## Core Verification

```bash
pnpm check
pnpm verify
cargo test -p sdkwork-drive-contract
cargo test -p sdkwork-drive-workspace-service
cargo test -p sdkwork-routes-drive-app-api
cargo test -p sdkwork-routes-drive-backend-api
```

## OpenAPI And SDK Tooling

SDK family naming and app integration rules are documented in
[docs/architecture/tech/TECH-drive-sdk-integration-standard.md](docs/architecture/tech/TECH-drive-sdk-integration-standard.md). The canonical SDK families are
`sdkwork-drive-sdk`, `sdkwork-drive-app-sdk`,
`sdkwork-drive-backend-sdk`, and `sdkwork-drive-admin-storage-sdk`.
OpenAPI authority names still use `sdkwork-drive-open-api`,
`sdkwork-drive-app-api`, and `sdkwork-drive-backend-api`; the dedicated storage
administration authority uses `sdkwork-drive-admin-storage-api`. Runtime Rust
route crates use the standard `sdkwork-routes-<capability>-<surface>` package
family under `crates/`.

Check contracts and schema quality without running SDK generation:

```bash
node tools/drive_sdk_generate.mjs --check
```

Check with explicit OpenAPI inputs (useful for CI workspace variants):

```bash
node tools/drive_sdk_generate.mjs --check \
  --app-input apis/app-api/drive/drive-app-api.openapi.json \
  --backend-input apis/backend-api/drive/drive-backend-api.openapi.json \
  --admin-storage-input apis/backend-api/drive/drive-admin-storage-api.openapi.json
```

Run the generation pipeline after installing the native workspace dependencies.
By default Drive resolves the generator at
`../sdkwork-sdk-generator/bin/sdkgen.js`; `SDKWORK_SDK_GENERATOR_BIN`
is only for an explicit nonstandard local override and must still point at that
canonical generator checkout.

```bash
node tools/drive_sdk_generate.mjs
```

Generate selected language SDKs only (example: Rust):

```bash
node tools/drive_sdk_generate.mjs --language rust
```

## SDKWork Documentation Contract

Domain: drive
Capability: workspace
Package type: app
Status: standard

### Public API

Public exports are declared in `specs/component.spec.json` under `contracts.publicExports`.

### Required SDK Surface

- None declared in `specs/component.spec.json`.

### Configuration

Configuration keys and runtime entrypoints are declared in `specs/component.spec.json`.

### SaaS/Private/Local Behavior

This module follows the canonical standards linked from `specs/component.spec.json`, including deployment and runtime configuration rules where applicable.

### Security

Do not add secrets, live tokens, manual auth headers, or app-local credential handling to this module.

### Extension Points

Extension points are limited to declared public exports, runtime entrypoints, SDK clients, events, and config keys.

### Verification

- `pnpm check`
- `pnpm verify`
- `pnpm test`

### Owner And Status

Owner and lifecycle status are tracked in `specs/component.spec.json`.

## Documentation Canon

- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)

## Application Roots

- [apps directory index](apps/README.md)
