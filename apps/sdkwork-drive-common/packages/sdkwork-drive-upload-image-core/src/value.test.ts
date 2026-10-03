import assert from "node:assert/strict";
import { test } from "node:test";
import { uploadResultFixture } from "./test-support";
import {
  buildDriveUploadImageValue,
  formatDriveImageUri,
  isDriveImageUri,
  mapDriveUploaderProgress,
  parseDriveImageUri,
  toDriveUploadImageFailure,
} from "./value";
import { DriveUploadImageError } from "./types";

test("builds the drive-backed value profile from an upload result", () => {
  const value = buildDriveUploadImageValue(uploadResultFixture(), {
    name: "cat.png",
    type: "image/png",
    size: 4,
  });
  assert.equal(value.source, "drive");
  assert.equal(value.uri, "drive://spaces/space_app_upload_01/nodes/node_01HR6P7ZJQ4A7M2CKA9F0P6R7S");
  assert.deepEqual(value.metadata?.drive, {
    spaceId: "space_app_upload_01",
    nodeId: "node_01HR6P7ZJQ4A7M2CKA9F0P6R7S",
    contentType: "image/png",
    contentLength: "4",
    originalFileName: "cat.png",
  });
});

test("keeps int64 content length as a string", () => {
  const value = buildDriveUploadImageValue(
    uploadResultFixture({ contentLength: "9007199254740993" }),
  );
  assert.equal(value.metadata?.drive?.contentLength, "9007199254740993");
});

test("throws a typed error when Drive returns no node identity", () => {
  const result = uploadResultFixture();
  result.uploadItem.spaceId = "";
  assert.throws(() => buildDriveUploadImageValue(result), DriveUploadImageError);
});

test("parses drive uris round-trip", () => {
  const uri = formatDriveImageUri({ spaceId: "s1", nodeId: "n1" });
  assert.equal(uri, "drive://spaces/s1/nodes/n1");
  assert.deepEqual(parseDriveImageUri(uri), { spaceId: "s1", nodeId: "n1" });
  assert.equal(isDriveImageUri(uri), true);
  assert.equal(isDriveImageUri("https://example.com/cat.png"), false);
  assert.equal(parseDriveImageUri("https://example.com/cat.png"), null);
});

test("wraps unknown failures into the typed upload failure", () => {
  const failure = toDriveUploadImageFailure(new Error("socket closed"));
  assert.equal(failure.code, "upload-failed");
  assert.equal(failure.message, "socket closed");
  const passthrough = toDriveUploadImageFailure(
    new DriveUploadImageError("missing-app-resource-id", "no anchor"),
  );
  assert.equal(passthrough.code, "missing-app-resource-id");
});

test("maps uploader progress onto percent", () => {
  const progress = mapDriveUploaderProgress({
    taskId: "t",
    uploadItemId: "i",
    uploadSessionId: "s",
    uploadedBytes: 50,
    totalBytes: 200,
    uploadedPartsCount: 1,
    totalParts: 4,
    status: "uploading",
  });
  assert.equal(progress.percent, 25);
  assert.equal(progress.status, "uploading");
});
