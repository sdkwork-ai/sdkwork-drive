# Drive Tools

`tools/` contains deterministic developer, validation, and generation tools for
SDKWork Drive. It is the tooling boundary governed by
`../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`.

## Current Tools

- `drive_sdk_generate.mjs`: SDK generation pipeline entrypoint. Validates
  OpenAPI contracts and runs `sdkgen` to produce SDK family workspaces under
  `sdks/`. Supports `--check` for contract-only validation and `--language`
  for selective generation.

## Allowed Content

- Node.js scripts for SDK generation, validation, and developer workflows.
- Deterministic code generation tools.
- Contract validation tools.

## Forbidden Content

- Thin command entrypoints (those live in `scripts/`).
- Runtime state, databases, logs, or caches.
- Manual one-off scripts without deterministic behavior.

## Related Specs

- `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`
- `../sdkwork-specs/SDK_SPEC.md`
- `../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`

## Verification

- `pnpm sdk:check` (runs `node tools/drive_sdk_generate.mjs --check`)
