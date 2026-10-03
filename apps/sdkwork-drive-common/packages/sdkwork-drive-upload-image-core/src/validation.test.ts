import assert from "node:assert/strict";
import { test } from "node:test";
import { validateDriveUploadImageFile } from "./validation";
import { byteFile } from "./test-support";
import { DRIVE_UPLOAD_IMAGE_DEFAULT_MAX_BYTES } from "./types";

test("admits a default png within the size ceiling", () => {
  assert.equal(validateDriveUploadImageFile(byteFile(1024)), null);
});

test("rejects non-image content types", () => {
  assert.equal(
    validateDriveUploadImageFile(byteFile(10, { type: "application/pdf" })),
    "invalid-file-type",
  );
});

test("rejects files outside the configured accept list", () => {
  assert.equal(
    validateDriveUploadImageFile(byteFile(10), { accept: ["image/svg+xml"] }),
    "invalid-file-type",
  );
  assert.equal(
    validateDriveUploadImageFile(byteFile(10, { type: "image/svg+xml" }), {
      accept: ["image/svg+xml"],
    }),
    null,
  );
});

test("matches extension-based accept entries for empty content types", () => {
  assert.equal(
    validateDriveUploadImageFile(byteFile(10, { type: undefined, name: "photo.PNG" })),
    null,
  );
  assert.equal(
    validateDriveUploadImageFile(byteFile(10, { type: undefined, name: "photo.txt" }), {
      accept: [".png", ".jpg"],
    }),
    "invalid-file-type",
  );
});

test("rejects empty and oversized files", () => {
  assert.equal(validateDriveUploadImageFile(byteFile(0)), "empty-file");
  assert.equal(
    validateDriveUploadImageFile(byteFile(DRIVE_UPLOAD_IMAGE_DEFAULT_MAX_BYTES + 1)),
    "file-too-large",
  );
  assert.equal(
    validateDriveUploadImageFile(byteFile(DRIVE_UPLOAD_IMAGE_DEFAULT_MAX_BYTES)),
    null,
  );
});

test("honors a configured size ceiling", () => {
  assert.equal(
    validateDriveUploadImageFile(byteFile(2048), { maxSizeBytes: 1024 }),
    "file-too-large",
  );
});
