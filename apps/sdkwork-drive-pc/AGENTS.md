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

## Human Review Rules

Human review is required for public app identity, breaking package or API
changes, security/auth behavior, generated SDK ownership, database migrations,
production deployment governance, and release publication.
