import 'package:drive_uploader_composed/drive_uploader_composed.dart';

import 'types.dart';

/// Functional slice that reads a bounded preview for a Drive node. Hosts bind
/// it to the generated Drive nodes content API so previews stay same-origin
/// and size-capped (`DRIVE_SPEC.md` section 8).
typedef DriveImagePreviewReader = Future<String?> Function({
  required String nodeId,
  required int maxBytes,
});

/// Service the host service layer builds once and injects into widgets.
abstract class DriveUploadImageService {
  Future<DriveUploadImageValue> upload({
    required DriveUploadImageSource file,
    required String appResourceId,
    void Function(DriveUploadImageProgress progress)? onProgress,
    bool Function()? shouldAbort,
  });

  /// Transient display URL for a stored value. Drive-backed values read
  /// through the bounded preview reader; external `http(s)/data/blob`
  /// values resolve to themselves. Results are presentation-only state
  /// (`DRIVE_SPEC.md` section 9) and never persisted.
  Future<String?> resolvePreview({
    required String uri,
    int? maxBytes,
  });
}

RegExp _driveUriPattern = RegExp(r'^drive://spaces/([^/]+)/nodes/([^/?#]+)');

/// Default binding over the composed Drive Uploader: the host passes the
/// `DriveUploaderClient` plus its declared upload intent
/// (`DRIVE_SPEC.md` section 18.3 — the service layer supplies declared
/// values; identity dimensions are derived server-side).
class DriveUploaderImageService implements DriveUploadImageService {
  DriveUploaderImageService({
    required DriveUploaderClient uploader,
    required DriveUploadImageDeclaration declaration,
    DriveImagePreviewReader? previewReader,
    this.previewMaxBytes = driveUploadImagePreviewMaxBytes,
  })  : _uploader = uploader,
        _declaration = declaration,
        _previewReader = previewReader {
    validateDriveUploadImageDeclaration(_declaration);
  }

  final DriveUploaderClient _uploader;
  final DriveUploadImageDeclaration _declaration;
  final DriveImagePreviewReader? _previewReader;
  final int previewMaxBytes;

  @override
  Future<DriveUploadImageValue> upload({
    required DriveUploadImageSource file,
    required String appResourceId,
    void Function(DriveUploadImageProgress progress)? onProgress,
    bool Function()? shouldAbort,
  }) async {
    if (appResourceId.trim().isEmpty) {
      throw DriveUploadImageException(
        DriveUploadImageErrorCode.missingAppResourceId,
        'Drive upload requires the identifier of an existing entity; persist '
        'the entity first, then upload (DRIVE_SPEC.md section 18.3).',
      );
    }
    final retention = _declaration.retention == 'temporary'
        ? DriveUploaderRetention.temporary(
            ttlSeconds: _declaration.retentionTtlSeconds?.toString(),
          )
        : DriveUploaderRetention.longTerm();
    final result = await _uploader.uploadByProfile(
      _declaration.uploadProfileCode.name,
      DriveUploaderRequest(
        blob: DriveUploaderBlob(
          bytes: file.bytes,
          fileName: file.fileName ?? 'image',
          contentType: file.contentType ?? 'application/octet-stream',
        ),
        appResourceType: _declaration.appResourceType,
        appResourceId: appResourceId,
        scene: _declaration.scene,
        source: _declaration.source,
        uploadProfileCode: _declaration.uploadProfileCode.name,
        retention: retention,
        onProgress: onProgress == null
            ? null
            : (progress) {
                onProgress(
                  DriveUploadImageProgress(
                    uploadedBytes: progress.uploadedBytes,
                    totalBytes: progress.totalBytes,
                    uploadedPartsCount: progress.uploadedPartsCount,
                    totalParts: progress.totalParts,
                    status: 'uploading',
                  ),
                );
              },
        shouldAbort: shouldAbort,
      ),
    );

    final spaceId = result.spaceId;
    final nodeId = result.nodeId;
    if (spaceId.isEmpty || nodeId.isEmpty) {
      throw DriveUploadImageException(
        DriveUploadImageErrorCode.invalidUploadResult,
        'Drive did not return the uploaded image identity.',
      );
    }
    return DriveUploadImageValue(
      uri: result.driveUri,
      source: 'drive',
      metadata: <String, dynamic>{
        'drive': <String, dynamic>{
          'spaceId': spaceId,
          'nodeId': nodeId,
          'contentType': file.contentType,
          'contentLength': '${file.sizeBytes}',
          'originalFileName': file.fileName,
        },
      },
    );
  }

  @override
  Future<String?> resolvePreview({required String uri, int? maxBytes}) async {
    if (!uri.startsWith('drive://')) {
      final isDirect = uri.startsWith('http://') ||
          uri.startsWith('https://') ||
          uri.startsWith('data:') ||
          uri.startsWith('blob:');
      return isDirect ? uri : null;
    }
    final reader = _previewReader;
    if (reader == null) {
      throw DriveUploadImageException(
        DriveUploadImageErrorCode.previewUnsupported,
        'No preview reader was bound; previews of drive-backed images are '
        'unavailable.',
      );
    }
    final matched = _driveUriPattern.firstMatch(uri);
    if (matched == null) {
      return null;
    }
    final nodeId = matched.group(2);
    if (nodeId == null || nodeId.isEmpty) {
      return null;
    }
    return reader(nodeId: nodeId, maxBytes: maxBytes ?? previewMaxBytes);
  }
}

/// Validates the host declaration against `DRIVE_SPEC.md` section 18 field
/// rules; unknown shapes fail fast when the service is created.
void validateDriveUploadImageDeclaration(DriveUploadImageDeclaration declaration) {
  void fail(String message) {
    throw DriveUploadImageException(
      DriveUploadImageErrorCode.invalidDeclaration,
      message,
    );
  }

  final resourceTypePattern = RegExp(r'^[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)+$');
  final labelPattern = RegExp(r'^[a-z0-9]+(-[a-z0-9]+)*$');

  if (!resourceTypePattern.hasMatch(declaration.appResourceType)) {
    fail('appResourceType must be a lowercase dotted business type with at '
        'least two segments.');
  }
  if (!<String>['application', 'entity', 'draft']
      .contains(declaration.appResourceIdKind)) {
    fail('appResourceIdKind must be one of: application, entity, draft.');
  }
  if (!labelPattern.hasMatch(declaration.scene) || declaration.scene == 'im') {
    fail('scene must be a lowercase kebab-case workflow label and must not '
        'use the reserved Drive scene `im`.');
  }
  if (!labelPattern.hasMatch(declaration.source)) {
    fail('source must be a stable lowercase kebab-case label, not a package '
        'name or module path.');
  }
  if (declaration.retention != 'long_term' &&
      declaration.retention != 'temporary') {
    fail('retention must be long_term or temporary.');
  }
  if (declaration.retention == 'temporary' &&
      (declaration.retentionTtlSeconds == null ||
          declaration.retentionTtlSeconds! <= 0)) {
    fail('A temporary declaration must declare a positive integer '
        'retentionTtlSeconds.');
  }
  if (declaration.purpose.trim().isEmpty) {
    fail('purpose must be a non-empty sentence.');
  }
}
