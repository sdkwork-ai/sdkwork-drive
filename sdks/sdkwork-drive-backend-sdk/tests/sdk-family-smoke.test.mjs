import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const testDir = path.dirname(fileURLToPath(import.meta.url));
const sdkRoot = path.resolve(testDir, "..");
const sdkName = "sdkwork-drive-backend-sdk";
const apiPrefix = "/backend/v3/api";
const languages = ["typescript", "rust", "java", "python", "go"];
const requiredOperations = [
  "labels.list",
  "labels.create",
  "labels.retrieve",
  "labels.update",
  "labels.delete",
  "quotas.retrieve",
  "quotas.update",
  "auditEvents.list",
  "maintenance.jobs.list",
  "maintenance.objectSweep",
  "maintenance.uploadSessionSweep",
  "maintenance.expiredUploadContentSweep",
  "maintenance.abandonedUploadTaskSweep",
  "spaces.admin.list",
  "downloadPackages.list",
];

const forbiddenOperations = [
  "storageProviders.list",
  "storageProviders.create",
  "storageProviderBindings.default.get",
];

test("sdkwork-drive-backend-sdk uses sdkwork-v3 profile", () => {
  const source = readFileSync(path.join(sdkRoot, "bin/generate-sdk.mjs"), "utf8");
  assert.match(source, /--standard-profile/);
  assert.match(source, /sdkwork-v3/);
});

test("sdkwork-drive-backend-sdk records family metadata outside generated output for every official language", () => {
  const manifest = JSON.parse(readFileSync(path.join(sdkRoot, "sdk-manifest.json"), "utf8"));
  assert.equal(manifest.sdkName, sdkName);
  assert.equal(manifest.apiPrefix, apiPrefix);
  assert.equal(manifest.standardProfile, "sdkwork-v3");
  assert.ok(
    manifest.ownerOnlyOperationCount >= 15,
    "family manifest should include backend operations admin surface only",
  );

  for (const language of languages) {
    const generatedOutput = `${sdkName}-${language}/generated/server-openapi`;
    assert.deepEqual(
      manifest.generatedPackages?.[language],
      {
        language,
        packageName: `${sdkName}-generated-${language}`,
        generatedOutput,
      },
      `${language} family manifest must record the generated package`,
    );
    assert.equal(
      existsSync(path.join(sdkRoot, generatedOutput, "sdk-manifest.json")),
      false,
      `${language} generated output must not carry SDK ownership manifest`,
    );

    const sourceOpenapi = readFileSync(
      path.join(
        sdkRoot,
        `${sdkName}-${language}`,
        "generated/server-openapi/source-openapi.json",
      ),
      "utf8",
    );
    for (const operationId of requiredOperations) {
      assert.match(
        sourceOpenapi,
        new RegExp(`"operationId": "${operationId.replace(".", "\\.")}"`),
        `${language} generated source OpenAPI should include ${operationId}`,
      );
    }
    for (const operationId of forbiddenOperations) {
      assert.doesNotMatch(
        sourceOpenapi,
        new RegExp(`"operationId": "${operationId.replace(".", "\\.")}"`),
        `${language} generated source OpenAPI must not include deprecated storage op ${operationId}`,
      );
    }
  }
});

test("sdkwork-drive-backend-sdk composed TypeScript operations include completed backend operations", () => {
  const source = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-backend-sdk-typescript",
      "composed/operations.ts",
    ),
    "utf8",
  );
  for (const operationId of requiredOperations) {
    assert.match(source, new RegExp(`"${operationId.replace(".", "\\.")}"`));
  }
  for (const operationId of forbiddenOperations) {
    assert.doesNotMatch(source, new RegExp(`"${operationId.replace(".", "\\.")}"`));
  }
});

test("sdkwork-drive-backend-sdk generated TypeScript does not carry ownership metadata", () => {
  const source = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-backend-sdk-typescript",
      "generated/server-openapi/src/index.ts",
    ),
    "utf8",
  );
  assert.doesNotMatch(source, /sdkMetadata/);
  assert.doesNotMatch(source, /sdkDependencies/);
});

test("sdkwork-drive-backend-sdk generated TypeScript exposes a real client surface", () => {
  const source = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-backend-sdk-typescript",
      "generated/server-openapi/src/index.ts",
    ),
    "utf8",
  );
  assert.match(source, /createClient/);
  assert.match(source, /export \* from ['"]\.\/api['"]/);
});

