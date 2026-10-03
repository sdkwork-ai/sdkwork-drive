# sdkwork-drive-pc-upload-image

PC (React) shells of the reusable Drive image-upload component family:

- `DriveUploadImage` — a single configurable image slot (avatar circle by
  default) with pick/replace, drag-drop, transient progress, remove, retry,
  and localized copy.
- `DriveUploadImageList` — a bounded multi-image thumbnail grid with per-item
  remove/retry and an add tile.
- `useDriveUploadImageController` / `useDriveUploadImageSnapshot` /
  `useDriveUploadImageControlledValue` — React bindings over the shared
  state machine from `@sdkwork/drive-upload-image-core`.

## Layering

The host application's service layer builds the service once from its
declared upload intent and the composed Drive uploader; components receive
only that service and never compose `appResourceType`/`scene`/`source`
inline (`DRIVE_SPEC.md` §18.3):

```tsx
import { DriveUploadImage } from "sdkwork-drive-pc-upload-image";
import { avatarImageService } from "../services/avatarImageService";

<DriveUploadImage
  service={avatarImageService}
  value={avatar}
  onChange={setAvatar}
  appResourceId={userId}          // entity anchor; a () => string supports persist-first flows
  shape="circle"
  sizePx={96}
  maxSizeBytes={5 * 1024 * 1024}
  label="Avatar"
  copy={{ pickImage: "上传头像" }} // localized copy overrides
/>
```

Uploads enter Drive only through `client.uploader.*`; the rendered value is
the persist-safe `DriveUploadImageValue`
(`drive://spaces/{spaceId}/nodes/{nodeId}` + `metadata.drive`), and preview
URLs stay transient component state (`DRIVE_SPEC.md` §9/§10).

## Verification

```bash
pnpm --filter sdkwork-drive-pc-upload-image test
pnpm --filter sdkwork-drive-pc-upload-image typecheck
```
