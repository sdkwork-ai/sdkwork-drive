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
): { accept: readonly string[]; maxSizeBytes: number } {
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

/**
 * Canonical mime for common image extensions. Platforms without a content
 * type on picked files (mini-program `wx.chooseMedia`) still admit and match
 * through this map; anything unmapped stays empty and is rejected.
 */
const EXTENSION_MIME: Readonly<Record<string, string>> = {
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".gif": "image/gif",
  ".webp": "image/webp",
  ".avif": "image/avif",
  ".heic": "image/heic",
};

function effectiveTypeOf(file: { type?: string; name?: string }): string {
  const declared = file.type?.toLowerCase() ?? "";
  if (declared !== "") {
    return declared;
  }
  const extension = extensionOf(file.name);
  if (extension === null) {
    return "";
  }
  return EXTENSION_MIME[extension] ?? "";
}

function matchesAcceptEntry(
  file: { type?: string; name?: string },
  entry: string,
  effectiveType: string,
): boolean {
  const normalized = entry.trim().toLowerCase();
  if (normalized === "") {
    return false;
  }
  if (normalized.startsWith(".")) {
    const extension = extensionOf(file.name);
    return extension !== null && extension === normalized;
  }
  if (effectiveType === "") {
    return false;
  }
  if (normalized.endsWith("/*")) {
    return effectiveType.startsWith(normalized.slice(0, -1));
  }
  return effectiveType === normalized;
}

function matchesAccept(
  file: { type?: string; name?: string },
  accept: readonly string[],
  effectiveType: string,
): boolean {
  return accept.some((entry) => matchesAcceptEntry(file, entry, effectiveType));
}

/**
 * Returns the rejection code for `file`, or `null` when the file is admitted.
 * `image/*` is a package invariant; `accept` narrows on top of it. Files
 * without a declared mime are identified through their image extension map.
 */
export function validateDriveUploadImageFile(
  file: Pick<DriveUploadImageFileLike, "size" | "type" | "name">,
  constraints: DriveUploadImageFileConstraints = {},
): DriveUploadImageRejectionCode | null {
  const { accept, maxSizeBytes } = resolveDriveUploadImageConstraints(constraints);
  if (!Number.isFinite(file.size) || file.size <= 0) {
    return "empty-file";
  }
  const effectiveType = effectiveTypeOf(file);
  if (!effectiveType.startsWith("image/") || !matchesAccept(file, accept, effectiveType)) {
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
