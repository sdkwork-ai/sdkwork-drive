import 'dart:typed_data';

/// Standard Drive upload profiles (`DRIVE_SPEC.md` section 8.1). A profile is
/// chosen by content shape; custom profile codes are forbidden.
const List<String> driveUploaderProfiles = <String>[
  'generic',
  'video',
  'image',
  'audio',
  'document',
  'archive',
  'text',
  'dataset',
  'attachment',
  'avatar',
  'thumbnail',
];

/// Bytes plus the metadata the uploader needs to describe the content.
class DriveUploaderBlob {
  const DriveUploaderBlob({
    required this.bytes,
    required this.fileName,
    required this.contentType,
  });

  final Uint8List bytes;
  final String fileName;
  final String contentType;

  int get sizeBytes => bytes.length;
}

/// Retention intent; the Drive server owns cleanup execution.
class DriveUploaderRetention {
  const DriveUploaderRetention.longTerm() : mode = 'long_term', ttlSeconds = null;

  const DriveUploaderRetention.temporary({this.ttlSeconds})
      : mode = 'temporary';

  final String mode;
  final String? ttlSeconds;

  Map<String, dynamic> toJson() => <String, dynamic>{
        'mode': mode,
        if (ttlSeconds != null) 'ttlSeconds': ttlSeconds,
      };
}

/// Upload business intent only (`DRIVE_SPEC.md` section 18.3). Identity
/// dimensions (tenant, organization, user, app) are derived by Drive from the
/// verified authenticated runtime and MUST NOT be added here.
class DriveUploaderRequest {
  const DriveUploaderRequest({
    required this.blob,
    required this.appResourceType,
    required this.appResourceId,
    required this.scene,
    required this.source,
    this.uploadProfileCode,
    this.retention = const DriveUploaderRetention.longTerm(),
    this.spaceId,
    this.parentNodeId,
    this.taskId,
    this.requestedPartTtlSeconds,
    this.onProgress,
    this.shouldAbort,
  });

  final DriveUploaderBlob blob;
  final String appResourceType;
  final String appResourceId;
  final String scene;
  final String source;

  /// Standard profile code; when omitted the helper method's implied profile
  /// applies (`uploadImage` implies `image`, and so on).
  final String? uploadProfileCode;
  final DriveUploaderRetention retention;
  final String? spaceId;
  final String? parentNodeId;

  /// Client-side idempotency task id; a stable value is derived when omitted.
  final String? taskId;

  /// Requested TTL for presigned part grants.
  final int? requestedPartTtlSeconds;

  /// Coarse progress, reported once per completed part.
  final void Function(DriveUploaderProgress progress)? onProgress;

  /// Polled before each part; return true to stop the upload.
  final bool Function()? shouldAbort;
}

/// Coarse upload progress, reported once per completed part.
class DriveUploaderProgress {
  const DriveUploaderProgress({
    required this.uploadedBytes,
    required this.totalBytes,
    required this.uploadedPartsCount,
    required this.totalParts,
  });

  final int uploadedBytes;
  final int totalBytes;
  final int uploadedPartsCount;
  final int totalParts;

  double get percent => totalBytes <= 0
      ? 0
      : (uploadedBytes / totalBytes).clamp(0.0, 1.0);
}

/// Stable Drive identity returned by a completed upload. Business form state
/// carries these references, never the presigned URLs.
class DriveUploaderUploadResult {
  const DriveUploaderUploadResult({
    required this.uploadItemId,
    required this.uploadSessionId,
    required this.spaceId,
    required this.nodeId,
    this.storageProviderId,
    required this.checksumSha256Hex,
  });

  final String uploadItemId;
  final String uploadSessionId;
  final String spaceId;
  final String nodeId;
  final String? storageProviderId;
  final String checksumSha256Hex;

  String get driveUri => 'drive://spaces/$spaceId/nodes/$nodeId';
}

/// Failure carrying the Drive envelope code and trace id for diagnostics.
class DriveUploaderException implements Exception {
  DriveUploaderException(this.message, {this.code, this.traceId});

  final String message;
  final int? code;
  final String? traceId;

  @override
  String toString() =>
      'DriveUploaderException: $message'
      '${code == null ? '' : ' (code=$code, traceId=$traceId)'}';
}
