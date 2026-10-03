/// Reusable Drive image-upload UI family for Flutter.
///
/// Uploads enter Drive only through the composed `DriveUploaderClient`
/// (`drive_uploader_composed`); the host service layer binds its declared
/// upload intent once via [DriveUploaderImageService] and injects the
/// resulting [DriveUploadImageService] into the widgets, which never compose
/// `appResourceType`/`scene`/`source` inline (`DRIVE_SPEC.md` section 18.3).
/// Values are persist-safe `DriveUploadImageValue`s — the stable
/// `drive://spaces/{spaceId}/nodes/{nodeId}` reference and the
/// `metadata.drive` block; previews stay transient widget state
/// (`DRIVE_SPEC.md` sections 9 and 10).
library;

export 'src/controller.dart';
export 'src/drive_upload_image_field.dart';
export 'src/drive_upload_image_grid.dart';
export 'src/service.dart';
export 'src/types.dart';
