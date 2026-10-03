import 'dart:typed_data';

/// Standard image-shaped upload profiles (`DRIVE_SPEC.md` section 8.1); a
/// profile is chosen by content shape and custom codes are forbidden.
enum DriveUploadImageProfile { image, avatar, thumbnail }

/// Rejection codes produced by file admission; every shell renders them
/// through `DriveUploadImageCopy`.
enum DriveUploadImageRejectionCode {
  invalidFileType,
  fileTooLarge,
  emptyFile,
  tooManyFiles,
}

/// Full error taxonomy for the image-upload family.
enum DriveUploadImageErrorCode {
  invalidFileType,
  fileTooLarge,
  emptyFile,
  tooManyFiles,
  invalidDeclaration,
  missingAppResourceId,
  invalidUploadResult,
  uploadFailed,
  previewUnsupported,
  controllerDestroyed,
}

/// The host application's declared upload intent (`DRIVE_SPEC.md` section
/// 18). The application service layer constructs it once from its declared
/// constants; UI code never composes these values inline.
class DriveUploadImageDeclaration {
  const DriveUploadImageDeclaration({
    required this.appResourceType,
    required this.appResourceIdKind,
    required this.scene,
    required this.source,
    required this.uploadProfileCode,
    required this.retention,
    this.retentionTtlSeconds,
    required this.purpose,
  });

  final String appResourceType;
  final String appResourceIdKind;
  final String scene;
  final String source;
  final DriveUploadImageProfile uploadProfileCode;
  final String retention;
  final int? retentionTtlSeconds;
  final String purpose;
}

/// Persist-safe upload value carried by business form state
/// (`DRIVE_SPEC.md` section 10): the stable Drive reference plus the
/// `metadata.drive` block — never presigned or delivery URLs.
class DriveUploadImageValue {
  const DriveUploadImageValue({
    required this.uri,
    required this.source,
    this.metadata = const <String, dynamic>{},
  });

  /// `drive://spaces/{spaceId}/nodes/{nodeId}` for Drive-backed content.
  final String uri;
  final String source;
  final Map<String, dynamic> metadata;

  Map<String, dynamic>? get driveMetadata {
    final metadata = this.metadata['drive'];
    return metadata is Map<String, dynamic> ? metadata : null;
  }

  Map<String, dynamic> toJson() => <String, dynamic>{
        'uri': uri,
        'source': source,
        'metadata': metadata,
      };

  static DriveUploadImageValue? fromJson(Object? json) {
    if (json is! Map) {
      return null;
    }
    final uri = json['uri'];
    final source = json['source'];
    if (uri is! String || uri.isEmpty || source is! String) {
      return null;
    }
    final metadata = json['metadata'];
    return DriveUploadImageValue(
      uri: uri,
      source: source,
      metadata: metadata is Map<String, dynamic>
          ? metadata
          : const <String, dynamic>{},
    );
  }
}

/// Bytes plus the metadata the uploader and validators need.
class DriveUploadImageSource {
  const DriveUploadImageSource({
    required this.bytes,
    this.fileName,
    this.contentType,
  });

  final Uint8List bytes;
  final String? fileName;
  final String? contentType;

  int get sizeBytes => bytes.length;
}

/// Normalized progress snapshot; `percent` is 0..100.
class DriveUploadImageProgress {
  const DriveUploadImageProgress({
    required this.uploadedBytes,
    required this.totalBytes,
    required this.uploadedPartsCount,
    required this.totalParts,
    required this.status,
  });

  final int uploadedBytes;
  final int totalBytes;
  final int uploadedPartsCount;
  final int totalParts;
  final String status;

  double get percent =>
      totalBytes <= 0 ? 0 : (uploadedBytes / totalBytes).clamp(0.0, 1.0) * 100;
}

/// Typed failure raised across the package.
class DriveUploadImageException implements Exception {
  DriveUploadImageException(this.code, this.message, [this.cause]);

  final DriveUploadImageErrorCode code;
  final String message;
  final Object? cause;

  @override
  String toString() => 'DriveUploadImageException(${code.name}): $message';
}

/// User-facing copy for every shell. Hosts pass a customized instance for
/// localization; the defaults are English (`I18N_SPEC.md` section 6.1).
class DriveUploadImageCopy {
  const DriveUploadImageCopy({
    this.pickImage = 'Upload image',
    this.replaceImage = 'Replace image',
    this.removeImage = 'Remove image',
    this.retryUpload = 'Retry upload',
    this.uploading = 'Uploading…',
    this.uploadFailed = 'Upload failed',
    this.invalidFileType = 'Only image files are supported.',
    this.fileTooLarge = 'This image is too large.',
    this.emptyFile = 'This file is empty.',
    this.tooManyFiles = 'Too many images selected.',
    this.previewUnavailable = 'Preview unavailable',
    this.chooseFromAlbum = 'Choose from album',
    this.takePhoto = 'Take photo',
  });

  final String pickImage;
  final String replaceImage;
  final String removeImage;
  final String retryUpload;
  final String uploading;
  final String uploadFailed;
  final String invalidFileType;
  final String fileTooLarge;
  final String emptyFile;
  final String tooManyFiles;
  final String previewUnavailable;
  final String chooseFromAlbum;
  final String takePhoto;
}

/// Shared package limits.
const int driveUploadImageDefaultMaxBytes = 5 * 1024 * 1024;
const int driveUploadImagePreviewMaxBytes = 2 * 1024 * 1024;
const List<String> driveUploadImageDefaultAccept = <String>['image/*'];

const Map<String, String> driveUploadImageExtensionMime = <String, String>{
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.gif': 'image/gif',
  '.webp': 'image/webp',
  '.avif': 'image/avif',
  '.heic': 'image/heic',
};

String? driveUploadImageExtensionOf(String? fileName) {
  if (fileName == null) {
    return null;
  }
  final dot = fileName.lastIndexOf('.');
  if (dot < 0 || dot == fileName.length - 1) {
    return null;
  }
  return fileName.substring(dot).toLowerCase();
}

/// Effective mime for admission: declared type first, then the image
/// extension map (mini-program/platforms without a declared type).
String driveUploadImageEffectiveType(String? contentType, String? fileName) {
  final declared = contentType?.toLowerCase() ?? '';
  if (declared.isNotEmpty) {
    return declared;
  }
  final extension = driveUploadImageExtensionOf(fileName);
  if (extension == null) {
    return '';
  }
  return driveUploadImageExtensionMime[extension] ?? '';
}

bool driveUploadImageMatchesAccept(
  String effectiveType,
  String? fileName,
  List<String> accept,
) {
  for (final rawEntry in accept) {
    final entry = rawEntry.trim().toLowerCase();
    if (entry.isEmpty) {
      continue;
    }
    if (entry.startsWith('.')) {
      final extension = driveUploadImageExtensionOf(fileName);
      if (extension != null && extension == entry) {
        return true;
      }
      continue;
    }
    if (effectiveType.isEmpty) {
      continue;
    }
    if (entry.endsWith('/*')) {
      if (effectiveType.startsWith(entry.substring(0, entry.length - 1))) {
        return true;
      }
      continue;
    }
    if (effectiveType == entry) {
      return true;
    }
  }
  return false;
}

/// File admission rules shared by every shell (`image/*` is a package
/// invariant; `accept` narrows on top of it).
DriveUploadImageRejectionCode? validateDriveUploadImageSource(
  DriveUploadImageSource source, {
  List<String> accept = driveUploadImageDefaultAccept,
  int maxSizeBytes = driveUploadImageDefaultMaxBytes,
}) {
  if (source.sizeBytes <= 0) {
    return DriveUploadImageRejectionCode.emptyFile;
  }
  final effectiveType =
      driveUploadImageEffectiveType(source.contentType, source.fileName);
  if (!effectiveType.startsWith('image/') ||
      !driveUploadImageMatchesAccept(effectiveType, source.fileName, accept)) {
    return DriveUploadImageRejectionCode.invalidFileType;
  }
  if (source.sizeBytes > maxSizeBytes) {
    return DriveUploadImageRejectionCode.fileTooLarge;
  }
  return null;
}

String formatDriveUploadImageBytes(int bytes) {
  if (bytes >= 1024 * 1024) {
    final mebibytes = bytes / (1024 * 1024);
    final label = mebibytes == mebibytes.roundToDouble()
        ? mebibytes.toStringAsFixed(0)
        : mebibytes.toStringAsFixed(1);
    return '$label MB';
  }
  if (bytes >= 1024) {
    return '${(bytes / 1024).round()} KB';
  }
  return '$bytes B';
}
