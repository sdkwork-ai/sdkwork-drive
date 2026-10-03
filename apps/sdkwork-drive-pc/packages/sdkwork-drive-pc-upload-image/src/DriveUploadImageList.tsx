import { useMemo, useRef, useState, type ChangeEvent, type ReactNode } from "react";
import { ImagePlus, X } from "lucide-react";
import {
  describeDriveUploadImageRejection,
  mergeDriveUploadImageCopy,
  DriveUploadImageController,
  type DriveUploadImageCopy,
  type DriveUploadImageError,
  type DriveUploadImageRejectionCode,
  type DriveUploadImageService,
  type DriveUploadImageValue,
} from "@sdkwork/drive-upload-image-core";
import {
  useDriveUploadImageControlledValue,
  useDriveUploadImageController,
  useDriveUploadImageSnapshot,
} from "./useDriveUploadImageController";

/**
 * PC shell for configurable multi-image upload: a bounded thumbnail grid with
 * per-item remove/retry and an add tile, driven by the same injected
 * `DriveUploadImageService` as the single field.
 */

export interface DriveUploadImageListProps {
  service: DriveUploadImageService;
  controller?: DriveUploadImageController;
  /** Controlled uploaded values; omit for uncontrolled usage. */
  value?: readonly DriveUploadImageValue[];
  onChange?: (values: DriveUploadImageValue[]) => void;
  appResourceId?: string | (() => string | null | undefined);
  /** Maximum admitted images; the add tile hides at the cap. */
  maxFiles?: number;
  accept?: readonly string[];
  maxSizeBytes?: number;
  itemSizePx?: number;
  disabled?: boolean;
  readOnly?: boolean;
  label?: string;
  description?: string;
  alt?: string;
  showProgress?: boolean;
  copy?: Partial<DriveUploadImageCopy>;
  className?: string;
  onFileRejected?: (rejection: { code: DriveUploadImageRejectionCode }) => void;
  onUploadError?: (error: DriveUploadImageError) => void;
}

function cx(...names: Array<string | false | null | undefined>): string {
  return names.filter((name) => typeof name === "string" && name !== "").join(" ");
}

export function DriveUploadImageList(props: DriveUploadImageListProps): ReactNode {
  const {
    service,
    controller: externalController,
    value,
    onChange,
    appResourceId,
    maxFiles = 9,
    accept,
    maxSizeBytes,
    itemSizePx = 88,
    disabled = false,
    readOnly = false,
    label,
    description,
    alt,
    showProgress = true,
    copy: copyOverride,
    className,
    onFileRejected,
    onUploadError,
  } = props;

  const copy = useMemo(() => mergeDriveUploadImageCopy(copyOverride), [copyOverride]);
  const inputRef = useRef<HTMLInputElement>(null);

  const resolveAppResourceId = useMemo(() => {
    if (typeof appResourceId === "function") {
      return appResourceId;
    }
    return () => appResourceId ?? null;
  }, [appResourceId]);

  const controller = useDriveUploadImageController(
    {
      service,
      maxFiles,
      replaceOnMax: false,
      accept,
      maxSizeBytes,
      resolveAppResourceId,
      onRejected: (rejection) => onFileRejected?.({ code: rejection.code }),
      onFailed: (failure) => onUploadError?.(failure.error),
      onUploaded: (values) => onChange?.([...values]),
    },
    externalController,
  );
  const snapshot = useDriveUploadImageSnapshot(controller);
  useDriveUploadImageControlledValue(controller, value);

  const atCap = snapshot.values.length >= maxFiles;
  const interactive = !disabled && !readOnly;

  const openPicker = (): void => {
    if (interactive && !atCap && !snapshot.isUploading) {
      inputRef.current?.click();
    }
  };

  const handleInputChange = (event: ChangeEvent<HTMLInputElement>): void => {
    const files = Array.from(event.target.files ?? []);
    event.target.value = "";
    if (files.length > 0) {
      void controller.addFiles(files);
    }
  };

  return (
    <div className={cx("flex flex-col gap-1.5", className)}>
      {label !== undefined && label !== "" ? (
        <span className="text-sm font-medium text-neutral-700">{label}</span>
      ) : null}
      <ul role="list" className="flex flex-wrap gap-2">
        {snapshot.items.map((item) => {
          const rejectionText =
            item.status === "rejected" && item.rejectionCode !== null
              ? describeDriveUploadImageRejection(item.rejectionCode, copy, { maxSizeBytes })
              : null;
          return (
            <li key={item.id} className="flex flex-col gap-0.5">
              <span
                className={cx(
                  "relative flex items-center justify-center overflow-hidden rounded-lg border-2 border-dashed",
                  item.status === "uploaded" ? "border-neutral-200 bg-neutral-50" : "border-neutral-300 bg-neutral-50",
                )}
                style={{ width: `${itemSizePx}px`, height: `${itemSizePx}px` }}
                title={item.fileName}
              >
                {item.previewUrl !== null ? (
                  <img
                    src={item.previewUrl}
                    alt={alt ?? item.fileName}
                    className="h-full w-full object-cover"
                    draggable={false}
                  />
                ) : (
                  <ImagePlus aria-hidden="true" size={Math.max(16, Math.round(itemSizePx / 4))} className="text-neutral-400" />
                )}
                {item.status === "uploading" && showProgress ? (
                  <span className="absolute inset-0 flex items-end bg-black/30">
                    <span
                      className="h-1.5 bg-blue-400 transition-all"
                      style={{ width: `${item.progressPercent ?? 0}%` }}
                    />
                  </span>
                ) : null}
                {interactive && item.status !== "uploading" ? (
                  <button
                    type="button"
                    className="absolute right-1 top-1 flex h-5 w-5 items-center justify-center rounded-full bg-black/50 text-white hover:bg-red-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-blue-400"
                    aria-label={`${copy.removeImage}: ${item.fileName}`}
                    onClick={() => controller.removeItem(item.id)}
                  >
                    <X aria-hidden="true" size={11} />
                  </button>
                ) : null}
              </span>
              {rejectionText !== null ? (
                <span role="alert" className="max-w-[item] text-[11px] text-red-600" style={{ maxWidth: `${itemSizePx}px` }}>
                  {rejectionText}
                </span>
              ) : null}
              {item.status === "error" ? (
                <button
                  type="button"
                  className="text-[11px] text-blue-600 hover:underline"
                  aria-label={copy.retryUpload}
                  disabled={disabled}
                  onClick={() => void controller.retryItem(item.id)}
                >
                  {copy.retryUpload}
                </button>
              ) : null}
            </li>
          );
        })}
        {!readOnly && !atCap ? (
          <li>
            <button
              type="button"
              className={cx(
                "flex items-center justify-center rounded-lg border-2 border-dashed border-neutral-300 bg-neutral-50 text-neutral-400 transition-colors",
                interactive && !snapshot.isUploading
                  ? "cursor-pointer hover:border-blue-500 hover:text-blue-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-blue-500"
                  : "cursor-default opacity-60",
              )}
              style={{ width: `${itemSizePx}px`, height: `${itemSizePx}px` }}
              aria-label={copy.pickImage}
              aria-disabled={!interactive || snapshot.isUploading}
              disabled={disabled || snapshot.isUploading}
              onClick={openPicker}
            >
              <ImagePlus aria-hidden="true" size={Math.max(18, Math.round(itemSizePx / 3.5))} />
            </button>
          </li>
        ) : null}
      </ul>
      {snapshot.hasPending ? (
        <span className="text-xs text-amber-600">{copy.uploading}</span>
      ) : null}
      {description !== undefined && description !== "" ? (
        <span className="text-xs text-neutral-500">{description}</span>
      ) : null}
      <input
        ref={inputRef}
        type="file"
        className="hidden"
        accept={(accept ?? ["image/*"]).join(",")}
        multiple={maxFiles > 1}
        disabled={!interactive}
        aria-hidden="true"
        tabIndex={-1}
        onChange={handleInputChange}
      />
    </div>
  );
}
