---
id: REQ-2026-0003
title: Pre-launch technical debt cleanup for SDKWork Drive
owner: SDKWork maintainers
status: accepted
source: platform
updated: 2026-07-23
problem: Before commercial GA, Drive must remove dead auth-policy wiring, standardize deployment profile env keys, eliminate stale documentation, align deploy/API/database framework contracts with sdkwork-specs, and prevent legacy HTTP envelope modules from returning.
goals:
  - Remove unused `auth_policy` state fields and deprecated `build_router_with_pool_and_iam_policy` builders.
  - Standardize `SDKWORK_DRIVE_DEPLOYMENT_PROFILE` across systemd units and deployment validators.
  - Provide `deployments/deploy.yaml` per SDKWORK_DEPLOY_SPEC.md.
  - Add a deployment validator that rejects deploy/topology profile drift and fails release-mode checks on `REPLACE_WITH_RELEASE_DIGEST` and non-immutable Kubernetes image references.
  - Route JWT/download crypto through `sdkwork-utils-rust` instead of direct `sha2`/`hmac` in workspace route crates.
  - Include `api:envelope:check` and `api:schema:check` in the root `pnpm check` gate.
  - Keep Drive OpenAPI authorities owner-only, consume IAM through declared SDK/runtime dependencies, and eliminate operationId/tag duplication before first release.
  - Provide an operator pre-launch checklist linked from deployment validation.
  - Remove legacy `sdkwork-drive-http` envelope modules (`response.rs`, `problem_detail.rs`); keep `api_problem.rs` as the sole ProblemDetail authority.
  - Keep `.env.postgres.example` on the unified `SDKWORK_DATABASE_*` identity only; application-specific database connection prefixes are forbidden.
  - Route App SDK uploader composed helpers through `@sdkwork/utils` (`hexEncode`, `uuid`).
  - Align `pnpm-workspace.yaml` and PC `peerDependencies` with sdkwork-specs workspace registry (`verify-repo`).
  - Remove retired service-layout tokens from public `package.json#scripts` and document standard dev command examples only.
  - Make strict release readiness fail closed on deferred signing, cross-platform checksum, and Catalog media evidence instead of reporting GA blockers as warnings.
  - Make release dispatcher lifecycle commands deterministic: no-argument phases must not crash, and `release:package` must build the standalone gateway instead of assuming a pre-existing release binary.
  - Materialize local release package evidence for web, Windows desktop, standalone gateway, Catalog media staging, release evidence, and SBOM without fabricating production signing, CDN, or cross-platform artifact evidence.
non_goals:
  - Enabling artifact signing or flipping catalog publish status to ACTIVE.
users:
  - platform operators
  - backend maintainers
acceptance_criteria:
  - No router crate exposes `build_router_with_pool_and_iam_policy`.
  - All systemd units in `deployments/systemd/` set `SDKWORK_DRIVE_DEPLOYMENT_PROFILE`.
  - `deployments/deploy.yaml` exists, uses only canonical topology v4 profile ids (`cloud.production`, `cloud.development`, `standalone.production`, `standalone.development`), and passes `pnpm deploy:validate`.
  - `check_drive_deployments.mjs` rejects deploy profiles that are not listed in `specs/topology.spec.json#profileFiles` or whose `overrides.topology.env` points at a missing/non-canonical env file.
  - `pnpm check` includes `api:envelope:check`, `api:schema:check`, and passes.
  - `pnpm deploy:validate` and `pnpm check:architecture-alignment` pass.
  - `node --test tools/check_drive_deployments.test.mjs` proves default-mode digest warnings, strict-mode placeholder rejection, strict-mode real digest acceptance, Redis rate-limit enforcement, and legacy topology profile rejection.
  - `docs/guides/operator/pre-launch-checklist.md` exists and is referenced by deployment checks.
  - `crates/sdkwork-drive-http/src/response.rs` and `problem_detail.rs` do not exist; `lib.rs` exposes only `api_problem`.
  - `.env.postgres.example` uses only the unified `SDKWORK_DATABASE_*` connection identity and contains no legacy application-specific database connection prefix.
  - `@sdkwork/drive-app-sdk` depends on `@sdkwork/utils`; `uploaderClient.ts` does not inline hex formatting.
  - `pnpm check:app-composition` (`verify-repo`) passes for workspace package wiring.
  - Drive App and Backend authorities contain no dependency-owned routes or composed-operation markers; generated SDKs are rebuilt from owner-only OpenAPI inputs.
  - `check-api-operation-patterns.mjs` passes without compatibility exceptions; pre-launch tag cleanup produces canonical `client.drive.<resource>.<action>()` SDK grouping.
  - `pnpm check:pnpm-script-standard` passes and scans root scripts, package manifests, docs, command JSON, and runner scripts.
  - The `test:sdkwork-command-dev-topology` contract proves cloud dev commands infer the cloud development topology internally without exposing retired public script tokens.
  - `node --test tools/check_sdkwork_drive_release_readiness.test.mjs` proves default-mode warnings remain non-blocking for development while `SDKWORK_RELEASE_VALIDATION=strict` fails on deferred signing, target-runner checksum evidence, and Catalog media evidence.
  - `node --test tests/contract/sdkwork-command-dev-topology.contract.test.mjs` proves `release:plan` does not fail with `TypeError: scriptArgs is not iterable` and `release:package` does not call `gateway-standalone-pack.mjs package --skip-build`.
  - `pnpm release:package` builds and stages local release artifacts, updates manifest checksums only from real local files, materializes Catalog media staging evidence, writes release evidence, and generates SBOM.
  - `pnpm release:validate` passes after local packaging while preserving strict GA blockers for signing, macOS/Linux target-runner checksums, and Catalog CDN publication.
  - `SDKWORK_RELEASE_VALIDATION=strict pnpm check:release-readiness` must not pass until real signing, checksum, and Catalog CDN evidence is materialized.
non_functional_requirements:
  reliability: Metrics and tracing use `deployment_profile` label via `resolve_deployment_profile_label()`.
affected_surfaces:
  - backend
  - deployments
  - docs
  - sdks
  - tests
verification:
  - cargo check --workspace
  - pnpm verify
  - pnpm check
  - pnpm api:envelope:check
  - pnpm api:schema:check
  - node ../sdkwork-specs/tools/check-api-operation-patterns.mjs --workspace .
  - node --test sdks/test/verify-sdk-ownership-boundaries.test.mjs
  - pnpm check:pnpm-script-standard
  - pnpm test:sdkwork-command-dev-topology
  - node --test tests/contract/sdkwork-command-dev-topology.contract.test.mjs
  - node --test tools/check_drive_deployments.test.mjs
  - node --test tools/check_sdkwork_drive_release_readiness.test.mjs
  - pnpm release:package
  - pnpm release:validate
  - pnpm test:release-evidence
  - pnpm deploy:validate
  - pnpm check:release-readiness
  - SDKWORK_RELEASE_VALIDATION=strict pnpm check:release-readiness (expected to fail until signing/checksum/Catalog evidence is materialized)
  - pnpm check:architecture-alignment
---
