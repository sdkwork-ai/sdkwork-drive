# RELEASE v0.1.0

Status: draft
Channel: STABLE
Release maturity: BETA
Updated: 2026-06-25

## Summary

Initial SDKWork Drive release with contract-first Rust APIs, PC browser/desktop client, generated SDK families, and standalone/cloud deployment templates.

## Highlights

- Drive spaces, nodes, uploader, download grants, share links, quotas, and install-worker maintenance
- IAM dual-token protected routers with backend personal-session rejection
- Sanitized internal HTTP error responses
- PC transfer center with download pause/resume and secure bootstrap failure rendering

## GA Blockers (remaining)

- Artifact signing credentials and signed desktop/web packages
- Catalog media CDN upload (remove placeholder metadata)
- macOS/Linux desktop checksum evidence materialization
- Kubernetes immutable digest replacement from release evidence

See [Commercial GA rollout plan](../engineering/plans/PLAN-2026-0625-commercial-ga.md) for the operator execution sequence.

## Verification Evidence

- `pnpm verify`
- `SDKWORK_RELEASE_VALIDATION=strict pnpm check:release-readiness`
- `cargo test -p sdkwork-routes-drive-backend-api --test iam_auth_guard`
- `cargo test -p sdkwork-routes-storage-backend-api --test iam_auth_guard`
