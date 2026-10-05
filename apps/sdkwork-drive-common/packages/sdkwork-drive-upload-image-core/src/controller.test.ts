import assert from "node:assert/strict";
import { test } from "node:test";
import { DriveUploadImageController } from "./controller";
import { createDriveUploadImageService } from "./service";
import {
  avatarDeclaration,
  byteFile,
  recordingUploader,
} from "./test-support";
import type {
  DriveUploadImageProgress,
  DriveUploadImageService,
  DriveUploadImageValue,
} from "./types";

const nullPreviews = {
  createObjectUrl: () => null,
  revokeObjectUrl: () => undefined,
};

function scriptedService(
  handler: (input: {
    file: { name?: string | undefined; size: number };
    appResourceId: string;
    onProgress?: (progress: DriveUploadImageProgress) => void;
  }) => Promise<DriveUploadImageValue>,
): DriveUploadImageService {
  return {
    upload: (input) =>
      handler({
        file: { name: input.file.name, size: input.file.size },
        appResourceId: input.appResourceId,
        ...(input.onProgress === undefined
          ? {}
          : { onProgress: input.onProgress }),
      }),
    resolvePreview: () => Promise.resolve(null),
  };
}

function driveValue(nodeId: string): DriveUploadImageValue {
  return {
    uri: `drive://spaces/space_1/nodes/${nodeId}`,
    source: "drive",
    metadata: { drive: { spaceId: "space_1", nodeId, originalFileName: `${nodeId}.png` } },
  };
}

test("rejects invalid files and keeps the reason on the item", async () => {
  const rejections: string[] = [];
  const controller = new DriveUploadImageController({
    service: scriptedService(() => Promise.resolve(driveValue("n"))),
    previewUrls: nullPreviews,
    maxSizeBytes: 1024,
    onRejected: ({ code }) => rejections.push(code),
  });

  await controller.addFiles([byteFile(2048)]);
  const snapshot = controller.getSnapshot();
  assert.equal(snapshot.items.length, 1);
  assert.equal(snapshot.items[0]?.status, "rejected");
  assert.equal(snapshot.items[0]?.rejectionCode, "file-too-large");
  assert.deepEqual(rejections, ["file-too-large"]);
  assert.equal(controller.getValues().length, 0);
  controller.destroy();
});

test("defers the upload when no entity anchor is available (persist-first)", async () => {
  const uploads: string[] = [];
  const controller = new DriveUploadImageController({
    service: scriptedService((input) => {
      uploads.push(input.appResourceId);
      return Promise.resolve(driveValue(`node_${uploads.length}`));
    }),
    previewUrls: nullPreviews,
  });

  await controller.addFiles([byteFile(8)]);
  assert.equal(controller.getSnapshot().hasPending, true);
  assert.equal(uploads.length, 0);

  const values = await controller.uploadPending({ appResourceId: "user_01" });
  assert.deepEqual(uploads, ["user_01"]);
  assert.equal(values.length, 1);
  assert.equal(controller.getSnapshot().items[0]?.status, "uploaded");
  controller.destroy();
});

test("uploads automatically once the anchor resolver supplies an id", async () => {
  const uploads: string[] = [];
  const uploadedSets: string[][] = [];
  const controller = new DriveUploadImageController({
    service: scriptedService((input) => {
      uploads.push(input.appResourceId);
      input.onProgress?.({
        uploadedBytes: 4,
        totalBytes: 8,
        uploadedPartsCount: 1,
        totalParts: 1,
        status: "completed",
        percent: 100,
      });
      return Promise.resolve(driveValue(`node_${uploads.length}`));
    }),
    previewUrls: nullPreviews,
    resolveAppResourceId: () => "user_42",
    onUploaded: (values) => uploadedSets.push(values.map((value) => value.uri)),
  });

  await controller.addFiles([byteFile(8)]);
  assert.deepEqual(uploads, ["user_42"]);
  assert.equal(controller.getSnapshot().isUploading, false);
  assert.equal(controller.getValues().length, 1);
  assert.equal(uploadedSets.length, 1);
  controller.destroy();
});

test("single-field mode replaces the stored image on a new pick", async () => {
  let counter = 0;
  const controller = new DriveUploadImageController({
    service: scriptedService(() => {
      counter += 1;
      return Promise.resolve(driveValue(`node_${counter}`));
    }),
    previewUrls: nullPreviews,
    resolveAppResourceId: () => "user_01",
  });

  await controller.addFiles([byteFile(8, { name: "first.png" })]);
  assert.deepEqual(controller.getValues().map((value) => value.uri), [
    "drive://spaces/space_1/nodes/node_1",
  ]);
  await controller.addFiles([byteFile(8, { name: "second.png" })]);
  assert.deepEqual(controller.getValues().map((value) => value.uri), [
    "drive://spaces/space_1/nodes/node_2",
  ]);
  assert.equal(controller.getSnapshot().items.length, 1);
  controller.destroy();
});

test("multi mode enforces maxFiles with a too-many-files rejection", async () => {
  let counter = 0;
  const rejections: string[] = [];
  const controller = new DriveUploadImageController({
    service: scriptedService(() => {
      counter += 1;
      return Promise.resolve(driveValue(`node_${counter}`));
    }),
    previewUrls: nullPreviews,
    maxFiles: 2,
    replaceOnMax: false,
    resolveAppResourceId: () => "user_01",
    onRejected: ({ code }) => rejections.push(code),
  });

  await controller.addFiles([byteFile(8), byteFile(8), byteFile(8)]);
  assert.equal(controller.getValues().length, 2);
  assert.deepEqual(rejections, ["too-many-files"]);
  controller.destroy();
});

test("upload failures surface typed state and retry recovers", async () => {
  let attempts = 0;
  const failures: string[] = [];
  const controller = new DriveUploadImageController({
    service: scriptedService(() => {
      attempts += 1;
      if (attempts === 1) {
        return Promise.reject(new Error("provider unreachable"));
      }
      return Promise.resolve(driveValue("node_ok"));
    }),
    previewUrls: nullPreviews,
    onFailed: ({ error }) => failures.push(error.code),
  });

  await controller.addFiles([byteFile(8)]);
  assert.equal(controller.getSnapshot().hasPending, true);
  await assert.rejects(
    controller.uploadPending({ appResourceId: "user_01" }),
    (error: unknown) => error instanceof Error && error.message === "provider unreachable",
  );
  const failed = controller.getSnapshot().items[0];
  assert.equal(failed?.status, "error");
  assert.equal(failed?.errorMessage, "provider unreachable");
  assert.deepEqual(failures, ["upload-failed"]);

  await controller.retryItem(failed?.id ?? "");
  assert.equal(controller.getValues().length, 1);
  assert.equal(controller.getSnapshot().items[0]?.status, "uploaded");
  controller.destroy();
});

test("setValues round-trips parent onChange without losing transient previews", async () => {
  const uploader = recordingUploader();
  const service = createDriveUploadImageService({ uploader, declaration: avatarDeclaration });
  const controller = new DriveUploadImageController({
    service,
    previewUrls: nullPreviews,
    resolveAppResourceId: () => "user_01",
  });

  const emitted: string[] = [];
  controller.subscribe(() => {
    for (const value of controller.getValues()) {
      if (!emitted.includes(value.uri)) {
        emitted.push(value.uri);
      }
    }
  });

  await controller.addFiles([byteFile(4)]);
  assert.equal(uploader.recorded.length, 1);
  const value = controller.getValues()[0];
  assert.ok(value !== undefined);

  // Parent stores the value and feeds it back through the controlled prop.
  controller.setValues([value]);
  assert.equal(controller.getSnapshot().items.length, 1);
  assert.equal(controller.getSnapshot().items[0]?.status, "uploaded");
  assert.deepEqual(emitted, [value.uri]);

  // A different external value replaces the displayed set.
  controller.setValues([driveValue("node_external")]);
  assert.deepEqual(controller.getValues().map((candidate) => candidate.uri), [
    "drive://spaces/space_1/nodes/node_external",
  ]);
  controller.destroy();
});

test("resolvePreview fills previews for externally-loaded drive values", async () => {
  const controller = new DriveUploadImageController({
    service: {
      upload: () => Promise.resolve(driveValue("unused")),
      resolvePreview: (input) =>
        Promise.resolve(input.uri.includes("node_loaded") ? "data:image/png;base64,AAA" : null),
    },
    previewUrls: nullPreviews,
  });

  controller.setValues([driveValue("node_loaded")]);
  assert.equal(controller.getSnapshot().items[0]?.previewUrl, null);
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(controller.getSnapshot().items[0]?.previewUrl, "data:image/png;base64,AAA");
  controller.destroy();
});

test("removing an uploaded item shrinks values and notifies", () => {
  const controller = new DriveUploadImageController({
    service: scriptedService(() => Promise.resolve(driveValue("node_keep"))),
    previewUrls: nullPreviews,
  });
  controller.setValues([driveValue("node_a"), driveValue("node_b")]);
  const firstItem = controller.getSnapshot().items[0];
  assert.ok(firstItem !== undefined);
  controller.removeItem(firstItem.id);
  assert.deepEqual(controller.getValues().map((value) => value.uri), [
    "drive://spaces/space_1/nodes/node_b",
  ]);
  controller.destroy();
});

test("destroy aborts in-flight work and rejects later use", async () => {
  let aborted = false;
  const controller = new DriveUploadImageController({
    service: {
      upload: (input) =>
        new Promise((_resolve, reject) => {
          input.signal?.addEventListener("abort", () => {
            aborted = true;
            reject(new Error("aborted"));
          });
        }),
      resolvePreview: () => Promise.resolve(null),
    },
    previewUrls: nullPreviews,
    resolveAppResourceId: () => "user_01",
  });

  const adding = controller.addFiles([byteFile(8)]);
  await new Promise((resolve) => setTimeout(resolve, 0));
  controller.destroy();
  await adding;
  assert.equal(aborted, true);
  await assert.rejects(async () => controller.uploadPending(), /destroyed/);
  assert.equal(controller.getSnapshot().items.length, 0);
});

test("clearing a destroyed controller is a no-op (StrictMode remount cleanup)", async () => {
  const controller = new DriveUploadImageController({
    service: scriptedService(() => Promise.resolve(driveValue("node_clear"))),
    previewUrls: nullPreviews,
  });

  controller.destroy();
  assert.doesNotThrow(() => controller.clear());
  assert.equal(controller.getSnapshot().items.length, 0);
});
