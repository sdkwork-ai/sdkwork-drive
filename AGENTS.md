# Repository Guidelines

<!-- SDKWORK-AGENTS-GENERATED: v2 -->

## SDKWORK Soul

Read `../sdkwork-specs/SOUL.md` before executing tasks in this root. Resolve the
task category before loading broad context, and verify the changed boundary
before completion.

## SDKWORK Standards

Canonical global authorities from this root:

- `../sdkwork-specs/README.md`
- `../sdkwork-specs/SOUL.md`
- `../sdkwork-specs/AGENTS_SPEC.md`
- `../sdkwork-specs/PNPM_SCRIPT_SPEC.md`
- `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`
- `../sdkwork-specs/SOURCE_CONFIG_SPEC.md`
- `../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`
- `../sdkwork-specs/SDKWORK_DEPLOY_SPEC.md`
- `../sdkwork-specs/CODE_STYLE_SPEC.md`
- `../sdkwork-specs/NAMING_SPEC.md`
- `../sdkwork-specs/TEST_SPEC.md`

Do not copy global normative bodies into this repository. Stop and report a
broken workspace layout if these relative paths do not resolve.

## Application Identity

`sdkwork.app.config.json` owns Drive identity, runtime declarations, release
metadata, and package inventory. `etc/sdkwork.deployment.config.json` owns the
source profile index; concrete URLs, binds, ports, and runtime values live
under `etc/`.

## Local Dictionary Structure

- `sdkwork.workflow.json`: package/release/deployment workflow declaration.
- `specs/topology.spec.json`: topology v5 process and surface contract.
- `etc/`: safe source configuration and profile instances.
- `deployments/`: deployment and infrastructure descriptors.
- `apis/`: Drive-owned API authorities and OpenAPI inputs.
- `apps/`: runnable application surfaces; use the nearest `AGENTS.md`.
- `crates/`: Rust libraries and process hosts.
- `sdks/`: generated SDK families and composed facades.
- `.sdkwork/`: local skills, plugins, and repository metadata.
- `scripts/`, `tools/`, `tests/`, `docs/`: lifecycle adapters, validators, tests, and Canon documentation.

Documentation entrypoints are `docs/README.md`, `docs/product/prd/PRD.md`, and
`docs/architecture/tech/TECH_ARCHITECTURE.md`.

## Spec Resolution Order

Use dynamic progressive loading before implementation files:

1. Read this file and any nearer `AGENTS.md`.
2. Read `sdkwork.app.config.json`, `etc/`, local `specs/`, or `.sdkwork/` only when the task reaches that contract.
3. Use the relevant row in `../sdkwork-specs/README.md` to select global authorities.
4. Inspect implementation only after resolving ownership and vocabulary.

Language-specific specs are on-demand and load only for the language being changed.

## Required Specs By Task Type

- Lifecycle/topology: `PNPM_SCRIPT_SPEC.md`, `APP_RUNTIME_TOPOLOGY_SPEC.md`, `CONFIG_SPEC.md`, `ENVIRONMENT_SPEC.md`, `TEST_SPEC.md`.
- Source config: `SOURCE_CONFIG_SPEC.md`, `CONFIG_SPEC.md`, `ENVIRONMENT_SPEC.md`, `RUNTIME_DIRECTORY_SPEC.md`.
- Release/deploy: `GITHUB_WORKFLOW_SPEC.md`, `RELEASE_SPEC.md`, `SUPPLY_CHAIN_SECURITY_SPEC.md`, `SDKWORK_DEPLOY_SPEC.md`, `QUALITY_GATE_SPEC.md`.
- API/SDK: `API_SPEC.md`, `SDK_SPEC.md`, `SDK_WORKSPACE_GENERATION_SPEC.md`, `WEB_FRAMEWORK_SPEC.md`, `TEST_SPEC.md`.
- Security/auth: `IAM_SPEC.md`, `IAM_LOGIN_INTEGRATION_SPEC.md`, `SECURITY_SPEC.md`, `PRIVACY_SPEC.md`.
- Rust: `RUST_CODE_SPEC.md`; TypeScript/Node: `TYPESCRIPT_CODE_SPEC.md`; frontend: `FRONTEND_CODE_SPEC.md` and the selected UI architecture spec.

## Int64 Wire Contract (API_SPEC §13.6)

- OpenAPI `int64` fields and parameters `MUST` be `type: string`, `format: int64`,
  a decimal `pattern` such as `^-?[0-9]+$`, and `x-sdkwork-int64-string: true`.
  `type: integer, format: int64` is a contract violation: generated TypeScript
  SDKs then emit `number`, and browsers silently round ids past
  `Number.MAX_SAFE_INTEGER` (2^53), replaying wrong ids into lookups.
- Rust response DTOs `MUST` serialize `i64` wire fields with
  `#[serde(with = "sdkwork_utils_rust::serde_int64")]` (or `::option`); request
  boundaries parse inbound strings with the same helper.
- Generated TypeScript SDKs keep `int64` as `string`; frontend code `MUST NOT`
  convert ids/snowflake ids/sequence ids to `number` for storage, comparison,
  or submission.
- Verification: `node <sdkwork-specs>/tools/check-api-operation-patterns.mjs --workspace .`

## Code Style Rules

Code changes follow `../sdkwork-specs/CODE_STYLE_SPEC.md` and
`../sdkwork-specs/NAMING_SPEC.md`. Build runners and `clean` follow
`CODE_STYLE_SPEC.md` section 7 and must not delete tracked build-critical
sources.

## Build, Test, and Verification

Public lifecycle commands are `pnpm dev`, `pnpm dev:standalone`,
`pnpm dev:cloud`, `pnpm stop`, `pnpm build`, `pnpm test`, `pnpm check`,
`pnpm verify`, and `pnpm clean`. Run the narrowest changed-surface check first,
then broaden only when the change crosses contracts.

Important local gates include:

```powershell
pnpm check:pnpm-script-standard
pnpm check:docs-standard
pnpm check:agent-workflow-standard
pnpm topology:validate
pnpm deploy:validate
node ../sdkwork-specs/tools/check-source-config-standard.mjs --root .
```

## Agent Execution Rules

Use the convention dictionary before broad source loading. Do not hand-edit
generated SDK output, replace composed SDKs with raw HTTP, or bypass the public
`sdkwork-app` lifecycle facade. Preserve unrelated worktree changes and record
exact verification evidence.

SDK consumer import work routes to
`../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md`,
`../sdkwork-specs/SDK_SPEC.md`, and
`../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`. Validate with
`node ../sdkwork-specs/tools/check-app-sdk-consumer-imports.mjs --workspace .`.

HTTP input/output work routes to `../sdkwork-specs/API_SPEC.md` sections 4.5 and
14-16 plus the affected framework, SDK, migration, and test authorities. Use
the API response and operation-pattern validators selected by those specs.

List/search work routes to `../sdkwork-specs/PAGINATION_SPEC.md` and its linked
API, database, backend, SDK, and frontend authorities. Validate with
`node ../sdkwork-specs/tools/check-pagination.mjs --workspace .`.

## Human Review Rules

Human review is required for public identity or naming changes, breaking API or
SDK changes, security/auth changes, database migrations, production deployment
governance, generated SDK ownership, and release publication.
