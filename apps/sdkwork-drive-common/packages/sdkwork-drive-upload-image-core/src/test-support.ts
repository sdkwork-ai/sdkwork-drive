import type {
  DriveUploaderRequest,
  DriveUploaderUploadResult,
} from "@sdkwork/drive-app-sdk/uploader";
import type {
  DriveUploadImageDeclaration,
  DriveUploadImageFileLike,
  DriveUploadImageUploaderLike,
} from "./types";

/**
 * Test fixtures shared by the core test files. They build byte-backed file
 * sources and a recording uploader double so every test runs without network
 * or DOM dependencies.
 */

export function byteFile(
  size: number,
  overrides: { type?: string | undefined; name?: string | undefined } = {},
): DriveUploadImageFileLike & { bytes: Uint8Array } {
  const bytes = new Uint8Array(Math.max(0, size));
  bytes.fill(7);
  const buffer = new ArrayBuffer(bytes.byteLength);
  new Uint8Array(buffer).set(bytes);
  return {
    size,
    type: overrides.type ?? "image/png",
    name: overrides.name ?? "cat.png",
    arrayBuffer: () => Promise.resolve(buffer),
    bytes,
  };
}

export function uploadResultFixture(
  overrides: Partial<{
    spaceId: string;
    nodeId: string;
    originalFileName: string;
    contentType: string;
    contentLength: string;
  }> = {},
): DriveUploaderUploadResult {
  const spaceId = overrides.spaceId ?? "space_app_upload_01";
  const nodeId = overrides.nodeId ?? "node_01HR6P7ZJQ4A7M2CKA9F0P6R7S";
  return {
    uploadItem: {
      id: nodeId,
      taskId: "task_01",
      actorType: "user",
      actorId: "user_01",
      appId: "sdkwork-test-app",
      appResourceType: "profile.avatar",
      appResourceId: "user_01",
      uploadProfileCode: "avatar",
      fileFingerprint: "fp_01",
      spaceId,
      nodeId,
      originalFileName: overrides.originalFileName ?? "cat.png",
      contentType: overrides.contentType ?? "image/png",
      contentTypeGroup: "image",
      contentLength: overrides.contentLength ?? "4",
      chunkSizeBytes: "8388608",
      totalParts: "1",
      uploadedPartsCount: "1",
      uploadedBytes: "4",
      status: "completed",
      retentionMode: "long_term",
      cleanupStatus: "none",
      postProcessStatus: "none",
    },
    uploadSession: {
      id: "session_01",
      spaceId,
      nodeId,
      bucket: "bucket",
      objectKey: "object/key",
      state: "completed",
      expiresAtEpochMs: "0",
      version: "1",
      storageProviderId: "provider_01",
      storageUploadId: "upload_01",
    },
    parts: [],
  };
}

export interface RecordedUpload {
  request: DriveUploaderRequest;
  profile: DriveUploadImageDeclaration["uploadProfileCode"];
}

export function recordingUploader(
  result: DriveUploaderUploadResult = uploadResultFixture(),
): DriveUploadImageUploaderLike & { recorded: RecordedUpload[] } {
  const recorded: RecordedUpload[] = [];
  const record =
    (profile: RecordedUpload["profile"]) =>
    (request: DriveUploaderRequest): Promise<DriveUploaderUploadResult> => {
      recorded.push({ profile, request });
      return Promise.resolve(result);
    };
  return {
    recorded,
    uploadImage: record("image"),
    uploadAvatar: record("avatar"),
    uploadThumbnail: record("thumbnail"),
  };
}

export const avatarDeclaration: DriveUploadImageDeclaration = {
  appResourceType: "profile.avatar",
  appResourceIdKind: "entity",
  scene: "avatar",
  source: "sdkwork-test-app",
  uploadProfileCode: "avatar",
  retention: "long_term",
  purpose: "Test avatar uploads for the reusable image component contract.",
};
