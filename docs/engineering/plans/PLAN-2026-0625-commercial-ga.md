# Commercial GA Rollout Plan

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-06

## Objective

Promote SDKWork Drive from controlled Beta to commercial GA with signed artifacts, CDN catalog media, and production deployment evidence.

## Prerequisites (code — done)

- IAM dual-token validation on protected routers
- Backend/admin personal-session rejection (`login_scope=TENANT`)
- Sanitized HTTP 500 responses
- PC download pause/resume and bootstrap XSS hardening
- `pnpm verify` and strict release-readiness checks passing locally
- App-api route modularization Phases 1–7 complete; Phase 8 batch 8a delegates space lifecycle and change-feed SQL to workspace-service (`space_lifecycle_service`, `change_feed_service`)
- **2026-07-06 alignment:** PAGINATION_SPEC default `page_size=20`; unified table-based maintenance leader (PG/SQLite); Redis rate-limit feature compile + Lua atomic window; outbox per-channel delivery ledger (migration 0005); upload session metadata reclaim; transactional orphan cleanup; SQLite baseline regenerated from `sqlite_core.sql`; PostgreSQL CI integration job

## Release gates (ops — required before ACTIVE)

| Gate | Owner | Evidence |
| --- | --- | --- |
| Artifact signing | Release / security | CI produces signed web + desktop packages; `signatureDeferred: false` |
| Cross-platform desktop | Release CI | Real checksums for Windows, macOS, Linux bundles |
| Catalog media CDN | Product / release | Upload icon, screenshots, preview; clear `generatedPlaceholder` |
| Kubernetes digest | Platform ops | Replace `REPLACE_WITH_RELEASE_DIGEST` from release evidence |
| Staging E2E | QA / platform | Mandatory workflow with staging secrets configured |
| Publish status | Product | Set `sdkwork.app.config.json` `publish.status` to `ACTIVE` only after all gates pass |

## Execution sequence

1. Run `pnpm verify` and `SDKWORK_RELEASE_VALIDATION=strict pnpm check:release-readiness` on release branch.
2. Materialize release evidence (`pnpm test:release-evidence`) on protected CI runners.
3. Upload catalog media and regenerate catalog evidence.
4. Deploy staging with IAM secrets and run staging E2E workflows.
5. Promote immutable digest to production Kubernetes manifest.
6. Flip catalog publish status to `ACTIVE` and announce GA.

## Related documents

- [Pre-launch checklist](../../guides/operator/pre-launch-checklist.md)
- [RELEASE v0.1.0](../../releases/RELEASE-v0.1.0.md)
- [Production operations runbook](../../runbooks/drive-production-operations.md)
