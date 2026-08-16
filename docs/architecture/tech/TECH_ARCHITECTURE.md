# SDKWork Drive Technical Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-21
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md, DATABASE_SPEC.md, DRIVE_SPEC.md,
SECURITY_SPEC.md

## Document Map

### Active standards

- [TECH-drive-observability-event-dictionary.md](TECH-drive-observability-event-dictionary.md)
- [TECH-drive-iam-integration-standard.md](TECH-drive-iam-integration-standard.md)
- [TECH-drive-sdk-integration-standard.md](TECH-drive-sdk-integration-standard.md)
- [TECH-drive-sandbox-explorer-standard.md](TECH-drive-sandbox-explorer-standard.md)
- [TECH-drive-sibling-naming-standard.md](TECH-drive-sibling-naming-standard.md)
- [TECH-drive-topology-standard.md](TECH-drive-topology-standard.md)
- [TECH-drive-uploader-standard.md](TECH-drive-uploader-standard.md)
- [TECH-database-architecture.md](TECH-database-architecture.md)
- [TECH-storage-key-layout.md](TECH-storage-key-layout.md)
- [TECH-storage-s3-architecture.md](TECH-storage-s3-architecture.md)

### Feature-specific shards

- [TECH-website-directory-resource-provider.md](TECH-website-directory-resource-provider.md) -
  active Website Space/root Provider architecture for cloud Sites; live directory delivery is
  implemented and atomic sync/UI/operations remain incomplete.
- [TECH-2026-06-06-drive-uploader.md](TECH-2026-06-06-drive-uploader.md)
- [TECH-2026-06-08-drive-node-version-policy-design.md](TECH-2026-06-08-drive-node-version-policy-design.md)
- [TECH-2026-06-08-drive-node-version-policy.md](TECH-2026-06-08-drive-node-version-policy.md)

### Redirect stubs (stable links)

- [TECH-2026-06-01-drive-observability-event-dictionary.md](TECH-2026-06-01-drive-observability-event-dictionary.md) → [TECH-drive-observability-event-dictionary.md](TECH-drive-observability-event-dictionary.md)
- [TECH-2026-06-01-sdkwork-drive-backend-design.md](TECH-2026-06-01-sdkwork-drive-backend-design.md) → [archive](../../archive/tech/TECH-2026-06-01-sdkwork-drive-backend-design.md)
- [TECH-2026-06-01-sdkwork-drive-backend.md](TECH-2026-06-01-sdkwork-drive-backend.md) → [archive](../../archive/tech/TECH-2026-06-01-sdkwork-drive-backend.md)

Historical backend design and implementation plans live under [docs/archive/tech/](../../archive/tech/).

## 1. Architecture Overview

SDKWork Drive is a contract-first file workspace with metadata/object separation. Rust HTTP routers expose app, backend, open, and admin-storage APIs. The PC client consumes generated SDK families only. Production authentication uses appbase IAM dual tokens validated through `IamWebRequestContextResolver`.

Deployment profiles:

- **standalone** — unified gateway embeds split routers plus embedded IAM app-api
- **cloud** — Kubernetes split services with Redis-backed application rate limits, ingress edge limits, and install-worker maintenance

## 2. Technology Choices

| Layer | Choice |
| --- | --- |
| Backend | Rust, Axum, sqlx (PostgreSQL/SQLite) |
| Client | React + Tauri desktop host |
| Contracts | OpenAPI → generated SDK families |
| Auth | appbase IAM dual-token + IAM DB session resolver |
| Object storage | S3-compatible, local filesystem, OpenDAL providers |
| Observability | Prometheus metrics, OTLP traces, structured logs |

## 3. System Boundaries And Modules

| Module | Responsibility |
| --- | --- |
| `sdkwork-routes-drive-app-api` | End-user file workspace HTTP surface |
| `sdkwork-routes-drive-backend-api` | Tenant admin operations: audit, maintenance, quotas, labels, spaces, download packages, sandbox volume/grant configuration |
| `sdkwork-routes-drive-open-api` | Public share-link resolve/download |
| `sdkwork-routes-storage-backend-api` | Storage provider admin APIs |
| `sdkwork-drive-workspace-service` | Domain logic, SQL, outbox, uploader orchestration |
| `sdkwork-drive-sandbox-*` | Authorized server sandbox contracts, policy, provider containment, and audit |
| `@sdkwork/drive-pc-sandbox-explorer` | Reusable browser/Tauri renderer capability for logical sandbox navigation |
| `sdkwork-drive-install-worker` | Maintenance leader, outbox dispatch loop |
| `sdkwork-api-drive-standalone-gateway` | Dev/all-in-one HTTP gateway |
| `apps/sdkwork-drive-pc` | Browser and desktop application shell |

PC application Canon: `apps/sdkwork-drive-pc/docs/product/prd/PRD.md` and `apps/sdkwork-drive-pc/docs/architecture/tech/TECH_ARCHITECTURE.md`.

## 4. Directory And Package Layout

See repository `AGENTS.md` dictionary. Generated SDK output lives under `sdks/` and is modified only through OpenAPI contracts and generator inputs.

## 5. API, SDK, And Data Ownership

- OpenAPI authorities under `apis/` materialize SDK families under `sdks/`.
- Drive-owned upload/download flows use uploader APIs; legacy global asset upload routes fail closed with explicit retired-route semantics.
- PostgreSQL is the production database; SQLite is for development, tests, and single-writer standalone use only.

## 6. Security, Privacy, And Observability

- Protected routers finalize with IAM database session resolution in production assembly paths.
- Backend and admin-storage APIs reject personal IAM sessions (`login_scope=TENANT`); organization-scoped tokens only.
- Backend and admin-storage route guards enforce per-operation capability scopes (`drive.audit.admin`, `drive.quota.admin`, `drive.sandboxes.admin`, and so on), with `drive.storage.admin` and `drive.*` as umbrella grants. IAM should provision least-privilege roles where operators do not require full storage administration.
- Sandbox app/open APIs never expose physical provider roots. Only organization-scoped backend operators with sandbox-admin privilege may receive `providerRootRef`; audit facts and logs remain path-free.
- HTTP 500 responses return generic client-safe problem details; SQL and internal errors are logged server-side only.
- Download tokens require production signing secrets (`SDKWORK_DRIVE_RUNTIME_PROFILE=production`).
- Webhook outbox dispatch validates HTTPS URLs and DNS-resolved addresses before egress.
- Health: `/healthz` liveness, `/readyz` database readiness, `/metrics` Prometheus scrape.
- See [drive-production-operations.md](../../runbooks/drive-production-operations.md) and [REQ-2026-0002-production-security-alignment.md](../../product/requirements/REQ-2026-0002-production-security-alignment.md).

## 7. Deployment And Runtime Topology

- Deploy contract: `deployments/deploy.yaml` per `SDKWORK_DEPLOY_SPEC.md` (validated by `pnpm deploy:validate` and `check-deploy-standard.mjs`)
- Cloud reference manifest: `deployments/kubernetes/drive-services.yaml`
- Container build descriptors: `deployments/docker/`
- Systemd units: `deployments/systemd/`
- Topology contract: `specs/topology.spec.json`

Cloud operators must set `SDKWORK_DRIVE_DOMAIN_OUTBOX_EMBEDDED_DISPATCH=false` on API pods when install-worker is deployed.
Cloud API Deployments must set `SDKWORK_DRIVE_RATE_LIMIT_BACKEND=redis`, source `SDKWORK_DRIVE_RATE_LIMIT_REDIS_URL` from the `sdkwork-drive-rate-limit` secret, and set `SDKWORK_DRIVE_RATE_LIMIT_FAIL_CLOSED=true`; ingress limits remain an edge protection layer.

## 8. Architecture Decision Index

- [ADR-20260721 Website Space Directory Resource](../decisions/ADR-20260721-website-space-directory-resource.md) - accepted multi-project Website Space and atomic root design.

| Topic | Shard / ADR |
| --- | --- |
| IAM integration | [TECH-drive-iam-integration-standard.md](TECH-drive-iam-integration-standard.md) |
| Backend personal-session rejection | [ADR-20260625-backend-personal-session-rejection.md](../../architecture/decisions/ADR-20260625-backend-personal-session-rejection.md) |
| HTTP error sanitization | [ADR-20260625-http-error-sanitization.md](../../architecture/decisions/ADR-20260625-http-error-sanitization.md) |
| App-api route modularization | [ADR-20260625-app-api-route-modularization.md](../../architecture/decisions/ADR-20260625-app-api-route-modularization.md) |
| SDK consumption | [TECH-drive-sdk-integration-standard.md](TECH-drive-sdk-integration-standard.md) |
| Sibling naming and conflict handling | [TECH-drive-sibling-naming-standard.md](TECH-drive-sibling-naming-standard.md) |
| Runtime topology | [TECH-drive-topology-standard.md](TECH-drive-topology-standard.md) |
| Database policy | [TECH-database-architecture.md](TECH-database-architecture.md) |
| Object storage | [TECH-storage-s3-architecture.md](TECH-storage-s3-architecture.md) |

## 9. Verification

- `pnpm verify`
- `pnpm check` (includes `api:envelope:check`, `api:schema:check`, `deploy:validate`, architecture alignment)
- `pnpm api:envelope:check`
- `pnpm api:schema:check`
- `pnpm deploy:validate`
- `pnpm check:architecture-alignment`
- `pnpm check:docs-standard`
- `SDKWORK_RELEASE_VALIDATION=strict pnpm check:release-readiness`
