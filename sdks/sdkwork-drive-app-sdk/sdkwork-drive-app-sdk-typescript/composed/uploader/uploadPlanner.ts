import type { DriveUploaderBlobLike, DriveUploaderPartPlan } from "./types";

export const DEFAULT_UPLOADER_CHUNK_SIZE_BYTES = 8 * 1024 * 1024;

export function planUploaderParts(
  file: Pick<DriveUploaderBlobLike, "size">,
  chunkSizeBytes: number = DEFAULT_UPLOADER_CHUNK_SIZE_BYTES,
): DriveUploaderPartPlan[] {
  if (!Number.isFinite(file.size) || file.size < 0) {
    throw new Error("Drive uploader file size must be a non-negative finite number.");
  }
  if (!Number.isFinite(chunkSizeBytes) || chunkSizeBytes <= 0) {
    throw new Error("Drive uploader chunk size must be a positive finite number.");
  }

  if (file.size === 0) {
    return [
      {
        partNo: 1,
        offsetBytes: 0,
        sizeBytes: 0,
      },
    ];
  }

  const parts: DriveUploaderPartPlan[] = [];
  let partNo = 1;
  for (let offsetBytes = 0; offsetBytes < file.size; offsetBytes += chunkSizeBytes) {
    parts.push({
      partNo,
      offsetBytes,
      sizeBytes: Math.min(chunkSizeBytes, file.size - offsetBytes),
    });
    partNo += 1;
  }
  return parts;
}

export function inferUploaderContentType(file: DriveUploaderBlobLike, fallback = "application/octet-stream"): string {
  const explicitType = file.type?.trim();
  if (explicitType) {
    return explicitType;
  }

  const name = file.name?.toLowerCase() || "";
  if (name.endsWith(".png")) return "image/png";
  if (name.endsWith(".jpg") || name.endsWith(".jpeg")) return "image/jpeg";
  if (name.endsWith(".webp")) return "image/webp";
  if (name.endsWith(".gif")) return "image/gif";
  if (name.endsWith(".mp4")) return "video/mp4";
  if (name.endsWith(".mov")) return "video/quicktime";
  if (name.endsWith(".mp3")) return "audio/mpeg";
  if (name.endsWith(".wav")) return "audio/wav";
  if (name.endsWith(".pdf")) return "application/pdf";
  if (name.endsWith(".zip")) return "application/zip";
  if (name.endsWith(".txt")) return "text/plain";
  if (name.endsWith(".md")) return "text/markdown";
  return fallback;
}

export function inferUploaderFileName(file: DriveUploaderBlobLike): string {
  return file.name?.trim() || "upload.bin";
}


