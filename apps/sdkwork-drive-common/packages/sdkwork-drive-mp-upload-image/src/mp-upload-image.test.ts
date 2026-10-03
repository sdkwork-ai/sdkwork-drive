import assert from "node:assert/strict";
import { test } from "node:test";
import {
  createMpDriveUploadImage,
  mapSnapshotForTemplate,
} from "./mp-upload-image";
import { createWxDriveImagePicker } from "./wx-picker";
import type { WxMiniProgramLike } from "./wx-types";
import type { DriveUploadImageService, DriveUploadImageValue } from "@sdkwork/drive-upload-image-core";

function fakeWx(options: {
  tempFiles: Array<{ tempFilePath: string; size: number }>;
  bytes?: Uint8Array;
}): WxMiniProgramLike & { chosenCounts: number[] } {
  const bytes = options.bytes ?? new Uint8Array(16).fill(9);
  return {
    chosenCounts: [],
    chooseMedia({ count, success }) {
      this.chosenCounts.push(count);
      success({ tempFiles: options.tempFiles });
    },
    getFileSystemManager() {
      return {
        readFile({ position = 0, length, success }) {
          const view = bytes.slice(position, position + (length ?? bytes.length - position));
          const buffer = new ArrayBuffer(view.byteLength);
          new Uint8Array(buffer).set(view);
          success({ data: buffer });
        },
      };
    },
  };
}

const service: DriveUploadImageService = {
  upload: (input) => {
    return Promise.resolve({
      uri: `drive://spaces/space_1/nodes/node_${input.appResourceId}`,
      source: "drive",
      metadata: {
        drive: {
          spaceId: "space_1",
          nodeId: `node_${input.appResourceId}`,
          originalFileName: input.file.name ?? "image",
        },
      },
    });
  },
  resolvePreview: () => Promise.resolve(null),
};

test("wx picker maps chooseMedia results into ranged byte sources", async () => {
  const wx = fakeWx({
    tempFiles: [
      { tempFilePath: "wxfile://tmp/img_1.png", size: 16 },
      { tempFilePath: "wxfile://tmp/img_2.jpg?sign=abc", size: 16 },
    ],
  });
  const picker = createWxDriveImagePicker(wx);
  const files = await picker.pick({ count: 2, sourceType: ["album"] });

  assert.equal(files.length, 2);
  assert.equal(files[0]?.name, "img_1.png");
  assert.equal(files[1]?.name, "img_2.jpg");
  assert.equal(files[0]?.path, "wxfile://tmp/img_1.png");
  const window = (await files[0]?.readRange?.(4, 4)) ?? null;
  assert.ok(window !== null);
  assert.equal(new Uint8Array(window)[0], 9);
});

test("picker treats a user cancel as an empty selection", async () => {
  const wx: WxMiniProgramLike = {
    chooseMedia({ fail }) {
      fail({ errMsg: "chooseMedia:fail cancel" });
    },
    getFileSystemManager() {
      throw new Error("not reached");
    },
  };
  const picker = createWxDriveImagePicker(wx);
  const files = await picker.pick({ count: 1, sourceType: ["album"] });
  assert.deepEqual(files, []);
});

test("chooseAndUpload picks, uploads, and exposes the drive value", async () => {
  const wx = fakeWx({ tempFiles: [{ tempFilePath: "wxfile://tmp/avatar.png", size: 16 }] });
  const binding = createMpDriveUploadImage({
    service,
    wx,
    maxSizeBytes: 1024,
  });

  const values = await binding.chooseAndUpload({ appResourceId: "user_9" });
  assert.equal(values.length, 1);
  assert.equal(values[0]?.uri, "drive://spaces/space_1/nodes/node_user_9");
  assert.equal(binding.getSnapshot().items[0]?.status, "uploaded");
  binding.destroy();
});

test("without an anchor the pick defers until uploadPending (persist-first)", async () => {
  const wx = fakeWx({ tempFiles: [{ tempFilePath: "wxfile://tmp/avatar.png", size: 16 }] });
  const binding = createMpDriveUploadImage({ service, wx });

  const afterPick = await binding.chooseAndUpload({});
  assert.equal(afterPick.length, 0);
  assert.equal(binding.getSnapshot().hasPending, true);

  const values = await binding.uploadPending({ appResourceId: "user_7" });
  assert.equal(values.length, 1);
  binding.destroy();
});

test("mapSnapshotForTemplate produces the template shape", () => {
  const wx = fakeWx({ tempFiles: [] });
  const binding = createMpDriveUploadImage({ service, wx, maxFiles: 3 });
  binding.setValues([
    { uri: "drive://spaces/space_1/nodes/node_a", source: "drive" },
  ]);

  const template = mapSnapshotForTemplate(binding.getSnapshot(), { maxFiles: 3 });
  assert.equal(template.items.length, 1);
  assert.equal(template.items[0]?.status, "uploaded");
  assert.equal(template.canAddMore, true);
  assert.equal(template.isUploading, false);

  const capped = mapSnapshotForTemplate(binding.getSnapshot(), { maxFiles: 1 });
  assert.equal(capped.canAddMore, false);
  binding.destroy();
});
