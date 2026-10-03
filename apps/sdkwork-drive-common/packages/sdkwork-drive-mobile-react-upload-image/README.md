# @sdkwork/drive-mobile-react-upload-image

Mobile-React (H5) shells of the reusable Drive image-upload component family:

- `DriveUploadImage` — single image slot with large touch targets and an
  optional album/camera source sheet (`sources={["album", "camera"]}`).
- `DriveUploadImageList` — bounded multi-image grid with per-item
  remove/retry and an add tile.
- `useDriveUploadImageController` / `useDriveUploadImageSnapshot` /
  `useDriveUploadImageControlledValue` — React bindings over the shared
  state machine from `@sdkwork/drive-upload-image-core`.

## Layering

The host application's service layer builds the service once from its
declared upload intent and the composed Drive uploader; components receive
only that service (`DRIVE_SPEC.md` §18.3):

```tsx
import { DriveUploadImage } from "@sdkwork/drive-mobile-react-upload-image";
import { avatarImageService } from "../services/avatarImageService";

<DriveUploadImage
  service={avatarImageService}
  value={avatar}
  onChange={setAvatar}
  appResourceId={userId}
  sources={["album", "camera"]}
  shape="circle"
  sizePx={80}
  copy={{ pickImage: "上传头像", takePhoto: "拍照", chooseFromAlbum: "从相册选择", cancel: "取消" }}
/>
```

Uploads enter Drive only through `client.uploader.*`; the rendered value is
the persist-safe `DriveUploadImageValue`, and preview URLs stay transient
(`DRIVE_SPEC.md` §9/§10). On mobile browsers the camera source sets the
`capture` attribute imperatively right before the picker opens.

## Verification

```bash
pnpm --filter @sdkwork/drive-mobile-react-upload-image typecheck
pnpm --filter @sdkwork/drive-mobile-react-upload-image test
```
