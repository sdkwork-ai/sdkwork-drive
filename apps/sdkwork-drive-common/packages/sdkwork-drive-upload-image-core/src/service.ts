import type {
  DriveUploaderProgress,
  DriveUploaderRequest,
  DriveUploaderUploadResult,
} from "@sdkwork/drive-app-sdk/uploader";
import { assertDriveUploadImageDeclaration } from "./declaration";
import {
  DRIVE_UPLOAD_IMAGE_PREVIEW_MAX_BYTES,
  type DriveImagePreviewReaderLike,
  type DriveUploadImageDeclaration,
  type DriveUploadImageFileLike,
  type DriveUploadImageProfile,
  type DriveUploadImageProgress,
  type DriveUploadImageService,
  type DriveUploadImageUploaderLike,
  type DriveUploadImageValue,
} from "./types";
import { DriveUploadImageError } from "./types";
import { buildDriveUploadImageValue, mapDriveUploaderProgress, parseDriveImageUri } from "./value";

/**
 * Service factory the host application's service layer calls once with its
 * declared upload intent and the composed Drive uploader — the component
 * receives only the resulting `DriveUploadImageService`
 * (`DRIVE_SPEC.md` §18.3: the service layer, not the UI, supplies declared
 * values; ambient identity is derived server-side).
 */

export interface DriveUploadImageServiceOptions {
  uploader: DriveUploadImageUploaderLike;
  declaration: DriveUploadImageDeclaration;
  /** Overrides `declaration.uploadProfileCode` when a shell needs a fixed one. */
  profile?: DriveUploadImageProfile;
  previewReader?: DriveImagePreviewReaderLike;
  /** Read ceiling for Drive-backed previews; default 2 MiB. */
  previewMaxBytes?: number;
  /**
   * Explicit Drive target for hosts whose contract pins an upload space or
   * parent folder (`DRIVE_SPEC.md` §9.4: explicit targets require Drive
   * permission validation). Omit for the default caller-owned Upload space.
   */
  spaceId?: string;
  parentNodeId?: string;
}

function uploaderMethodForProfile(
  uploader: DriveUploadImageUploaderLike,
  profile: DriveUploadImageProfile,
): (request: DriveUploaderRequest) => Promise<DriveUploaderUploadResult> {
  switch (profile) {
    case "avatar":
      return (request) => uploader.uploadAvatar(request);
    case "thumbnail":
      return (request) => uploader.uploadThumbnail(request);
    case "image":
      return (request) => uploader.uploadImage(request);
  }
}

/**
 * Retention travels from the declaration (`DRIVE_SPEC.md` §18.1): the
 * declared mode is what Drive records; temporary declarations carry their
 * declared TTL (wire type is int64-as-string).
 */
function retentionRequest(declaration: DriveUploadImageDeclaration): {
  retention?: { mode: "long_term" | "temporary"; ttlSeconds?: string };
} {
  if (declaration.retention === "temporary") {
    return {
      retention: {
        mode: "temporary",
        ...(declaration.retentionTtlSeconds === undefined
          ? {}
          : { ttlSeconds: String(declaration.retentionTtlSeconds) }),
      },
    };
  }
  return { retention: { mode: "long_term" } };
}

/**
 * Adapts the loose byte-source contract onto the composed uploader's
 * `DriveUploaderBlobLike`. Ranged sources (mini-program file adapters) pass
 * through untouched; `arrayBuffer` sources are served from their bytes, which
 * stay bounded by the validated image size limit.
 */
async function toUploaderBlob(
  file: DriveUploadImageFileLike,
): Promise<DriveUploaderRequest["file"]> {
  const identity = {
    ...(file.type === undefined ? {} : { type: file.type }),
    ...(file.name === undefined ? {} : { name: file.name }),
  };
  if (typeof file.readRange === "function") {
    const readRange = file.readRange.bind(file);
    return {
      size: file.size,
      ...identity,
      readRange: (offsetBytes, lengthBytes) => readRange(offsetBytes, lengthBytes),
    };
  }
  if (typeof file.arrayBuffer === "function") {
    const bytes = new Uint8Array(await file.arrayBuffer());
    return {
      size: bytes.byteLength,
      ...identity,
      readRange: (offsetBytes, lengthBytes) =>
        Promise.resolve(
          bytes.slice(offsetBytes, offsetBytes + lengthBytes).buffer,
        ),
    };
  }
  throw new DriveUploadImageError(
    "upload-failed",
    "The picked image exposes neither readRange nor arrayBuffer bytes.",
  );
}

/**
 * Builds a bounded, cached preview reader over the generated Drive nodes
 * content API. Hosts pass `drive.drive.nodes` from the app SDK client. The
 * reader returns transient data URLs only — results are presentation state
 * and must never enter business form state (`DRIVE_SPEC.md` §9).
 */
export function createDriveNodesImagePreviewReader(nodes: {
  content: {
    retrieve(
      nodeId: string,
      options: { encoding: "base64"; maxBytes: number; signal?: AbortSignal },
    ): Promise<{ content?: string; contentType?: string; hasMore?: boolean }>;
  };
}): DriveImagePreviewReaderLike {
  const cache = new Map<string, string>();
  const inflight = new Map<string, Promise<string | null>>();
  const maxCacheEntries = 64;

  return {
    readImagePreview({ nodeId, maxBytes, signal }) {
      const cacheKey = `${nodeId}:${maxBytes}`;
      const cached = cache.get(cacheKey);
      if (cached !== undefined) {
        return Promise.resolve(cached);
      }
      const pending = inflight.get(cacheKey);
      if (pending !== undefined) {
        return pending;
      }
      const read = nodes.content
        .retrieve(nodeId, {
          encoding: "base64",
          maxBytes,
          ...(signal === undefined ? {} : { signal }),
        })
        .then((content) => {
          if (content.hasMore || typeof content.content !== "string" || content.content === "") {
            return null;
          }
          const dataUrl = `data:${content.contentType || "image/png"};base64,${content.content}`;
          cache.set(cacheKey, dataUrl);
          if (cache.size > maxCacheEntries) {
            const oldest = cache.keys().next();
            if (oldest.done !== true) {
              cache.delete(oldest.value);
            }
          }
          return dataUrl;
        })
        .finally(() => {
          inflight.delete(cacheKey);
        });
      inflight.set(cacheKey, read);
      return read;
    },
  };
}

export function createDriveUploadImageService(
  options: DriveUploadImageServiceOptions,
): DriveUploadImageService {
  assertDriveUploadImageDeclaration(options.declaration);
  const profile = options.profile ?? options.declaration.uploadProfileCode;
  const previewMaxBytes = options.previewMaxBytes ?? DRIVE_UPLOAD_IMAGE_PREVIEW_MAX_BYTES;

  return {
    async upload({ file, appResourceId, signal, onProgress }) {
      if (typeof appResourceId !== "string" || appResourceId.trim() === "") {
        throw new DriveUploadImageError(
          "missing-app-resource-id",
          "Drive upload requires the identifier of an existing entity; persist the entity first, then upload (DRIVE_SPEC.md §18.3).",
        );
      }
      const upload = uploaderMethodForProfile(options.uploader, profile);
      const blob = await toUploaderBlob(file);
      const result = await upload({
        file: blob,
        appResourceType: options.declaration.appResourceType,
        appResourceId,
        scene: options.declaration.scene,
        source: options.declaration.source,
        uploadProfileCode: options.declaration.uploadProfileCode,
        ...(options.spaceId === undefined ? {} : { spaceId: options.spaceId }),
        ...(options.parentNodeId === undefined ? {} : { parentNodeId: options.parentNodeId }),
        ...retentionRequest(options.declaration),
        ...(file.name === undefined ? {} : { originalFileName: file.name }),
        ...(file.type === undefined ? {} : { contentType: file.type }),
        ...(signal === undefined ? {} : { signal }),
        ...(onProgress === undefined
          ? {}
          : {
              onProgress: (progress: DriveUploaderProgress) => {
                onProgress(mapDriveUploaderProgress(progress));
              },
            }),
      });
      return buildDriveUploadImageValue(result, {
        name: file.name,
        type: file.type,
        size: file.size,
      });
    },

    async resolvePreview({ uri, maxBytes, signal }) {
      if (!uri.startsWith("drive://")) {
        return uri.startsWith("http://") ||
          uri.startsWith("https://") ||
          uri.startsWith("data:") ||
          uri.startsWith("blob:")
          ? uri
          : null;
      }
      const reader = options.previewReader;
      if (reader === undefined) {
        throw new DriveUploadImageError(
          "preview-unsupported",
          "No preview reader was bound; previews of drive-backed images are unavailable.",
        );
      }
      const parts = parseDriveImageUri(uri);
      if (parts === null) {
        return null;
      }
      return reader.readImagePreview({
        nodeId: parts.nodeId,
        maxBytes: maxBytes ?? previewMaxBytes,
        signal,
      });
    },
  };
}
