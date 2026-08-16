# SDKWork Drive PRD

Status: active
Owner: SDKWork maintainers
Application: sdkwork-drive
Updated: 2026-07-23
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md, DRIVE_SPEC.md, DEPLOYMENT_SPEC.md, SECURITY_SPEC.md

## Document Map

- [PRD-website-space-publishing.md](PRD-website-space-publishing.md) - active Website Space and
  WebsiteRoot publication contract; live Space-root/folder-root Provider and atomic sync/rollback/
  cleanup lifecycle are implemented, while complete user/admin UI, a real bundle pilot, quota and
  metering reconciliation, and commercial acceptance evidence remain.
- [REQ-2026-0001-production-readiness.md](../requirements/REQ-2026-0001-production-readiness.md)
- [REQ-2026-0002-production-security-alignment.md](../requirements/REQ-2026-0002-production-security-alignment.md)
- [REQ-2026-0003-pre-launch-debt-cleanup.md](../requirements/REQ-2026-0003-pre-launch-debt-cleanup.md)
- [REQ-2026-0004-website-space-directory-publishing.md](../requirements/REQ-2026-0004-website-space-directory-publishing.md)

## 1. Background And Problem

Organizations need a contract-first cloud drive that works in SDKWork SaaS, customer VPC, and standalone desktop/browser packages without forking upload, IAM, or storage logic. Existing ad hoc file APIs leak storage credentials, break SDK generation, and cannot scale operationally across split cloud services.

SDKWork Drive must provide a professional file workspace with metadata/object separation, generated SDK surfaces, appbase IAM integration, and deployment profiles that stay API-compatible between `standalone` and `cloud`.

## 2. Target Users

- **End users**: Upload, browse, share, preview, and download files through the PC/browser client.
- **Tenant administrators**: Configure storage providers, tenant quota caps, maintenance sweeps, and audit review; manage labels, inspect spaces, and monitor download packages through backend APIs and the PC admin UI.
- **Platform operators**: Deploy split cloud services or standalone gateway bundles with observability and release evidence.
- **Application integrators**: Consume Drive through generated app/backend/open SDK families without raw HTTP or local presign tables.

## 3. Goals And Non-Goals

### Goals

- Deliver Drive spaces, nodes, upload sessions, uploader, download grants, share links, trash, quotas, and watch/outbox change delivery.
- Keep all protected APIs on dual-token IAM with RFC 9457 problem responses.
- Support PostgreSQL production and SQLite development with governed database lifecycle assets.
- Ship browser and desktop clients that consume generated SDKs only.
- Publish release artifacts with checksums, SBOM, and deployment descriptors suitable for commercial distribution.
- Provide explicit `website` Spaces whose default Space-root or selected-folder WebsiteRoots can be
  served through Deploy/Web Server without exposing ordinary Spaces.

### Non-Goals (current release)

- Desktop sync client with delta sync comparable to Google Drive desktop.
- Online collaborative document editing.
- Full DLP/eDiscovery product workflows.
- Azure Blob storage adapter until a concrete provider exists.
- Public domains, TLS, Variants, Mounts, or HTTP delivery policy; these belong to Deploy/Web Server.

## 4. Scope

| Surface | In scope |
|---------|----------|
| Rust backend routers | app-api, backend-api, open-api, admin storage-api, install-worker, standalone-gateway |
| PC client | sdkwork-drive-pc browser + Tauri desktop |
| SDK families | sdkwork-drive-app-sdk, backend SDK, open SDK |
| Storage backends | local filesystem, S3-compatible, OpenDAL providers |
| Deployment | standalone unified gateway; cloud split services on Kubernetes |
| Website source provider | implemented Website Space, `SPACE_ROOT`/`FOLDER` live WebsiteRoots, generated Internal SDK Provider, and App API/generated SDK atomic sync, rollback, retention, and cleanup lifecycle |

Legacy `/app/v3/api/assets/upload*` routes remain unavailable; global assets must use Drive uploader APIs per DRIVE_SPEC.

## 5. User Scenarios

1. **Authenticated upload**: User signs in through appbase IAM, creates an upload session or uploader item, completes multipart upload, and sees the file in their space.
2. **Share link access**: Anonymous recipient resolves a share link through open-api, optionally enters an access code, and receives a short-lived download grant.
3. **Admin storage setup**: Tenant admin registers an S3-compatible provider and binds it to a space through admin storage APIs and the PC admin UI.
4. **Admin operations**: Tenant admin reviews audit logs, runs maintenance sweeps, sets tenant quota caps, and inspects labels, spaces, and download packages through backend APIs and the PC admin UI.
5. **Cloud operations**: Operator deploys split services with `/readyz` probes, install-worker maintenance, Redis-backed global rate limits, ingress edge limits, and immutable release digests.
6. **Desktop secure session**: Desktop client persists auth/session tokens in OS secure storage via Tauri keychain commands.
7. **Website directory publication**: User creates a Website Space -> uses the default whole-Space
   root or selects a descendant folder -> uploads or atomically synchronizes a tree -> connects the
   stable WebsiteRoot to a Deploy Site -> opens the bound domain without changing Drive hierarchy.

## 6. Success Metrics

| Metric | Target (GA) |
|--------|----------------|
| API contract drift | `pnpm api:check` and `pnpm sdk:check` pass on main |
| Verification pipeline | `pnpm verify` pass on main |
| Release readiness | strict release check passes after real multi-platform artifacts |
| Staging smoke | weekly open-api + optional PC share-claim smoke against staging secrets |
| Upload/download success | integration and e2e suites pass for share-link and uploader flows |
| Production startup | production profile rejects missing download-token signing secrets |
| Website root confinement | Space-root/folder-root, reserved namespace, and cross-tenant suites pass |

## 7. Phases

| Phase | Outcome | Status |
|-------|---------|--------|
| P0 Core backend | Spaces, nodes, upload/download, permissions, storage abstraction | Done |
| P1 PC client | AuthGate, SDK-backed file browser, transfer center, desktop host | Done |
| P2 Production hardening | Outbox singleton, readyz, K8s spec alignment, secure desktop storage, CSP, IAM DB resolver wiring | Done |
| P3 Release & ops | Signed multi-platform artifacts, catalog media, staging smoke schedule, ACTIVE publish | In progress - code alignment complete; remaining gates are artifact signing, Catalog CDN, and staging operations evidence |
| P4 Differentiation | Delta/changes API, Website Space UI/pilot/operations certification, knowledge/AI profiles, and storage-provider expansion | In progress - live WebsiteRoot Provider and atomic sync lifecycle implemented; remaining UI, pilot, production, and commercial evidence are open |

## 8. Linked Requirements

- [REQ-2026-0001-production-readiness.md](../requirements/REQ-2026-0001-production-readiness.md)
- [REQ-2026-0002-production-security-alignment.md](../requirements/REQ-2026-0002-production-security-alignment.md)
- [REQ-2026-0003-pre-launch-debt-cleanup.md](../requirements/REQ-2026-0003-pre-launch-debt-cleanup.md)
- [REQ-2026-0004-website-space-directory-publishing.md](../requirements/REQ-2026-0004-website-space-directory-publishing.md)

## 9. Open Questions

- Exact timeline for enabling artifact signing in CI (`signatureRequired`).
- Minimum supported desktop platforms for macOS/Linux store publication beyond Windows x64.
