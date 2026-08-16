# sdkwork-drive-pc-admin-core

Domain: drive
Capability: pc-admin-core
Package type: PC internal admin React package
Surface: backend-admin
Status: standard

This package owns the Drive PC admin runtime core: admin access guards, Drive
runtime composition for admin surfaces, and the generated Drive admin storage
SDK client boundary. It is consumed by sibling `sdkwork-drive-pc-admin-*`
packages and must not construct raw HTTP requests, manual auth headers, or
generated SDK internals.

## Public API

- `.`: aggregated exports for admin access, runtime, and SDK client.
- `./auth/adminAccess`: admin access guard helpers.
- `./runtime/driveRuntime`: Drive admin runtime composition.
- `./sdk/driveAdminStorageSdkClient`: generated Drive admin storage SDK client boundary.

## Required SDK Surface

- `@sdkwork/drive-admin-storage-sdk`

## Verification

- `pnpm typecheck`
