# sdkwork-drive-pc-file-preview

Domain: drive
Capability: file-preview
Package type: PC internal React package
Surface: shared
Status: standard

File preview and editing components shared by the Drive console and the storage
admin console. Each preview is its own module; `FilePreviewSurface` resolves a
file to a preview kind and mounts exactly one of them. Host coupling is limited
to two injected ports — `FilePreviewResource` for content and `FilePreviewLabels`
for copy — so a host mounts these components over its own service and its own
dictionary without the package importing either.

## Public API

- `.`: preview kinds, the resource/label ports, the individual preview
  components, and `FilePreviewSurface`.

## Verification

- `pnpm typecheck`
- `pnpm test`

## Related Specs

- `../../../../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md`
- `../../../../sdkwork-specs/FRONTEND_CODE_SPEC.md`
- `../../../../sdkwork-specs/DRIVE_SPEC.md`
