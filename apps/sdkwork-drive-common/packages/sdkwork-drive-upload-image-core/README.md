# @sdkwork/drive-upload-image-core

Platform-agnostic core for the reusable Drive image-upload component family.
It owns the standardized contract that the PC, mobile-React (H5), WeChat
mini-program, and Flutter shells bind to:

- `assertDriveUploadImageDeclaration` — validates the host application's
  declared upload intent against `DRIVE_SPEC.md` §18 field rules.
- `createDriveUploadImageService` — builds the injected
  `DriveUploadImageService` from the composed Drive uploader
  (`client.uploader`) plus the declaration. UI components receive this
  service; they never compose `appResourceType`/`scene`/`source` inline and
  never see the SDK client (`DRIVE_SPEC.md` §18.3).
- `createDriveNodesImagePreviewReader` — bounded (default 2 MiB), cached,
  same-origin preview reads over `drive.drive.nodes.content.retrieve`,
  surfaced as transient data URLs (`DRIVE_SPEC.md` §9).
- `DriveUploadImageController` — the upload state machine (validation,
  sequential uploads with progress, persist-first deferral, retry, replace
  semantics, transient preview lifecycle, destroy/abort) shared by every
  platform shell.
- Value contract: `DriveUploadImageValue` carries only the stable
  `drive://spaces/{spaceId}/nodes/{nodeId}` reference and the
  `metadata.drive` block (`DRIVE_SPEC.md` §10) — never presigned URLs.

## Usage (host application service layer)

```ts
import { createClient } from "@sdkwork/drive-app-sdk";
import {
  createDriveNodesImagePreviewReader,
  createDriveUploadImageService,
} from "@sdkwork/drive-upload-image-core";
import { MY_APP_USER_AVATAR_UPLOAD } from "../uploadDeclaration";

const drive = createClient({ /* app config */ });

// Service layer: bind once with the declared intent (§18).
export const avatarImageService = createDriveUploadImageService({
  uploader: drive.uploader,
  declaration: MY_APP_USER_AVATAR_UPLOAD,
  previewReader: createDriveNodesImagePreviewReader(drive.drive.nodes),
});
```

The platform shells consume the service plus their own rendering props:

| Platform | Package |
| --- | --- |
| PC (React) | `sdkwork-drive-pc-upload-image` |
| H5 / mobile (React) | `@sdkwork/drive-mobile-react-upload-image` |
| WeChat mini-program | `@sdkwork/drive-mp-upload-image` |
| Flutter | `drive_upload_image_composed` (composed over the Drive app SDK Flutter uploader) |

## Verification

```bash
pnpm --filter @sdkwork/drive-upload-image-core typecheck
pnpm --filter @sdkwork/drive-upload-image-core test
```
