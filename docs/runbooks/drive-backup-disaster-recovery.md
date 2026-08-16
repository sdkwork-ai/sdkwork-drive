# SDKWork Drive Backup and Disaster Recovery

Status: active
Owner: SDKWork maintainers
Updated: 2026-06-24
Specs: DEPLOYMENT_SPEC.md, RELEASE_SPEC.md, DATABASE_FRAMEWORK_SPEC.md

## 1. Scope

This runbook covers PostgreSQL metadata, S3-compatible object storage, and release rollback for SDKWork Drive cloud and standalone deployments.

## 2. Recovery Objectives

| Asset | RPO target | RTO target |
| --- | --- | --- |
| PostgreSQL metadata | 15 minutes (PITR) | 60 minutes |
| Object storage blobs | Near-zero with versioning | 30 minutes |
| Application release | Last signed artifact | 30 minutes |

## 3. PostgreSQL

1. Enable continuous backup / PITR for the Drive database cluster.
2. Store backups in a separate region/account from production.
3. Quarterly restore drill:
   - Restore to an isolated cluster.
   - Run `pnpm db:status` and `pnpm db:drift:check` against the restored instance.
   - Smoke `/readyz` on app-api and open-api.
4. Migration rollback: apply paired `down.sql` from `database/migrations/postgres/` only after operator review.

## 4. Object Storage

1. Enable bucket versioning on production storage providers.
2. For multi-region resilience, configure cross-region replication on the primary bucket.
3. Never delete production buckets without a 30-day retention hold policy.

## 5. Release Rollback

1. Keep the previous container digest / binary artifact for every production promotion.
2. Roll back Kubernetes deployments to the prior immutable digest recorded in the release manifest.
3. Re-run `SDKWORK_RELEASE_VALIDATION=strict pnpm release:validate` before re-promoting.

## 6. Outbox and Maintenance

1. Cloud API pods: set `SDKWORK_DRIVE_DOMAIN_OUTBOX_EMBEDDED_DISPATCH=false`.
2. Run exactly one install-worker replica (or rely on PostgreSQL advisory lock leader election).
3. After failover, verify `drive_domain_outbox_pending_total` returns to baseline.

## 7. Verification Checklist

- [ ] PostgreSQL backup job succeeded in the last 24 hours
- [ ] Object storage versioning enabled
- [ ] Prior release artifact retained
- [ ] `/readyz` green on all split services
- [ ] Staging smoke workflow green when secrets configured
