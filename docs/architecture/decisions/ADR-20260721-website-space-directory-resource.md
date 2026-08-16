# ADR-20260721 Website Space Directory Resource

Status: accepted
Requirement: REQ-2026-0004
Owner: SDKWork Drive maintainers
Date: 2026-07-22
Specs: ARCHITECTURE_DECISION_SPEC.md, DATABASE_SPEC.md, DRIVE_SPEC.md, API_SPEC.md,
SDK_SPEC.md, SECURITY_SPEC.md, PERFORMANCE_SPEC.md, MIGRATION_SPEC.md

## Context

Static website output is a directory graph, not a single artifact. One Drive Space should represent
one website project, while a selected folder behaves as the document root. Ordinary Drive Spaces
must not become public by accident. Framework bundles also need an atomic tree switch because
uploading `index.html` and hashed assets independently can create a broken mixed deployment.

At decision time, the Drive Space type enum had no `website`, and its owner/type uniqueness rule was
incompatible with one owner having multiple website projects. The accepted baseline now materializes
the type-aware model and stable WebsiteRoot provider contract; the remaining implementation work is
tracked separately from this decision.

## Decision

1. Add `website` as an explicit Drive Space type and treat it as multi-instance per owner within
   entitlements. Preserve separately declared singleton invariants for Space types that require them.
2. Space is the project, ownership, security, storage, and quota boundary. Each `WebsiteRoot` has an
   immutable selector: `SPACE_ROOT` for the complete eligible user-content tree or `FOLDER` for one
   explicit same-Space descendant folder. Reserved/internal namespaces are excluded in both modes.
3. Add a Drive-owned stable `WebsiteRoot` handle with selector, `LIVE_TREE` or
   `ATOMIC_GENERATION` content mode, one active folder node, and monotonic generation. Deploy
   `DRIVE_DIRECTORY` resources reference only the stable WebsiteRoot UUID.
4. Creating a Website Space/WebsiteRoot grants eligibility only. Publication requires Deploy
   resource, Variant, Mount, Binding, and active revision.
5. Ordinary file writes use existing Drive node/version/upload lifecycle and produce path/version
   events. They do not create a website Release.
6. `ATOMIC_SYNC` stages a normal Drive folder tree, validates a complete manifest, and switches the
   WebsiteRoot active node/generation in one transaction. Prior roots follow Drive retention policy.
7. The resource provider resolves normalized paths only under the active root and returns typed
   public-eligible metadata/version/streams to authenticated Deploy/Web Server consumers. It never
   exposes object-storage identity as business state.
8. V1 rejects shortcuts in public WebsiteRoot trees to eliminate cross-root escape ambiguity.
9. Domain, URL, Variant, TLS, cache/header, request analytics, and delivery metering remain Deploy/Web
   Server concerns. Drive owns storage/sync/version usage only.
10. Every Website Space provisions a default `SPACE_ROOT` in `LIVE_TREE` mode. Additional folder roots are explicit and
    entitlement-controlled. The same selector is idempotent and the same WebsiteRoot may be reused
    by multiple Sites/Variants/Mounts.
11. Changing the selected Space/folder root creates or reuses another WebsiteRoot and updates Deploy
    configuration, producing a SiteRevision. File changes and atomic generation switches behind an
    existing root remain provider lifecycle and produce only root/path events.

## Alternatives

- Expose any Space/folder with a public flag: rejected because it weakens explicit eligibility and
  makes ordinary collaboration storage an accidental hosting surface.
- Point Deploy directly at a folder node and update it for each atomic bundle: rejected because every
  content switch would become a Site configuration revision.
- Copy bundle files into the existing root one by one: rejected because readers can observe mixed
  generations and rollback is expensive.
- Zip every bundle as one file: rejected because Web Server would need extraction/build ownership and
  would lose ordinary directory WYSIWYG behavior.
- Persist object-store origin URLs: rejected by Drive ownership, security, portability, and expiry
  requirements.

## Consequences

- Drive now carries the reviewed enum/constraint/index baseline, WebsiteRoot aggregate, App/Internal
  API authorities, and generated SDK/provider surfaces in PostgreSQL and SQLite.
- The WebsiteSync schema is reserved, while its command API, fenced worker, rollback, cleanup, and
  operational workflows remain implementation gates.
- Temporary staging storage must be reserved, metered, expired, and cleaned.
- Deploy/Web Server receive stable root generation events and can cache safely without a SiteRevision.
- Existing Space owner/type uniqueness must become type-aware before multiple Website Spaces launch.

## Verification

- database migration and dual-engine contract tests;
- Drive type/domain/API/SDK/UI alignment checks;
- atomic switch/concurrency/crash/rollback/cleanup tests;
- provider root confinement and tenant isolation tests;
- event/freshness/load/large-tree tests;
- absence of object key/presigned/raw storage consumers.

## Supersedes / Superseded By

This decision supersedes any implied design that treats every Drive Space as publishable or requires
a Deploy Release/SiteRevision for each Website Space file change. It is accepted and materialized in
the Drive schema and owner contracts; commercial activation remains gated by the outstanding atomic
sync, UI, operations, and production-evidence work recorded in the PRD and technical architecture.
