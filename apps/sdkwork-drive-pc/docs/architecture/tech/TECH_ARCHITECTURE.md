# SDKWork Drive PC Technical Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-06-25
Specs: ARCHITECTURE_DECISION_SPEC.md, APP_PC_ARCHITECTURE_SPEC.md, DESKTOP_APP_ARCHITECTURE_SPEC.md, APP_SDK_INTEGRATION_SPEC.md

Platform-wide backend topology is documented in [../../../../docs/architecture/tech/TECH_ARCHITECTURE.md](../../../../docs/architecture/tech/TECH_ARCHITECTURE.md). This entry covers the PC renderer and desktop host only.

## 1. Architecture Overview

`sdkwork-drive-pc` is a thin React application root with feature packages. Bootstrap constructs appbase IAM runtime, TokenManager, and generated Drive SDK clients. Feature UI receives typed services through context/props and never constructs raw HTTP or manual auth headers.

## 2. Technology Choices

| Layer | Choice |
| --- | --- |
| Renderer | React, Vite, Vitest |
| Desktop host | Tauri 2 (`sdkwork-drive-pc-desktop`) |
| SDKs | `@sdkwork/drive-app-sdk`, `@sdkwork/drive-admin-storage-sdk`, `@sdkwork/drive-backend-sdk`, appbase auth runtime |
| i18n | `sdkwork-drive-pc-commons` LanguageProvider |
| Session | Browser storage adapter or Tauri OS secure storage |

## 3. System Boundaries And Modules

| Module | Owns | Must not own |
| --- | --- | --- |
| `src/main.tsx` | Global providers, error boundary shell | Feature business UI |
| `createDrivePcRuntime.ts` | SDK clients, TokenManager, service facades | Raw Tauri APIs in features |
| `sdkwork-drive-pc-file` | File browser, download manager | SDK construction |
| `sdkwork-drive-pc-transfer` | Transfer center UI | Upload/download protocol logic |
| `sdkwork-drive-pc-desktop` | Tauri commands, permissions, bundle | Drive business rules |
| `sdkwork-drive-pc-admin-core` | Admin storage + backend SDK clients | Feature admin UI |
| `sdkwork-drive-pc-admin-storage-providers` | Storage provider/binding admin UI | SDK construction |
| `sdkwork-drive-pc-admin-operations` | Audit, maintenance, quota, labels, spaces, download packages admin UI | Raw HTTP or manual auth headers |

## 4. Directory And Package Layout

```text
apps/sdkwork-drive-pc/
  src/                 bootstrap + root composition
  packages/
    sdkwork-drive-pc-core/
    sdkwork-drive-pc-file/
    sdkwork-drive-pc-transfer/
    sdkwork-drive-pc-commons/
    sdkwork-drive-pc-desktop/
    sdkwork-drive-pc-admin-storage-providers/
    sdkwork-drive-pc-admin-operations/
  docs/                application Canon (this tree)
  specs/               component.spec.json contracts
```

Contract tests in `src/__tests__/desktop-architecture.contract.test.ts` enforce these boundaries.

## 5. API, SDK, And Data Ownership

- File operations use `DriveFileService` → generated Drive app SDK facade.
- Admin storage operations use the admin storage SDK wrapper in `sdkwork-drive-pc-admin-core`.
- Backend operations admin (audit, maintenance, quotas, labels, spaces, download packages) uses `runtime.admin.backend` → `@sdkwork/drive-backend-sdk` through `sdkwork-drive-pc-admin-operations`.
- IAM login, refresh, and logout are owned by appbase auth runtime; Drive exposes `DriveAuthGate` only.
- Server-side list sort uses `nodes.list` `sortBy`/`sortOrder`; client fallback applies only for non-server-sort sections.

## 6. Security, Privacy, And Observability

- Desktop session tokens persist through Tauri secure storage commands.
- Browser preferences use injected storage from `src/bootstrap/browserPreferenceStorage.ts`.
- CSP and production runtime defaults follow repository security requirements.
- No secrets, live tokens, or manual credential headers in renderer packages.

## 7. Deployment And Runtime Topology

- **Browser**: Vite dev server on `127.0.0.1:5183`; production static assets from app manifest build targets.
- **Desktop**: Tauri bundles Windows/macOS/Linux artifacts declared in `sdkwork.app.config.json`.
- Runtime config keys use `VITE_DRIVE_PC_*` surfaces documented in application `README.md`.

## 8. Architecture Decision Index

| Topic | Reference |
| --- | --- |
| IAM integration | [TECH-drive-iam-integration-standard.md](../../../../docs/architecture/tech/TECH-drive-iam-integration-standard.md) |
| SDK integration | [TECH-drive-sdk-integration-standard.md](../../../../docs/architecture/tech/TECH-drive-sdk-integration-standard.md) |
| Uploader flows | [TECH-drive-uploader-standard.md](../../../../docs/architecture/tech/TECH-drive-uploader-standard.md) |

## 9. Verification

```bash
pnpm --dir apps/sdkwork-drive-pc test
pnpm --dir apps/sdkwork-drive-pc typecheck
pnpm check:pc-standard
node ../../../sdkwork-specs/tools/check-repository-docs-standard.mjs --root apps/sdkwork-drive-pc
```

Repository-wide gates: `pnpm verify` from `sdkwork-drive` root.
