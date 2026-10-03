import { useMemo, useRef, useState, type ChangeEvent, type ReactNode } from "react";
import { Camera, ImagePlus, Images, X } from "lucide-react";
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
 * Mobile-React (H5) shell for configurable multi-image upload: a bounded
 * thumbnail grid with per-item remove/retry and an add tile, driven by the
 * same injected `DriveUploadImageService` as the single field.
 */

export type DriveUploadImageSource = "album" | "camera";

export interface DriveUploadImageListProps {
  service: DriveUploadImageService;
  controller?: DriveUploadImageController;
  /** Controlled uploaded values; omit for uncontrolled usage. */
  value?: readonly DriveUploadImageValue[];
  onChange?: (values: DriveUploadImageValue[]) => void;
  appResourceId?: string | (() => string | null | undefined);
  /** Pick sources offered to the user; default album only. */
  sources?: readonly DriveUploadImageSource[];
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
    sources = ["album"],
    maxFiles = 9,
    accept,
    maxSizeBytes,
    itemSizePx = 76,
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
  const [sheetOpen, setSheetOpen] = useState(false);

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

  const openPicker = (source: DriveUploadImageSource): void => {
    setSheetOpen(false);
    const input = inputRef.current;
    if (input === null || !interactive) {
      return;
    }
    if (source === "camera") {
      input.setAttribute("capture", "environment");
    } else {
      input.removeAttribute("capture");
    }
    input.click();
  };

  const handleTrigger = (): void => {
    if (!interactive || atCap || snapshot.isUploading) {
      return;
    }
    if (sources.length > 1) {
      setSheetOpen(true);
      return;
    }
    openPicker(sources[0] ?? "album");
  };

  const handleInputChange = (event: ChangeEvent<HTMLInputElement>): void => {
    const files = Array.from(event.target.files ?? []);
    event.target.value = "";
    if (files.length > 0) {
      void controller.addFiles(files);
    }
  };

  return (
    <div className={cx("flex flex-col gap-2", className)}>
      {label !== undefined && label !== "" ? (
        <span className="text-base font-medium text-neutral-800">{label}</span>
      ) : null}
      <ul role="list" className="flex flex-wrap gap-2">
        {snapshot.items.map((item) => {
          const rejectionText =
            item.status === "rejected" && item.rejectionCode !== null
              ? describeDriveUploadImageRejection(item.rejectionCode, copy, { maxSizeBytes })
              : null;
          return (
            <li key={item.id} className="flex flex-col gap-1">
              <span
                className={cx(
                  "relative flex items-center justify-center overflow-hidden rounded-2xl border-2 border-dashed",
                  item.status === "uploaded"
                    ? "border-neutral-200 bg-neutral-50"
                    : "border-neutral-300 bg-neutral-50",
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
                  <ImagePlus aria-hidden="true" size={Math.max(18, Math.round(itemSizePx / 4))} className="text-neutral-400" />
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
                    className="absolute right-1 top-1 flex h-6 w-6 items-center justify-center rounded-full bg-black/50 text-white active:bg-red-600"
                    aria-label={`${copy.removeImage}: ${item.fileName}`}
                    onClick={() => controller.removeItem(item.id)}
                  >
                    <X aria-hidden="true" size={12} />
                  </button>
                ) : null}
              </span>
              {rejectionText !== null ? (
                <span role="alert" className="text-xs text-red-600" style={{ maxWidth: `${itemSizePx}px` }}>
                  {rejectionText}
                </span>
              ) : null}
              {item.status === "error" ? (
                <button
                  type="button"
                  className="text-xs text-blue-600 active:underline"
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
                "flex items-center justify-center rounded-2xl border-2 border-dashed border-neutral-300 bg-neutral-50 text-neutral-400 transition-colors",
                interactive && !snapshot.isUploading ? "active:border-blue-500 active:bg-blue-50" : "opacity-60",
              )}
              style={{ width: `${itemSizePx}px`, height: `${itemSizePx}px` }}
              aria-label={copy.pickImage}
              aria-disabled={!interactive || snapshot.isUploading}
              disabled={disabled || snapshot.isUploading}
              onClick={handleTrigger}
            >
              <ImagePlus aria-hidden="true" size={Math.max(20, Math.round(itemSizePx / 3.2))} />
            </button>
          </li>
        ) : null}
      </ul>

      {sheetOpen ? (
        <div
          className="fixed inset-0 z-50 flex items-end bg-black/40"
          role="dialog"
          aria-modal="true"
          aria-label={copy.pickImage}
          onClick={() => setSheetOpen(false)}
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              setSheetOpen(false);
            }
          }}
        >
          <div
            className="w-full rounded-t-2xl bg-white p-2 pb-6"
            role="presentation"
            onClick={(event) => event.stopPropagation()}
          >
            {sources.includes("camera") ? (
              <button
                type="button"
                className="flex w-full items-center gap-3 rounded-xl px-4 py-3 text-base text-neutral-800 active:bg-neutral-100"
                onClick={() => openPicker("camera")}
              >
                <Camera aria-hidden="true" size={18} />
                {copy.takePhoto}
              </button>
            ) : null}
            {sources.includes("album") ? (
              <button
                type="button"
                className="flex w-full items-center gap-3 rounded-xl px-4 py-3 text-base text-neutral-800 active:bg-neutral-100"
                onClick={() => openPicker("album")}
              >
                <Images aria-hidden="true" size={18} />
                {copy.chooseFromAlbum}
              </button>
            ) : null}
            <button
              type="button"
              className="mt-2 w-full rounded-xl border-t border-neutral-100 px-4 py-3 text-base text-neutral-500 active:bg-neutral-100"
              onClick={() => setSheetOpen(false)}
            >
              {copy.cancel}
            </button>
          </div>
        </div>
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
