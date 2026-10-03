# sdkwork-drive-pc-admin-storage-buckets

Domain: drive
Capability: storage-buckets
Package type: PC internal admin React package
Surface: backend-admin
Status: standard

This package owns the Drive storage bucket browser on the backend-admin surface.
It reads the buckets of a storage provider account through the shared
`sdkwork-drive-pc-admin-storage-providers` service (which owns the generated
Drive admin storage SDK calls) and manages each bucket's objects in a dialog
with content classification on the left and the object list on the right.

Bucket selection travels on the request (`bucket`), because the operator browses
the buckets that exist on the provider account rather than only the one named in
the provider configuration; endpoint, region, and credentials still come from
that configuration. Reads and writes are bounded by the API's 8 MiB object
content limit and say so instead of failing silently.

## Public API

- `.`: the storage bucket admin page.

## Required SDK Surface

- `@sdkwork/drive-admin-storage-sdk` (consumed through `sdkwork-drive-pc-admin-core`)

## Verification

- `pnpm typecheck`
- `pnpm test`

## Related Specs

- `../../../../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md`
- `../../../../sdkwork-specs/BACKEND_UI_SPEC.md`
- `../../../../sdkwork-specs/DRIVE_SPEC.md`
