# @sdkwork/drive-mp-upload-image

WeChat mini-program binding of the reusable Drive image-upload family:

- `createMpDriveUploadImage(options)` — owns a shared
  `DriveUploadImageController` and exposes `chooseAndUpload` /
  `chooseFromAlbum` / `takePhoto` / `uploadPending` / `removeItem` /
  `retryItem` built on `wx.chooseMedia` + ranged `FileSystemManager` reads.
- `createWxDriveImagePicker(wx)` — the picker adapter alone; byte sources
  stream through `readRange`, so nothing loads whole files up front.
- `mapSnapshotForTemplate(snapshot, { maxFiles, readOnly })` — stable
  `setData` shape for the reference native component.
- `templates/` — copy-in reference component
  (`upload-image.wxml/.wxss/.js/.json`) showing the page binding.

## Layering

The host's service layer builds the service once from its declared upload
intent and the composed Drive uploader; pages and the reference component
receive only that service (`DRIVE_SPEC.md` §18.3). The `wx` host object is
passed explicitly (or resolved from the global), keeping every code path
unit-testable under Node.

```ts
import { createMpDriveUploadImage } from "@sdkwork/drive-mp-upload-image";
import { driveUploadImageService } from "../services/driveUploadImageService";

const binding = createMpDriveUploadImage({
  service: driveUploadImageService,
  maxFiles: 1,
  resolveAppResourceId: () => pageEntityId ?? null, // persist-first supported
});
const unsubscribe = binding.subscribe(() =>
  this.setData({ state: mapSnapshotForTemplate(binding.getSnapshot(), { maxFiles: 1 }) }),
);
```

Uploads enter Drive only through `client.uploader.*` via the injected
service; rendered values are persist-safe `DriveUploadImageValue`s. The
WeChat byte transport is bound by the host (mini-program runtimes need a
`readRange`-based uploader path or an app-provided service).

## Verification

```bash
pnpm --filter @sdkwork/drive-mp-upload-image typecheck
pnpm --filter @sdkwork/drive-mp-upload-image test
```
