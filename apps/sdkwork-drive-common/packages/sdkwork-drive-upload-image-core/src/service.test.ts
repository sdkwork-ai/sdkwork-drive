import assert from "node:assert/strict";
import { test } from "node:test";
import { createDriveNodesImagePreviewReader, createDriveUploadImageService } from "./service";
import { avatarDeclaration, byteFile, recordingUploader, uploadResultFixture } from "./test-support";
import { DriveUploadImageError } from "./types";

test("service uploads through the declared profile with declared intent only", async () => {
  const uploader = recordingUploader();
  const service = createDriveUploadImageService({
    uploader,
    declaration: avatarDeclaration,
  });
  const file = byteFile(4);
  const value = await service.upload({
    file,
    appResourceId: "user_01",
    onProgress: () => undefined,
  });

  assert.equal(uploader.recorded.length, 1);
  const { profile, request } = uploader.recorded[0] ?? {};
  assert.equal(profile, "avatar");
  assert.equal(request?.appResourceType, "profile.avatar");
  assert.equal(request?.scene, "avatar");
  assert.equal(request?.source, "sdkwork-test-app");
  assert.equal(request?.appResourceId, "user_01");
  assert.equal(request?.uploadProfileCode, "avatar");
  assert.equal(request?.contentType, "image/png");
  assert.equal(request?.originalFileName, "cat.png");
  assert.equal("tenantId" in (request ?? {}), false);
  assert.equal(value.source, "drive");
  assert.equal(value.uri, "drive://spaces/space_app_upload_01/nodes/node_01HR6P7ZJQ4A7M2CKA9F0P6R7S");
});

test("service adapts arrayBuffer sources into ranged uploader blobs", async () => {
  const uploader = recordingUploader();
  const service = createDriveUploadImageService({
    uploader,
    declaration: avatarDeclaration,
  });
  const file = byteFile(4096);
  await service.upload({ file, appResourceId: "user_01" });
  const request = uploader.recorded[0]?.request;
  assert.ok(request);
  assert.equal(request.file.size, 4096);
  assert.equal(typeof request.file.readRange, "function");
  const ranged = await request.file.readRange?.(0, 4);
  assert.ok(ranged !== undefined);
  assert.equal(new Uint8Array(ranged)[0], 7);
});

test("service refuses uploads without an entity anchor", async () => {
  const service = createDriveUploadImageService({
    uploader: recordingUploader(),
    declaration: avatarDeclaration,
  });
  await assert.rejects(
    service.upload({ file: byteFile(4), appResourceId: "  " }),
    (error: unknown) => error instanceof DriveUploadImageError,
  );
});

test("service rejects a non-conforming declaration up front", () => {
  assert.throws(
    () =>
      createDriveUploadImageService({
        uploader: recordingUploader(),
        declaration: { ...avatarDeclaration, scene: "im" },
      }),
    DriveUploadImageError,
  );
});

test("resolvePreview passes external urls through and reads drive nodes bounded", async () => {
  const reads: Array<{ nodeId: string; maxBytes: number }> = [];
  const service = createDriveUploadImageService({
    uploader: recordingUploader(),
    declaration: avatarDeclaration,
    previewReader: {
      readImagePreview: async ({ nodeId, maxBytes }) => {
        reads.push({ nodeId, maxBytes });
        return `data:image/png;base64,${nodeId}`;
      },
    },
  });

  assert.equal(await service.resolvePreview({ uri: "https://cdn.example.com/a.png" }), "https://cdn.example.com/a.png");
  const preview = await service.resolvePreview({
    uri: "drive://spaces/s1/nodes/node_9",
    maxBytes: 1234,
  });
  assert.equal(preview, "data:image/png;base64,node_9");
  assert.deepEqual(reads, [{ nodeId: "node_9", maxBytes: 1234 }]);
});

test("preview reader caches and de-duplicates concurrent reads", async () => {
  let readCount = 0;
  const reader = createDriveNodesImagePreviewReader({
    content: {
      retrieve: async () => {
        readCount += 1;
        await new Promise((resolve) => setTimeout(resolve, 1));
        return { content: "AAAA", contentType: "image/webp" };
      },
    },
  });

  const [first, second] = await Promise.all([
    reader.readImagePreview({ nodeId: "n1", maxBytes: 10 }),
    reader.readImagePreview({ nodeId: "n1", maxBytes: 10 }),
  ]);
  assert.equal(first, "data:image/webp;base64,AAAA");
  assert.equal(second, first);
  assert.equal(readCount, 1);

  assert.equal(
    await reader.readImagePreview({ nodeId: "n1", maxBytes: 10 }),
    "data:image/webp;base64,AAAA",
  );
  assert.equal(readCount, 1);
});

test("preview reader degrades oversized content to null", async () => {
  const reader = createDriveNodesImagePreviewReader({
    content: {
      retrieve: async () => ({ content: "AAAA", contentType: "image/png", hasMore: true }),
    },
  });
  assert.equal(await reader.readImagePreview({ nodeId: "n1", maxBytes: 10 }), null);
});

test("resolvePreview without a reader fails typed for drive uris", async () => {
  const service = createDriveUploadImageService({
    uploader: recordingUploader(uploadResultFixture()),
    declaration: avatarDeclaration,
  });
  await assert.rejects(
    service.resolvePreview({ uri: "drive://spaces/s1/nodes/n1" }),
    (error: unknown) => error instanceof DriveUploadImageError,
  );
});
