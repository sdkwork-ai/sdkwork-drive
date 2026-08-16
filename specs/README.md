# SDKWork Drive Repository Root Component

This is the repository root component spec for `sdkwork-drive`. It governs the
Rust backend workspace and the PC application root under `apps/sdkwork-drive-pc/`.

## Component Identity

- **Domain**: `drive`
- **Capability**: `workspace`
- **Component type**: `rust-workspace`
- **Languages**: `rust`, `typescript`
- **Root**: `.`

## Public Exports

- Rust workspace members under `crates/`
- PC application root under `apps/sdkwork-drive-pc/`
- API contracts under `apis/`
- SDK families under `sdks/` (generated)

## Runtime Entrypoints

- `crates/sdkwork-api-drive-standalone-gateway/src/main.rs`: standalone gateway binary
- `crates/sdkwork-routes-drive-app-api/src/main.rs`: app-api server binary
- `crates/sdkwork-routes-drive-backend-api/src/main.rs`: backend-api server binary
- `crates/sdkwork-routes-drive-open-api/src/main.rs`: open-api server binary
- `crates/sdkwork-routes-storage-backend-api/src/main.rs`: storage backend-api server binary
- `crates/sdkwork-drive-install-worker/src/main.rs`: install worker binary

## Verification

- `cargo test --workspace`
- `pnpm test`
- `pnpm check`
- `pnpm verify`

## Related Specs

- `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`
- `../sdkwork-specs/CODE_STYLE_SPEC.md`
- `../sdkwork-specs/NAMING_SPEC.md`
- `../sdkwork-specs/RUST_CODE_SPEC.md`
- `../sdkwork-specs/TYPESCRIPT_CODE_SPEC.md`
- `../sdkwork-specs/DRIVE_SPEC.md`
- `../sdkwork-specs/API_SPEC.md`
- `../sdkwork-specs/SDK_SPEC.md`
