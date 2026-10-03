# drive_upload_image_composed

Flutter shells of the reusable Drive image-upload component family,
composed over the `drive_uploader_composed` uploader facade
(`DRIVE_SPEC.md` sections 8.1, 9, 10, and 18):

- `DriveUploadImageField` — single configurable image slot (avatar circle by
  default) with pick, transient progress, remove, retry, and localized copy.
- `DriveUploadImageGridView` — bounded multi-image grid with per-item
  remove/retry and an add tile.
- `DriveUploadImageController` — the shared upload state machine
  (validation, sequential uploads, persist-first deferral, replace
  semantics, preview resolution, dispose/abort) as a `ChangeNotifier`.
- `DriveUploaderImageService` — the service-layer binding: host passes the
  `DriveUploaderClient` plus its declared `DriveUploadImageDeclaration`
  once; widgets receive only the resulting `DriveUploadImageService`.

## Layering contract

`DRIVE_SPEC.md` section 18.3: the service layer, not the UI, supplies the
declared upload intent. Hosts build the service once and inject it; widgets
never compose `appResourceType`/`scene`/`source` inline and never touch the
SDK client. Rendered values are persist-safe `DriveUploadImageValue`s — the
stable `drive://spaces/{spaceId}/nodes/{nodeId}` reference plus the
`metadata.drive` block; preview URLs stay transient widget state.

```dart
final imageService = DriveUploaderImageService(
  uploader: driveAppClient.uploader,
  declaration: const DriveUploadImageDeclaration(
    appResourceType: 'profile.avatar',
    appResourceIdKind: 'entity',
    scene: 'avatar',
    source: 'my-app-flutter',          // declared constant, not a package name
    uploadProfileCode: DriveUploadImageProfile.avatar,
    retention: 'long_term',
    purpose: 'User avatar uploaded from the Flutter app profile page.',
  ),
  previewReader: ({required nodeId, required maxBytes}) =>
      myBoundedNodePreview(nodeId, maxBytes),
);

DriveUploadImageField(
  service: imageService,
  onPick: () async => pickImagesWithHostPlugin(),
  value: avatar,
  onChanged: (next) => setState(() => avatar = next),
  appResourceId: () => userId,        // persist-first flows return null until the entity exists
  shape: DriveUploadImageShape.circle,
  size: 96,
  copy: const DriveUploadImageCopy(pickImage: '上传头像'),
)
```

## Picking

The package is picker-plugin agnostic: hosts implement
`DriveImagePicker`/`onPick` with their plugin of choice. With
`image_picker`:

```dart
final picker = ImagePicker();
Future<List<DriveUploadImageSource>> pickFromGallery() async {
  final file = await picker.pickImage(source: ImageSource.gallery);
  if (file == null) return const [];
  return [
    DriveUploadImageSource(
      bytes: await file.readAsBytes(),
      fileName: file.name,
      contentType: file.mimeType,
    ),
  ];
}
```

## Verification

```bash
flutter pub get
flutter analyze
flutter test
```
