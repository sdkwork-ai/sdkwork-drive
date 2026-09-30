import 'dart:math';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:http/http.dart' as http;
import 'package:sdkwork_drive_app_sdk_generated_flutter/sdkwork_drive_app_sdk_generated_flutter.dart';

import 'types.dart';
import 'uploader_state_store.dart';

/// Composed Drive Uploader (`DRIVE_SPEC.md` section 8.1).
///
/// Owns the whole client upload flow — prepare, per-part presign, raw byte PUT
/// to the Drive-granted provider URL, part marking, completion, progress, and
/// resumable state — so feature code never reimplements upload-session
/// orchestration. Identity dimensions are derived server-side; callers send
/// upload business intent only (section 18.3).
class DriveUploaderClient {
  DriveUploaderClient({
    required SdkworkAppClient client,
    http.Client? uploadHttp,
    DriveUploaderStateStore? stateStore,
    int defaultChunkSizeBytes = 8 * 1024 * 1024,
  })  : _client = client,
        _uploadHttp = uploadHttp ?? http.Client(),
        _stateStore = stateStore ?? InMemoryDriveUploaderStateStore(),
        _defaultChunkSizeBytes = defaultChunkSizeBytes;

  final SdkworkAppClient _client;
  final http.Client _uploadHttp;
  final DriveUploaderStateStore _stateStore;
  final int _defaultChunkSizeBytes;

  /// `uploader.upload` — implied profile `generic`.
  Future<DriveUploaderUploadResult> upload(DriveUploaderRequest request) =>
      uploadByProfile('generic', request);

  /// `uploadImage` — implied profile `image`.
  Future<DriveUploaderUploadResult> uploadImage(DriveUploaderRequest request) =>
      uploadByProfile('image', request);

  /// `uploadVideo` — implied profile `video`.
  Future<DriveUploaderUploadResult> uploadVideo(DriveUploaderRequest request) =>
      uploadByProfile('video', request);

  /// `uploadAudio` — implied profile `audio`.
  Future<DriveUploaderUploadResult> uploadAudio(DriveUploaderRequest request) =>
      uploadByProfile('audio', request);

  /// `uploadDocument` — implied profile `document`.
  Future<DriveUploaderUploadResult> uploadDocument(
          DriveUploaderRequest request) =>
      uploadByProfile('document', request);

  /// `uploadArchive` — implied profile `archive`.
  Future<DriveUploaderUploadResult> uploadArchive(
          DriveUploaderRequest request) =>
      uploadByProfile('archive', request);

  /// `uploadText` — implied profile `text`.
  Future<DriveUploaderUploadResult> uploadText(DriveUploaderRequest request) =>
      uploadByProfile('text', request);

  /// `uploadDataset` — implied profile `dataset`.
  Future<DriveUploaderUploadResult> uploadDataset(
          DriveUploaderRequest request) =>
      uploadByProfile('dataset', request);

  /// `uploadAttachment` — implied profile `attachment`.
  Future<DriveUploaderUploadResult> uploadAttachment(
          DriveUploaderRequest request) =>
      uploadByProfile('attachment', request);

  /// `uploadAvatar` — implied profile `avatar`.
  Future<DriveUploaderUploadResult> uploadAvatar(DriveUploaderRequest request) =>
      uploadByProfile('avatar', request);

  /// `uploadThumbnail` — implied profile `thumbnail`.
  Future<DriveUploaderUploadResult> uploadThumbnail(
          DriveUploaderRequest request) =>
      uploadByProfile('thumbnail', request);

  /// `uploadByProfile` — the one orchestration entry; every helper delegates
  /// here with its implied standard profile.
  Future<DriveUploaderUploadResult> uploadByProfile(
    String profile,
    DriveUploaderRequest request,
  ) async {
    _assertStandardProfile(profile, request.uploadProfileCode);
    final effectiveProfile = request.uploadProfileCode ?? profile;

    final checksum = sha256.convert(request.blob.bytes).toString();
    final taskId = request.taskId ?? _newId();
    final totalBytes = request.blob.sizeBytes;

    final prepared = await _prepare(
      request: request,
      profile: effectiveProfile,
      checksumHex: checksum,
      taskId: taskId,
    );

    final chunkSize = _resolveChunkSize(prepared);
    final totalParts = _resolveTotalParts(totalBytes, chunkSize);
    final sessionId = prepared.uploadSessionId;
    if (sessionId == null || sessionId.isEmpty) {
      throw DriveUploaderException(
        'Drive prepare did not return an upload session id.',
      );
    }

    final knownParts = await _stateStore.completedParts(taskId);
    var uploadedBytes = 0;
    final completed = <CompletedUploadPart>[];

    for (var partNo = 1; partNo <= totalParts; partNo++) {
      if (request.shouldAbort?.call() == true) {
        await _abortQuietly(sessionId);
        await _stateStore.clear(taskId);
        throw DriveUploaderException('Upload aborted by the caller.');
      }

      final offset = (partNo - 1) * chunkSize;
      final end =
          offset + chunkSize > totalBytes ? totalBytes : offset + chunkSize;
      final size = end - offset;

      final knownEtag = knownParts[partNo];
      String etag;
      if (knownEtag != null) {
        // Resume: this part was already granted, uploaded, and accepted in a
        // previous run of the same task; Drive's prepare stays authoritative.
        etag = knownEtag;
      } else {
        final presigned = await _presign(
          sessionId: sessionId,
          partNo: partNo,
          requestedTtlSeconds: request.requestedPartTtlSeconds,
        );
        etag = await _putPart(
          url: presigned.uploadUrl,
          headers: presigned.headers,
          bytes: Uint8List.sublistView(request.blob.bytes, offset, end),
        );
        await _stateStore.markPartUploaded(
          taskId,
          partNo,
          etag,
          offsetBytes: offset,
          sizeBytes: size,
        );
      }

      final marked = await _client.drive.uploaderUploadsPartsUpdate(
        prepared.id,
        partNo,
        MarkUploaderPartUploadedRequest(
          uploadSessionId: sessionId,
          offsetBytes: '$offset',
          sizeBytes: '$size',
          etag: etag,
          checksumSha256Hex: 'sha256:$checksum',
        ),
      );
      _assertEnvelope(marked?.code, marked?.traceId, 'mark part $partNo');

      completed.add(CompletedUploadPart(partNo: partNo, etag: etag));
      uploadedBytes += size;
      request.onProgress?.call(
        DriveUploaderProgress(
          uploadedBytes: uploadedBytes,
          totalBytes: totalBytes,
          uploadedPartsCount: partNo,
          totalParts: totalParts,
        ),
      );
    }

    final completedSession = await _client.drive.uploadSessionsComplete(
      sessionId,
      CompleteUploadSessionRequest(
        contentType: request.blob.contentType,
        contentLength: '$totalBytes',
        checksumSha256Hex: 'sha256:$checksum',
        parts: completed,
      ),
    );
    _assertEnvelope(
      completedSession?.code,
      completedSession?.traceId,
      'complete upload session',
    );

    await _stateStore.clear(taskId);

    return DriveUploaderUploadResult(
      uploadItemId: prepared.id,
      uploadSessionId: sessionId,
      spaceId: prepared.spaceId,
      nodeId: prepared.nodeId,
      storageProviderId: prepared.storageProviderId,
      checksumSha256Hex: 'sha256:$checksum',
    );
  }

  Future<UploaderUploadItem> _prepare({
    required DriveUploaderRequest request,
    required String profile,
    required String checksumHex,
    required String taskId,
  }) async {
    final response = await _client.drive.uploaderUploadsCreate(
      PrepareUploaderUploadRequest(
        id: _newId(),
        taskId: taskId,
        appResourceType: request.appResourceType,
        appResourceId: request.appResourceId,
        uploadProfileCode: profile,
        fileFingerprint: checksumHex,
        originalFileName: request.blob.fileName,
        contentType: request.blob.contentType,
        contentLength: '${request.blob.sizeBytes}',
        chunkSizeBytes: '$_defaultChunkSizeBytes',
        spaceId: request.spaceId,
        parentNodeId: request.parentNodeId,
        retention: UploaderRetentionRequest(
          mode: request.retention.mode,
          ttlSeconds: request.retention.ttlSeconds,
        ),
        scene: request.scene,
        source: request.source,
      ),
    );
    _assertEnvelope(response?.code, response?.traceId, 'prepare upload');
    final data = response!.data;
    if (data is! Map<String, dynamic>) {
      throw DriveUploaderException('Drive prepare returned no upload item.');
    }
    return UploaderUploadItem.fromJson(data);
  }

  Future<PresignedUploadPart> _presign({
    required String sessionId,
    required int partNo,
    required int? requestedTtlSeconds,
  }) async {
    final response = await _client.drive.uploadSessionsPartsUpdate(
      sessionId,
      partNo,
      PresignUploadPartRequest(requestedTtlSeconds: requestedTtlSeconds),
    );
    _assertEnvelope(response?.code, response?.traceId, 'presign part $partNo');
    final data = response!.data;
    if (data is! Map<String, dynamic>) {
      throw DriveUploaderException('Drive presign returned no grant.');
    }
    return PresignedUploadPart.fromJson(data);
  }

  /// The only permitted raw byte transport: one PUT of the part slice to the
  /// short-lived provider URL Drive granted, with Drive-supplied headers.
  Future<String> _putPart({
    required String url,
    required Map<String, String> headers,
    required Uint8List bytes,
  }) async {
    final response = await _uploadHttp.put(
      Uri.parse(url),
      headers: headers,
      body: bytes,
    );
    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw DriveUploaderException(
        'Provider part upload failed with HTTP ${response.statusCode}.',
      );
    }
    final etag = response.headers['etag'];
    if (etag == null || etag.isEmpty) {
      throw DriveUploaderException(
        'Provider part upload did not return an etag.',
      );
    }
    return etag.replaceAll('"', '');
  }

  Future<void> _abortQuietly(String sessionId) async {
    try {
      await _client.drive.uploadSessionsAbort(
        sessionId,
        NodeCommandRequest(),
      );
    } catch (_) {
      // Abort is best-effort during caller cancellation; Drive maintenance
      // expires the session regardless.
    }
  }

  int _resolveChunkSize(UploaderUploadItem item) {
    final declared = int.tryParse(item.chunkSizeBytes);
    if (declared == null || declared <= 0) {
      return _defaultChunkSizeBytes;
    }
    return declared;
  }

  int _resolveTotalParts(int totalBytes, int chunkSize) {
    if (totalBytes == 0) {
      return 1;
    }
    return (totalBytes + chunkSize - 1) ~/ chunkSize;
  }

  void _assertStandardProfile(String profile, String? requested) {
    if (!driveUploaderProfiles.contains(profile)) {
      throw DriveUploaderException(
        'Profile "$profile" is not a DRIVE_SPEC.md section 8.1 standard profile.',
      );
    }
    if (requested != null && !driveUploaderProfiles.contains(requested)) {
      throw DriveUploaderException(
        'Requested profile "$requested" is not a standard profile.',
      );
    }
  }

  void _assertEnvelope(int? code, String? traceId, String step) {
    if (code == null) {
      throw DriveUploaderException('Drive $step returned no envelope.');
    }
    if (code != 0) {
      throw DriveUploaderException(
        'Drive $step failed.',
        code: code,
        traceId: traceId,
      );
    }
  }

  String _newId() {
    final random = Random.secure();
    final values = List<int>.generate(16, (_) => random.nextInt(256));
    values[6] = (values[6] & 0x0f) | 0x40;
    values[8] = (values[8] & 0x3f) | 0x80;
    final hex = values
        .map((value) => value.toRadixString(16).padLeft(2, '0'))
        .join();
    return '${hex.substring(0, 8)}-${hex.substring(8, 12)}-'
        '${hex.substring(12, 16)}-${hex.substring(16, 32)}';
  }
}
