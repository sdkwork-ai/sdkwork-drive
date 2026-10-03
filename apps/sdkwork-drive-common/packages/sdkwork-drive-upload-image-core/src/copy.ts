import { formatDriveUploadImageBytes } from "./validation";
import type { DriveUploadImageRejectionCode } from "./types";

/**
 * User-facing copy for the image-upload shells. Reusable packages ship
 * overridable defaults instead of hard-wiring a locale (`I18N_SPEC.md` §6.1):
 * hosts pass `copy: { ... }` with their localized strings and every shell
 * merges it over these defaults.
 */

export interface DriveUploadImageCopy {
  /** Accessible name / caption of the pick control. */
  pickImage: string;
  replaceImage: string;
  removeImage: string;
  retryUpload: string;
  uploading: string;
  uploadFailed: string;
  invalidFileType: string;
  fileTooLarge: string;
  emptyFile: string;
  tooManyFiles: string;
  /** `{max}` interpolates the human-readable configured ceiling. */
  fileTooLargeDetail: string;
  previewUnavailable: string;
}

export const defaultDriveUploadImageCopy: DriveUploadImageCopy = {
  pickImage: "Upload image",
  replaceImage: "Replace image",
  removeImage: "Remove image",
  retryUpload: "Retry upload",
  uploading: "Uploading…",
  uploadFailed: "Upload failed",
  invalidFileType: "Only image files are supported.",
  fileTooLarge: "This image is too large.",
  emptyFile: "This file is empty.",
  tooManyFiles: "Too many images selected.",
  fileTooLargeDetail: "Images must be {max} or smaller.",
  previewUnavailable: "Preview unavailable",
};

export function mergeDriveUploadImageCopy(
  override?: Partial<DriveUploadImageCopy>,
): DriveUploadImageCopy {
  return { ...defaultDriveUploadImageCopy, ...override };
}

export function formatDriveUploadImageMessage(
  template: string,
  params: Record<string, string | number>,
): string {
  return template.replace(/\{(\w+)\}/g, (matched, key: string) => {
    const value = params[key];
    return value === undefined ? matched : String(value);
  });
}

export function describeDriveUploadImageRejection(
  code: DriveUploadImageRejectionCode,
  copy: DriveUploadImageCopy,
  context?: { maxSizeBytes?: number },
): string {
  switch (code) {
    case "invalid-file-type":
      return copy.invalidFileType;
    case "file-too-large":
      return context?.maxSizeBytes === undefined
        ? copy.fileTooLarge
        : formatDriveUploadImageMessage(copy.fileTooLargeDetail, {
            max: formatDriveUploadImageBytes(context.maxSizeBytes),
          });
    case "empty-file":
      return copy.emptyFile;
    case "too-many-files":
      return copy.tooManyFiles;
  }
}
