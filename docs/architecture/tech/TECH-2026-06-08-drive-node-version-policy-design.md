> Owner: SDKWork maintainers

## Goal

Add Drive-owned logical node versions and version policy so SDKWork Notes and future business domains can store durable file content in Drive without duplicating revision tables.

## Context

Drive already stores concrete bytes in `dr_drive_storage_object` with `version_no`. Existing App API routes expose `/app/v3/api/drive/nodes/{nodeId}/versions` by reading that storage-object table directly. That is useful for basic file history, but it does not provide a stable logical version fact that can carry product attribution, restore lineage, AI provenance, policy decisions, or future retention behavior.

Notes must not create `notes_note_revision` or any app-local object storage/version table. Notes should store `drive_space_id`, `drive_node_id`, `drive_uri`, `current_drive_version_id`, and `current_drive_version_no`, while Drive owns version lifecycle.

## Design

Drive will keep `dr_drive_storage_object` as the byte/object fact and add `dr_drive_node_version` as the logical file-version fact. A node version references one storage object when bytes are Drive-owned. The logical version id is the id that Notes and other domains may persist.

Drive will also add policy tables:

- `dr_drive_space_version_policy`: default versioning behavior for a space.
- `dr_drive_node_version_policy`: optional node-level override.

The effective policy is node override, then space policy, then Drive default. The first phase stores policy facts and creates logical versions for upload/copy/extract flows, while preserving existing API route paths and response compatibility.

## Tables

`dr_drive_node_version`:

- `id`, `tenant_id`, `space_id`, `node_id`, `version_no`
- `storage_object_id`
- `content_type`, `content_length`, `checksum_sha256_hex`
- `version_kind`: `auto`, `manual`, `restore`, `import`, `ai_generated`, `system`
- `version_label`, `change_source`, `change_summary`
- `restored_from_version_id`
- `app_id`, `app_resource_type`, `app_resource_id`
- `scene`, `source`
- `lifecycle_status`
- `created_by`, `updated_by`, `created_at`, `updated_at`

`dr_drive_space_version_policy`:

- `id`, `tenant_id`, `space_id`
- `versioning_enabled`
- `default_version_kind`
- `retention_mode`
- `max_versions`
- `keep_deleted_versions`
- `created_by`, `updated_by`, `created_at`, `updated_at`

`dr_drive_node_version_policy`:

- `id`, `tenant_id`, `space_id`, `node_id`
- same policy fields as the space policy table
- `created_by`, `updated_by`, `created_at`, `updated_at`

## Runtime Behavior

Upload completion inserts both `dr_drive_storage_object` and `dr_drive_node_version` in the same request path. The logical version id is derived from the storage object id for deterministic first-phase compatibility.

Existing version list/get/delete/restore routes read logical node versions when present. During migration they fall back to `dr_drive_storage_object` facts when no logical rows exist.

Delete and restore update both the logical version row and the referenced storage object row when the logical row exists. Existing storage-object-only data remains operable.

## Notes Integration Contract

Notes will create or use a Drive space with a Notes-specific Drive-owned space type/profile when the Drive schema supports it, or an `app_upload` space bound to `sdkwork-notes` during the first implementation phase. Page content is saved as a Drive file node. Notes metadata points to the current Drive version, not to any Notes-local revision table.

## Non-Goals

This phase does not regenerate Drive SDK output, change the public route prefix, or add a Notes-specific table inside Drive. Future work can add explicit policy management endpoints and OpenAPI response fields once the generic storage facts are stable.

## Verification

The implementation must add schema contract tests for SQLite and PostgreSQL DDL text, product store/service tests for node version insertion and fallback reads, App API route tests for logical version ids, and formatting/workspace tests where feasible.

