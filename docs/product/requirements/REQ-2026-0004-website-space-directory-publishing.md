# REQ-2026-0004 Website Space Directory Publishing

```yaml
id: REQ-2026-0004
title: Provide eligible Website Spaces and atomic directory roots for cloud site publication
owner: SDKWork Drive maintainers
status: in-progress
source: product
problem: Static application builds need directory-faithful live publication without exposing ordinary Spaces, leaking storage topology, or creating a Deploy Release for every file.
goals:
  - add a multi-instance website Space type
  - expose stable WebsiteRoot resources for the whole Space root or an explicit descendant folder
  - separate root selection from live-tree or atomic-generation content updates
  - support live ordinary changes and atomic complete-tree sync
  - provide typed public-resource resolution and versioned invalidation events
non_goals:
  - own domain, TLS, Variant, Mount, or public delivery policy
  - render Knowledgebase Wikis
  - allow anonymous arbitrary Drive node access
affected_surfaces:
  - database
  - api
  - sdk
  - backend
  - pc
  - uploader
  - storage
```

Specs: REQUIREMENTS_SPEC.md, ARCHITECTURE_DECISION_SPEC.md, DATABASE_SPEC.md, DRIVE_SPEC.md,
API_SPEC.md, SDK_SPEC.md, APP_SDK_INTEGRATION_SPEC.md, SECURITY_SPEC.md, PRIVACY_SPEC.md,
PERFORMANCE_SPEC.md, OBSERVABILITY_SPEC.md, TEST_SPEC.md, MIGRATION_SPEC.md

## Requirements

1. Add canonical `website` Space type across PostgreSQL/SQLite contracts, Rust domain, OpenAPI,
   generated SDKs, validators, storage binding policy, and UI.
2. Permit multiple Website Spaces per tenant owner subject within entitlement; revise the existing
   owner/type uniqueness rule without weakening singleton Space types.
3. Add a stable Drive-owned WebsiteRoot with immutable `sourceRootMode` (`SPACE_ROOT` or `FOLDER`),
   nullable selected folder identity, explicit `contentMode` (`LIVE_TREE` or
   `ATOMIC_GENERATION`), one active folder node, and monotonically increasing generation.
4. Validate website resource eligibility without marking the Space/root public by itself.
5. Resolve provider paths only inside the active folder through a typed authenticated service port or
   generated SDK; return stable version/metadata/stream and no provider topology.
6. Ordinary committed file mutations shall emit path/version events and require no Deploy Release or
   SiteRevision.
7. `ATOMIC_SYNC` shall stage/validate a complete normal Drive tree, atomically switch WebsiteRoot,
   retain prior generation by policy, and emit one generation event.
8. V1 public WebsiteRoot resolution shall reject shortcuts and all incomplete/quarantined/deleted or
   cross-root nodes.
9. Interactive/CI uploads shall reuse Drive Uploader sessions and attribution rather than add a
   second upload subsystem.
10. Drive user/admin views and commercial meters shall cover Website Space, WebsiteRoot, sync,
    versions, quotas, provider health, resource consumers, and audit.
11. Provision one default `SPACE_ROOT` WebsiteRoot in `LIVE_TREE` mode for every Website Space. Permit additional
    `FOLDER` roots within entitlement, reject reserved/internal/cross-Space selectors, and make an
    identical selector idempotently reuse one active WebsiteRoot.
12. Treat WebsiteRoot selector changes as a new/reused provider resource plus Deploy configuration
    change. Treat ordinary file updates and atomic generation switches behind an existing root as
    provider lifecycle changes that create no SiteRevision.
13. Own a versioned Drive AsyncAPI contract for generic node version/path/eligibility/delete events
    and WebsiteRoot generation events. Each event shall include Drive-produced root-scope matches;
    consumers must not infer membership from an unqualified node event.
14. Permit an authorized Knowledgebase `sources/raw` root-scoped subscription to the generic Drive
    node event stream without granting `website` type or Drive public-resource eligibility. Expose
    subscriptions/events through `sdkwork-drive-internal-api`, generated
    `sdkwork-drive-internal-sdk`, or an equivalent standalone typed port.

## Acceptance Criteria

- Multi-Website-Space and singleton-Space database/domain tests pass.
- WebsiteRoot ownership, active-node, generation, optimistic switch, retention, and deletion-impact
  contracts pass on PostgreSQL and SQLite.
- Space-root/folder-root selector, default provisioning, immutability, idempotency, overlapping-root,
  reserved namespace, folder move/rename/invalidation, and content-mode tests pass.
- React/Vite directory upload and atomic-sync E2E preserves paths and never exposes mixed roots.
- Cross-tenant/Space/root, traversal, shortcut, guessed ID, quarantine, incomplete, and deleted-node
  security tests fail closed.
- Event duplicate/gap/out-of-order and read-through reconciliation meet freshness targets.
- Drive AsyncAPI, internal SDK, producer inventory, root-scoped WebsiteRoot/Knowledgebase
  subscription, old/new move path, replay/dead-letter/checkpoint and bounded reconciliation
  contract tests pass.
- No Drive/provider object key, bucket, presigned URL, or raw storage SDK appears in public resource
  state or consumers.
- Generated SDK ownership/consumer import, uploader, pagination, quota, migration, observability,
  load, cleanup, crash recovery, and UI acceptance checks pass.

## Trace

- PRD: `docs/product/prd/PRD-website-space-publishing.md`
- Decision: `docs/architecture/decisions/ADR-20260721-website-space-directory-resource.md`
- Architecture: `docs/architecture/tech/TECH-website-directory-resource-provider.md`
- Cross-repository authority: `sdkwork-deployments` REQ-2026-0001

## Current Status

The Website Space, `SPACE_ROOT`/`FOLDER` WebsiteRoot, live-tree provider, atomic sync lifecycle,
retained generation activation/rollback, App/Internal API and generated SDK boundaries, and
provider invalidation contracts are implemented with focused executable evidence. The requirement
remains `in-progress` because its acceptance criteria still require complete user/admin UI, a real
React/Vite bundle pilot, deployed multi-node freshness/load/security evidence, and commercial
metering/operations acceptance. Content updates behind an existing WebsiteRoot do not create a
Deploy Release, Deployment, or SiteRevision.
