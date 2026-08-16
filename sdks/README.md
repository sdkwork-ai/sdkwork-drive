# Drive SDK Families

`sdks/` contains SDKWork Drive SDK family workspaces, SDK generation manifests,
composed facades, and generated SDK artifacts. It is the SDK boundary governed by
`../sdkwork-specs/SDK_SPEC.md` and
`../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`.

## Canonical SDK Families

- `sdkwork-drive-sdk/`: Drive open-api SDK family (generated from
  `apis/open-api/drive/drive-open-api.openapi.json`).
- `sdkwork-drive-app-sdk/`: Drive app-api SDK family (generated from
  `apis/app-api/drive/drive-app-api.openapi.json`).
- `sdkwork-drive-backend-sdk/`: Drive backend-api SDK family (generated from
  `apis/backend-api/drive/drive-backend-api.openapi.json`).
- `sdkwork-drive-admin-storage-sdk/`: Drive admin storage SDK family (generated
  from `apis/backend-api/drive/drive-admin-storage-api.openapi.json`).

## Generation

SDK generation is owned by the repository root and uses the canonical
`@sdkwork/sdk-generator` / `sdkgen` toolchain. Generated transport output lives
under each SDK family directory; do not hand-edit generated files.

## Allowed Content

- SDK family directories with `sdk-manifest.json` manifests.
- Generated language workspaces (e.g., `sdkwork-drive-app-sdk-typescript/`).
- Composed facade packages that wrap generated clients.
- SDK generation manifests and derived OpenAPI inputs.

## Forbidden Content

- Authored API contracts (those live under `apis/`).
- Runtime state, databases, logs, caches, or secrets.
- Manual backups or local-only artifacts.

## Related Specs

- `../sdkwork-specs/SDK_SPEC.md`
- `../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`
- `../sdkwork-specs/API_SPEC.md`

## Verification

- `pnpm sdk:check` (validate SDK generation manifests and contract alignment)
- `pnpm sdk:generate` (run SDK generation pipeline)
