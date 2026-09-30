> Owner: SDKWork maintainers

# SDKWork Drive Storage Key Layout

## Goal

Drive product directories and object-store keys are separate concerns.

- `dr_drive_node` owns the user-visible directory tree, file names, shortcuts, and lifecycle state.
- `dr_drive_storage_object` maps Drive node versions to immutable physical objects.
- S3-compatible object stores keep bytes under internal keys. Object keys are not user paths.

This avoids copying objects when users rename files, move folders, restore items, create shortcuts, or change sharing rules.

## Binding Root Prefix

Provider bindings add a mount root before Drive-generated content keys. The stored physical key is:

```text
{storageRootPrefix}/{standardContentKey}
```

Default binding roots are:

```text
sdkwork-drive/v1/tenants/{tenantId}
sdkwork-drive/v1/tenants/{tenantId}/spaces/{spaceId}
```

`dr_drive_storage_provider_binding.storage_root_prefix` may override the default so a tenant or space can be mounted under a different S3 account layout. The prefix is a trimmed relative object prefix: UTF-8 1-512 bytes, no leading or trailing slash, no `//`, no NUL, and no `.` or `..` path segment.

## Standard Content Key

All newly generated content objects must contain this Drive-controlled suffix:

```text
sdkwork-drive/v1/t/{tenantShard}/tenants/{tenantId}/spaces/{spaceId}/nodes/n/{nodeShard}/{nodeId}/versions/{versionNoPadded}/{objectId}/content
```

Example:

```text
sdkwork-drive/v1/t/f0/tenants/tenant-8f3a9c/spaces/space-team-01/nodes/n/4a/node-a74392/versions/0000000007/obj-01hrz4m5nn9k9/content
```

## Segment Rules

```text
sdkwork-drive
  Product root prefix. This prevents collisions when a bucket hosts multiple systems.

v1
  Physical layout version. Future layouts must use a new version prefix instead of rewriting old keys.

t/{tenantShard}
  Two-character lowercase hex shard from the tenant id SHA-256 first byte.

tenants/{tenantId}
  Tenant isolation boundary for audits, migrations, and lifecycle scans.

spaces/{spaceId}
  Space boundary for personal, team, knowledge-base, AI-generated, Git repository, deployment, and app-upload storage.

nodes/n/{nodeShard}/{nodeId}
  Node boundary. The shard is the two-character lowercase hex SHA-256 first byte of the node id.

versions/{versionNoPadded}
  Drive-controlled file version number padded to 10 digits. Version numbers start at 1.

{objectId}
  Physical object identifier. Upload session id is acceptable for initial uploads because it is stable under idempotency.

content
  Content leaf object.
```

## Product Rules

1. User-visible folders are rows in `dr_drive_node`, not S3 folders.
2. Object keys must not contain user file names, user folder names, or user-submitted paths.
3. App upload and file creation store physical object keys under the active binding root prefix.
4. User-supplied `objectKey` fields are compatibility inputs only and must be ignored for new object placement.
5. Each Drive file version gets a new immutable object key.
6. Rename, move, share, restore, and trash operations update SQL metadata only.
7. App APIs should not treat `objectKey` as a durable user-facing identifier.
8. Backend provider object APIs may expose object keys because they are administrative tools.
9. Object-key generation belongs in product application code, not in API routes or provider SDK adapters.

## Reserved Prefixes

These prefixes are namespaced by this layout and MUST NOT be used for content objects:

```text
sdkwork-drive/v1/t/{tenantShard}/tenants/{tenantId}/spaces/{spaceId}/derived/
sdkwork-drive/v1/t/{tenantShard}/tenants/{tenantId}/spaces/{spaceId}/uploads/
sdkwork-drive/v1/t/{tenantShard}/tenants/{tenantId}/spaces/{spaceId}/quarantine/
sdkwork-drive/v1/t/{tenantShard}/tenants/{tenantId}/system/manifests/
sdkwork-drive/v1/t/{tenantShard}/tenants/{tenantId}/system/repair/
sdkwork-drive/v1/t/{tenantShard}/tenants/{tenantId}/system/export/
```

## Provider Location And History

The key layout is provider-neutral. The same `{storageRootPrefix}/{standardContentKey}` shape is
produced whether the target is a local store or any S3-compatible provider.

- Switching the tenant or space binding changes the root prefix and the physical provider for
  newly written objects only. It never rewrites, moves, or re-addresses an existing key.
- `dr_drive_storage_object` records the provider id and the `DriveProviderLocation`
  (`endpoint_url`, `bucket`, `path_style`) alongside the key, so a read resolves to the same
  physical location it was written to even after the active provider changes.
- Because the key is stable and provider-neutral, a provider migration is a byte copy plus a
  location metadata update, not an application-level rewrite.
- Bindings are resolved on the write path through explicit provider, explicit bucket, space
  binding, space-type binding, then tenant binding. The read path prefers the object's recorded
  location.

Object-key generation stays in the Drive storage key service. Adapters receive a finished key and
never compose one.

