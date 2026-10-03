import {
  DRIVE_UPLOAD_IMAGE_DEFAULT_ACCEPT,
  DRIVE_UPLOAD_IMAGE_DEFAULT_MAX_BYTES,
  type DriveUploadImageFileConstraints,
  type DriveUploadImageFileLike,
  type DriveUploadImageRejectionCode,
} from "./types";

/**
 * File admission rules for every platform shell. An image-upload component
 * only ever admits `image/*` content; the configurable `accept` list narrows
 * that further. Rules are pure so every platform rejects identically.
 */

export function resolveDriveUploadImageConstraints(
  constraints: DriveUploadImageFileConstraints,
): Required<Pick<DriveUploadImageFileConstraints, "accept">> & {
  maxSizeBytes: number;
} {
  return {
    accept: constraints.accept ?? DRIVE_UPLOAD_IMAGE_DEFAULT_ACCEPT,
    maxSizeBytes: constraints.maxSizeBytes ?? DRIVE_UPLOAD_IMAGE_DEFAULT_MAX_BYTES,
  };
}

function extensionOf(fileName: string | undefined): string | null {
  if (fileName === undefined) {
    return null;
  }
  const dot = fileName.lastIndexOf(".");
  if (dot < 0 || dot === fileName.length - 1) {
    return null;
  }
  return fileName.slice(dot).toLowerCase();
}

function matchesAcceptEntry(
  file: { type?: string; name?: string },
  entry: string,
): boolean {
  const normalized = entry.trim().toLowerCase();
  if (normalized === "") {
    return false;
  }
  if (normalized.startsWith(".")) {
    const extension = extensionOf(file.name);
    return extension !== null && extension === normalized;
  }
  const type = file.type?.toLowerCase() ?? "";
  if (type === "") {
    return false;
  }
  if (normalized.endsWith("/*")) {
    return type.startsWith(normalized.slice(0, -1));
  }
  return type === normalized;
}

function matchesAccept(
  file: { type?: string; name?: string },
  accept: readonly string[],
): boolean {
  return accept.some((entry) => matchesAcceptEntry(file, entry));
}

/**
 * Returns the rejection code for `file`, or `null` when the file is admitted.
 * `image/*` is a package invariant; `accept` narrows on top of it.
 */
export function validateDriveUploadImageFile(
  file: Pick<DriveUploadImageFileLike, "size" | "type" | "name">,
  constraints: DriveUploadImageFileConstraints = {},
): DriveUploadImageRejectionCode | null {
  const { accept, maxSizeBytes } = resolveDriveUploadImageConstraints(constraints);
  if (!Number.isFinite(file.size) || file.size <= 0) {
    return "empty-file";
  }
  const type = file.type?.toLowerCase() ?? "";
  const extension = extensionOf(file.name);
  const looksLikeImage =
    type.startsWith("image/") ||
    (type === "" && extension !== null && extension === ".png") ||
    (type === "" && extension !== null && extension === ".jpg") ||
    (type === "" && extension !== null && extension === ".jpeg") ||
    (type === "" && extension !== null && extension === ".gif") ||
    (type === "" && extension !== null && extension === ".webp");
  if (!looksLikeImage || !matchesAccept(file, accept)) {
    return "invalid-file-type";
  }
  if (file.size > maxSizeBytes) {
    return "file-too-large";
  }
  return null;
}

export function formatDriveUploadImageBytes(bytes: number): string {
  if (bytes >= 1024 * 1024) {
    const mebibytes = bytes / (1024 * 1024);
    return `${Number.isInteger(mebibytes) ? mebibytes : mebibytes.toFixed(1)} MB`;
  }
  if (bytes >= 1024) {
    return `${Math.round(bytes / 1024)} KB`;
  }
  return `${bytes} B`;
}
