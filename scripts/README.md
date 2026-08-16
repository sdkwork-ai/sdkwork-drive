# Drive Scripts

`scripts/` contains thin command entrypoints for SDKWork Drive. It is the
script boundary governed by `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md` and
`../sdkwork-specs/PNPM_SCRIPT_SPEC.md`.

## Allowed Content

- Thin command entrypoints that delegate to tools, crates, or packages.
- Standard command dispatcher (`sdkwork-command.mjs`).
- Lifecycle scripts for dev, build, test, and deployment workflows.

## Forbidden Content

- Reusable logic (belongs in `tools/` or a proper package/crate).
- Runtime state, databases, logs, or caches.

## Related Specs

- `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`
- `../sdkwork-specs/PNPM_SCRIPT_SPEC.md`

## Verification

- `pnpm check:pnpm-script-standard` (validate package script standardization)
