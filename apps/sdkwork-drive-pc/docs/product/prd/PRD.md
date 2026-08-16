# SDKWork Drive PC PRD

Status: active
Owner: SDKWork maintainers
Application: sdkwork-drive-pc
Updated: 2026-06-25
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md, APP_PC_ARCHITECTURE_SPEC.md, DESKTOP_APP_ARCHITECTURE_SPEC.md

Repository-wide product scope and GA phases live in [../../../../docs/product/prd/PRD.md](../../../../docs/product/prd/PRD.md). This document narrows the PC client surface only.

## 1. Background And Problem

End users need a browser and desktop file workspace that consumes Drive through generated SDKs and appbase IAM without raw HTTP, manual auth headers, or renderer-local business APIs. The PC application root must stay thin while feature packages deliver file browsing, transfers, settings, admin storage provider management, and backend operations administration.

## 2. Target Users

- **End users**: Browse, upload, download, share, preview, and manage transfers in browser or Tauri desktop.
- **Tenant administrators**: Configure storage providers through the admin storage UI backed by `@sdkwork/drive-admin-storage-sdk`, and manage audit, maintenance, quotas, labels, spaces, and download packages through `@sdkwork/drive-backend-sdk`.
- **Embedding hosts**: Consume Drive packages with `integrationMode="host-managed"` without mounting the standalone shell.

## 3. Goals And Non-Goals

### Goals

- AuthGate-protected product routes with appbase IAM session bridge and TokenManager-owned SDK credentials.
- SDK-backed file browser with server-side sort, list/grid virtualization, i18n, and accessible error surfaces.
- Transfer center and download manager with localized interruption messages and retry flows.
- Tauri desktop host with OS secure session storage and bundle metadata aligned to release manifest.
- Architecture contract tests enforcing package, SDK, and host boundaries.

### Non-Goals

- Desktop sync client with delta sync (repository P4).
- Renderer-local file data modes or sample data when the backend is unavailable.
- Raw `fetch`, manual authorization headers, or generated SDK edits inside feature packages.

## 4. Scope

| Package | Responsibility |
| --- | --- |
| `src/` | Bootstrap, providers, AuthGate wiring, root composition |
| `sdkwork-drive-pc-core` | Runtime config, session, SDK clients, service facades |
| `sdkwork-drive-pc-file` | File browser, download manager, Drive page |
| `sdkwork-drive-pc-transfer` | Transfer center UI |
| `sdkwork-drive-pc-commons` | Shared UI, i18n, preferences |
| `sdkwork-drive-pc-desktop` | Tauri shell, commands, permissions |
| `sdkwork-drive-pc-admin-storage-providers` | Storage provider and binding admin UI |
| `sdkwork-drive-pc-admin-operations` | Backend operations admin UI (audit, maintenance, quotas, labels, spaces, download packages) |

## 5. User Scenarios

1. User signs in through appbase IAM, browses a space with server-sorted pages, and uploads through Drive uploader APIs.
2. User monitors active transfers, reads localized failure reasons, and retries interrupted uploads or downloads.
3. Administrator registers an S3-compatible provider and binds it to a space through the admin panel.
4. Administrator reviews audit events, runs maintenance sweeps, sets tenant quota caps, and inspects labels, spaces, and download packages through backend admin routes.
5. Desktop user persists session tokens in OS secure storage and opens completed downloads through the Tauri host.

## 6. Success Metrics

| Metric | Target |
| --- | --- |
| Architecture contract tests | `pnpm test` passes in `apps/sdkwork-drive-pc` |
| SDK boundary | No raw HTTP or manual auth headers in feature packages |
| UX completeness | Settings, transfer errors, and file browser sort/virtualization covered by tests |
| Release alignment | Follow repository [releases/GA checklist](../../../../docs/releases/README.md) before ACTIVE publish |

## 7. Phases

| Phase | Outcome | Status |
| --- | --- | --- |
| P1 Core PC shell | AuthGate, SDK file service, transfer UI, desktop host | Done |
| P2 UX hardening | i18n, virtualization, server sort, interruption messages | Done |
| P2.5 Admin operations | Backend admin UI for audit, maintenance, quotas, labels, spaces, download packages | Done |
| P3 Release | Signed desktop artifacts and catalog media | Blocked on platform signing/CDN (repository P3) |

## 8. Linked Requirements

- [REQ-2026-0001-production-readiness.md](../../../../docs/product/requirements/REQ-2026-0001-production-readiness.md)
- [REQ-2026-0002-production-security-alignment.md](../../../../docs/product/requirements/REQ-2026-0002-production-security-alignment.md)
- [REQ-2026-0003-pre-launch-debt-cleanup.md](../../../../docs/product/requirements/REQ-2026-0003-pre-launch-debt-cleanup.md)

## 9. Open Questions

- macOS/Linux store publication timeline beyond Windows x64 (repository release manifest).
