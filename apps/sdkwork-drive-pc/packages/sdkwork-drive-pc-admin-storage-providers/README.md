# sdkwork-drive-pc-admin-storage-providers

Domain: drive
Capability: storage-providers
Package type: PC internal admin React package
Status: standard

This package owns internal operator UI for Drive storage provider configuration management. It consumes the generated Drive admin storage SDK through the Drive PC runtime service boundary and must not construct raw HTTP requests, manual auth headers, provider SDK clients, or generated SDK internals.

## Public API

- `.`

## Required SDK Surface

- `@sdkwork/drive-admin-storage-sdk`

## Verification

- `pnpm test -- packages/sdkwork-drive-pc-admin-storage-providers/tests/storageProviderAdminService.test.ts`
- `pnpm typecheck`
