---
id: REQ-2026-0001
title: Production and commercial readiness for SDKWork Drive
owner: SDKWork maintainers
status: in-progress
source: platform
updated: 2026-07-23
problem: SDKWork Drive must reach production-operable and commercially publishable quality before public GA, without leaving backend, client, release, or deployment technical debt.
goals:
  - All HTTP services expose health/readiness/metrics appropriate to their role.
  - Production startup fails fast when download-token signing or auth policy is misconfigured.
  - Database lifecycle uses baseline-plus-migrations with at least one governed postgres migration after baseline.
  - Install-worker maintenance is safe under multi-replica cloud deployment.
  - Release and deployment evidence align with RELEASE_SPEC and DEPLOYMENT_SPEC before ACTIVE publication.
non_goals:
  - Desktop sync client or collaborative editing in this requirement.
  - Managed Redis service provisioning itself; this requirement consumes a Redis URL from deployment secrets.
users:
  - platform operators
  - tenant administrators
  - end users of sdkwork-drive-pc
acceptance_criteria:
  - `pnpm verify` passes on main.
  - `pnpm deploy:validate` passes.
  - Strict deployment validation rejects placeholder Kubernetes image digests until release evidence replaces them.
  - Cloud Kubernetes API deployments configure Redis-backed rate limiting with fail-closed behavior.
  - Postgres migration 0002 exists with paired up/down SQL under database/migrations/postgres/.
  - install-worker uses postgres advisory lock for maintenance tasks in cloud replicas.
  - staging-e2e workflow runs on schedule and skips gracefully when secrets are absent.
  - PRD status is ready and links this requirement.
non_functional_requirements:
  security: production download-token signing required; browser CSP aligned with SECURITY_SPEC; desktop tokens in OS secure storage.
  privacy: none beyond root standards; no secrets in release artifacts.
  performance: outbox partial index migration; file list content-visibility optimization in PC client; Redis-backed global rate limiting for cloud replicas.
  reliability: singleton outbox dispatcher; /readyz database checks; maintenance leader election for install-worker; fail-closed rate limiting when Redis is unavailable in production cloud.
affected_surfaces:
  - backend
  - pc
  - api
  - sdk
trace:
  specs:
    - REQUIREMENTS_SPEC.md
    - RELEASE_SPEC.md
    - DEPLOYMENT_SPEC.md
    - DATABASE_FRAMEWORK_SPEC.md
    - SECURITY_SPEC.md
  components:
    - crates/sdkwork-drive-workspace-service
    - crates/sdkwork-drive-install-worker
    - apps/sdkwork-drive-pc
verification:
  - pnpm verify
  - pnpm deploy:validate
  - node --test tools/check_drive_deployments.test.mjs
  - pnpm db:validate
  - SDKWORK_RELEASE_VALIDATION=strict node tools/check_sdkwork_drive_release_readiness.mjs
