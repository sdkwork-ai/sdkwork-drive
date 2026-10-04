import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
  type ChangeEvent,
  type ForwardedRef,
  type ReactNode,
} from "react";
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
 * Mobile-React (H5) shell of the reusable Drive image-upload field: a large
 * touch target with optional album/camera source choice, backed by the
 * injected `DriveUploadImageService`. The component never composes upload
 * intent (`DRIVE_SPEC.md` §18.3) and renders only transient previews.
 *
 * Persist-first hosts pass `appResourceId` as a function returning `null` and
 * flush the parked pick through the ref handle (`uploadPending`) after the
 * entity exists, mirroring the PC shell and the mini-program instance API.
 */

export type DriveUploadImageSource = "album" | "camera";

export type DriveUploadImageShape = "circle" | "rounded" | "square";

/** Imperative surface for hosts driving the persist-first flow. */
export interface DriveUploadImageHandle {
  /**
   * Uploads parked pending files against the given entity anchor and resolves
   * with the full uploaded value set. Resolves with the current values when
   * nothing is parked.
   */
  uploadPending(
    options?: { appResourceId?: string | null },
  ): Promise<readonly DriveUploadImageValue[]>;
  /** `true` while a picked file waits for its entity anchor. */
  hasPending(): boolean;
  /** Uploaded values currently held by the field. */
  getValues(): readonly DriveUploadImageValue[];
  /** Drops every value, including a parked pending pick. */
  clear(): void;
}

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
  /** Pick sources offered to the user; default album only. */
  sources?: readonly DriveUploadImageSource[];
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
  /**
   * Fires whenever the field starts or stops holding a parked pending pick;
   * persist-first hosts wire it to their submit-button state.
   */
  onPendingChange?: (hasPending: boolean) => void;
  onFileRejected?: (rejection: { code: DriveUploadImageRejectionCode }) => void;
  onUploadError?: (error: DriveUploadImageError) => void;
}

function cx(...names: Array<string | false | null | undefined>): string {
  return names.filter((name) => typeof name === "string" && name !== "").join(" ");
}

const SHAPE_RADIUS: Record<DriveUploadImageShape, string> = {
  circle: "rounded-full",
  rounded: "rounded-2xl",
  square: "rounded-none",
};

export const DriveUploadImage = forwardRef<DriveUploadImageHandle, DriveUploadImageProps>(
  function DriveUploadImage(
    props: DriveUploadImageProps,
    ref: ForwardedRef<DriveUploadImageHandle>,
  ): ReactNode {
    const {
      service,
      controller: externalController,
      value,
      onChange,
      appResourceId,
      sources = ["album"],
      accept,
      maxSizeBytes,
      shape = "circle",
      sizePx = 80,
      disabled = false,
      readOnly = false,
      label,
      description,
      alt,
      showProgress = true,
      copy: copyOverride,
      className,
      onPendingChange,
      onFileRejected,
      onUploadError,
    } = props;

    const copy = useMemo(() => mergeDriveUploadImageCopy(copyOverride), [copyOverride]);
    const inputRef = useRef<HTMLInputElement>(null);
    const [sheetOpen, setSheetOpen] = useState(false);
    const onPendingChangeRef = useRef(onPendingChange);
    onPendingChangeRef.current = onPendingChange;

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

    useImperativeHandle(
      ref,
      (): DriveUploadImageHandle => ({
        uploadPending: (options) => controller.uploadPending(options),
        hasPending: () => controller.getSnapshot().hasPending,
        getValues: () => controller.getValues(),
        clear: () => controller.clear(),
      }),
      [controller],
    );

    const hasPending = snapshot.hasPending;
    useEffect(() => {
      onPendingChangeRef.current?.(hasPending);
    }, [hasPending]);

    const displayed = snapshot.items[snapshot.items.length - 1] ?? null;
    const previewSrc = displayed?.previewUrl ?? null;
    const isUploading = displayed?.status === "uploading";
    const interactive = !disabled && !readOnly && !isUploading;

    const openPicker = (source: DriveUploadImageSource): void => {
      setSheetOpen(false);
      const input = inputRef.current;
      if (input === null || !interactive) {
        return;
      }
      // The capture attribute must be current at click time; React state
      // updates settle after this handler returns.
      if (source === "camera") {
        input.setAttribute("capture", "environment");
      } else {
        input.removeAttribute("capture");
      }
      input.click();
    };

    const handleTrigger = (): void => {
      if (!interactive) {
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

    const sizeStyle = { width: `${sizePx}px`, height: `${sizePx}px` };
    const iconSize = Math.max(24, Math.round(sizePx / 3));
    const rejectionText =
      displayed?.status === "rejected" && displayed.rejectionCode !== null
        ? describeDriveUploadImageRejection(displayed.rejectionCode, copy, { maxSizeBytes })
        : null;
    const errorText =
      displayed?.status === "error" ? (displayed.errorMessage ?? copy.uploadFailed) : null;

    return (
      <div className={cx("inline-flex flex-col gap-2", className)}>
        {label !== undefined && label !== "" ? (
          <span className="text-base font-medium text-neutral-800">{label}</span>
        ) : null}
        <div
          className={cx(
            "relative flex items-center justify-center overflow-hidden border-2 border-dashed transition-colors",
            SHAPE_RADIUS[shape],
            interactive ? "active:border-blue-500 active:bg-blue-50" : "",
            previewSrc !== null ? "border-transparent" : "border-neutral-300 bg-neutral-50",
            disabled || readOnly ? "opacity-60" : "",
          )}
          style={sizeStyle}
          role="button"
          tabIndex={interactive ? 0 : -1}
          aria-label={displayed?.status === "uploaded" ? copy.replaceImage : copy.pickImage}
          aria-busy={isUploading}
          aria-disabled={!interactive}
          onClick={handleTrigger}
          onKeyDown={(event) => {
            if (event.key === "Enter" || event.key === " ") {
              event.preventDefault();
              handleTrigger();
            }
          }}
        >
          {previewSrc !== null ? (
            <img
              src={previewSrc}
              alt={alt ?? displayed?.fileName ?? copy.pickImage}
              className="h-full w-full object-cover"
              draggable={false}
            />
          ) : (
            <span className="flex flex-col items-center gap-1 p-2 text-neutral-400">
              <ImagePlus aria-hidden="true" size={iconSize} />
              <span className="max-w-full truncate px-1 text-xs">{copy.pickImage}</span>
            </span>
          )}
          {isUploading && showProgress ? (
            <span className="absolute inset-0 flex items-end bg-black/40">
              <span
                className="h-1.5 bg-blue-400 transition-all"
                style={{ width: `${displayed?.progressPercent ?? 0}%` }}
              />
            </span>
          ) : null}
        </div>

        <span className="flex items-center gap-3">
          {displayed !== null && !readOnly ? (
            <button
              type="button"
              className={cx(
                "inline-flex items-center gap-1 rounded-lg px-2 py-1 text-sm",
                disabled || isUploading
                  ? "cursor-not-allowed text-neutral-400"
                  : "text-neutral-500 active:bg-red-50 active:text-red-600",
              )}
              aria-label={copy.removeImage}
              disabled={disabled || isUploading}
              onClick={() => controller.removeItem(displayed.id)}
            >
              <X aria-hidden="true" size={14} />
              {copy.removeImage}
            </button>
          ) : null}
          {displayed?.status === "error" ? (
            <button
              type="button"
              className="inline-flex items-center gap-1 rounded-lg px-2 py-1 text-sm text-blue-600 active:bg-blue-50"
              aria-label={copy.retryUpload}
              disabled={disabled}
              onClick={() => void controller.retryItem(displayed.id)}
            >
              {copy.retryUpload}
            </button>
          ) : null}
        </span>

        {rejectionText !== null || errorText !== null ? (
          <span role="alert" className="text-sm text-red-600">
            {rejectionText ?? errorText}
          </span>
        ) : null}
        {description !== undefined && description !== "" ? (
          <span className="text-sm text-neutral-500">{description}</span>
        ) : null}

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
          multiple={false}
          disabled={!interactive}
          aria-hidden="true"
          tabIndex={-1}
          onChange={handleInputChange}
        />
      </div>
    );
  },
);
