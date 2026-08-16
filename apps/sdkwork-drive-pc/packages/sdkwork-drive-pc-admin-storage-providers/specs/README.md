# sdkwork-drive-pc-admin-storage-providers

Domain: drive
Capability: storage-providers
Package type: PC internal admin React package
Surface: backend-admin
Status: standard

This package owns internal operator UI services for Drive storage provider
configuration management. It consumes the generated Drive admin storage SDK
through the Drive PC runtime service boundary and must not construct raw HTTP
requests, manual auth headers, provider SDK clients, or generated SDK internals.

## Public API

- `.`: capability binding, service factory, and storage provider summary types.

## Required SDK Surface

- `@sdkwork/drive-admin-storage-sdk` (consumed through `sdkwork-drive-pc-admin-core`)

## Verification

- `pnpm typecheck`
- `pnpm test`

## Related Specs

- `../../../../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md`
- `../../../../sdkwork-specs/BACKEND_UI_SPEC.md`
- `../../../../sdkwork-specs/DRIVE_SPEC.md`
