# Drive Admin Storage API Changelog

## 0.2.0

- New object content endpoints: `GET`/`PUT /backend/v3/api/drive/storage/providers/{providerId}/objects/{objectKey}/content` (`storageProviders.objects.content.retrieve` / `storageProviders.objects.content.update`). Reads return base64 content up to 8 MiB with size and SHA-256 checksum; writes accept utf8 or base64 payloads with an optional content type. A trailing-slash object key with empty content creates a directory placeholder object.
- Object content endpoints moved from `.../objects/{objectKey}/content` to `.../object-contents/{objectKey}`: the previous path collided with the `{*objectKey}` tail wildcard of the object head/delete route and panicked at router construction in production assembly (axum 0.8 wildcard overlap). The new path keeps the wildcard at the end of the route.
- Object keys in path parameters are decoded exactly once by the framework; the previous manual second percent-decode silently rewrote literal `%` characters (e.g. `50%20off.txt` became `50 off.txt`) and rejected keys with trailing percent sequences. Trailing-slash keys are normalized to a single slash, double-slash keys are rejected, and directory placeholder objects are now deletable through `DELETE .../objects/{key}/` like any other object.
- The storage provider/bucket management surface becomes the canonical owner for the CloudRouter storage center: CloudRouter no longer serves `/backend/v3/api/storage/providers|buckets`, and its admin UI browses bucket objects through this API (see CloudRouter backend API changelog 0.12.0).

## 0.1.0 — 2026-06-24

- Initial nested storage admin API surface under `/backend/v3/api/drive/storage/...`.
- Storage provider CRUD, bucket/object admin, provider bindings, and activation lifecycle.
- Canonical authority for PC admin UI via `sdkwork-drive-admin-storage-sdk`.
