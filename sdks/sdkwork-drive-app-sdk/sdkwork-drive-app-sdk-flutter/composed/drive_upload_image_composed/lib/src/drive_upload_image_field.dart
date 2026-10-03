import 'dart:typed_data';

import 'package:flutter/material.dart';

import 'controller.dart';
import 'service.dart';
import 'types.dart';

/// Flutter shell of the reusable Drive image-upload family: a single
/// configurable image slot (avatar circle by default) backed by the injected
/// [DriveUploadImageService]. The widget never composes upload intent
/// (`DRIVE_SPEC.md` section 18.3) and renders only transient previews.
enum DriveUploadImageShape { circle, rounded, square }

class DriveUploadImageField extends StatefulWidget {
  const DriveUploadImageField({
    super.key,
    required this.service,
    required this.onPick,
    this.controller,
    this.value,
    this.onChanged,
    this.appResourceId,
    this.accept = driveUploadImageDefaultAccept,
    this.maxSizeBytes = driveUploadImageDefaultMaxBytes,
    this.shape = DriveUploadImageShape.circle,
    this.size = 96,
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

  /// Injected by the host service layer (`DriveUploaderImageService`).
  final DriveUploadImageService service;

  /// Host-owned pick adapter (image_picker or any plugin); invoked when the
  /// field is tapped. Selected sources are handed to the controller.
  final Future<List<DriveUploadImageSource>> Function() onPick;

  /// Inject a pre-built controller (tests, shared state); optional.
  final DriveUploadImageController? controller;

  /// Controlled uploaded value; omit for uncontrolled usage.
  final DriveUploadImageValue? value;
  final ValueChanged<DriveUploadImageValue?>? onChanged;

  /// Entity anchor at upload time; supports persist-first flows.
  final String? Function()? appResourceId;

  /// MIME or extension accept list, e.g. `[".png", "image/jpeg"]`.
  final List<String> accept;
  /// Upload ceiling in bytes; the shared default is 5 MiB.
  final int maxSizeBytes;

  final DriveUploadImageShape shape;
  final double size;
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
  State<DriveUploadImageField> createState() => _DriveUploadImageFieldState();
}

class _DriveUploadImageFieldState extends State<DriveUploadImageField> {
  DriveUploadImageController? _owned;
  late DriveUploadImageController _effective;

  @override
  void initState() {
    super.initState();
    final external = widget.controller;
    _effective = external ??
        (_owned = DriveUploadImageController(
          service: widget.service,
          accept: widget.accept,
          maxSizeBytes: widget.maxSizeBytes,
          resolveAppResourceId: () => widget.appResourceId?.call(),
          onRejected: (item, code) => widget.onFileRejected?.call(code),
          onFailed: (item, error) => widget.onUploadError?.call(error),
        ));
    if (widget.value != null) {
      _effective.setValues(<DriveUploadImageValue>[widget.value!]);
    }
    _effective.addListener(_handleControllerChanged);
  }

  /// Reports value changes for owned and injected controllers alike; the uri
  /// comparison suppresses the parent's `onChanged` round-trip.
  void _handleControllerChanged() {
    final values = _effective.values;
    final last = values.isEmpty ? null : values.last;
    if ((last?.uri ?? '') == (widget.value?.uri ?? '')) {
      return;
    }
    widget.onChanged?.call(last);
  }

  @override
  void didUpdateWidget(covariant DriveUploadImageField oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.value?.uri != oldWidget.value?.uri) {
      _effective.setValues(widget.value == null
          ? null
          : <DriveUploadImageValue>[widget.value!]);
    }
  }

  @override
  void dispose() {
    _effective.removeListener(_handleControllerChanged);
    _owned?.dispose();
    super.dispose();
  }

  Future<void> _handleTap() async {
    if (widget.disabled || widget.readOnly) {
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
        final items = _effective.snapshot.items;
        final displayed = items.isEmpty ? null : items.last;
        final isUploading = displayed?.status == DriveUploadImageItemStatus.uploading;
        final interactive = !widget.disabled && !widget.readOnly && !isUploading;
        final radius = switch (widget.shape) {
          DriveUploadImageShape.circle => 999.0,
          DriveUploadImageShape.rounded => 16.0,
          DriveUploadImageShape.square => 0.0,
        };

        final rejectionText = displayed?.rejectionCode == null
            ? null
            : _rejectionText(displayed!.rejectionCode!);
        final errorText =
            displayed?.status == DriveUploadImageItemStatus.error
                ? (displayed?.errorMessage ?? widget.copy.uploadFailed)
                : null;

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
            Semantics(
              button: true,
              enabled: interactive,
              label: displayed?.status == DriveUploadImageItemStatus.uploaded
                  ? widget.copy.replaceImage
                  : widget.copy.pickImage,
              child: GestureDetector(
                onTap: interactive ? _handleTap : null,
                child: Container(
                  width: widget.size,
                  height: widget.size,
                  clipBehavior: Clip.antiAlias,
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(radius),
                    border: Border.all(
                      color: Theme.of(context).colorScheme.outlineVariant,
                      style: BorderStyle.solid,
                    ),
                    color: Theme.of(context).colorScheme.surfaceContainerHighest,
                  ),
                  child: Stack(
                    fit: StackFit.expand,
                    children: [
                      if (displayed?.previewBytes != null)
                        Builder(builder: (context) {
                          final previewBytes = displayed?.previewBytes;
                          if (previewBytes == null) {
                            return const SizedBox.shrink();
                          }
                          return Image.memory(
                            previewBytes,
                            fit: BoxFit.cover,
                            semanticLabel: widget.alt ?? displayed?.fileName,
                          );
                        })
                      else if (displayed?.previewUrl != null)
                        Builder(builder: (context) {
                          final previewUrl = displayed?.previewUrl;
                          if (previewUrl == null) {
                            return const SizedBox.shrink();
                          }
                          return Image.network(
                            previewUrl,
                            fit: BoxFit.cover,
                            semanticLabel: widget.alt ?? displayed?.fileName,
                            errorBuilder: (_, __, ___) => Icon(
                              Icons.image_outlined,
                              size: widget.size / 3,
                              color: Theme.of(context).colorScheme.outline,
                            ),
                          );
                        })
                      else
                        Center(
                          child: Column(
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              Icon(
                                Icons.add_photo_alternate_outlined,
                                size: widget.size / 3,
                                color: Theme.of(context).colorScheme.outline,
                              ),
                              const SizedBox(height: 4),
                              Text(
                                widget.copy.pickImage,
                                style: Theme.of(context).textTheme.labelSmall,
                                textAlign: TextAlign.center,
                              ),
                            ],
                          ),
                        ),
                      if (isUploading && widget.showProgress)
                        ColoredBox(
                          color: Colors.black38,
                          child: Center(
                            child: Column(
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                Text(
                                  widget.copy.uploading,
                                  style: const TextStyle(
                                      color: Colors.white, fontSize: 11),
                                ),
                                const SizedBox(height: 4),
                                SizedBox(
                                  width: widget.size * 0.6,
                                  child: LinearProgressIndicator(
                                    value: displayed?.progressPercent == null
                                        ? null
                                        : displayed!.progressPercent! / 100,
                                    minHeight: 3,
                                  ),
                                ),
                              ],
                            ),
                          ),
                        ),
                    ],
                  ),
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Wrap(
                spacing: 8,
                crossAxisAlignment: WrapCrossAlignment.center,
                children: [
                  if (displayed != null && !widget.readOnly)
                    ActionChip(
                      label: Text(widget.copy.removeImage),
                      avatar: const Icon(Icons.close, size: 14),
                      onPressed: isUploading || widget.disabled
                          ? null
                          : () => _effective.removeItem(displayed.id),
                    ),
                  if (displayed?.status == DriveUploadImageItemStatus.error)
                    ActionChip(
                      label: Text(widget.copy.retryUpload),
                      avatar: const Icon(Icons.refresh, size: 14),
                      onPressed: widget.disabled || displayed == null
                          ? null
                          : () => _effective.retryItem(displayed.id),
                    ),
                ],
              ),
            ),
            if (rejectionText != null || errorText != null)
              Padding(
                padding: const EdgeInsets.only(top: 2),
                child: Text(
                  rejectionText ?? errorText!,
                  style: TextStyle(
                      color: Theme.of(context).colorScheme.error, fontSize: 12),
                ),
              ),
            if (widget.description != null)
              Padding(
                padding: const EdgeInsets.only(top: 2),
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

  String _rejectionText(DriveUploadImageRejectionCode code) {
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
}

/// Convenience bytes source used by host pickers.
DriveUploadImageSource driveUploadImageSourceFromBytes({
  required Uint8List bytes,
  String? fileName,
  String? contentType,
}) {
  return DriveUploadImageSource(
    bytes: bytes,
    fileName: fileName,
    contentType: contentType,
  );
}
