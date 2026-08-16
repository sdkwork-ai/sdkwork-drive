# Drive Plugins

`plugins/` is reserved for Drive application/runtime plugin source. It is the
plugin boundary governed by `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`.

## Allowed Content

- Drive application plugin packages.
- Runtime extension plugins.

## Forbidden Content

- Agent plugin bundles (those live under `.sdkwork/plugins/`).
- Runtime state, databases, logs, or caches.

## Related Specs

- `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`
- `../sdkwork-specs/MODULE_SPEC.md`

## Verification

- `pnpm verify` (repository-wide verification)
