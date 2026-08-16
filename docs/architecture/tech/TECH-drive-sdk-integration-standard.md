> Owner: SDKWork maintainers

# SDKWork Drive SDK Integration Standard

## Purpose

This document defines the Drive-local SDK family naming and frontend/backend integration boundary. It is a Drive-local implementation note, not a replacement for the global SDK standards.

Normative order:

- `../sdkwork-specs/SDK_SPEC.md` is the primary SDK standard. It owns SDK semantics, canonical SDK/API naming vocabulary, package semantics, generated client surface, auth handling, service facade boundaries, and generated client quality.
- `../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md` is the subordinate workspace/generation detail standard. It owns physical `sdks/` layout, SDK family directory placement, OpenAPI authority/derived inputs, generated-output placement, and generation workflow.
- This Drive document records the Drive-specific mapping and checks required to apply those two standards in this repository.

## API Authority And SDK Family Mapping

OpenAPI files keep the `*-api` naming because they describe runtime API authority. Runtime Rust route crates use the standard `sdkwork-routes-<capability>-<surface>` naming:

- `crates/sdkwork-routes-drive-open-api` -> `apis/open-api/drive/drive-open-api.openapi.json`
- `crates/sdkwork-routes-drive-app-api` -> `apis/app-api/drive/drive-app-api.openapi.json`
- `crates/sdkwork-routes-drive-backend-api` -> `apis/backend-api/drive/drive-backend-api.openapi.json`
- `crates/sdkwork-routes-storage-backend-api` -> `apis/backend-api/drive/drive-admin-storage-api.openapi.json`

Generated SDK families use `*-sdk` naming because they are consumable client SDK workspaces:

- Open/default API authority `drive-open-api` -> `sdks/sdkwork-drive-sdk`
- App API authority `drive-app-api` -> `sdks/sdkwork-drive-app-sdk`
- Backend API authority `drive-backend-api` -> `sdks/sdkwork-drive-backend-sdk`
- Admin storage API authority `drive-admin-storage-api` -> `sdks/sdkwork-drive-admin-storage-sdk`

The default/open SDK is `sdkwork-drive-sdk`. It is not `sdkwork-drive-open-api` and not `sdkwork-drive-open-sdk`.
The admin storage SDK is `sdkwork-drive-admin-storage-sdk`. It is not `sdkwork-drive-admin-storage-api` and not `drive-admin-storage-sdk`.

Admin storage uses the `/backend/v3/api` runtime prefix. The canonical generator's `sdkwork-v3` backend profile maps to `/backend/v3/api`, so `sdkwork-drive-admin-storage-sdk` uses generator SDK type `custom` and Drive-local standard profile metadata `sdkwork-drive-admin-storage-v3`. It still declares dual-token security and consumes appbase backend IAM through `sdkDependencies` instead of copying appbase routes.

## Canonical SDK Names

Canonical SDK family directories:

- `sdks/sdkwork-drive-sdk`
- `sdks/sdkwork-drive-app-sdk`
- `sdks/sdkwork-drive-backend-sdk`
- `sdks/sdkwork-drive-admin-storage-sdk`

Canonical language output directories:

- `sdks/sdkwork-drive-sdk/sdkwork-drive-sdk-{language}`
- `sdks/sdkwork-drive-app-sdk/sdkwork-drive-app-sdk-{language}`
- `sdks/sdkwork-drive-backend-sdk/sdkwork-drive-backend-sdk-{language}`
- `sdks/sdkwork-drive-admin-storage-sdk/sdkwork-drive-admin-storage-sdk-{language}`

Canonical generated package names:

- `sdkwork-drive-sdk-generated-{language}`
- `sdkwork-drive-app-sdk-generated-{language}`
- `sdkwork-drive-backend-sdk-generated-{language}`
- `sdkwork-drive-admin-storage-sdk-generated-{language}`

The SDK family root `sdk-manifest.json`, assembly `workspace`, generator `SDK_NAME`, and generator `--sdk-name` value must all use the canonical SDK family name. Generated `sdkwork-sdk.json` also uses the SDK family name, but generated source files must not be post-processed with Drive ownership metadata.

Do not infer SDK family names from route crate names. The route crates are API authority implementations; the SDK families are client distribution workspaces. When adding or reviewing any generated SDK surface, apply this mapping mechanically:

| API authority or service | SDK family | TypeScript output | Generated package base |
| --- | --- | --- | --- |
| `sdkwork-drive-open-api` / `drive-open-api` | `sdkwork-drive-sdk` | `sdkwork-drive-sdk-typescript` | `sdkwork-drive-sdk-generated` |
| `sdkwork-drive-app-api` / `drive-app-api` | `sdkwork-drive-app-sdk` | `sdkwork-drive-app-sdk-typescript` | `sdkwork-drive-app-sdk-generated` |
| `sdkwork-drive-backend-api` / `drive-backend-api` | `sdkwork-drive-backend-sdk` | `sdkwork-drive-backend-sdk-typescript` | `sdkwork-drive-backend-sdk-generated` |
| `sdkwork-drive-admin-storage-api` / `drive-admin-storage-api` | `sdkwork-drive-admin-storage-sdk` | `sdkwork-drive-admin-storage-sdk-typescript` | `sdkwork-drive-admin-storage-sdk-generated` |

Forbidden SDK family directories:

- `sdks/sdkwork-drive-open-api`
- `sdks/sdkwork-drive-app-api`
- `sdks/sdkwork-drive-backend-api`
- `sdks/sdkwork-drive-admin-storage-api`
- `sdks/drive-open-sdk`
- `sdks/drive-app-sdk`
- `sdks/drive-backend-sdk`
- `sdks/drive-admin-storage-sdk`

`sdkwork-drive-open-api`, `sdkwork-drive-app-api`, `sdkwork-drive-backend-api`, and `sdkwork-drive-admin-storage-api` are valid only for OpenAPI authority naming, generated OpenAPI filenames, and API/SDK metadata. Runtime Rust packages use `sdkwork-routes-*`. The API authority names are not valid SDK family names.

Forbidden SDK metadata and package values:

- `"sdkName": "sdkwork-drive-open-api"`
- `"sdkName": "sdkwork-drive-app-api"`
- `"sdkName": "sdkwork-drive-backend-api"`
- `"sdkName": "sdkwork-drive-admin-storage-api"`
- `sdkwork-drive-open-api-generated-{language}`
- `sdkwork-drive-app-api-generated-{language}`
- `sdkwork-drive-backend-api-generated-{language}`
- `sdkwork-drive-admin-storage-api-generated-{language}`

Drive-specific runtime operation maps, such as the PC app's Drive App SDK operation registry, belong in `sdkwork-drive-*-sdk-typescript/composed/operations.ts`. They must not be appended to `generated/server-openapi/src/index.ts`.

## Generation Flow

All Drive SDK family output must be generated by the canonical SDKWork generator:

```text
../../sdkwork-sdk-generator/bin/sdkgen.js
```

The generator package is `@sdkwork/sdk-generator`, and the CLI identity is `sdkgen`. `sdkwork-code-generator` is only an alias/wrapper name when it executes `../../sdkwork-sdk-generator/bin/sdkgen.js`; it is not a separate Drive SDK generator.

Drive-local scripts are thin wrappers only. They may select the Drive OpenAPI input, SDK family name, language, output directory, API prefix, SDK type, and strict profile arguments, but they must call the canonical generator above. They must not call copied generator code, a PATH-resolved generator from another location, a repository-local stub, an independent `sdkwork-code-generator`, or an ad hoc OpenAPI client generator for committed SDK output.

The repository generation entrypoint is:

```powershell
node tools/drive_sdk_generate.mjs
```

Use `--check` for contract and schema validation without writing SDK output:

```powershell
node tools/drive_sdk_generate.mjs --check
```

Use `--all-languages` when SDK family naming, OpenAPI contracts, or generator standards change:

```powershell
node tools/drive_sdk_generate.mjs --all-languages
```

Generated output is not hand-edited. Fix the OpenAPI contract, generator script, family-root manifest, composed facade, or assembly manifest first, then regenerate.

Generated `server-openapi` output must remain canonical `sdkgen` output:

- `sdkwork-sdk.json`, generated package manifests, `.sdkwork/*` reports, and generated source files must not carry `sdkOwner`, `apiAuthority`, `sdkDependencies`, `standardProfile`, or other ownership standard overlay fields.
- Drive family ownership and dependency metadata belongs in `sdk-manifest.json` and family-root `sdk-manifest.json`.
- Drive runtime operation maps belong in `composed/operations.ts` outside generated output.

## Storage Administration SDK Boundary

The App SDK is a user workflow SDK. It must expose file, folder, upload, download, archive, workspace, and user-facing Drive operations only.

The App SDK must not expose storage administration routes, operationIds, schemas, or composed helper methods for:

- storage provider CRUD, capability inspection, health test, activation/deactivation, or credential rotation;
- bucket discovery, head, create, update, or delete;
- low-level provider object list, head, delete, or copy;
- storage provider binding list, set, get, or delete.

Those capabilities belong to backend/admin control-plane SDK families:

- application storage administration and S3 account management: `sdks/sdkwork-drive-admin-storage-sdk` at `/backend/v3/api/drive/storage/*`;
- backend operations administration (audit, maintenance, quotas, labels, spaces, download packages): `sdks/sdkwork-drive-backend-sdk`.

The legacy flat `/backend/v3/api/drive/storage_providers/*` routes and `storageProviders.*` operations were removed from `drive-backend-api`; do not reintroduce them.

The OpenAPI export layer must remove App storage administration paths and unreachable storage administration schemas before App SDK generation. Do not hand-delete generated SDK files as the primary fix; fix API routing, OpenAPI export filters, schema quality gates, and SDK family smoke tests, then regenerate with `sdkgen` through Drive wrapper scripts.

## Backend Admin Permission Scopes

`drive-backend-api` and `drive-admin-storage-api` enforce per-route authorization through the web framework `AuthorizationPolicy` and `operationId` manifest mapping:

- umbrella: `drive.storage.admin` (all backend + storage admin operations)
- wildcard: `drive.*`
- granular: capability scopes such as `drive.audit.admin`, `drive.maintenance.admin`, `drive.quota.admin`, `drive.labels.admin`, `drive.spaces.admin`, and `drive.download_packages.admin`

Storage admin routes require `drive.storage.admin` or `drive.*`. Backend operations routes require the matching capability scope or the umbrella/wildcard scopes. PC admin navigation mirrors the same scope evaluation through `resolveDriveAdminSectionAccess` in `sdkwork-drive-pc-admin-core`.

## PC Frontend Boundary

The PC app must keep UI/product design separate from remote business transport.

Canonical call path:

```text
feature UI -> injected DriveFileService -> sdkwork-drive-pc-core service facade
  -> createDriveAppSdkClient -> sdks/sdkwork-drive-app-sdk/... composed operations + generated transport
  -> crates/sdkwork-routes-drive-app-api
```

Rules:

- Feature packages do not call `fetch`, `axios`, generic HTTP helpers, or handwritten Drive API clients.
- Feature packages do not assemble `Authorization`, `Access-Token`, or SDKWork AppContext headers.
- The shared App SDK wrapper may project the current SDKWork IAM session into required transport headers.
- `sdkwork-drive-pc-core` may use dedicated byte-transfer helpers (`uploadFetch`, `downloadFetch`) only for signed storage URLs returned by the generated App SDK; this is not a Drive metadata API bypass.
- Local demo data mode (`VITE_DRIVE_USE_LOCAL_DEMO_DATA`) was removed. All production and standalone PC runtimes require real appbase IAM login; contract tests forbid reintroducing local-demo session routing.
- Runtime bootstrap must not seed `tenant-local-demo` or `user-local-demo` sessions. Persisted demo sessions must be cleared on bootstrap when upgrading from legacy builds.

## Verification

Minimum verification for SDK naming and PC frontend/backend wiring:

```powershell
node tools/drive_sdk_generate.mjs --check --language typescript
pnpm --dir apps/sdkwork-drive-pc test
pnpm --dir apps/sdkwork-drive-pc typecheck
node sdks\sdkwork-drive-app-sdk\tests\sdk-family-smoke.test.mjs
node sdks\test\verify-sdk-ownership-boundaries.test.mjs
```

Before completing SDK family changes, scan for forbidden SDK family paths and metadata:

```powershell
rg -n "sdks/sdkwork-drive-(open|app|backend|admin-storage)-api|sdks/drive-(open|app|backend|admin-storage)-sdk|sdkName\": \"sdkwork-drive-(open|app|backend|admin-storage)-api" README.md docs apps sdks tools crates -S
```

Use targeted scans when the shell makes regex quoting ambiguous:

```powershell
Get-ChildItem sdks | Select-Object Name
rg -n 'sdkwork-drive-(open|app|backend|admin-storage)-api' sdks -S
rg -n 'sdkwork-drive-(open|app|backend|admin-storage)-api-generated|name: "sdkwork-drive-(open|app|backend|admin-storage)-api"|"sdkName": "sdkwork-drive-(open|app|backend|admin-storage)-api"' sdks apps tools crates -S
rg -n 'sdkOwner|apiAuthority|sdkDependencies|standardProfile' sdks/*/*/generated/server-openapi -g 'sdk-manifest.json' -g '*.ts' -g '*.json'
rg -n 'sdks/sdkwork-drive-(open|app|backend)-api|sdks/drive-(open|app|backend)-sdk' README.md docs apps tools crates -S
```

Expected results:

- `Get-ChildItem sdks` shows only Drive SDK family directories such as `sdkwork-drive-sdk`, `sdkwork-drive-app-sdk`, `sdkwork-drive-backend-sdk`, and `sdkwork-drive-admin-storage-sdk`, plus repository-local test fixtures.
- The `sdks` scan for `sdkwork-drive-(open|app|backend|admin-storage)-api` returns no matches.
- The metadata/package scan returns no forbidden SDK family names.
- The generated ownership scan returns no matches for generated `sdk-manifest.json` or generated source metadata overlays.
- The path scan may match this document's forbidden-name section only; it must not match implementation, generated output, tooling, README examples, or active design instructions.

