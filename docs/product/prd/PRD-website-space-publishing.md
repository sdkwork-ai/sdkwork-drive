# SDKWork Drive Website Space Publishing PRD

Status: active
Owner: SDKWork Drive maintainers
Application: sdkwork-drive
Updated: 2026-07-23
Requirement: REQ-2026-0004
Parent: [PRD.md](PRD.md)
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md, DRIVE_SPEC.md, DATABASE_SPEC.md,
API_SPEC.md, SDK_SPEC.md, APP_SDK_INTEGRATION_SPEC.md, SECURITY_SPEC.md,
PRIVACY_SPEC.md, PERFORMANCE_SPEC.md, OBSERVABILITY_SPEC.md, TEST_SPEC.md,
MIGRATION_SPEC.md

Implementation boundary: the Website Space type, default `SPACE_ROOT`, explicit `FOLDER`
WebsiteRoots, dual-engine persistence, App/Internal API contracts, generated SDKs, provider
validation, path resolution/open, root-scoped change events, and `ATOMIC_SYNC` create/retrieve/
finalize/abort/activation are implemented. The atomic lifecycle includes uploader-confined staging,
leased and fenced validation, serializable generation switching, retained-generation rollback,
retention, expiry, and cleanup. Complete user/admin UI, a real React/Vite pilot, deployed
Provider-cache freshness/load evidence, commercial metering reconciliation, and production
load/security/backup/restore/operations evidence remain open.

## 1. Purpose

Enable a Drive directory tree to act as the source of a professional static website while keeping
Drive as the authority for Spaces, nodes, uploads, versions, storage, quotas, and content lifecycle.
One Website Space represents one independent website project. Each Drive-owned `WebsiteRoot`
selects either the complete user-content namespace of the Space (`SPACE_ROOT`) or one explicit
descendant folder (`FOLDER`). Its hierarchy is preserved when that stable resource is mounted by the
Deploy control plane.

A Website Space is eligible for publication but is not public by itself. Public exposure requires
an active Deploy Site, `DRIVE_DIRECTORY` resource, Variant, Mount, Binding, and verified runtime
revision.

## 2. Problem

Application builds such as React/Vite produce a complete directory with `index.html`, hashed assets,
icons, manifests, fonts, and nested paths. Treating that output as individual unrelated public files
breaks relative paths and makes publishing cumbersome. Requiring a Deploy Release for every file
edit also defeats the direct, WYSIWYG behavior expected from cloud storage and static hosting.

At the same time, exposing arbitrary Drive Spaces or handing object-storage URLs to Web Server would
bypass Drive ownership, tenant isolation, versioning, provider portability, and security policy.

## 3. Users

- Developers uploading a static application build.
- Designers and content owners maintaining a directory-based website.
- Tenant administrators governing Space ownership, storage, versions, and publication integration.
- CI/CD automation synchronizing a complete build through the generated Drive SDK.
- Deploy and Web Server resource consumers resolving eligible public directory content.
- Drive platform administrators operating storage, scanners, quotas, syncs, and provider health.

## 4. Goals

- Add `website` as an explicit multi-instance Drive Space type.
- Treat the Space as project/security/quota boundary and support both the Space root and an explicit
  descendant folder as document-root selections.
- Create one default `SPACE_ROOT` WebsiteRoot for every Website Space and support additional
  `FOLDER` WebsiteRoots within entitlement.
- Keep root selection (`SPACE_ROOT`/`FOLDER`) independent from update mode
  (`LIVE_TREE`/`ATOMIC_GENERATION`).
- Preserve directory/file names and relative hierarchy through the resource provider contract.
- Support ordinary live file mutation without a Deploy Release.
- Support complete-tree `ATOMIC_SYNC` and rollback so hashed application bundles never expose mixed
  generations.
- Provide stable WebsiteRoot identity even when the active physical folder generation changes.
- Resolve content through stable Drive identities and streams, never bucket/object/presigned URLs.
- Emit versioned events and public-resource observations for cache freshness.
- Provide complete user and Drive-admin workflows with plan/quota visibility and audit.

## 5. Non-Goals

- Make personal, team, app-upload, deployment, knowledge-base, or other Space types public.
- Own domains, paths, device Variants, TLS, delivery headers, CDN, or public request analytics.
- Render Markdown into a Wiki or decide Knowledgebase page visibility.
- Build source code, run package managers, execute uploaded server code, or run arbitrary hooks.
- Let anonymous browsers retrieve Drive nodes by guessed Space/node/version IDs.
- Persist or return object keys and presigned URLs as durable website identity.

## 6. Product Model

```text
Website Space (project, tenant, owner, quota)
  -> WebsiteRoot (stable provider resource handle)
     -> sourceRootMode = SPACE_ROOT | FOLDER
     -> selectedFolderNode = null | stable descendant folder UUID
     -> contentMode = LIVE_TREE | ATOMIC_GENERATION
     -> active Folder Node (effective provider document root)
        -> Files and subfolders (website hierarchy)
     -> generation N (atomic switch/version observation)
  -> Atomic Sync attempts
  -> Drive node and content versions

Deploy Site
  -> DRIVE_DIRECTORY resource -> WebsiteRoot UUID
  -> Variant -> Mount -> Host/path Binding
```

Every Website Space provisions one default `SPACE_ROOT` WebsiteRoot in `LIVE_TREE` mode. Users may add explicit
`FOLDER` roots for `apps/admin`, `apps/mobile`, `docs`, or other subtrees within entitlement. A
selector is immutable after creation: changing from the Space root to a folder, or from one folder
to another, creates or reuses another WebsiteRoot and updates the Deploy Site Resource. That is a
Site configuration change. File mutations and atomic generation switches behind an existing
WebsiteRoot remain provider lifecycle changes and create no SiteRevision.

The same WebsiteRoot is reused by any number of authorized Deploy Sites, Variants, and Mounts; it is
not duplicated for each domain or client type. Drive returns an existing active root for the same
normalized selector or rejects a conflicting duplicate according to the idempotency contract.

## 7. Space Eligibility

- `spaceType=website` is created only through an operation authorized for website project creation.
- Website Spaces are multi-instance. A user or organization can own multiple website projects
  within entitlement and storage quotas.
- Space conversion from another type is not supported in V1; import/copy into a new Website Space is
  safer and auditable.
- Deleting, transferring, suspending, or changing tenant ownership of a Website Space requires
  Deploy dependency checks and explicit impact confirmation.
- Space state, owner, tenant, storage provider binding, malware policy, and quota must be valid before
  a WebsiteRoot can be connected or synchronized.
- Eligibility is not equivalent to publication. Drive does not create a public domain or Binding.

## 8. Directory Semantics

- `SPACE_ROOT` resolves from the Website Space's canonical root node and includes all active
  user-content descendants except Drive-reserved, internal, trash, staging, version, and provider
  management namespaces.
- `FOLDER` requires one active folder UUID in the same Website Space, below the canonical root and
  outside every reserved/internal namespace. The selected folder itself maps to provider path `/`.
- `sourceRootMode` and the selected folder identity are immutable WebsiteRoot identity fields.
  Folder rename or same-Space move preserves the stable resource; deletion, quarantine, archive, or
  move outside the eligible namespace invalidates it.
- `LIVE_TREE` serves the effective selected root directly. `ATOMIC_GENERATION` serves the active
  managed generation for that logical selector. Switching content mode is a privileged Drive root
  operation with validation, audit, generation fencing, and cache invalidation.
- Node names and parent relationships define the resource path exactly after canonical normalization.
- Name uniqueness among active siblings follows the Drive sibling naming standard.
- File names are case-sensitive at the canonical contract boundary; case-collision validation is
  available for portability to case-insensitive hosts/tooling.
- Folder nodes are not listed publicly by default. Index and listing behavior belongs to the Deploy
  Mount/delivery policy.
- V1 public WebsiteRoot resolution rejects Drive shortcut nodes. A later same-root shortcut profile
  requires a separate path-confinement decision.
- Deleted, quarantined, incomplete-upload, failed-scan, expired, or non-active nodes are not
  resolvable.
- Renames, moves, writes, and deletes update public paths immediately after commit and event/cache
  propagation. Old paths exist only when Deploy owns an explicit redirect.
- Relative links are preserved because Web Server maps the complete remaining URL path into the
  active folder tree.

## 9. Upload Workflows

### 9.1 Interactive Upload

The Website explorer uses the composed Drive Uploader. Users can drag/drop files and folders,
preserve relative paths, resolve conflicts, retry failed parts, and observe scan/version status.
Ordinary upload commits each file according to Drive semantics and becomes live independently.

### 9.2 CLI/CI Directory Sync

The generated SDK/composed uploader supports manifest planning, changed-file detection, bounded
parallel multipart upload, resume, checksum verification, delete policy, dry run, and idempotency.
The client sends stable path/checksum/size metadata; Drive remains responsible for provider object
identity.

### 9.3 ATOMIC_SYNC

`ATOMIC_SYNC` is recommended for framework bundles whose HTML and hashed assets must change as a
unit:

1. Create a sync against a stable WebsiteRoot, its immutable source selector, content mode, and
   expected current generation.
2. Upload the complete target tree into an isolated staging folder using Drive Uploader sessions.
3. Submit/finalize a bounded manifest with path, type, size, and checksum.
4. Validate sibling names, paths, completeness, checksums, quotas, malware/format policy, reserved
   names, symlink/shortcut absence, and required entry files when requested.
5. Atomically switch `WebsiteRoot.activeNodeId` and increment `generation` in one transaction.
6. Emit one root-generation event plus bounded diagnostic counts.
7. Retain the prior root under the configured rollback/version policy, then clean it asynchronously.

Readers see generation N or N+1, never a partially uploaded mixture. A failed or expired sync leaves
the active generation unchanged. This is a Drive content operation, not a Deploy Release,
Deployment, or SiteRevision.

### 9.4 Rollback

An authorized user can switch the WebsiteRoot to a retained prior generation using optimistic
concurrency and audit. Rollback emits the same generation-change event and does not alter domain,
Variant, Mount, or TLS configuration.

## 10. User Interface Views

### 10.1 Space List

Add a Website filter and project rows/cards with owner, active root, generation, storage, last sync,
connected Sites, public-domain summary from Deploy, and health. Normal Space rows do not imply
public status.

### 10.2 Create Website Space

The wizard collects project name/slug, owner scope, storage region/provider policy, initial folder
or starter files, version retention, and optional connection to an existing/new Deploy Site. It
shows storage and Website Space entitlements before creation.

### 10.3 Website Explorer

| Area | Required behavior |
| --- | --- |
| Toolbar | upload files/folder, new folder/file, atomic sync, preview/open Site, rollback, settings |
| Tree/list | path, type, size, version, checksum state, scan state, active generation, modified by/time |
| Root banner | WebsiteRoot identity, active folder, generation, connected Site/Mount, freshness |
| Selection panel | node versions, MIME, checksum, public URL preview when connected, activity |
| Sync drawer | plan, adds/changes/deletes, bytes, conflicts, progress, validation, cancel/retry |
| Problems view | missing entry, case collision, invalid path, scan failure, broken relative asset hints |

Every view includes loading, empty, permission-denied, quota-exceeded, partial upload, validation
failure, provider degradation, stale integration state, and retry states.

### 10.4 Website Settings

Manage project identity, owners/members, the default Space-root resource, additional folder roots,
root selector, content mode, active generation, retention, sync defaults, case policy,
reserved-file policy, storage provider, quota alerts, connected Deploy Sites, audit, archive, and
guarded deletion. Domains/TLS/Variant/delivery policy link to Deploy rather than being duplicated in
Drive.

## 11. Drive Admin Views

| View | Purpose |
| --- | --- |
| Website Spaces | tenant/owner/project inventory, status, storage, connected resources, entitlement |
| WebsiteRoots | active node/generation, stale references, switches, retention, rollback health |
| Atomic Syncs | state, duration, file/byte counts, failure reasons, orphan staging cleanup |
| Storage/scan health | provider latency/errors, malware queue, checksum failures, capacity |
| Quotas | Website Space count, stored bytes, version bytes, upload/sync rates, rejected operations |
| Resource consumers | authorized Deploy resource references and last validation/open observations |
| Audit/investigation | create/transfer/sync/switch/rollback/delete/provider and privileged actions |

Drive admins cannot create domains, issue certificates, or change Deploy routing from these views.
Cross-product links use generated SDK-backed console integration.

## 12. Permissions

Conceptual roles are owner, maintainer, content publisher, content editor, viewer, and Drive platform
administrator. Only owner/maintainer can manage WebsiteRoots, transfer/archive/delete, and connect a
Deploy resource. Publisher can run atomic sync and rollback if granted. Editor can mutate ordinary
files but cannot switch generations by default. Final permission tokens are defined by the Drive
permission manifest and enforced server-side.

## 13. Commercial And Quota Model

Drive owns entitlements and metering for Website Space count, stored bytes, version bytes, upload
bytes, atomic-sync staging bytes, sync frequency/concurrency, retained generations, and storage
region/provider tier. Deploy owns public requests/egress/domains/certificates. Commerce owns pricing
and billing. Cross-product UI labels each dimension with its billing owner to prevent double charge.

Quota checks occur before session creation and again before commit/switch. Staging capacity is
reserved and released on completion/abort/expiry. A plan downgrade does not delete content or switch
the active root; it blocks new excess capacity according to policy and gives an actionable remedy.

## 14. Security And Privacy

- Resource provider operations require authenticated Deploy/Web Server service identity, tenant,
  WebsiteRoot, trace, and deadline; anonymous node lookup is not a provider contract.
- Normalize and bound every manifest path, node name, depth, count, size, checksum, and MIME value.
- Reject traversal, absolute paths, drive letters, separators in node names, control characters,
  shortcut/symlink escape, duplicate canonical paths, and reserved internal paths.
- Malware/quarantine policy blocks active-root switch when required checks are not ready or fail.
- Streams are tenant/root/version confined and range-bounded. Cacheable metadata contains no provider
  bucket/object/presigned URL.
- Audit records use stable identities and reason/result codes; manifests and customer file names are
  not copied into unbounded metrics.
- Transfer/deletion follows privacy, retention, legal hold, backup, and data export requirements.

## 15. Performance And Reliability

- Path lookup and public open use indexed `space/root/parent/name` or provider-approved equivalent,
  not full-tree collection.
- Directory upload/sync supports bounded page/manifest size and bounded parallelism with streaming.
- Root switch is one short transaction independent of total object bytes.
- Sync validation and cleanup are asynchronous, leased, idempotent, restart-safe, and fenced.
- Event delivery is at-least-once with idempotent consumers and a reconciliation checkpoint.
- Drive publishes a versioned AsyncAPI node lifecycle stream and root-qualified scope matches;
  authorized Knowledgebase `sources/raw` subscriptions reuse that generic stream without granting
  website eligibility to the Knowledgebase Space.
- Active generation remains available when a staging sync fails or expires.
- Provider outage, checksum mismatch, quota race, scan delay, duplicate finalize, worker crash, and
  storage retry have deterministic failure/recovery behavior.

## 16. Success Metrics

- A valid React/Vite `dist/` tree can be uploaded and connected without changing its hierarchy.
- At least 99% of committed ordinary changes meet the platform freshness objective.
- Atomic sync never exposes mixed generations in contract, fault-injection, or load tests.
- 100% of provider resolutions use stable Drive resource/node/version identity and no object key or
  presigned business identity.
- Orphan staging trees are reconciled within the retention objective.
- Storage and sync usage reconcile with Drive quota and Commerce meter exports.

## 17. Acceptance Criteria

- Multiple Website Spaces can be created for one owner subject within entitlements; singleton Space
  invariants remain intact for types that require them.
- Ordinary Space types fail website resource eligibility.
- Every Website Space receives one default `SPACE_ROOT`; creating `FOLDER` roots validates stable
  same-Space descendant identity and both modes resolve the complete selected hierarchy.
- Root selector immutability, same-selector idempotency, multiple folder roots, overlapping subtree
  authorization, reserved namespace exclusion, folder move/rename, invalidation, and deletion-impact
  tests pass.
- Interactive and CI upload use the Drive Uploader and persist no provider topology as business state.
- Atomic sync success, failure, cancellation, duplicate finalize, generation conflict, rollback,
  expiry, cleanup, and crash recovery tests pass.
- Cross-root, cross-Space, cross-tenant, shortcut, traversal, incomplete, quarantined, and guessed-ID
  access tests fail closed.
- Provider event and read-through reconciliation tests meet freshness targets.
- Drive input event schema, generated internal SDK, component producer inventory, root-scoped
  WebsiteRoot and Knowledgebase subscription matching, old/new move paths, duplicate/order/gap/
  replay/dead-letter, and bounded reconciliation tests pass.
- User and Drive-admin views cover project, files, sync, versions, integration, quotas, health, and
  audit through generated SDKs.
- PostgreSQL/SQLite contracts, API/SDK generation, tenant isolation, performance, and migration gates
  pass before implementation is declared ready.

## 18. Dependencies

- Deploy PRD: `sdkwork-deployments/docs/product/prd/PRD-cloud-site-publishing-platform.md`
- Drive architecture: [TECH-website-directory-resource-provider.md](../../architecture/tech/TECH-website-directory-resource-provider.md)
- Drive decision: [ADR-20260721 Website Space Directory Resource](../../architecture/decisions/ADR-20260721-website-space-directory-resource.md)
