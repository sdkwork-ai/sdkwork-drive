import type {
  CompleteUploadSessionRequest,
  CreateUploadSessionRequest,
  DriveUploadSession,
  MarkUploaderPartUploadedRequest,
  NodeCommandRequest,
  PrepareUploaderUploadRequest,
  PrepareUploaderUploadResponse,
  PresignedUploadPart,
  PresignUploadPartRequest,
  UploadSessionMutationResponse,
  UploaderUploadItem,
  UploaderUploadPart,
} from "../../generated/server-openapi/src/types";

export type DriveUploaderProfile =
  | "generic"
  | "video"
  | "image"
  | "audio"
  | "document"
  | "archive"
  | "text"
  | "dataset"
  | "attachment"
  | "avatar"
  | "thumbnail";

export interface DriveUploaderBlobLike {
  readonly size: number;
  readonly type?: string;
  readonly name?: string;
  arrayBuffer?(): Promise<ArrayBuffer>;
  slice(start?: number, end?: number, contentType?: string): Blob;
  readRange?(offsetBytes: number, lengthBytes: number): Promise<ArrayBuffer>;
}

export interface DriveUploaderRequest
  extends Omit<
    PrepareUploaderUploadRequest,
    | "id"
    | "taskId"
    | "uploadProfileCode"
    | "fileFingerprint"
    | "originalFileName"
    | "contentType"
    | "contentLength"
    | "chunkSizeBytes"
    | "nowEpochMs"
  > {
  file: DriveUploaderBlobLike;
  id?: string;
  taskId?: string;
  uploadProfileCode?: DriveUploaderProfile;
  fileFingerprint?: string;
  originalFileName?: string;
  contentType?: string;
  chunkSizeBytes?: number;
  checksumSha256Hex?: string;
  uploadFetch?: typeof fetch;
  requestedPartTtlSeconds?: number;
  nowEpochMs?: string;
  signal?: AbortSignal;
  onProgress?: (progress: DriveUploaderProgress) => void;
}

export interface DriveUploaderReplaceNodeContentRequest {
  file: DriveUploaderBlobLike;
  appResourceType: string;
  appResourceId: string;
  scene?: string;
  source?: string;
  uploadProfileCode?: DriveUploaderProfile;
  fileFingerprint?: string;
  originalFileName?: string;
  contentType?: string;
  spaceId: string;
  nodeId: string;
  sessionId?: string;
  idempotencyKey?: string;
  expiresAtEpochMs?: string;
  chunkSizeBytes?: number;
  checksumSha256Hex?: string;
  uploadFetch?: typeof fetch;
  requestedPartTtlSeconds?: number;
  nowEpochMs?: string;
  signal?: AbortSignal;
}

export interface DriveUploaderPartPlan {
  partNo: number;
  offsetBytes: number;
  sizeBytes: number;
}

export interface DriveUploaderProgress {
  taskId: string;
  uploadItemId: string;
  uploadSessionId: string;
  nodeId?: string;
  partNo?: number;
  uploadedBytes: number;
  totalBytes: number;
  uploadedPartsCount: number;
  totalParts: number;
  status: "prepared" | "uploading" | "part_uploaded" | "completing" | "completed";
}

export interface DriveUploaderCompletedPart {
  partNo: number;
  etag: string;
  offsetBytes: number;
  sizeBytes: number;
  checksumSha256Hex?: string;
}

export interface DriveUploaderStateSnapshot {
  taskId: string;
  uploadItemId: string;
  uploadSessionId: string;
  storageUploadId?: string;
  uploadedParts: DriveUploaderCompletedPart[];
  updatedAtEpochMs: number;
}

export interface DriveUploaderStateStore {
  get(taskId: string): Promise<DriveUploaderStateSnapshot | undefined>;
  put(snapshot: DriveUploaderStateSnapshot): Promise<void>;
  clear(taskId: string): Promise<void>;
}

export interface DriveUploaderTransportOptions {
  signal?: AbortSignal;
}

export interface DriveUploaderTransport {
  drive: {
    uploader: {
      uploads: {
        create(
          body: PrepareUploaderUploadRequest,
          options?: DriveUploaderTransportOptions,
        ): Promise<PrepareUploaderUploadResponse>;
        parts: {
          update(
            uploadItemId: string,
            partNo: number,
            body: MarkUploaderPartUploadedRequest,
            options?: DriveUploaderTransportOptions,
          ): Promise<UploaderUploadPart>;
        };
      };
    };
    uploadSessions: {
      create(
        body: CreateUploadSessionRequest,
        options?: DriveUploaderTransportOptions,
      ): Promise<DriveUploadSession>;
      parts: {
        update(
          uploadSessionId: string,
          partNo: number,
          body: PresignUploadPartRequest,
          options?: DriveUploaderTransportOptions,
        ): Promise<PresignedUploadPart>;
      };
      complete(
        uploadSessionId: string,
        body: CompleteUploadSessionRequest,
        options?: DriveUploaderTransportOptions,
      ): Promise<UploadSessionMutationResponse>;
      abort?(
        uploadSessionId: string,
        body: NodeCommandRequest,
        options?: DriveUploaderTransportOptions,
      ): Promise<UploadSessionMutationResponse>;
    };
  };
}

export interface DriveUploaderClientOptions {
  transport: DriveUploaderTransport;
  stateStore?: DriveUploaderStateStore;
  uploadFetch?: typeof fetch;
  defaultChunkSizeBytes?: number;
}

export interface DriveUploaderUploadResult {
  uploadItem: UploaderUploadItem;
  uploadSession: UploadSessionMutationResponse;
  parts: DriveUploaderCompletedPart[];
}

export interface DriveUploaderReplaceNodeContentResult {
  uploadSession: UploadSessionMutationResponse;
  parts: DriveUploaderCompletedPart[];
}


