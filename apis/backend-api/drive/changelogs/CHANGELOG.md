# Drive Backend API Changelog

## Unreleased

- Add `files.admin.list` (`GET /backend/v3/api/drive/files`): a tenant-scoped,
  metadata-only admin file listing with `space_id`, `space_type`, `node_type`,
  `lifecycle_status`, `content_type_group`, `keyword`, and `owner_subject_id`
  filters, cursor pagination, and an ancestor-resolved `filePath`. The projection
  never exposes object keys, storage provider ids, or credentials. An omitted
  `lifecycle_status` means live (`active`) nodes.
- Require `drive.files.admin` for `files.admin.list`; `drive.spaces.admin`
  continues to guard `spaces.admin.list`.
- The Drive admin storage contribution
  (`assemble_backend_admin_storage_contribution_with_pool`) now mounts the
  workspace governance routes next to `/drive/storage/*`, and
  `backend_admin_storage_route_manifest` projects both route families so
  same-origin hosts (the Cloud Router storage center) compose one admin
  contribution without `404 route-not-in-manifest`.

## 0.2.0 - 2026-07-16

- Add organization-scoped sandbox volume list/create/retrieve/update/delete operations.
- Add explicit user and organization sandbox grant list/create/retrieve/update/delete operations.
- Define server-canonical local filesystem roots, optimistic volume lifecycle updates,
  strict SDKWork response envelopes, standard pagination, and RFC 9457 errors.
- Keep `providerRootRef` limited to protected backend-admin sandbox responses; app/open
  APIs, audit facts, and logs remain path-free.
- Expose only the runtime-backed `local_filesystem` provider; `s3` and `opendal` are
  rejected until a working sandbox provider adapter is registered.
- Apply the framework `auth-critical` rate-limit tier to sandbox volume and grant
  mutations.

## 0.1.0 — 2026-06-25

- Mark legacy `/backend/v3/api/drive/storage_providers*` and `storage_provider_bindings/default` operations as **deprecated**; use `drive-admin-storage-api` canonical `/backend/v3/api/drive/storage/*` routes instead.
- Document tenant quota policy update (`PUT /backend/v3/api/drive/quotas`) in the contract authority.

## 0.1.0 — 2026-06-24

- Initial SDKWork Drive backend API surface under `/backend/v3/api`.
- Storage provider administration, maintenance sweeps, quotas, labels, and audit APIs.
