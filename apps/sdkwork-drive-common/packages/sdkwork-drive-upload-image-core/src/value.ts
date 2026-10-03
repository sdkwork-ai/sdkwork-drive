import type {
  DriveUploaderProgress,
  DriveUploaderUploadResult,
} from "@sdkwork/drive-app-sdk/uploader";
import type {
  DriveUploadImageDriveMetadata,
  DriveUploadImageProgress,
  DriveUploadImageValue,
} from "./types";
import { DriveUploadImageError } from "./types";

/**
 * Value-space helpers shared by every platform shell: mapping composed
 * uploader results into persist-safe `DriveUploadImageValue`s
 * (`DRIVE_SPEC.md` §10), parsing `drive://` references, and normalizing
 * progress snapshots. Ids stay strings end to end (`API_SPEC.md` §13.6).
 */

const DRIVE_URI_PATTERN = /^drive:\/\/spaces\/([^/]+)\/nodes\/([^/?#]+)/;

export function isDriveImageUri(uri: string): boolean {
  return DRIVE_URI_PATTERN.test(uri);
}

export interface DriveImageUriParts {
  spaceId: string;
  nodeId: string;
}

export function parseDriveImageUri(uri: string): DriveImageUriParts | null {
  const matched = DRIVE_URI_PATTERN.exec(uri);
  if (matched === null) {
    return null;
  }
  return { spaceId: matched[1] ?? "", nodeId: matched[2] ?? "" };
}

export function formatDriveImageUri(parts: DriveImageUriParts): string {
  return `drive://spaces/${parts.spaceId}/nodes/${parts.nodeId}`;
}

/**
 * Maps a completed composed-uploader result into the persist-safe value
 * stored in business form state. Presigned URLs and grants are dropped.
 */
export function buildDriveUploadImageValue(
  result: DriveUploaderUploadResult,
  file?: {
    name?: string | undefined;
    type?: string | undefined;
    size?: number | undefined;
  },
): DriveUploadImageValue {
  const spaceId = result.uploadItem.spaceId;
  const nodeId = result.uploadItem.nodeId;
  if (typeof spaceId !== "string" || spaceId === "" || typeof nodeId !== "string" || nodeId === "") {
    throw new DriveUploadImageError(
      "invalid-upload-result",
      "Drive did not return the uploaded image identity (spaceId/nodeId).",
    );
  }
  const driveMetadata: DriveUploadImageDriveMetadata = { spaceId, nodeId };
  const contentType = result.uploadItem.contentType || file?.type;
  if (contentType !== undefined && contentType !== "") {
    driveMetadata.contentType = contentType;
  }
  const contentLength =
    result.uploadItem.contentLength || (file?.size === undefined ? undefined : String(file.size));
  if (contentLength !== undefined && contentLength !== "") {
    driveMetadata.contentLength = contentLength;
  }
  const originalFileName = result.uploadItem.originalFileName || file?.name;
  if (originalFileName !== undefined && originalFileName !== "") {
    driveMetadata.originalFileName = originalFileName;
  }
  const checksumSha256Hex = result.uploadItem.checksumSha256Hex;
  if (checksumSha256Hex !== undefined && checksumSha256Hex !== "") {
    driveMetadata.checksumSha256Hex = checksumSha256Hex;
  }
  return {
    uri: formatDriveImageUri({ spaceId, nodeId }),
    source: "drive",
    metadata: { drive: driveMetadata },
  };
}

/** Wraps any thrown value into the package's typed upload failure. */
export function toDriveUploadImageFailure(error: unknown): DriveUploadImageError {
  if (error instanceof DriveUploadImageError) {
    return error;
  }
  const message = error instanceof Error ? error.message : String(error);
  return new DriveUploadImageError("upload-failed", message, { cause: error });
}

export function mapDriveUploaderProgress(
  progress: DriveUploaderProgress,
): DriveUploadImageProgress {
  const percent =
    progress.totalBytes > 0
      ? Math.min(100, Math.round((progress.uploadedBytes / progress.totalBytes) * 100))
      : 0;
  return {
    uploadedBytes: progress.uploadedBytes,
    totalBytes: progress.totalBytes,
    uploadedPartsCount: progress.uploadedPartsCount,
    totalParts: progress.totalParts,
    status: progress.status,
    percent,
  };
}
