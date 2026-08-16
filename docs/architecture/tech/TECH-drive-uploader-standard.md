> Owner: SDKWork maintainers

# SDKWork Drive Uploader Standard

## Purpose

Drive Uploader is the shared upload capability for SDKWork Drive. It is not an
application-local widget or private upload helper. Every client and server
workflow that uploads content into Drive must use the same uploader contracts so
space ownership, resumable upload state, retention, deletion logs, audit events,
and usage statistics stay consistent.

## Required Entry Points

Drive Uploader must expose three entry points backed by one business core:

```text
crates/sdkwork-drive-workspace-service::uploader
  Rust workspace service/component for server-side callers.

crates/sdkwork-routes-drive-app-api /app/v3/api/drive/uploader/*
  HTTP App API surface for browser, desktop, mobile, and other remote app
  clients.

sdks/sdkwork-drive-app-sdk client.uploader.*
  App SDK high-level API for client applications.
```

The App API is an adapter over the workspace service. The App SDK is an adapter
over the App API. Server-side Rust callers should import the workspace uploader
component directly instead of calling the App API through HTTP.

## Hard Rules

- Client applications must call `client.uploader.*` for Drive uploads.
- Server-side Rust services must call `sdkwork_drive_workspace_service::uploader` for
  direct Drive uploads.
- App API route handlers must not duplicate uploader business logic.
- Generated SDK output must not be hand-edited. High-level TypeScript uploader
  code belongs in the App SDK composed layer.
- Applications must not resolve Upload spaces, build object keys, create upload
  sessions, maintain part facts, or implement Drive retention cleanup on their
  own.
- All uploader-created content must be attributable by tenant, organization,
  user or anonymous actor, app, resource, upload profile, content type, and
  space.

## Space Ownership

The business display name is `Upload`. The persisted Drive space type remains
the existing `app_upload` value.

Anonymous uploads use an app-owned Upload space:

```text
tenant_id = resolved tenant
owner_subject_type = app
owner_subject_id = app:{appId}:anonymous
space_type = app_upload
display_name = Upload
```

Logged-in user uploads use the user's Upload space:

```text
tenant_id = current tenant
owner_subject_type = user
owner_subject_id = userId
space_type = app_upload
display_name = Upload
```

Organization attribution is recorded on uploader metadata. It does not require
placing every logged-in upload into an organization-owned space.

Tenant, organization, authenticated user, anonymous actor, operator, and app
identity are ambient caller facts resolved from the verified request context.
They are not client-writable fields on generated or composed uploader requests.
Client services provide only business attribution such as resource type/id,
scene, source, profile, target, and retention. `shareToken` remains an explicit
target-folder authorization credential; it is not an anonymous actor identity.

Explicit target-space uploads are supported for application workflows that need
to drop files into an existing Drive folder. The uploader must validate the
target space is active before creating a node. Logged-in users may upload when
they own the space or hold an active `writer`/`owner` permission on the target
folder. Anonymous or external uploads to an explicit target folder require an
active writer share link token; the raw `shareToken` is accepted only by the
prepare request, is hashed before lookup, and must never be stored on uploader
metadata or returned by App API/SDK responses.

## Upload Profiles

`client.uploader.upload()` is the generic upload entry. Type-specific helpers
select an upload profile and then use the same upload pipeline:

```text
client.uploader.upload()
client.uploader.uploadByProfile()
client.uploader.uploadVideo()
client.uploader.uploadImage()
client.uploader.uploadAudio()
client.uploader.uploadDocument()
client.uploader.uploadArchive()
client.uploader.uploadText()
```

Standard profile codes:

```text
generic
video
image
audio
document
archive
text
dataset
attachment
avatar
thumbnail
```

Profile selection controls validation, default retention, chunk size,
concurrency, checksum requirements, and optional post-processing hints. It does
not create a separate upload implementation.

## Resumable Upload

Resumable upload is a first-class uploader capability. The server stores upload
task and upload part facts; the SDK may also keep local state for fast recovery.
Server state is authoritative.

Required flow:

```text
1. SDK computes a file fingerprint.
2. SDK calls prepare or resume.
3. Workspace service resolves/creates the Upload space and upload task.
4. Workspace service returns existing uploaded parts.
5. SDK uploads missing parts only.
6. SDK reports each uploaded part.
7. Workspace service validates parts before completion.
8. Workspace service commits Drive storage object metadata and uploader metadata.
```

## Retention And Cleanup

Uploader content supports:

```text
retention_mode = long_term
retention_expires_at_epoch_ms = null
```

or:

```text
retention_mode = temporary
retention_expires_at_epoch_ms = now + ttl
cleanup_action = soft_delete | hard_delete
hard_delete_after_epoch_ms = optional second-stage deletion time
```

Expired completed uploader content is handled by maintenance jobs:

```text
expired_upload_content_sweep
abandoned_upload_task_sweep
```

Every automatic soft delete or hard delete must write both an audit event and a
file-sensitive-operation record with the pre-delete object snapshot.

## Server-Side Rust Contract

The workspace service crate must export uploader domain types and services for direct
server use:

```rust
use sdkwork_drive_workspace_service::uploader::{
    DriveUploaderService, PrepareUploaderUploadCommand, UploadBytesCommand,
};
```

Server-side helpers such as `upload_video_bytes` are profile shortcuts. They
must delegate to the same profile-driven core as generic uploads.

## Client SDK Contract

The TypeScript App SDK must expose `client.uploader.*` from its composed layer.
The high-level uploader may use generated operations, state stores, part
planners, and queue helpers, but it must not hand-code business HTTP routes
outside the SDK boundary or restore ambient identity fields removed from the
generated request contracts.

## Persistence Model

The workspace service owns these uploader tables:

```text
dr_drive_upload_item
dr_drive_upload_part
dr_drive_file_sensitive_operation
```

`dr_drive_upload_item` is the upload task/content lifecycle fact table.
`dr_drive_upload_part` is the resumable multipart fact table.
`dr_drive_file_sensitive_operation` records sensitive file operations such as
upload completion, soft delete, hard delete, restore, share changes, permission
changes, and download grants. Delete operations preserve pre-delete snapshots,
including hard-delete snapshots that would otherwise disappear from object
metadata.

## Verification

Required verification for uploader implementation:

```powershell
cargo test -p sdkwork-drive-workspace-service uploader_service sqlite_schema_contract maintenance_service
cargo test -p sdkwork-routes-drive-app-api uploader
node tools/drive_sdk_generate.mjs --check --language typescript
pnpm --dir apps/sdkwork-drive-pc test
pnpm --dir apps/sdkwork-drive-pc typecheck
```
