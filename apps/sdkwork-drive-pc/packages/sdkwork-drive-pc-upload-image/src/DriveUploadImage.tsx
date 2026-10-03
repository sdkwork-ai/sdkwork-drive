import { useMemo, useRef, useState, type ChangeEvent, type DragEvent, type ReactNode } from "react";
import { ImagePlus, RefreshCw, X } from "lucide-react";
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
 * PC shell of the reusable Drive image-upload field: a single configurable
 * image slot (avatar circle by default) backed by the injected
 * `DriveUploadImageService`. The component never composes upload intent
 * (`DRIVE_SPEC.md` §18.3) and renders only transient previews.
 */

export type DriveUploadImageShape = "circle" | "rounded" | "square";

export interface DriveUploadImageProps {
  /** Injected by the host service layer (`createDriveUploadImageService`). */
  service: DriveUploadImageService;
  /** Inject a pre-built controller (tests, shared state); optional. */
  controller?: DriveUploadImageController;
  /** Controlled uploaded value; omit for uncontrolled usage. */
  value?: DriveUploadImageValue | null;
  onChange?: (value: DriveUploadImageValue | null) => void;
  /** Entity anchor at upload time; a function supports persist-first flows. */
  appResourceId?: string | (() => string | null | undefined);
  accept?: readonly string[];
  maxSizeBytes?: number;
  shape?: DriveUploadImageShape;
  sizePx?: number;
  disabled?: boolean;
  /** Renders the image without pick/remove affordances. */
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

const SHAPE_RADIUS: Record<DriveUploadImageShape, string> = {
  circle: "rounded-full",
  rounded: "rounded-xl",
  square: "rounded-none",
};

export function DriveUploadImage(props: DriveUploadImageProps): ReactNode {
  const {
    service,
    controller: externalController,
    value,
    onChange,
    appResourceId,
    accept,
    maxSizeBytes,
    shape = "circle",
    sizePx = 96,
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
  const [dragOver, setDragOver] = useState(false);

  const resolveAppResourceId = useMemo(() => {
    if (typeof appResourceId === "function") {
      return appResourceId;
    }
    return () => appResourceId ?? null;
  }, [appResourceId]);

  const controller = useDriveUploadImageController(
    {
      service,
      accept,
      maxSizeBytes,
      resolveAppResourceId,
      onRejected: (rejection) => onFileRejected?.({ code: rejection.code }),
      onFailed: (failure) => onUploadError?.(failure.error),
      onUploaded: (values) => onChange?.(values[values.length - 1] ?? null),
    },
    externalController,
  );
  const snapshot = useDriveUploadImageSnapshot(controller);
  useDriveUploadImageControlledValue(
    controller,
    value === undefined ? undefined : value === null ? [] : [value],
  );

  const displayed = snapshot.items[snapshot.items.length - 1] ?? null;
  const previewSrc = displayed?.previewUrl ?? null;
  const isUploading = displayed?.status === "uploading";
  const interactive = !disabled && !readOnly && !isUploading;

  const openPicker = (): void => {
    if (interactive) {
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

  const handleDrop = (event: DragEvent<HTMLDivElement>): void => {
    event.preventDefault();
    setDragOver(false);
    if (interactive) {
      const files = Array.from(event.dataTransfer.files);
      if (files.length > 0) {
        void controller.addFiles(files);
      }
    }
  };

  const removeImage = (): void => {
    if (disabled || isUploading || displayed === null) {
      return;
    }
    controller.removeItem(displayed.id);
  };

  const sizeStyle = { width: `${sizePx}px`, height: `${sizePx}px` };
  const rejectionText =
    displayed?.status === "rejected" && displayed.rejectionCode !== null
      ? describeDriveUploadImageRejection(displayed.rejectionCode, copy, { maxSizeBytes })
      : null;
  const errorText = displayed?.status === "error" ? (displayed.errorMessage ?? copy.uploadFailed) : null;

  return (
    <div className={cx("inline-flex flex-col gap-1.5", className)}>
      {label !== undefined && label !== "" ? (
        <span className="text-sm font-medium text-neutral-700">{label}</span>
      ) : null}
      <div
        className={cx(
          "group relative flex items-center justify-center overflow-hidden border-2 border-dashed transition-colors",
          SHAPE_RADIUS[shape],
          interactive ? "cursor-pointer hover:border-blue-500" : "cursor-default",
          dragOver && interactive ? "border-blue-500 bg-blue-50" : "border-neutral-300 bg-neutral-50",
          disabled || readOnly ? "opacity-60" : "",
        )}
        style={sizeStyle}
        role="button"
        tabIndex={interactive ? 0 : -1}
        aria-label={
          displayed?.status === "uploaded" ? copy.replaceImage : copy.pickImage
        }
        aria-disabled={!interactive}
        onClick={openPicker}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            openPicker();
          }
        }}
        onDragOver={(event) => {
          event.preventDefault();
          setDragOver(true);
        }}
        onDragLeave={() => setDragOver(false)}
        onDrop={handleDrop}
      >
        {previewSrc !== null ? (
          <img
            src={previewSrc}
            alt={alt ?? displayed?.fileName ?? copy.pickImage}
            className="h-full w-full object-cover"
            draggable={false}
          />
        ) : (
          <span className="flex flex-col items-center gap-1 text-neutral-400">
            <ImagePlus aria-hidden="true" size={Math.max(20, Math.round(sizePx / 3.5))} />
            <span className="px-1 text-center text-[11px] leading-tight">{copy.pickImage}</span>
          </span>
        )}

        {isUploading && showProgress ? (
          <span className="absolute inset-0 flex flex-col items-center justify-center gap-1 bg-black/40 text-white">
            <span className="text-[11px] font-medium">{copy.uploading}</span>
            <span className="absolute bottom-0 left-0 h-1 bg-blue-400 transition-all" style={{ width: `${displayed?.progressPercent ?? 0}%` }} />
          </span>
        ) : null}
      </div>

      <span className="flex items-center gap-2">
        {displayed !== null && !readOnly ? (
          <button
            type="button"
            className={cx(
              "inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-xs",
              disabled || isUploading
                ? "cursor-not-allowed text-neutral-400"
                : "text-neutral-500 hover:bg-red-50 hover:text-red-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-blue-500",
            )}
            aria-label={copy.removeImage}
            disabled={disabled || isUploading}
            onClick={removeImage}
          >
            <X aria-hidden="true" size={12} />
            {copy.removeImage}
          </button>
        ) : null}
        {displayed?.status === "error" ? (
          <button
            type="button"
            className="inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-xs text-blue-600 hover:bg-blue-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-blue-500"
            aria-label={copy.retryUpload}
            disabled={disabled}
            onClick={() => void controller.retryItem(displayed.id)}
          >
            <RefreshCw aria-hidden="true" size={12} />
            {copy.retryUpload}
          </button>
        ) : null}
      </span>

      {rejectionText !== null || errorText !== null ? (
        <span role="alert" className="text-xs text-red-600">
          {rejectionText ?? errorText}
        </span>
      ) : null}
      {description !== undefined && description !== "" ? (
        <span className="text-xs text-neutral-500">{description}</span>
      ) : null}

      <input
        ref={inputRef}
        type="file"
        className="hidden"
        accept={(accept ?? ["image/*"]).join(",")}
        multiple={false}
        disabled={!interactive}
        aria-hidden="true"
        tabIndex={-1}
        onChange={handleInputChange}
      />
    </div>
  );
}
