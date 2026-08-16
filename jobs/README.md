# Drive Jobs

`jobs/` owns schedules, queues, batch definitions, and runbooks for
SDKWork Drive background work. It is the job boundary governed by
`../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`.

Rust worker implementations belong in
`crates/sdkwork-drive-<capability>-worker/` (for example
`crates/sdkwork-drive-install-worker/`). This directory holds job schedule
definitions, queue manifests, and operational runbooks that reference those
workers.

## Allowed Content

- Schedule definitions (cron expressions, interval configs).
- Queue manifests and batch job descriptors.
- Operational runbooks for recurring maintenance jobs.

## Forbidden Content

- Rust worker source code (belongs in `crates/`).
- Runtime state, databases, logs, or caches.

## Related Specs

- `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`
- `../sdkwork-specs/DRIVE_SPEC.md`

## Verification

- `pnpm verify` (repository-wide verification)
