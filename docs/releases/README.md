# SDKWork Drive Releases

Release notes and promotion evidence for SDKWork Drive follow `../../sdkwork-specs/RELEASE_SPEC.md` and `../product/prd/PRD.md`.

## Current train

| Channel | Version | Status | Notes |
| --- | --- | --- | --- |
| STABLE | 0.1.0 | BETA / DRAFT | Merge and verification gates pass; GA blocked on signing, catalog CDN, and multi-platform artifact evidence |

## CI release workflow

Package and multi-platform artifacts via GitHub Actions:

1. Open **Actions → Package Application** (`.github/workflows/package.yml`).
2. Run **workflow_dispatch** with platform/architecture filters, or push a semver tag (`v*`).
3. Workflow uses `sdkwork.workflow.json` targets (web, Windows zip, macOS dmg, Linux appimage, container OCI).
4. After CI success, materialize evidence locally or in CI:

```bash
pnpm release:evidence
pnpm release:validate
SDKWORK_DEPLOY_VALIDATION=strict pnpm deploy:validate
SDKWORK_RELEASE_VALIDATION=strict pnpm check:release-readiness
```

Pilot deployment steps: [pilot-deployment.md](../guides/operator/pilot-deployment.md).

## GA promotion checklist

Before setting `publish.status` to `ACTIVE` in `sdkwork.app.config.json`:

1. Run `pnpm verify` and `pnpm deploy:validate` on the release commit.
2. Materialize signed release artifacts for every enabled install package.
3. Upload catalog media to CDN and clear `generatedPlaceholder` metadata.
4. Replace Kubernetes `REPLACE_WITH_RELEASE_DIGEST` placeholders with immutable digests.
5. Run `SDKWORK_DEPLOY_VALIDATION=strict pnpm deploy:validate` and `SDKWORK_RELEASE_VALIDATION=strict node tools/check_sdkwork_drive_release_readiness.mjs` with zero blocking failures.
6. Record staging smoke evidence from `.github/workflows/staging-e2e.yml`.

See [pre-launch-checklist.md](../guides/operator/pre-launch-checklist.md) for the operator-facing checklist.

## Version history

Release notes are recorded in `sdkwork.app.config.json` → `release.notes` until a tagged GA train publishes dedicated notes in this directory.
