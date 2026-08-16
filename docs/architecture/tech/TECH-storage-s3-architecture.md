> Owner: SDKWork maintainers

# SDKWork Drive S3 Storage Architecture

## Scope

This document defines the current storage model for `sdkwork-drive`. It covers S3-compatible storage, bucket and object management, provider binding rules, admin storage modules, optional plugin adapters, and repository directory responsibilities.

The production implementation uses the official Rust AWS S3 SDK through `aws-sdk-s3`. The Drive codebase must not hand-write S3 HTTP signing, multipart XML, presigned URL generation, or provider-specific protocol details. The OpenDAL S3 plugin is optional and default disabled; it is available as a filesystem-style object adapter for deployments that explicitly wire it in.

## Core Rules

1. Drive metadata and object bytes are separate.
   - Drive nodes, versions, upload sessions, and storage object metadata live in SQL.
   - File bytes live in the configured object store.

2. `dr_drive_storage_provider` is the physical storage configuration.
   - `provider_kind` identifies the adapter family, such as `s3_compatible`, `aliyun_oss`, `tencent_cos`, `huawei_obs`, `volcengine_tos`, `google_cloud_storage`, or `custom:<vendor>`. AWS S3 uses `s3_compatible` with the AWS endpoint/region/bucket profile rather than a separate `aws_s3` database enum.
   - `endpoint_url`, `region`, `bucket`, `path_style`, `strict_tls`, and `credential_ref` configure the provider.
   - `endpoint_url` and `bucket` must already be normalized when they reach a storage adapter. The adapters reject leading/trailing whitespace rather than silently trimming configuration values.
   - `strict_tls` is provider-level transport policy. HTTPS endpoints default to `true`; private HTTP endpoints such as MinIO default to `false`; explicitly setting `strict_tls=true` requires an HTTPS endpoint.
   - Credentials are referenced, not exposed as raw API facts. Supported `credential_ref` forms are `plain:<accessKeyId>:<secretAccessKey>[:<sessionToken>]`, `env:<accessKeyEnv>:<secretKeyEnv>[:<sessionTokenEnv>]`, `secret:<ref>`, `kms:<ref>`, and `vault:<ref>`.
   - `secret:`, `kms:`, and `vault:` references are resolved by a host/application secret materialization layer before the storage adapter is built. The storage runtime reads `SDKWORK_DRIVE_STORAGE_CREDENTIAL__<sanitized_ref>__ACCESS_KEY_ID`, `SDKWORK_DRIVE_STORAGE_CREDENTIAL__<sanitized_ref>__SECRET_ACCESS_KEY`, and optional `SDKWORK_DRIVE_STORAGE_CREDENTIAL__<sanitized_ref>__SESSION_TOKEN`; `<sanitized_ref>` keeps ASCII letters/digits and maps other characters to `_`.
   - Multiple provider accounts are supported by storing one row per account/bucket/profile combination.

3. `dr_drive_storage_provider_binding` maps Drive business scope to storage.
   - Tenant default binding: `tenant_id + space_id=NULL + purpose=primary`.
   - Space default binding: `tenant_id + space_id + purpose=primary`.
   - Space binding wins over tenant binding.
   - Each active binding owns `storage_root_prefix`, the object-store mount root for that tenant or space.
   - Default tenant root: `sdkwork-drive/v1/tenants/{tenantId}`.
   - Default space root: `sdkwork-drive/v1/tenants/{tenantId}/spaces/{spaceId}`.
   - App, app-upload, personal, team, AI-generated, and knowledge-base spaces can each bind to different providers.

4. Storage administration is backend/admin-storage only.
   - App clients upload, download, browse, and package Drive files through Drive business workflows.
   - Backend admins and dedicated admin modules manage provider accounts, provider bindings, bucket existence, and low-level objects.
   - Provider, bucket, object, and binding administration is not part of normal end-user file workflows.

5. Object keys are internal.
   - App APIs should avoid exposing object keys as durable business identifiers.
   - Backend object-management APIs may expose object keys because they are administrative storage tools.
   - App upload workflows must generate object keys through the Drive storage key service.
   - The stored key is `{storageRootPrefix}/{standardContentKey}`. The standard content key remains `sdkwork-drive/v1/t/{tenantShard}/tenants/{tenantId}/spaces/{spaceId}/nodes/n/{nodeShard}/{nodeId}/versions/{versionNo}/{objectId}/content`.
   - Completion/version validation must parse and validate the standard content key suffix even when it is nested below a binding root.
   - Client-supplied object keys are compatibility inputs only and must not control physical storage layout.

6. Upload sessions store both Drive id and provider multipart id.
   - `dr_drive_upload_session.id` is the Drive business session id.
   - `dr_drive_upload_session.storage_upload_id` is the provider-side multipart upload id.
   - Presign, complete, and abort must use `storage_upload_id`.
   - Client-supplied `uploadId` is optional and, when present, must match `storage_upload_id`.

## API Surface

### App API

App API focuses on Drive workflows:

- Create upload session.
- Presign upload part.
- Complete upload session.
- Abort upload session.
- Create and resolve download URLs.

App API must not expose storage provider, bucket, object, or provider-binding administration routes. That includes provider CRUD/test/capabilities/activation/credential rotation, bucket head/create/delete/list, low-level object list/head/delete/copy, and default provider binding management. App upload and download workflows may resolve the active provider/binding internally, but the App OpenAPI and App SDK surface remain user-workflow only.

The app upload session response includes `storageUploadId` so clients can correlate presign and complete calls with the object-store multipart upload id when needed.

The app upload session response also includes `objectKey` for diagnostics and low-level correlation. It is an internal physical key under the active binding root and is not a user-facing path.

### Admin Storage API

`crates/sdkwork-routes-storage-backend-api` owns storage administration and uses `/backend/v3/api/drive/storage/...` routes:

- `GET|POST /backend/v3/api/drive/storage/providers`
- `GET|PATCH|DELETE /backend/v3/api/drive/storage/providers/{providerId}`
- `GET /backend/v3/api/drive/storage/providers/{providerId}/capabilities`
- `POST /backend/v3/api/drive/storage/providers/{providerId}/test`
- `POST /backend/v3/api/drive/storage/providers/{providerId}/activate`
- `POST /backend/v3/api/drive/storage/providers/{providerId}/deactivate`
- `POST /backend/v3/api/drive/storage/providers/{providerId}/credentials/rotate`
- `GET /backend/v3/api/drive/storage/providers/{providerId}/buckets`
- `GET|PUT|DELETE /backend/v3/api/drive/storage/providers/{providerId}/bucket`
- `GET /backend/v3/api/drive/storage/providers/{providerId}/objects`
- `GET|DELETE /backend/v3/api/drive/storage/providers/{providerId}/objects/{objectKey...}`
- `POST /backend/v3/api/drive/storage/providers/{providerId}/objects/copy`
- `GET /backend/v3/api/drive/storage/bindings`
- `GET|PUT|DELETE /backend/v3/api/drive/storage/bindings/default`

It is a standalone runtime service in the local Drive launch plan and binds `127.0.0.1:18083` by default. The same crate also exposes router builders so a host application can embed the admin-storage module without duplicating Drive storage logic. Compatibility alias `/admin/v3/api/drive/storage/*` mirrors the canonical `/backend/v3/api/drive/storage/*` handlers for legacy hosts; new clients must use the canonical prefix and `@sdkwork/drive-admin-storage-sdk`.

Admin-storage runtime routes under `/backend/v3/api/drive/storage/*` require the same dual-token contract as app and backend APIs. Context is token-derived; projection headers are forbidden.
`/healthz` is the only public admin-storage runtime route. Embedded hosts may use explicit
no-IAM router builders only for isolated business tests when tokens are supplied through test
fixtures rather than forged context headers.

`GET /storage/providers/{providerId}/buckets` lists the buckets visible to the configured S3 account and marks the currently configured provider bucket, which lets administrators discover and choose the correct bucket before mounting a provider to tenant or space storage. `GET /storage/bindings` requires `tenantId` and can filter by `spaceId`, `providerId`, and `lifecycleStatus` so administrators can inspect which provider account is mounted to each tenant or space. `PUT /storage/bindings/default` accepts optional `storageRootPrefix`; when omitted, Drive derives the tenant or space default root. `DELETE /storage/bindings/default` soft-deletes the tenant or space primary binding and records `storage_provider_binding.default_deleted`.

All admin-storage mutation routes must receive a real non-empty `operatorId` and record a Drive audit event. Provider and binding mutations carry `operatorId` in the JSON body except provider deletion, bucket create/delete, object delete, and default binding deletion, which use the required `operatorId` query parameter. Object copy carries `operatorId` in the request body. Low-level bucket and object audit events use the storage provider id as the audit resource id; object keys remain operational parameters, not audit resource identifiers.

Future backend-admin route modules must follow the `crates/sdkwork-routes-<capability>-backend-api` naming pattern, where `<capability>` is a bounded business capability such as `storage`, `audit`, `workspace`, `policy`, or `repair`. Each module must expose only its own backend-admin surface and reuse Drive service crates/contracts instead of duplicating Drive business logic.

The corresponding generated SDK family is `sdks/sdkwork-drive-admin-storage-sdk`, not `sdks/sdkwork-drive-admin-storage-api`. Its authority is `sdkwork-drive.admin.storage`, its OpenAPI input is `apis/backend-api/drive/drive-admin-storage-api.openapi.json`, and it exposes the storage administration operations through a dedicated client surface. Because the canonical `sdkwork-v3` backend generator profile is reserved for `/backend/v3/api`, this SDK uses generator type `custom` with Drive-local profile metadata `sdkwork-drive-admin-storage-v3`; it still declares dual-token security and appbase backend IAM dependency metadata.

## Storage Contract

`crates/sdkwork-drive-storage-contract` is the boundary between Drive and storage providers.

It owns provider-neutral types and traits:

- Object data: put, head, delete, range read.
- Multipart lifecycle: create, presign part, complete, abort.
- Presigned download.
- Bucket management: head, create, delete.
- Object management: list, copy.

Provider SDK types must not cross this boundary.

## S3 Adapter

`crates/sdkwork-drive-storage-s3` is the production S3-compatible adapter.

Implementation rules:

- Use `aws-sdk-s3::Client` for all S3 operations.
- Use AWS SDK presigning APIs for upload part and download URLs.
- Support endpoint override and path-style addressing for MinIO and compatible providers.
- Use the persisted provider `strict_tls` value when constructing the S3 client instead of relying on process-wide environment defaults.
- Infer provider profiles from `provider_kind` and endpoint where possible.
- Preserve explicit provider kinds such as `tencent_cos`, `huawei_obs`, and `volcengine_tos` in the storage contract instead of collapsing them to generic `s3_compatible`.
- Treat `http://` endpoints as non-strict TLS by default for MinIO/private deployments; `https://` endpoints remain strict by default.

Supported profiles:

- AWS S3
- MinIO
- Cloudflare R2
- Aliyun OSS
- Tencent COS
- Huawei OBS
- Google Cloud Storage S3-compatible mode
- Backblaze B2 S3-compatible mode
- Generic S3-compatible providers

## OpenDAL S3 Plugin

`crates/sdkwork-drive-storage-opendal` integrates the Rust OpenDAL S3 service as an optional plugin. It is a workspace member for compilation and testing, but app/open/backend API crates must not depend on it by default.

Rules:

- OpenDAL is opt-in plugin infrastructure, not the default production S3 path.
- The admin storage crate exposes the optional Cargo feature `opendal-s3-plugin`; default builds do not enable it.
- Runtime selection is explicit through `SDKWORK_DRIVE_ADMIN_STORAGE_OBJECT_STORE_ADAPTER`. The default value is `aws_sdk_s3`; `opendal_s3` is accepted only when the admin storage module is built with the `opendal-s3-plugin` feature.
- Selecting `opendal_s3` affects filesystem-style object operations in `crates/sdkwork-routes-storage-backend-api`; app/open/backend APIs and the Drive workspace service continue to use the full AWS SDK S3 adapter.
- OpenDAL configuration must receive and enforce the persisted provider `strict_tls` value; it must reject `strict_tls=true` for `http://` endpoints just like the full AWS SDK S3 adapter.
- The plugin adapts OpenDAL S3 to `DriveObjectStore` for filesystem-style object operations: put, head, delete, range read, list, copy, and presigned download where supported by OpenDAL.
- It does not fake Drive multipart semantics. Drive multipart upload, upload-part presign, complete, and abort remain the responsibility of `sdkwork-drive-storage-s3`.
- Bucket administration remains on the AWS SDK S3 adapter unless an explicit future plugin capability is added and tested. This includes `HeadBucket`, `ListBuckets`, `CreateBucket`, `DeleteBucket`, and provider test health checks.
- The OpenDAL operator is bound to one bucket/root. Cross-bucket administrative operations should use one provider per bucket or the AWS SDK S3 adapter.

## Directory Responsibilities

The storage code should remain organized as follows:

```text
crates/
  sdkwork-drive-storage-contract/
    src/lib.rs
    src/types.rs
  sdkwork-drive-storage-s3/
    src/config.rs
    src/s3_store.rs
  sdkwork-drive-storage-opendal/
    src/config.rs
    src/opendal_store.rs
  sdkwork-drive-storage-local/
    src/local_store.rs

crates/
  sdkwork-drive-workspace-service/
    src/domain/storage_provider.rs
    src/domain/upload.rs
    src/application/storage_provider_service.rs
    src/application/storage_key_service.rs
    src/application/upload_service.rs
    src/ports/storage_provider_store.rs
    src/ports/storage_object_store.rs
    src/infrastructure/sql/storage_provider_store.rs
    src/infrastructure/sql/upload_session_store.rs
    src/infrastructure/sql/storage_object_store.rs
  sdkwork-routes-drive-app-api/
    src/lib.rs
  sdkwork-routes-drive-backend-api/
    src/lib.rs
  sdkwork-routes-storage-backend-api/
    src/lib.rs
    src/main.rs
  sdkwork-routes-<capability>-backend-api/
    src/lib.rs
    src/main.rs

apis/
  openapi/
    drive-app-api.openapi.json
    drive-backend-api.openapi.json
    drive-admin-storage-api.openapi.json

sdks/
  sdkwork-drive-admin-storage-sdk/
    bin/generate-sdk.mjs
    sdk-manifest.json
    sdkwork-drive-admin-storage-sdk-typescript/
    sdkwork-drive-admin-storage-sdk-rust/
    sdkwork-drive-admin-storage-sdk-java/
    sdkwork-drive-admin-storage-sdk-python/
    sdkwork-drive-admin-storage-sdk-go/
```

Rules:

- Storage-provider SDK construction stays in adapter/helper code, not scattered across routes.
- Object-key generation stays in product application code, not in API routes or provider SDK adapters.
- Product domain owns metadata and business rules.
- App API owns user file workflows.
- Backend API owns administrative storage operations.
- Admin API modules own application management operations such as provider accounts, bucket/object administration, and space storage binding.
- OpenAPI files must reflect every exposed route and response field.
- App OpenAPI and App SDK must not expose storage administration paths, operationIds, request/response schemas, or generated client methods. Storage administration belongs to backend/admin SDK families only.
- Generated backend-admin SDKs must live under the owning SDK family in `sdks/` and must use the canonical `../../sdkwork-sdk-generator/bin/sdkgen.js` generator through Drive wrapper scripts.

## Current Implementation Notes

- S3 multipart upload is now real in the app workflow:
  - create upload session calls `CreateMultipartUpload` when an active S3-compatible provider exists.
  - presign uses the persisted provider multipart id.
  - complete calls `CompleteMultipartUpload` before committing Drive object metadata.
  - abort calls `AbortMultipartUpload` before marking the upload session aborted.

- Bucket/object management is available through backend provider routes and the dedicated admin storage module. Admin storage additionally supports S3 account-level bucket discovery for provider configuration and validation.
- Dedicated admin-storage provider, binding, bucket, and object mutations record audit events with the provided `operatorId`; generated OpenAPI and SDKs must expose those `operatorId` requirements. Binding administration supports listing provider mounts and soft-deleting tenant or space default mounts.
- Provider test routes in backend and admin-storage surfaces perform a real provider health check by calling `HeadBucket` for S3-compatible providers; status-only checks are not sufficient for account validation. App API does not expose provider test routes.
- Provider test routes may run against `active` or `disabled` providers so administrators can validate accounts before enabling traffic. Bucket/object mutation and inspection routes still require an `active` provider.

- S3-compatible provider profiles currently include AWS S3, MinIO, Cloudflare R2, Aliyun OSS, Tencent COS, Huawei OBS, Volcengine TOS, Google Cloud Storage S3-compatible mode, Backblaze B2, and generic S3-compatible endpoints.

- Both the AWS SDK S3 adapter and the optional OpenDAL S3 plugin use the same Drive storage credential resolver from `sdkwork-drive-storage-contract`. This prevents provider create/rotate APIs from accepting `secret:`, `kms:`, or `vault:` values that later fail under one adapter but not the other.

- Local storage supports local bucket and object management for development, but still does not support multipart presign semantics.

- App upload, file creation, and archive extraction now generate physical object keys with the binding root plus the standard content suffix:
  `{storageRootPrefix}/sdkwork-drive/v1/t/{tenantShard}/tenants/{tenantId}/spaces/{spaceId}/nodes/n/{nodeShard}/{nodeId}/versions/{versionNo}/{objectId}/content`.
  User file names and folder paths are not embedded in object keys.

## Future Work

Recommended next steps:

1. Expand MinIO integration tests for bucket lifecycle, list/head/delete/copy object, multipart upload, presign upload part, presign download, and range read.
2. Add object-store copy and server-side encryption options to Drive business workflows only where a user-facing workflow needs them.
3. Add derived-object, quarantine, export, and repair manifests as their workflows are implemented.

