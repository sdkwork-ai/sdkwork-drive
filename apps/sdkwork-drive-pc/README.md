# SDKWork Drive PC

SDKWork Drive PC is the renderer application and desktop composition layer for the Drive desktop client. It follows `DESKTOP_APP_ARCHITECTURE_SPEC.md`: the root app stays thin, product UI lives in feature packages, remote business access is routed through the generated Drive App SDK wrapper, and native capabilities are isolated in a Tauri desktop package.

## Documentation

- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)
- Repository docs: [../../docs/README.md](../../docs/README.md)

## Package Shape

```text
apps/sdkwork-drive-pc/
  src/
    App.tsx
    bootstrap/createDrivePcRuntime.ts
  packages/
    sdkwork-drive-pc-core/       runtime config, session, SDK wrapper, host adapter, service facades
    sdkwork-drive-pc-desktop/    Tauri shell, commands, permissions, bundle metadata
    sdkwork-drive-pc-file/       file feature UI backed by injected DriveFileService
    sdkwork-drive-pc-commons/    shared UI components and i18n
    sdkwork-drive-pc-transfer/   transfer feature UI
    sdkwork-drive-pc-types/      compatibility re-export for core-owned Drive UI/domain types
    sdkwork-drive-pc-admin-core/ admin SDK wrappers (storage + backend)
    sdkwork-drive-pc-admin-storage-providers/ storage provider admin UI
    sdkwork-drive-pc-admin-operations/ backend operations admin UI
```

## Runtime Boundary

- `src/App.tsx` only composes the runtime provider and feature pages.
- `sdkwork-drive-pc-core` owns runtime config, session persistence, generated SDK access, typed host adapters, and service facade creation.
- Feature packages receive services through props/context and do not import raw Tauri APIs, raw `fetch`, or manual authorization headers.
- `sdkwork-drive-pc-desktop` owns Tauri CLI, Rust commands, permissions, capabilities, icons, and bundle metadata.

## IAM Login Boundary

- Drive mounts protected product pages behind `DriveAuthGate`.
- Anonymous visits to product routes redirect to `/auth/login?redirect=<target>`.
- `/auth/*` is an appbase IAM route host. Drive does not implement login, refresh, logout, current-session, or `/app/v3/api/auth/*` endpoints.
- The appbase IAM runtime owns session state, refresh, logout, current user, and the global TokenManager. Drive SDK clients are constructed in runtime/bootstrap and receive credentials through that TokenManager rather than feature packages assembling auth headers.
- When appbase writes or clears session state, `DriveAuthGate` refreshes the persisted session before rendering protected content.
- Embedding applications that already own IAM should consume Drive packages directly and either omit `DriveAuthGate` or pass `integrationMode="host-managed"`. They should not mount `apps/sdkwork-drive-pc/src/App.tsx`, because that file is the standalone shell.
- Drive core and feature packages do not depend on `@sdkwork/auth-pc-react` or `@sdkwork/appbase-pc-react`; those packages belong to the top-level host shell.

Detailed standalone versus embeddable rules are documented in [../../docs/architecture/tech/TECH-drive-iam-integration-standard.md](../../docs/architecture/tech/TECH-drive-iam-integration-standard.md).

## Configuration

Copy `.env.example` to `.env.local` for local development.

Runtime configuration separates lifecycle environment, config profile, build mode, deployment profile, and runtime target:

- `VITE_DRIVE_PC_ENVIRONMENT`: `development`, `test`, `staging`, or `production`.
- `VITE_DRIVE_PC_CONFIG_PROFILE`: `dev`, `test`, `staging`, or `prod`.
- `VITE_DRIVE_PC_BUILD_MODE`: build-tool mode, normally `development`, `test`, `staging`, or `production`.
- `VITE_DRIVE_PC_DEPLOYMENT_PROFILE`: `standalone` or `cloud`. Standalone development topologies use localhost gateway defaults; IAM identity still comes from dual-token JWT claims after login.
- `VITE_DRIVE_PC_RUNTIME_TARGET`: `browser`, `desktop`, `tablet-ipados`, `tablet-android`, `server`, `container`, or `test-runner`.

SDK base URLs are configured per surface and per dependency:

- `VITE_DRIVE_PC_APP_API_BASE_URL`: default Drive app-api URL for the PC runtime.
- `VITE_DRIVE_PC_BACKEND_API_BASE_URL`: default backend/admin URL for the PC runtime.
- `VITE_DRIVE_PC_APPBASE_APP_API_BASE_URL`: appbase app SDK URL when it differs from the Drive app-api URL.
- `VITE_DRIVE_PC_DRIVE_APP_API_BASE_URL`: Drive app SDK URL.
- `VITE_DRIVE_PC_DRIVE_ADMIN_STORAGE_API_BASE_URL`: Drive admin storage SDK URL.

Local defaults use `http://127.0.0.1:18080` for Drive app-api and
`http://127.0.0.1:18083` for Drive admin storage. Release-safe defaults use
`https://drive.sdkwork.com/app/v3/api` and
`https://drive.sdkwork.com/backend/v3/api`; private deployments override the
specific `VITE_DRIVE_PC_*_BASE_URL` value they need.

The file service calls `sdkwork-drive-pc-core` `DriveFileService` in every
runtime mode. `DriveFileService` wraps the stable `@sdkwork/drive-app-sdk`
facade, so file listing, Drive Uploader composition, download grants, archive
packages, metadata updates, restore, delete, and space operations all require a
real SDKWork IAM session and a Drive App API reachable at
`VITE_DRIVE_PC_DRIVE_APP_API_BASE_URL` or `VITE_DRIVE_PC_APP_API_BASE_URL`.

Storage provider, bucket, object, and default binding management use the
stable `@sdkwork/drive-admin-storage-sdk` facade through the shared PC core
wrapper. Those management operations require the Admin Storage API at
`VITE_DRIVE_PC_DRIVE_ADMIN_STORAGE_API_BASE_URL`.

Tenant operations administration (audit logs, maintenance sweeps, quota policy,
labels, spaces, and download packages) uses `@sdkwork/drive-backend-sdk` through
`sdkwork-drive-pc-admin-operations` and `sdkwork-drive-pc-admin-core`. Configure
the backend API base URL with `VITE_DRIVE_PC_BACKEND_API_BASE_URL` (defaults to the
admin storage gateway when unset).

There is no renderer-local file data mode. When the backend is unavailable, the
renderer should surface the real App SDK error instead of replacing Drive data
with local samples.

## Commands

Install dependencies:

```powershell
pnpm --dir apps/sdkwork-drive-pc install
```

Run the web renderer:

```powershell
pnpm dev
```

The renderer dev server uses `127.0.0.1:5183` with `--strictPort` because the Tauri shell `devUrl` points to the same port. Stop any other service on 5183 before running `pnpm dev` or `pnpm dev:desktop`.

Start the Drive services from the repository root before opening the renderer:

```powershell
pnpm dev
```

Run desktop development shell:

```powershell
pnpm dev:desktop
```

Build renderer assets:

```powershell
pnpm --dir apps/sdkwork-drive-pc build
```

Build a local debug desktop bundle:

```powershell
pnpm --dir apps/sdkwork-drive-pc build:desktop:local
```

## Verification

```powershell
pnpm --dir apps/sdkwork-drive-pc test
pnpm --dir apps/sdkwork-drive-pc typecheck
pnpm --dir apps/sdkwork-drive-pc build
cargo check --manifest-path apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-desktop/src-tauri/Cargo.toml
pnpm --dir apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-desktop exec tauri info
```

The architecture contract test checks the package boundary, SDK boundary, host boundary, Tauri config, release-safe runtime defaults, and that generated SDK output is not edited by the PC app.

## SDKWork Documentation Contract

Domain: drive
Capability: pc
Package type: react-package
Status: standard

### Public API

Public exports are declared in `specs/component.spec.json` under `contracts.publicExports`.

### Required SDK Surface

Required SDK dependencies are declared in `specs/component.spec.json` under
`contracts.sdkDependencies`, including appbase app SDK, Drive app SDK, Drive
admin storage SDK, and Drive backend SDK dependencies.

### Configuration

Configuration keys and runtime entrypoints are declared in `specs/component.spec.json`.

### SaaS/Private/Local Behavior

This module follows the canonical standards linked from `specs/component.spec.json`, including deployment and runtime configuration rules where applicable.

### Security

Do not add secrets, live tokens, manual auth headers, or app-local credential handling to this module.

### Extension Points

Extension points are limited to declared public exports, runtime entrypoints, SDK clients, events, and config keys.

### Verification

- `pnpm typecheck`

### Owner And Status

Owner and lifecycle status are tracked in `specs/component.spec.json`.
