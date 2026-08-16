# Drive Documentation

Documentation for SDKWork Drive follows `../sdkwork-specs/DOCUMENTATION_SPEC.md`.

## Canon

| Document | Path |
| --- | --- |
| Product PRD | [product/prd/PRD.md](product/prd/PRD.md) |
| Technical architecture | [architecture/tech/TECH_ARCHITECTURE.md](architecture/tech/TECH_ARCHITECTURE.md) |
| Website Space PRD | [product/prd/PRD-website-space-publishing.md](product/prd/PRD-website-space-publishing.md) |
| Website directory provider | [architecture/tech/TECH-website-directory-resource-provider.md](architecture/tech/TECH-website-directory-resource-provider.md) |
| Production operations | [runbooks/drive-production-operations.md](runbooks/drive-production-operations.md) |
| Backup and DR | [runbooks/drive-backup-disaster-recovery.md](runbooks/drive-backup-disaster-recovery.md) |

## Standards (TECH shards)

- [TECH-drive-iam-integration-standard.md](architecture/tech/TECH-drive-iam-integration-standard.md)
- [TECH-drive-sdk-integration-standard.md](architecture/tech/TECH-drive-sdk-integration-standard.md)
- [TECH-drive-topology-standard.md](architecture/tech/TECH-drive-topology-standard.md)
- [TECH-drive-uploader-standard.md](architecture/tech/TECH-drive-uploader-standard.md)
- [TECH-drive-sibling-naming-standard.md](architecture/tech/TECH-drive-sibling-naming-standard.md)
- [TECH-drive-observability-event-dictionary.md](architecture/tech/TECH-drive-observability-event-dictionary.md)
- [TECH-database-architecture.md](architecture/tech/TECH-database-architecture.md)
- [TECH-storage-s3-architecture.md](architecture/tech/TECH-storage-s3-architecture.md)
- [TECH-website-directory-resource-provider.md](architecture/tech/TECH-website-directory-resource-provider.md)

## Requirements

- [REQ-2026-0001-production-readiness.md](product/requirements/REQ-2026-0001-production-readiness.md)
- [REQ-2026-0002-production-security-alignment.md](product/requirements/REQ-2026-0002-production-security-alignment.md)
- [REQ-2026-0003-pre-launch-debt-cleanup.md](product/requirements/REQ-2026-0003-pre-launch-debt-cleanup.md)
- [REQ-2026-0004-website-space-directory-publishing.md](product/requirements/REQ-2026-0004-website-space-directory-publishing.md)

## Evidence and operator guides

| Document | Path |
| --- | --- |
| Evidence and review | [reviews/FULL_CODE_REVIEW_REPORT.md](reviews/FULL_CODE_REVIEW_REPORT.md) |
| Controlled pilot | [guides/operator/pilot-deployment.md](guides/operator/pilot-deployment.md) |
| Pre-launch checklist (GA) | [guides/operator/pre-launch-checklist.md](guides/operator/pre-launch-checklist.md) |
| Operator guide index | [guides/operator/README.md](guides/operator/README.md) |
| Releases and GA promotion | [releases/README.md](releases/README.md) |
| Changelog index | [changelogs/README.md](changelogs/README.md) |

## Release maturity

Drive is **Beta / DRAFT** for catalog publication. Code alignment is complete for controlled pilot (atomic space lifecycle, pagination, dual-engine parity, PostgreSQL CI). Commercial GA requires signed artifacts, CDN catalog media, immutable K8s digests, and staging smoke evidence — see [releases/README.md](releases/README.md) and [reviews/FULL_CODE_REVIEW_REPORT.md](reviews/FULL_CODE_REVIEW_REPORT.md).

## Verification

- `pnpm verify` (includes `check:app-composition`)
- `pnpm check` — full merge gate (includes `test:staging-admin-smoke-contract`)
- `pnpm smoke:staging-admin` — staging admin API smoke (requires `SDKWORK_DRIVE_STAGING_*` secrets)
- `pnpm api:envelope:check`
- `pnpm api:schema:check`
- `pnpm deploy:validate`
- `pnpm api:assembly:validate`
- `pnpm check:docs-standard`

Legacy root-level filenames under `docs/` redirect to the TECH shards above. Do not add new content outside the canon paths.
