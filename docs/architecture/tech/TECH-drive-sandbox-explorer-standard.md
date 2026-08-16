# Drive Sandbox Explorer Standard

Status: active
Owner: SDKWork Drive maintainers
Updated: 2026-07-16
Specs: `DRIVE_SPEC.md`, `API_SPEC.md`, `SECURITY_SPEC.md`, `APP_PC_REACT_UI_SPEC.md`

## Purpose

Drive owns the reusable explorer for server-authorized file spaces. An end-user
browser or desktop renderer selects a Drive sandbox by opaque identifier and
navigates its logical node hierarchy. End-user app and open APIs never receive a
deployment-host physical path. The protected backend-admin configuration API is
the only HTTP surface allowed to return a canonical `providerRootRef` to an
organization-scoped operator with `drive.sandboxes.admin` privilege.

## Security Model

- A sandbox root is configured by an operator and is the only physical root a
  server-side provider may resolve for that sandbox.
- `defaultAccess` defaults to `full`, but is configuration metadata rather than an
  authorization rule. Every end-user access decision requires an explicit grant;
  no tenant-wide, organization-wide, or user access is inferred from the default.
- An authorized full grant enables browse, read, write, create, rename, move,
  delete, upload, download, and search. A read-only grant or volume lifecycle may
  narrow these operations; access never crosses a sandbox root.
- App and open APIs return `sandboxId`, node identifiers, revisions, and
  root-relative logical paths only. Absolute paths, credentials, provider
  configuration, and host topology remain private provider state.
- Backend-admin may return `providerRootRef` only on sandbox volume responses. It
  must never enter app/open API DTOs, structured logs, error details, or audit facts.
- Every mutation must authorize the subject and sandbox before filesystem access,
  canonicalize the requested target, reject traversal and symlink escape, perform a
  revision check when supplied, and append an audit fact.
- Browser UI permission hints are advisory. Drive service authorization is decisive.

## Backend Administration

The Drive backend API owns `/backend/v3/api/drive/sandbox_volumes` and nested
`grants` resources. Volume operations support list, create, retrieve, update, and delete;
updates transition lifecycle status through `active`, `read_only`, and `disabled`
with optimistic `expectedVersion` checks. Deleting a volume removes its configuration,
grants, and operation records but never deletes provider files. Grant operations support
list, create, retrieve, update, and delete. Every management query and mutation is
scoped to both the verified tenant and the active organization; resources owned by a
different organization are returned as not found.

Volume creation defaults `providerKind` to `local_filesystem` and `defaultAccess`
to `full`. Unless explicitly disabled, it creates a separate current verified-user
grant with `full` access; callers may override that initial grant to `read_only`.
Local filesystem roots must already exist, must be directories, and are canonicalized
server-side before transaction commit. `local_filesystem` is the only advertised and
accepted sandbox provider until another provider has a complete runtime adapter;
`s3` and `opendal` are not configurable placeholders. User and organization grants are supported.
Workspace and role grants fail validation until an authoritative membership resolver
is integrated. Every successful write and automatically created initial grant appends
a tenant-scoped `dr_drive_audit_event` in the same database transaction. Volume and
grant mutations use the framework `auth-critical` rate-limit tier in both OpenAPI and
the generated runtime route manifest.

## Runtime Model

`@sdkwork/drive-pc-sandbox-explorer` is a domain capability embed package. It exports
`SandboxExplorerView`, `SandboxDirectoryPickerView`, and
`configureDriveSandboxExplorerRuntime`. Hosts inject a narrow `SandboxExplorerPort`
created from their authenticated `@sdkwork/drive-app-sdk` client. The embed package
does not construct SDK clients, read tokens, set headers, or access physical paths.

Tauri uses the same remote sandbox port for server workspaces. A native local filesystem
adapter remains a device-local capability; it must not be represented as a server sandbox
or serialized to a remote project record.

## Consumer Integration

BirdCoder and other products bind their domain object to a selected Drive sandbox entry
using `sandboxId`, `entryId`, and `rootRelativePath`. They do not duplicate Drive explorer
components or import Drive private source files. Project-specific import, execution, and
Git semantics remain owned by the consuming product.

## Required Verification

- API operation, response-envelope, and pagination checks pass for app and
  backend-admin sandbox routes.
- OpenAPI remains the SDK authority. Generated SDK output is never hand-edited;
  generation occurs only in the governed SDK generation workflow.
- Tests cover authorization, tenant and organization isolation, traversal rejection, symlink escape,
  pagination, local-root canonicalization, optimistic revision conflicts, explicit
  grant semantics, audit recording, path leakage, and browser/Tauri adapter parity.
- `pnpm --filter @sdkwork/drive-pc-sandbox-explorer typecheck` passes after dependencies
  are installed from a lockfile-aligned workspace.
