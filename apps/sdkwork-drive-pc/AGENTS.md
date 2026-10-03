# Repository Guidelines

<!-- SDKWORK-AGENTS-GENERATED: v2 -->

## SDKWORK Soul

Read `../../../sdkwork-specs/SOUL.md` before changing this application root.
Use progressive loading and the nearest package-level instructions.

## SDKWORK Standards

Canonical authorities are `../../../sdkwork-specs/README.md`,
`../../../sdkwork-specs/SOUL.md`, and
`../../../sdkwork-specs/AGENTS_SPEC.md`. Command and packaging work also loads
`../../../sdkwork-specs/PNPM_SCRIPT_SPEC.md` and
`../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`. Source configuration follows
`../../../sdkwork-specs/SOURCE_CONFIG_SPEC.md`. Do not copy their normative
bodies locally.

## Application Identity

This is the SDKWork PC application root for `sdkwork-drive-pc`.
`sdkwork.app.config.json` owns its application declaration. Repository-level
source profiles and deployment values are owned by `../../etc/`.

## Local Dictionary Structure

- `src/`: thin application bootstrap and root composition.
- `packages/`: PC runtime, features, shared UI, and Tauri desktop host packages.
- `specs/`: application-local component contracts.
- `etc/`: renderer-safe source configuration and the repository profile reference.
- `.sdkwork/`: local application skills and plugins.
- `package.json`: application-surface scripts and dependencies.

Documentation entrypoints are `docs/README.md`, `docs/product/prd/PRD.md`, and
`docs/architecture/tech/TECH_ARCHITECTURE.md`.

## Spec Resolution Order

1. Read this file and the owning repository `../../AGENTS.md`.
2. Load the app manifest or local specs only when the task touches their contract.
3. Select task authorities from `../../../sdkwork-specs/README.md`.
4. Inspect implementation after ownership and vocabulary are resolved.

Language-specific specs are on-demand. Use `TYPESCRIPT_CODE_SPEC.md` for TypeScript, `FRONTEND_CODE_SPEC.md` and
`APP_PC_REACT_UI_SPEC.md` for UI, and `APP_PC_ARCHITECTURE_SPEC.md` plus
`DESKTOP_APP_ARCHITECTURE_SPEC.md` for PC/Tauri host changes.

## Required Specs By Task Type

- TypeScript/Node: `../../../sdkwork-specs/TYPESCRIPT_CODE_SPEC.md`.
- UI: `../../../sdkwork-specs/FRONTEND_CODE_SPEC.md` and `../../../sdkwork-specs/APP_PC_REACT_UI_SPEC.md`.
- PC/Tauri: `../../../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md` and `../../../sdkwork-specs/DESKTOP_APP_ARCHITECTURE_SPEC.md`.
- SDK integration: `../../../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md`, `../../../sdkwork-specs/SDK_SPEC.md`, and `../../../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`.
- List/search: `../../../sdkwork-specs/PAGINATION_SPEC.md` and its linked API, database, SDK, and frontend authorities.

## Code Style Rules

Follow `../../../sdkwork-specs/CODE_STYLE_SPEC.md` and
`../../../sdkwork-specs/NAMING_SPEC.md`. Root `src/` stays thin; features live
in owned packages. Build and clean commands preserve tracked build-critical
sources.

## Build, Test, and Verification

Run the narrowest applicable app command: `pnpm dev`, `pnpm dev:desktop`,
`pnpm build`, `pnpm test`, `pnpm typecheck`, or `pnpm lint`. From the repository
root, run `pnpm check:pc-standard`, `pnpm check:pnpm-script-standard`, and
`pnpm check:agent-workflow-standard` when changing application structure,
commands, packaging, or agent entrypoints.

## Agent Execution Rules

Use dynamic progressive loading before implementation. Do not hand-edit
generated SDK output or replace composed facades with raw HTTP.

SDK integration routes to `../../../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md`,
`../../../sdkwork-specs/SDK_SPEC.md`, and
`../../../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`. Do not replace
composed SDK facades with raw HTTP or generated transport imports.

HTTP contract work routes to `../../../sdkwork-specs/API_SPEC.md` sections 4.5
and 14-16 plus the affected SDK/frontend/test authorities. Do not duplicate the
wire contract in this file.

List/search work routes to `../../../sdkwork-specs/PAGINATION_SPEC.md` and its
linked API, SDK, database, backend, and frontend authorities. Validate with
`node ../../../sdkwork-specs/tools/check-pagination.mjs --workspace ../..`.

## HTTP API Response Envelope

All L2+ SDKWork-owned custom HTTP contracts, including `app-api`, `backend-api`, and SDKWork-owned business `open-api`, `MUST` follow `API_SPEC.md` section 4.5, section 14, and section 15:

- **Default classification:** omitted `x-sdkwork-wire-protocol` means SDKWork-owned custom API (`sdkwork-v3`); only operation-level `x-sdkwork-wire-protocol: external` plus `x-sdkwork-external-protocol-id` identifies a third-party compatibility `open-api` operation.
- **Input:** typed request bodies, section 14.1 list/search/command input, `SdkWorkListQuery`, and `q` for free-text search.
- **Success output:** `SdkWorkApiResponse` with `{ "code": 0, "data": <payload>, "traceId": "<server-uuid>" }`.
- **Error output:** HTTP 4xx/5xx `application/problem+json` (`ProblemDetail`) with numeric `code` and `traceId`; SDKWork-owned errors may include `i18nKey` and `locale` presentation metadata.
- Success `code` is numeric `int32`; HTTP 2xx JSON bodies `MUST` use `0` only. REST semantics remain on HTTP status (`201`, `202`, etc.).
- Platform error codes are numeric non-zero values per section 15.3 (`40001`, `40101`, `40401`, …).
- Single resource: `data.item`
- Lists: `data.items` + `data.pageInfo` (`PageInfo.mode` is `offset` or `cursor`)
- Commands: `data.accepted` plus optional `resourceId` / `status`
- Async accept (`202`): `data.operationId`, `data.status`, optional `pollUrl`
- Operation patterns: retrieve/list/search/create/update/delete/command/async/bulk semantics follow `API_SPEC.md` section 15.4; create uses `201`, delete uses `204` with no JSON body, and `PUT`/`PATCH` use SDK action `update`.

Vendor compatibility `open-api` routes that mirror upstream tool or provider wire (for example OpenAI `/v1/*`, Anthropic/Claude `/anthropic/v1/*`, Google/Gemini `/google/v1beta/*`, Claude Code, or Codex) `MAY` opt out only when every exempt operation declares operation-level `x-sdkwork-wire-protocol: external` and `x-sdkwork-external-protocol-id` per `API_SPEC.md` section 4.5.2. SDKWork-owned business `open-api` operations `MUST NOT` opt out. Mixed OpenAPI documents are validated per operation; one external operation never exempts SDKWork-owned operations in the same document.

Errors `MUST` use HTTP 4xx/5xx with `application/problem+json` (`ProblemDetail`) including required numeric `code` and `traceId`. Optional `i18nKey` and `locale` are display metadata only. Business failures `MUST NOT` use HTTP 2xx with non-zero `code`, string wire codes, `success`, or human `message`.

Forbidden legacy envelopes and fields: `PlusApiResult`, `AppbaseApiResult`, `StoreApiResult`, `SdkWorkResponse`, per-domain `*ApiResult`, wire field `requestId`, bare domain DTOs at the HTTP root, and top-level `{ items, pageInfo, traceId }` without `data`.

Handlers `MUST` serialize success and map errors through `sdkwork-web-framework` response mapping. Generated HTTP SDKs (`--standard-profile sdkwork-v3`) unwrap `data` by default and expose typed numeric `ProblemDetail.code` / `traceId` and returned localization metadata on errors; use `.raw` when the full envelope is required.

Before completing API contract, SDK generation, or frontend service work, run:

```bash
node <sdkwork-specs>/tools/check-api-operation-patterns.mjs --workspace <workspace-root>
node <sdkwork-specs>/tools/check-api-response-envelope.mjs --workspace <workspace-root>
```

Authority: `sdkwork-specs/API_SPEC.md` section 4.5 and sections 14–16, `SDK_SPEC.md` section 4.2, `FRONTEND_SPEC.md`, `MIGRATION_SPEC.md` section 4.2.

## Human Review Rules

Human review is required for public app identity, breaking package or API
changes, security/auth behavior, generated SDK ownership, database migrations,
production deployment governance, and release publication.
