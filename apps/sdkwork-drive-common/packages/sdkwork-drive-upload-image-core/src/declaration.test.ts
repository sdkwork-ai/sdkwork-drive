import assert from "node:assert/strict";
import { test } from "node:test";
import { assertDriveUploadImageDeclaration } from "./declaration";
import { avatarDeclaration } from "./test-support";

test("accepts a conforming declaration", () => {
  assert.doesNotThrow(() => assertDriveUploadImageDeclaration(avatarDeclaration));
});

test("accepts a temporary declaration with a positive ttl", () => {
  assert.doesNotThrow(() =>
    assertDriveUploadImageDeclaration({
      ...avatarDeclaration,
      retention: "temporary",
      retentionTtlSeconds: 3600,
    }),
  );
});

test("rejects unknown fields", () => {
  assert.throws(
    () =>
      assertDriveUploadImageDeclaration({
        ...avatarDeclaration,
        tenantId: "tenant_01",
      }),
    /Unknown upload declaration field: tenantId/,
  );
});

test("rejects single-segment appResourceType", () => {
  assert.throws(
    () => assertDriveUploadImageDeclaration({ ...avatarDeclaration, appResourceType: "avatar" }),
    /dotted business type/,
  );
});

test("rejects uppercase or slashed source labels", () => {
  assert.throws(
    () => assertDriveUploadImageDeclaration({ ...avatarDeclaration, source: "@sdkwork/test-app" }),
    /kebab-case/,
  );
  assert.throws(
    () => assertDriveUploadImageDeclaration({ ...avatarDeclaration, source: "Test App" }),
    /kebab-case/,
  );
});

test("rejects the reserved Drive im scene", () => {
  assert.throws(
    () => assertDriveUploadImageDeclaration({ ...avatarDeclaration, scene: "im" }),
    /reserved/,
  );
});

test("rejects non-image-shaped profiles", () => {
  assert.throws(
    () =>
      assertDriveUploadImageDeclaration({
        ...avatarDeclaration,
        uploadProfileCode: "archive",
      }),
    /image-shaped profiles/,
  );
});

test("rejects temporary retention without ttl", () => {
  assert.throws(
    () => assertDriveUploadImageDeclaration({ ...avatarDeclaration, retention: "temporary" }),
    /retentionTtlSeconds/,
  );
});

test("rejects an empty purpose", () => {
  assert.throws(
    () => assertDriveUploadImageDeclaration({ ...avatarDeclaration, purpose: "   " }),
    /non-empty sentence/,
  );
});
