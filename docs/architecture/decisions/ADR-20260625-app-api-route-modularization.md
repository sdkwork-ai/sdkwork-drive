# ADR-20260625: Incremental App-API Route Modularization

Status: accepted
Owner: SDKWork maintainers
Updated: 2026-06-25

## Context

`crates/sdkwork-routes-drive-app-api/src/routes.rs` grew into a monolithic router assembly file with thousands of lines and direct SQL in handlers. This violates WEB_BACKEND layering goals and slows review, testing, and security hardening.

## Decision

1. Extract shared route support into focused modules (`route_change`, domain handler modules) instead of growing `routes.rs`.
2. Phase 1 (done): move share-link HTTP handlers to `share_link_handlers.rs` and change recording to `route_change.rs`.
3. Phase 2 (done): extract comment and permission handler groups to `comment_handlers.rs` and `permission_handlers.rs`; move `resolve_node_path` to `node_repository.rs`.
4. Phase 3 (done): extract watch handler group to `watch_handlers.rs`; move `insert_watch_channel` to `watch_repository.rs`.
5. Phase 4 (done): extract quota (`quota_handlers.rs`), trash (`trash_handlers.rs`), and library view handlers (`library_handlers.rs`); move node lifecycle transitions to `node_lifecycle.rs` and git-repository root validation to `space_repository.rs`.
6. Phase 5 (done): extract change feed (`change_handlers.rs`), search (`search_handlers.rs`), file versions (`version_handlers.rs`), and node metadata/labels (`metadata_handlers.rs`).
7. Phase 6 (done): extract space (`space_handlers.rs`), node CRUD/archive (`node_handlers.rs` + `node_support.rs`), upload (`upload_handlers.rs` + `upload_support.rs`), and download (`download_handlers.rs`); `routes.rs` retains router wiring only.
8. Phase 7 (done): trim duplicated import surfaces via `cargo fix` and manual cleanup; resolve handler-module visibility warnings; `cargo check -p sdkwork-routes-drive-app-api` is warning-free.
9. Phase 8 (in progress): delegate remaining direct SQL in handler modules to workspace-service application commands. Batch 8a (done): `space_lifecycle_service` (`bootstrap_team_space_creator_access`, `retire_space_contents`) and `change_feed_service` (`list_changes`, `query_start_page_token`) replace SQL in `space_handlers.rs` and `change_handlers.rs`.
10. Phase 8 follow-up (next): search, library, trash, node, version, metadata, permission, comment, share-link, watch, upload, and download handler SQL.
11. Keep `routes.rs` responsible for router wiring, middleware ordering, and builder exports only.

## Consequences

- Handler modules remain Axum-specific thin adapters; domain logic stays in workspace-service crates.
- Integration tests and alignment checks assert extracted modules exist to prevent re-monolithization.
- Full elimination of direct SQL in app-api remains a tracked follow-up, not a single risky rewrite.

## Verification

- `cargo check -p sdkwork-routes-drive-app-api`
- `cargo test -p sdkwork-routes-drive-app-api --test share_link_cross_api_e2e`
- `pnpm check:architecture-alignment`
- `node tests/integration/drive-alignment.integration.test.mjs`
