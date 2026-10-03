import 'package:flutter/material.dart';

import 'controller.dart';
import 'service.dart';
import 'types.dart';

/// Flutter shell for configurable multi-image upload: a bounded thumbnail
/// grid with per-item remove/retry and an add tile, driven by the same
/// injected [DriveUploadImageService] as the single field.
class DriveUploadImageGridView extends StatefulWidget {
  const DriveUploadImageGridView({
    super.key,
    required this.service,
    required this.onPick,
    this.controller,
    this.value,
    this.onChanged,
    this.appResourceId,
    this.maxFiles = 9,
    this.accept = driveUploadImageDefaultAccept,
    this.maxSizeBytes = driveUploadImageDefaultMaxBytes,
    this.itemSize = 88,
    this.disabled = false,
    this.readOnly = false,
    this.label,
    this.description,
    this.alt,
    this.showProgress = true,
    this.copy = const DriveUploadImageCopy(),
    this.onFileRejected,
    this.onUploadError,
  });

  final DriveUploadImageService service;
  final Future<List<DriveUploadImageSource>> Function() onPick;
  final DriveUploadImageController? controller;
  final List<DriveUploadImageValue>? value;
  final ValueChanged<List<DriveUploadImageValue>>? onChanged;
  final String? Function()? appResourceId;
  final int maxFiles;
  /// MIME or extension accept list, e.g. `[".png", "image/jpeg"]`.
  final List<String> accept;
  /// Upload ceiling in bytes; the shared default is 5 MiB.
  final int maxSizeBytes;
  final double itemSize;
  final bool disabled;
  final bool readOnly;
  final String? label;
  final String? description;
  final String? alt;
  final bool showProgress;
  final DriveUploadImageCopy copy;
  final void Function(DriveUploadImageRejectionCode code)? onFileRejected;
  final void Function(DriveUploadImageException error)? onUploadError;

  @override
  State<DriveUploadImageGridView> createState() =>
      _DriveUploadImageGridViewState();
}

class _DriveUploadImageGridViewState extends State<DriveUploadImageGridView> {
  DriveUploadImageController? _owned;
  late DriveUploadImageController _effective;

  @override
  void initState() {
    super.initState();
    final external = widget.controller;
    _effective = external ??
        (_owned = DriveUploadImageController(
          service: widget.service,
          maxFiles: widget.maxFiles,
          replaceOnMax: false,
          accept: widget.accept,
          maxSizeBytes: widget.maxSizeBytes,
          resolveAppResourceId: () => widget.appResourceId?.call(),
          onRejected: (item, code) => widget.onFileRejected?.call(code),
          onFailed: (item, error) => widget.onUploadError?.call(error),
        ));
    if (widget.value != null) {
      _effective.setValues(widget.value);
    }
    _effective.addListener(_handleControllerChanged);
  }

  /// Reports value-set changes for owned and injected controllers alike; the
  /// joined-uri comparison suppresses the parent's `onChanged` round-trip.
  void _handleControllerChanged() {
    final uris =
        _effective.values.map((entry) => entry.uri).join('\n');
    final widgetUris =
        widget.value?.map((entry) => entry.uri).join('\n') ?? '';
    if (uris == widgetUris) {
      return;
    }
    widget.onChanged?.call(List.of(_effective.values));
  }

  @override
  void didUpdateWidget(covariant DriveUploadImageGridView oldWidget) {
    super.didUpdateWidget(oldWidget);
    final oldUris = oldWidget.value?.map((entry) => entry.uri).toList() ?? const <String>[];
    final newUris = widget.value?.map((entry) => entry.uri).toList() ?? const <String>[];
    if (oldUris.join('\n') != newUris.join('\n')) {
      _effective.setValues(widget.value);
    }
  }

  @override
  void dispose() {
    _effective.removeListener(_handleControllerChanged);
    _owned?.dispose();
    super.dispose();
  }

  Future<void> _handleAdd() async {
    if (widget.disabled || widget.readOnly || _effective.snapshot.isUploading) {
      return;
    }
    try {
      final sources = await widget.onPick();
      await _effective.addSources(sources);
    } on DriveUploadImageException catch (error) {
      widget.onUploadError?.call(error);
    }
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: _effective,
      builder: (context, _) {
        final snapshot = _effective.snapshot;
        final atCap = snapshot.values.length >= widget.maxFiles;

        String rejectionText(DriveUploadImageItem item) {
          final code = item.rejectionCode;
          if (code == null) {
            return item.errorMessage ?? '';
          }
          switch (code) {
            case DriveUploadImageRejectionCode.invalidFileType:
              return widget.copy.invalidFileType;
            case DriveUploadImageRejectionCode.fileTooLarge:
              return '${widget.copy.fileTooLarge} '
                  '(<= ${formatDriveUploadImageBytes(widget.maxSizeBytes)})';
            case DriveUploadImageRejectionCode.emptyFile:
              return widget.copy.emptyFile;
            case DriveUploadImageRejectionCode.tooManyFiles:
              return widget.copy.tooManyFiles;
          }
        }

        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            if (widget.label != null)
              Padding(
                padding: const EdgeInsets.only(bottom: 6),
                child: Text(widget.label!,
                    style: Theme.of(context).textTheme.titleSmall),
              ),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                for (final item in snapshot.items)
                  SizedBox(
                    width: widget.itemSize,
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Stack(
                          children: [
                            Container(
                              width: widget.itemSize,
                              height: widget.itemSize,
                              clipBehavior: Clip.antiAlias,
                              decoration: BoxDecoration(
                                borderRadius: BorderRadius.circular(12),
                                border: Border.all(
                                  color: Theme.of(context)
                                      .colorScheme
                                      .outlineVariant,
                                ),
                                color: Theme.of(context)
                                    .colorScheme
                                    .surfaceContainerHighest,
                              ),
                              child: Builder(builder: (context) {
                                final previewBytes = item.previewBytes;
                                final previewUrl = item.previewUrl;
                                if (previewBytes != null) {
                                  return Image.memory(
                                    previewBytes,
                                    fit: BoxFit.cover,
                                    semanticLabel:
                                        widget.alt ?? item.fileName,
                                  );
                                }
                                if (previewUrl != null) {
                                  return Image.network(
                                    previewUrl,
                                    fit: BoxFit.cover,
                                    semanticLabel:
                                        widget.alt ?? item.fileName,
                                  );
                                }
                                return Icon(
                                  Icons.image_outlined,
                                  size: widget.itemSize / 4,
                                  color:
                                      Theme.of(context).colorScheme.outline,
                                );
                              }),
                            ),
                            if (item.status ==
                                DriveUploadImageItemStatus.uploading)
                              Positioned(
                                left: 0,
                                right: 0,
                                bottom: 0,
                                child: LinearProgressIndicator(
                                  value: item.progressPercent == null
                                      ? null
                                      : item.progressPercent! / 100,
                                  minHeight: 4,
                                ),
                              ),
                            if (!widget.readOnly &&
                                item.status !=
                                    DriveUploadImageItemStatus.uploading)
                              Positioned(
                                top: 4,
                                right: 4,
                                child: InkWell(
                                  onTap: () => _effective.removeItem(item.id),
                                  child: Container(
                                    width: 20,
                                    height: 20,
                                    decoration: const BoxDecoration(
                                      color: Colors.black54,
                                      shape: BoxShape.circle,
                                    ),
                                    child: const Icon(Icons.close,
                                        size: 12, color: Colors.white),
                                  ),
                                ),
                              ),
                          ],
                        ),
                        if (item.rejectionCode != null ||
                            item.status == DriveUploadImageItemStatus.error)
                          Text(
                            rejectionText(item),
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                            style: TextStyle(
                              fontSize: 10,
                              color: Theme.of(context).colorScheme.error,
                            ),
                          ),
                        if (item.status == DriveUploadImageItemStatus.error)
                          TextButton(
                            style: TextButton.styleFrom(
                              padding: const EdgeInsets.symmetric(horizontal: 4),
                              minimumSize: Size.zero,
                              tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                            ),
                            onPressed: widget.disabled
                                ? null
                                : () => _effective.retryItem(item.id),
                            child: Text(widget.copy.retryUpload,
                                style: const TextStyle(fontSize: 11)),
                          ),
                      ],
                    ),
                  ),
                if (!widget.readOnly && !atCap)
                  InkWell(
                    onTap: _handleAdd,
                    borderRadius: BorderRadius.circular(12),
                    child: Container(
                      width: widget.itemSize,
                      height: widget.itemSize,
                      decoration: BoxDecoration(
                        borderRadius: BorderRadius.circular(12),
                        border: Border.all(
                          color: Theme.of(context).colorScheme.outlineVariant,
                        ),
                        color:
                            Theme.of(context).colorScheme.surfaceContainerHighest,
                      ),
                      child: Icon(
                        Icons.add_photo_alternate_outlined,
                        size: widget.itemSize / 3,
                        color: Theme.of(context).colorScheme.outline,
                        semanticLabel: widget.copy.pickImage,
                      ),
                    ),
                  ),
              ],
            ),
            if (widget.description != null)
              Padding(
                padding: const EdgeInsets.only(top: 4),
                child: Text(
                  widget.description!,
                  style: Theme.of(context)
                      .textTheme
                      .bodySmall
                      ?.copyWith(color: Theme.of(context).colorScheme.outline),
                ),
              ),
          ],
        );
      },
    );
  }
}
