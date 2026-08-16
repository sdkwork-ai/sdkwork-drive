# SDKWork Drive Pre-Launch Checklist

Use this checklist before promoting Drive from controlled pilot to commercial GA. All items marked **required** must pass; **recommended** items should pass unless explicitly waived with owner sign-off.

## Security and IAM

- [ ] **required** Protected routers (`app-api`, `backend-api`, `open-api`, `admin-storage-api`) run with `IamWebRequestContextResolver` via `wrap_router_with_web_framework_from_env`.
- [ ] **required** IAM signing keys and database credentials are mounted from secrets (`sdkwork-drive-iam`), not plaintext env files.
- [ ] **required** Webhook outbox dispatch uses `validate_webhook_https_url_for_dispatch` (HTTPS, DNS resolution, no private IP targets).
- [ ] **required** Install worker health endpoint binds to loopback (`127.0.0.1`) in production unless an explicit override is documented.
- [ ] **required** Upload content policy mode is `enforce` on app-api in production (`SDKWORK_DRIVE_UPLOAD_CONTENT_POLICY_MODE`).
- [ ] **required** Backend and admin-storage APIs reject personal IAM sessions (`login_scope=TENANT`); only organization-scoped tokens are accepted.
- [ ] **required** HTTP 500 responses do not leak SQL, stack traces, or internal error strings to clients (generic problem detail only).

## Deployment and Observability

- [ ] **required** All systemd units set `SDKWORK_DRIVE_DEPLOYMENT_PROFILE` (`cloud` or `standalone`) and `SDKWORK_DRIVE_RUNTIME_PROFILE=production`.
- [ ] **required** Prometheus scrapes `/metrics`; dashboards alert on error rate, latency (`sdkwork_drive_http_request_duration_seconds`), rate-limit saturation, and route-level counters (`drive_http_requests_by_route_total`).
- [ ] **required** OTEL exporter endpoint and service names are configured per deployment block in `deployments/kubernetes/drive-services.yaml` or equivalent env files.
- [ ] **required** Kubernetes image references use immutable `@sha256:<64 hex>` digests; `SDKWORK_DEPLOY_VALIDATION=strict pnpm deploy:validate` passes.
- [ ] **required** Cloud API Deployments set `SDKWORK_DRIVE_RATE_LIMIT_BACKEND=redis`, source `SDKWORK_DRIVE_RATE_LIMIT_REDIS_URL` from `sdkwork-drive-rate-limit`, and use `SDKWORK_DRIVE_RATE_LIMIT_FAIL_CLOSED=true`.
- [ ] **recommended** Edge rate limiting is active (nginx `limit-rps` or Ingress annotation) as an additional protection layer.

## Release and Catalog

- [ ] **required** Artifact signing is enabled and release CI produces signed binaries for all shipped platforms.
- [ ] **required** Desktop release checksums are real (not placeholders) for Windows, macOS, and Linux where those platforms ship.
- [ ] **required** Catalog media assets are uploaded to CDN; `generatedPlaceholder` is false in release metadata.
- [ ] **required** `sdkwork.app.config.json` `publish.status` is `ACTIVE` only after signing and catalog gates pass.

## Verification Commands

Run from repository root before go-live:

```bash
pnpm check
pnpm verify
pnpm api:envelope:check
pnpm api:schema:check
pnpm deploy:validate
SDKWORK_DEPLOY_VALIDATION=strict pnpm deploy:validate
pnpm api:assembly:validate
pnpm check:architecture-alignment
node ../sdkwork-specs/tools/check-deploy-standard.mjs
```

## Admin Operations Smoke (required)

With a tenant admin session (`drive.storage.admin`, `drive.*`, or a granular `drive.<capability>.admin` scope) against a staging or pre-production backend:

Automated helper (skips when staging secrets are unset; contract validated in CI via `pnpm test:staging-admin-smoke-contract`):

```bash
export SDKWORK_DRIVE_STAGING_BACKEND_BASE_URL="https://<staging-host>/backend/v3/api/drive"
export SDKWORK_DRIVE_STAGING_AUTH_TOKEN="<org-scoped-auth-jwt>"
export SDKWORK_DRIVE_STAGING_ACCESS_TOKEN="<org-scoped-access-jwt>"
# Optional RBAC pairs:
# SDKWORK_DRIVE_STAGING_AUDIT_AUTH_TOKEN / SDKWORK_DRIVE_STAGING_AUDIT_ACCESS_TOKEN
# SDKWORK_DRIVE_STAGING_QUOTA_AUTH_TOKEN / SDKWORK_DRIVE_STAGING_QUOTA_ACCESS_TOKEN
pnpm smoke:staging-admin
```

Manual / scripted checklist:

- [ ] **required** Audit-only role (`drive.audit.admin`) can list audit events but receives `403` on quota, label mutation, and storage provider routes.
- [ ] **required** Quota-only role (`drive.quota.admin`) can read/update quotas but receives `403` on audit and storage provider routes.
- [ ] **required** Run maintenance object sweep in dry-run mode and confirm a completed job record.
- [ ] **required** Read quota summary, set a tenant quota cap (`PUT /backend/v3/api/drive/quotas`), then clear the policy.
- [ ] **required** Create, update, and delete a label through backend admin routes.
- [ ] **required** List tenant spaces and download packages from backend admin routes.
- [ ] **required** PC admin UI surfaces (audit, maintenance, quotas, labels, spaces, download packages) load without raw HTTP or manual auth headers.

## Related Documents

- [Controlled pilot deployment](./pilot-deployment.md) — run before this GA checklist
- [Production operations runbook](../../runbooks/drive-production-operations.md)
- [Audit investigation runbook](../../runbooks/drive-audit-investigation.md)
- [Backup and disaster recovery runbook](../../runbooks/drive-backup-disaster-recovery.md)
- [REQ-2026-0002 Production security alignment](../../product/requirements/REQ-2026-0002-production-security-alignment.md)
- [REQ-2026-0003 Pre-launch technical debt cleanup](../../product/requirements/REQ-2026-0003-pre-launch-debt-cleanup.md)
