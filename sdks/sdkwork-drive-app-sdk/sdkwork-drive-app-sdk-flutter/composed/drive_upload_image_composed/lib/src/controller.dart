import 'package:flutter/foundation.dart';

import 'service.dart';
import 'types.dart';

/// Shared upload state machine for the Flutter shells. Sequential uploads,
/// persist-first deferral, replace semantics, transient preview lifecycle,
/// and dispose/abort — mirroring `@sdkwork/drive-upload-image-core`.
///
/// The controller never sees the SDK client and never composes upload
/// intent; it only drives the injected [DriveUploadImageService].
enum DriveUploadImageItemStatus {
  pending,
  uploading,
  uploaded,
  error,
  rejected,
}

@immutable
class DriveUploadImageItem {
  const DriveUploadImageItem({
    required this.id,
    required this.status,
    required this.fileName,
    required this.sizeBytes,
    this.contentType,
    this.progressPercent,
    this.previewBytes,
    this.previewUrl,
    this.value,
    this.rejectionCode,
    this.errorMessage,
  });

  final String id;
  final DriveUploadImageItemStatus status;
  final String fileName;
  final int sizeBytes;
  final String? contentType;
  final double? progressPercent;
  /// Transient local preview bytes for a just-picked image.
  final Uint8List? previewBytes;
  /// Transient display URL (data URL or external address).
  final String? previewUrl;
  final DriveUploadImageValue? value;
  final DriveUploadImageRejectionCode? rejectionCode;
  final String? errorMessage;
}

@immutable
class DriveUploadImageSnapshot {
  const DriveUploadImageSnapshot({
    required this.items,
    required this.values,
    required this.isUploading,
    required this.hasPending,
  });

  final List<DriveUploadImageItem> items;
  final List<DriveUploadImageValue> values;
  final bool isUploading;
  final bool hasPending;
}

/// Picker abstraction; hosts adapt their plugin of choice (image_picker,
/// wechat_camera_picker, desktop file dialogs) in a few lines. See the
/// package README for the image_picker adapter.
abstract class DriveImagePicker {
  Future<List<DriveUploadImageSource>> pick({
    required int count,
    required bool preferCamera,
  });
}

class DriveUploadImageController extends ChangeNotifier {
  DriveUploadImageController({
    required DriveUploadImageService service,
    this.maxFiles = 1,
    bool? replaceOnMax,
    this.accept = driveUploadImageDefaultAccept,
    this.maxSizeBytes = driveUploadImageDefaultMaxBytes,
    this.resolveAppResourceId,
    this.onUploaded,
    this.onRejected,
    this.onFailed,
  })  : _service = service,
        replaceOnMax = replaceOnMax ?? maxFiles == 1;

  final DriveUploadImageService _service;
  final int maxFiles;
  final bool replaceOnMax;
  final List<String> accept;
  final int maxSizeBytes;
  final String? Function()? resolveAppResourceId;
  final void Function(List<DriveUploadImageValue> values)? onUploaded;
  final void Function(DriveUploadImageItem item,
      DriveUploadImageRejectionCode code)? onRejected;
  final void Function(DriveUploadImageItem item, DriveUploadImageException error)?
      onFailed;

  final List<_InternalItem> _items = <_InternalItem>[];
  final Map<String, String> _resolvedPreviews = <String, String>{};
  DriveUploadImageSnapshot _snapshot = const DriveUploadImageSnapshot(
    items: <DriveUploadImageItem>[],
    values: <DriveUploadImageValue>[],
    isUploading: false,
    hasPending: false,
  );
  int _nextItemId = 1;
  bool _draining = false;
  bool _destroyed = false;
  String? _lastAnchor;

  DriveUploadImageSnapshot get snapshot => _snapshot;

  List<DriveUploadImageValue> get values => _snapshot.values;

  /// Reconciles externally-owned values (form load, parent reset). Identical
  /// uri sequences are ignored so the parent's `onChanged` round-trip does
  /// not drop transient previews. Does not fire `onUploaded`.
  void setValues(List<DriveUploadImageValue>? values) {
    _assertAlive();
    final incoming = values ?? const <DriveUploadImageValue>[];
    final current =
        _items.where((item) => item.status == DriveUploadImageItemStatus.uploaded).toList();
    if (incoming.length == current.length) {
      var identical = true;
      for (var index = 0; index < incoming.length; index++) {
        if (incoming[index].uri != current[index].value?.uri) {
          identical = false;
          break;
        }
      }
      if (identical) {
        return;
      }
    }
    for (final item in current.toList()) {
      _dropItem(item.id);
    }
    for (final value in incoming) {
      _items.add(_InternalItem(
        id: 'image-${_nextItemId++}',
        status: DriveUploadImageItemStatus.uploaded,
        fileName: _fileNameOf(value),
        sizeBytes: 0,
        value: value,
        previewUrl: _transientPreviewOf(value),
      ));
    }
    _publish();
    _resolveMissingPreviews();
  }

  /// Admits picked sources: validates each, keeps admissible ones pending,
  /// and starts uploads for those that already have an entity anchor.
  Future<void> addSources(List<DriveUploadImageSource> sources) async {
    _assertAlive();
    for (final source in sources) {
      final rejectionCode = validateDriveUploadImageSource(
        source,
        accept: accept,
        maxSizeBytes: maxSizeBytes,
      );
      if (rejectionCode != null) {
        final item = _pushRejected(source, rejectionCode);
        onRejected?.call(item.snapshot, rejectionCode);
        continue;
      }
      if (replaceOnMax && maxFiles == 1) {
        for (final item in _items.toList()) {
          if (item.status != DriveUploadImageItemStatus.rejected) {
            _dropItem(item.id);
          }
        }
      } else if (_items
              .where((item) => item.status != DriveUploadImageItemStatus.rejected)
              .length >=
          maxFiles) {
        final item = _pushRejected(source, DriveUploadImageRejectionCode.tooManyFiles);
        onRejected?.call(item.snapshot, DriveUploadImageRejectionCode.tooManyFiles);
        continue;
      }
      _items.add(_InternalItem(
        id: 'image-${_nextItemId++}',
        status: DriveUploadImageItemStatus.pending,
        fileName: source.fileName ?? 'image',
        sizeBytes: source.sizeBytes,
        contentType: source.contentType,
        source: source,
      ));
    }
    _publish();
    final anchor = resolveAppResourceId?.call();
    if (anchor != null && anchor.trim().isNotEmpty) {
      await uploadPending(appResourceId: anchor);
    }
  }

  /// Uploads pending items with the given entity anchor (or the bound
  /// resolver) and resolves with the full uploaded set — the persist-first
  /// flow calls this after the entity exists.
  Future<List<DriveUploadImageValue>> uploadPending({String? appResourceId}) {
    _assertAlive();
    return _runUploads(appResourceId);
  }

  Future<List<DriveUploadImageValue>> retryItem(String itemId) async {
    _assertAlive();
    final item = _items
        .where((candidate) => candidate.id == itemId)
        .where((candidate) => candidate.status == DriveUploadImageItemStatus.error)
        .firstOrNull;
    if (item == null) {
      return values;
    }
    item.status = DriveUploadImageItemStatus.pending;
    item.errorMessage = null;
    _publish();
    return _runUploads(null);
  }

  void removeItem(String itemId) {
    _assertAlive();
    _dropItem(itemId);
    _publish();
    onUploaded?.call(values);
  }

  void clear() {
    _assertAlive();
    for (final item in _items.toList()) {
      _dropItem(item.id);
    }
    _publish();
  }

  @override
  void dispose() {
    _destroyed = true;
    _items.clear();
    super.dispose();
  }

  void _assertAlive() {
    if (_destroyed) {
      throw DriveUploadImageException(
        DriveUploadImageErrorCode.controllerDestroyed,
        'This controller has been disposed.',
      );
    }
  }

  _InternalItem _pushRejected(
    DriveUploadImageSource source,
    DriveUploadImageRejectionCode code,
  ) {
    final item = _InternalItem(
      id: 'image-${_nextItemId++}',
      status: DriveUploadImageItemStatus.rejected,
      fileName: source.fileName ?? 'image',
      sizeBytes: source.sizeBytes,
      contentType: source.contentType,
      rejectionCode: code,
    );
    _items.add(item);
    _publish();
    return item;
  }

  Future<List<DriveUploadImageValue>> _runUploads(String? queueAnchor) async {
    if (_draining) {
      return values;
    }
    _draining = true;
    try {
      final anchor = queueAnchor ??
          resolveAppResourceId?.call() ??
          _lastAnchor;
      if (anchor == null || anchor.trim().isEmpty) {
        return values;
      }
      _lastAnchor = anchor;
      for (final item in _items
          .where((item) => item.status == DriveUploadImageItemStatus.pending)
          .toList()) {
        if (_destroyed || item.status != DriveUploadImageItemStatus.pending) {
          continue;
        }
        final source = item.source;
        if (source == null) {
          continue;
        }
        item
          ..status = DriveUploadImageItemStatus.uploading
          ..progressPercent = 0
          ..errorMessage = null;
        _publish();
        try {
          final value = await _service.upload(
            file: source,
            appResourceId: anchor,
            onProgress: (progress) {
              item.progressPercent = progress.percent;
              notifyListeners();
            },
          );
          item
            ..status = DriveUploadImageItemStatus.uploaded
            ..value = value
            ..progressPercent = null
            ..source = null;
        } catch (error) {
          final failure = error is DriveUploadImageException
              ? error
              : DriveUploadImageException(
                  DriveUploadImageErrorCode.uploadFailed,
                  error.toString(),
                  error,
                );
          item
            ..status = DriveUploadImageItemStatus.error
            ..progressPercent = null
            ..errorMessage = failure.message;
          _publish();
          onFailed?.call(item.snapshot, failure);
          rethrow;
        }
        _publish();
      }
      onUploaded?.call(values);
      return values;
    } finally {
      _draining = false;
    }
  }

  void _dropItem(String itemId) {
    _items.removeWhere((item) => item.id == itemId);
  }

  String _fileNameOf(DriveUploadImageValue value) {
    final drive = value.driveMetadata;
    final name = drive?['originalFileName'];
    if (name is String && name.isNotEmpty) {
      return name;
    }
    return value.uri;
  }

  String? _transientPreviewOf(DriveUploadImageValue value) {
    if (value.source == 'drive') {
      return _resolvedPreviews[value.uri];
    }
    final isDirect = value.uri.startsWith('http://') ||
        value.uri.startsWith('https://') ||
        value.uri.startsWith('data:') ||
        value.uri.startsWith('blob:');
    return isDirect ? value.uri : null;
  }

  Future<void> _resolveMissingPreviews() async {
    for (final item in _items.toList()) {
      final value = item.value;
      if (item.status != DriveUploadImageItemStatus.uploaded ||
          value == null ||
          (item.previewUrl != null || item.previewBytes != null)) {
        continue;
      }
      try {
        final previewUrl = await _service.resolvePreview(uri: value.uri);
        if (previewUrl != null) {
          _resolvedPreviews[value.uri] = previewUrl;
          item.previewUrl = previewUrl;
          _publish();
        }
      } catch (_) {
        item.previewUrl = null;
      }
    }
  }

  void _publish() {
    _snapshot = DriveUploadImageSnapshot(
      items: _items.map((item) => item.snapshot).toList(growable: false),
      values: _items
          .where((item) =>
              item.status == DriveUploadImageItemStatus.uploaded &&
              item.value != null)
          .map((item) => item.value!)
          .toList(growable: false),
      isUploading: _items.any(
          (item) => item.status == DriveUploadImageItemStatus.uploading),
      hasPending: _items
          .any((item) => item.status == DriveUploadImageItemStatus.pending),
    );
    notifyListeners();
  }
}

class _InternalItem {
  _InternalItem({
    required this.id,
    required this.status,
    required this.fileName,
    required this.sizeBytes,
    this.contentType,
    this.previewUrl,
    this.value,
    this.rejectionCode,
    this.source,
  });

  final String id;
  DriveUploadImageItemStatus status;
  String fileName;
  int sizeBytes;
  String? contentType;
  double? progressPercent;
  Uint8List? previewBytes;
  String? previewUrl;
  DriveUploadImageValue? value;
  DriveUploadImageRejectionCode? rejectionCode;
  String? errorMessage;
  DriveUploadImageSource? source;

  DriveUploadImageItem get snapshot => DriveUploadImageItem(
        id: id,
        status: status,
        fileName: fileName,
        sizeBytes: sizeBytes,
        contentType: contentType,
        progressPercent: progressPercent,
        previewBytes: previewBytes,
        previewUrl: previewUrl,
        value: value,
        rejectionCode: rejectionCode,
        errorMessage: errorMessage,
      );
}
