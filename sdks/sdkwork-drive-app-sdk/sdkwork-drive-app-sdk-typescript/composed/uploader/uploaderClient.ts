import { hexEncode, uuid as createUuid, Sha256Hasher } from "@sdkwork/utils";
import { DEFAULT_UPLOADER_CHUNK_SIZE_BYTES, inferUploaderContentType, inferUploaderFileName, planUploaderParts } from "./uploadPlanner";
import { createInMemoryUploaderStateStore } from "./uploadStateStore";
import type {
  DriveUploaderBlobLike,
  DriveUploaderClientOptions,
  DriveUploaderCompletedPart,
  DriveUploaderProgress,
  DriveUploaderReplaceNodeContentRequest,
  DriveUploaderReplaceNodeContentResult,
  DriveUploaderRequest,
  DriveUploaderStateSnapshot,
  DriveUploaderStateStore,
  DriveUploaderTransport,
  DriveUploaderTransportOptions,
  DriveUploaderUploadResult,
  DriveUploaderProfile,
} from "./types";

interface NormalizedUploadRequest extends DriveUploaderRequest {
  id: string;
  taskId: string;
  uploadProfileCode: DriveUploaderProfile;
  fileFingerprint: string;
  originalFileName: string;
  contentType: string;
  chunkSizeBytes: number;
  checksumSha256Hex: string;
}

interface NormalizedReplaceNodeContentRequest extends DriveUploaderReplaceNodeContentRequest {
  sessionId: string;
  idempotencyKey: string;
  originalFileName: string;
  contentType: string;
  chunkSizeBytes: number;
  checksumSha256Hex: string;
  expiresAtEpochMs: string;
}

function makeUploaderId(prefix: string): string {
  return `${prefix}-${createUuid()}`;
}

function defaultFileFingerprint(fileName: string, contentType: string, contentLength: number): string {
  const normalizedName = fileName
    .trim()
    .replace(/[^A-Za-z0-9._:@-]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 96);
  return `name:${normalizedName || "file"}:size:${contentLength}:type:${contentType.replace("/", ".")}`;
}

function defaultReplacementIdempotencyKey(nodeId: string, sessionId: string): string {
  return `drive-node-content-${nodeId}-${sessionId}`;
}

function defaultReplacementExpiryEpochMs(nowEpochMs?: string): string {
  const now = nowEpochMs ? Number(nowEpochMs) : Date.now();
  return String(now + 60 * 60 * 1000);
}

async function readUploadPartBody(
  file: DriveUploaderBlobLike,
  offsetBytes: number,
  sizeBytes: number,
  contentType: string,
): Promise<Blob> {
  if (file.readRange) {
    const bytes = await file.readRange(offsetBytes, sizeBytes);
    return readableBlob(new Blob([bytes], { type: contentType }));
  }
  return readableBlob(file.slice(offsetBytes, offsetBytes + sizeBytes, contentType));
}

const UPLOADER_CHECKSUM_CHUNK_BYTES = 4 * 1024 * 1024;

type BlobReadable = Blob & {
  arrayBuffer?: () => Promise<ArrayBuffer>;
  bytes?: () => Promise<Uint8Array>;
};

function arrayBufferFromView(view: Uint8Array): ArrayBuffer {
  const buffer = new ArrayBuffer(view.byteLength);
  new Uint8Array(buffer).set(view);
  return buffer;
}

async function readBlobByRuntimeFallback(blob: BlobReadable): Promise<ArrayBuffer> {
  if (typeof blob.bytes === "function") {
    return arrayBufferFromView(await blob.bytes());
  }

  if (typeof FileReader === "function") {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        if (reader.result && typeof reader.result !== "string") {
          resolve(reader.result);
          return;
        }
        reject(new Error("Drive uploader Blob reader did not return binary data."));
      };
      reader.onerror = () => reject(reader.error || new Error("Drive uploader Blob read failed."));
      reader.onabort = () => reject(new Error("Drive uploader Blob read was aborted."));
      reader.readAsArrayBuffer(blob);
    });
  }

  throw new Error("Drive uploader runtime cannot read Blob data.");
}

async function blobToArrayBuffer(blob: Blob): Promise<ArrayBuffer> {
  const readable = blob as BlobReadable;
  if (typeof readable.arrayBuffer === "function") {
    return readable.arrayBuffer();
  }
  return readBlobByRuntimeFallback(readable);
}

function readableBlob(blob: Blob): Blob {
  const readable = blob as BlobReadable;
  if (typeof readable.arrayBuffer === "function") {
    return blob;
  }

  Object.defineProperty(readable, "arrayBuffer", {
    configurable: true,
    enumerable: false,
    value: () => readBlobByRuntimeFallback(readable),
  });
  return blob;
}

async function sha256Checksum(file: DriveUploaderBlobLike): Promise<string> {
  const hasher = new Sha256Hasher();
  if (file.size === 0) {
    return `sha256:${hexEncode(hasher.digest())}`;
  }

  for (let offset = 0; offset < file.size; offset += UPLOADER_CHECKSUM_CHUNK_BYTES) {
    const length = Math.min(UPLOADER_CHECKSUM_CHUNK_BYTES, file.size - offset);
    if (file.readRange) {
      const bytes = await file.readRange(offset, length);
      hasher.update(new Uint8Array(bytes));
      continue;
    }
    const blob = file.slice(offset, offset + length, file.type);
    hasher.update(new Uint8Array(await blobToArrayBuffer(blob)));
  }

  return `sha256:${hexEncode(hasher.digest())}`;
}

function etagFromResponse(response: Response): string {
  const etag = response.headers.get("ETag") || response.headers.get("etag");
  if (!etag) {
    throw new Error("Drive uploader signed upload response did not return an ETag.");
  }
  return etag;
}

function emitProgress(
  request: NormalizedUploadRequest,
  snapshot: Pick<DriveUploaderStateSnapshot, "uploadItemId" | "uploadSessionId"> & {
    nodeId?: string;
  },
  progress: Omit<
    DriveUploaderProgress,
    "taskId" | "uploadItemId" | "uploadSessionId" | "totalBytes" | "nodeId"
  >,
): void {
  request.onProgress?.({
    taskId: request.taskId,
    uploadItemId: snapshot.uploadItemId,
    uploadSessionId: snapshot.uploadSessionId,
    nodeId: snapshot.nodeId,
    totalBytes: request.file.size,
    ...progress,
  });
}

function completedPartsByPartNo(parts: DriveUploaderCompletedPart[]): Map<number, DriveUploaderCompletedPart> {
  return new Map(parts.map((part) => [part.partNo, part]));
}

function sortedCompletedParts(parts: DriveUploaderCompletedPart[]): DriveUploaderCompletedPart[] {
  return [...completedPartsByPartNo(parts).values()].sort((left, right) => left.partNo - right.partNo);
}

async function uploadPresignedPart({
  uploadFetch,
  url,
  method,
  headers,
  body,
  signal,
}: {
  uploadFetch: typeof fetch;
  url: string;
  method: string;
  headers?: Record<string, string>;
  body: Blob;
  signal?: AbortSignal;
}): Promise<Response> {
  const response = await uploadFetch(url, {
    method,
    headers,
    body,
    signal,
  });
  if (!response.ok) {
    throw new Error(`Drive uploader signed upload failed with HTTP ${response.status}.`);
  }
  return response;
}

export class DriveUploaderClient {
  private readonly transport: DriveUploaderTransport;
  private readonly stateStore: DriveUploaderStateStore;
  private readonly uploadFetch: typeof fetch;
  private readonly defaultChunkSizeBytes: number;

  constructor({
    transport,
    stateStore = createInMemoryUploaderStateStore(),
    uploadFetch = fetch,
    defaultChunkSizeBytes = DEFAULT_UPLOADER_CHUNK_SIZE_BYTES,
  }: DriveUploaderClientOptions) {
    this.transport = transport;
    this.stateStore = stateStore;
    this.uploadFetch = uploadFetch;
    this.defaultChunkSizeBytes = defaultChunkSizeBytes;
  }

  async upload(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult> {
    return this.uploadByProfile("generic", request);
  }

  async uploadVideo(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult> {
    return this.uploadByProfile("video", request);
  }

  async uploadImage(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult> {
    return this.uploadByProfile("image", request);
  }

  async uploadAudio(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult> {
    return this.uploadByProfile("audio", request);
  }

  async uploadDocument(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult> {
    return this.uploadByProfile("document", request);
  }

  async uploadArchive(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult> {
    return this.uploadByProfile("archive", request);
  }

  async uploadText(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult> {
    return this.uploadByProfile("text", request);
  }

  async uploadDataset(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult> {
    return this.uploadByProfile("dataset", request);
  }

  async uploadAttachment(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult> {
    return this.uploadByProfile("attachment", request);
  }

  async uploadAvatar(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult> {
    return this.uploadByProfile("avatar", request);
  }

  async uploadThumbnail(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult> {
    return this.uploadByProfile("thumbnail", request);
  }

  async uploadByProfile(
    profile: DriveUploaderProfile,
    request: DriveUploaderRequest,
  ): Promise<DriveUploaderUploadResult> {
    const normalized = await this.normalizeRequest(profile, request);
    const parts = planUploaderParts(normalized.file, normalized.chunkSizeBytes);
    const prepared = await this.transport.drive.uploader.uploads.create({
      id: normalized.id,
      taskId: normalized.taskId,
      appResourceType: normalized.appResourceType,
      appResourceId: normalized.appResourceId,
      scene: normalized.scene,
      source: normalized.source,
      uploadProfileCode: normalized.uploadProfileCode,
      fileFingerprint: normalized.fileFingerprint,
      originalFileName: normalized.originalFileName,
      contentType: normalized.contentType,
      contentLength: String(normalized.file.size),
      chunkSizeBytes: String(normalized.chunkSizeBytes),
      spaceId: normalized.spaceId,
      parentNodeId: normalized.parentNodeId,
      shareToken: normalized.shareToken,
      retention: normalized.retention,
      nowEpochMs: normalized.nowEpochMs,
    }, { signal: normalized.signal });
    const uploadItem = prepared.uploadItem;
    const uploadSession = prepared.uploadSession;
    const uploadSessionId = uploadItem.uploadSessionId || uploadSession.id;
    const storageUploadId = uploadItem.storageUploadId || uploadSession.storageUploadId;
    const preparedNodeId = uploadItem.nodeId;
    const previousState = await this.stateStore.get(normalized.taskId);
    const carryUploadedParts =
      previousState?.uploadSessionId === uploadSessionId
        ? sortedCompletedParts(previousState.uploadedParts)
        : [];
    const baseState: DriveUploaderStateSnapshot = {
      taskId: normalized.taskId,
      uploadItemId: uploadItem.id,
      uploadSessionId,
      storageUploadId,
      uploadedParts: carryUploadedParts,
      updatedAtEpochMs: Date.now(),
    };
    const progressSnapshot = {
      uploadItemId: baseState.uploadItemId,
      uploadSessionId: baseState.uploadSessionId,
      nodeId: preparedNodeId,
    };

    emitProgress(normalized, progressSnapshot, {
      uploadedBytes: baseState.uploadedParts.reduce((sum, part) => sum + part.sizeBytes, 0),
      uploadedPartsCount: baseState.uploadedParts.length,
      totalParts: parts.length,
      status: "prepared",
    });

    const completedParts = completedPartsByPartNo(baseState.uploadedParts);
    try {
      for (const part of parts) {
        if (completedParts.has(part.partNo)) {
          continue;
        }

        emitProgress(normalized, progressSnapshot, {
          partNo: part.partNo,
          uploadedBytes: [...completedParts.values()].reduce((sum, item) => sum + item.sizeBytes, 0),
          uploadedPartsCount: completedParts.size,
          totalParts: parts.length,
          status: "uploading",
        });

        const presigned = await this.transport.drive.uploadSessions.parts.update(uploadSessionId, part.partNo, {
          uploadId: storageUploadId,
          requestedTtlSeconds: normalized.requestedPartTtlSeconds,
        }, { signal: normalized.signal });
        const body = await readUploadPartBody(
          normalized.file,
          part.offsetBytes,
          part.sizeBytes,
          normalized.contentType,
        );
        const response = await uploadPresignedPart({
          uploadFetch: normalized.uploadFetch || this.uploadFetch,
          url: presigned.uploadUrl,
          method: presigned.method || "PUT",
          headers: presigned.headers,
          body,
          signal: normalized.signal,
        });
        const completedPart: DriveUploaderCompletedPart = {
          partNo: part.partNo,
          etag: etagFromResponse(response),
          offsetBytes: part.offsetBytes,
          sizeBytes: part.sizeBytes,
        };

        await this.transport.drive.uploader.uploads.parts.update(uploadItem.id, part.partNo, {
          uploadSessionId,
          offsetBytes: String(part.offsetBytes),
          sizeBytes: String(part.sizeBytes),
          etag: completedPart.etag,
        }, { signal: normalized.signal });

        completedParts.set(part.partNo, completedPart);
        await this.stateStore.put({
          ...baseState,
          uploadedParts: sortedCompletedParts([...completedParts.values()]),
          updatedAtEpochMs: Date.now(),
        });

        emitProgress(normalized, progressSnapshot, {
          partNo: part.partNo,
          uploadedBytes: [...completedParts.values()].reduce((sum, item) => sum + item.sizeBytes, 0),
          uploadedPartsCount: completedParts.size,
          totalParts: parts.length,
          status: "part_uploaded",
        });
      }

      const finalParts = sortedCompletedParts([...completedParts.values()]);
      emitProgress(normalized, progressSnapshot, {
        uploadedBytes: finalParts.reduce((sum, part) => sum + part.sizeBytes, 0),
        uploadedPartsCount: finalParts.length,
        totalParts: parts.length,
        status: "completing",
      });

      const completedSession = await this.transport.drive.uploadSessions.complete(uploadSessionId, {
        uploadId: storageUploadId,
        contentType: normalized.contentType,
        contentLength: String(normalized.file.size),
        checksumSha256Hex: normalized.checksumSha256Hex,
        parts: finalParts.map((part) => ({
          partNo: part.partNo,
          etag: part.etag,
        })),
      }, { signal: normalized.signal });
      await this.stateStore.clear(normalized.taskId);
      emitProgress(normalized, progressSnapshot, {
        uploadedBytes: normalized.file.size,
        uploadedPartsCount: finalParts.length,
        totalParts: parts.length,
        status: "completed",
      });
      return {
        uploadItem,
        uploadSession: completedSession,
        parts: finalParts,
      };
    } catch (error) {
      await this.stateStore.put({
        ...baseState,
        uploadedParts: sortedCompletedParts([...completedParts.values()]),
        updatedAtEpochMs: Date.now(),
      });
      // Industry behavior: keep sessions alive for retry/resume unless the caller explicitly aborted.
      // Aborting on transient network/storage errors prevents true resumable uploads.
      if (error instanceof DOMException && error.name === 'AbortError') {
        await this.abortUploadSession(
          {
            signal: normalized.signal,
          },
          uploadSessionId,
        );
      }
      throw error;
    }
  }

  async replaceNodeContent(
    request: DriveUploaderReplaceNodeContentRequest,
  ): Promise<DriveUploaderReplaceNodeContentResult> {
    const normalized = await this.normalizeReplaceNodeContentRequest(request);
    const uploadSession = await this.transport.drive.uploadSessions.create({
      sessionId: normalized.sessionId,
      spaceId: normalized.spaceId,
      nodeId: normalized.nodeId,
      idempotencyKey: normalized.idempotencyKey,
      expiresAtEpochMs: normalized.expiresAtEpochMs,
    }, { signal: normalized.signal });
    const uploadSessionId = uploadSession.id || normalized.sessionId;
    const storageUploadId = uploadSession.storageUploadId;
    const parts = planUploaderParts(normalized.file, normalized.chunkSizeBytes);

    try {
      const completedParts: DriveUploaderCompletedPart[] = [];
      for (const part of parts) {
        const presigned = await this.transport.drive.uploadSessions.parts.update(
          uploadSessionId,
          part.partNo,
          {
            uploadId: storageUploadId,
            requestedTtlSeconds: normalized.requestedPartTtlSeconds,
          },
          { signal: normalized.signal },
        );
        const response = await uploadPresignedPart({
          uploadFetch: normalized.uploadFetch || this.uploadFetch,
          url: presigned.uploadUrl,
          method: presigned.method || "PUT",
          headers: presigned.headers,
          body: await readUploadPartBody(
            normalized.file,
            part.offsetBytes,
            part.sizeBytes,
            normalized.contentType,
          ),
          signal: normalized.signal,
        });
        completedParts.push({
          partNo: presigned.partNo || part.partNo,
          etag: etagFromResponse(response),
          offsetBytes: part.offsetBytes,
          sizeBytes: part.sizeBytes,
        });
      }

      const completedSession = await this.transport.drive.uploadSessions.complete(uploadSessionId, {
        uploadId: storageUploadId,
        contentType: normalized.contentType,
        contentLength: String(normalized.file.size),
        checksumSha256Hex: normalized.checksumSha256Hex,
        parts: completedParts.map((part) => ({
          partNo: part.partNo,
          etag: part.etag,
        })),
      }, { signal: normalized.signal });

      return {
        uploadSession: completedSession,
        parts: completedParts,
      };
    } catch (error) {
      await this.abortUploadSession(
        {
          signal: normalized.signal,
        },
        uploadSessionId,
      );
      throw error;
    }
  }

  private async normalizeRequest(
    profile: DriveUploaderProfile,
    request: DriveUploaderRequest,
  ): Promise<NormalizedUploadRequest> {
    const originalFileName = request.originalFileName || inferUploaderFileName(request.file);
    const contentType = request.contentType || inferUploaderContentType(request.file);
    const uploadProfileCode = request.uploadProfileCode || profile;
    const chunkSizeBytes = request.chunkSizeBytes || this.defaultChunkSizeBytes;
    const fileFingerprint =
      request.fileFingerprint ||
      defaultFileFingerprint(originalFileName, contentType, request.file.size);
    const taskId = request.taskId || `uploader-${fileFingerprint}`;
    return {
      ...request,
      id: request.id || makeUploaderId("upload-item"),
      taskId,
      uploadProfileCode,
      fileFingerprint,
      originalFileName,
      contentType,
      chunkSizeBytes,
      checksumSha256Hex: request.checksumSha256Hex || await sha256Checksum(request.file),
    };
  }

  private async normalizeReplaceNodeContentRequest(
    request: DriveUploaderReplaceNodeContentRequest,
  ): Promise<NormalizedReplaceNodeContentRequest> {
    const sessionId = request.sessionId || makeUploaderId("upload");
    return {
      ...request,
      sessionId,
      idempotencyKey: request.idempotencyKey || defaultReplacementIdempotencyKey(request.nodeId, sessionId),
      originalFileName: request.originalFileName || inferUploaderFileName(request.file),
      contentType: request.contentType || inferUploaderContentType(request.file),
      chunkSizeBytes: request.chunkSizeBytes || this.defaultChunkSizeBytes,
      checksumSha256Hex: request.checksumSha256Hex || await sha256Checksum(request.file),
      expiresAtEpochMs: request.expiresAtEpochMs || defaultReplacementExpiryEpochMs(request.nowEpochMs),
    };
  }

  private async abortUploadSession(
    request: {
      signal?: AbortSignal;
    },
    uploadSessionId: string,
  ): Promise<void> {
    if (!this.transport.drive.uploadSessions.abort) {
      return;
    }

    try {
      await this.transport.drive.uploadSessions.abort(uploadSessionId, {
        }, { signal: request.signal });
    } catch {
      // Preserve the original upload failure; Drive will expire abandoned sessions.
    }
  }
}

export function createDriveUploaderClient(options: DriveUploaderClientOptions): DriveUploaderClient {
  return new DriveUploaderClient(options);
}


