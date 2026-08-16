import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const testDir = path.dirname(fileURLToPath(import.meta.url));
const sdkRoot = path.resolve(testDir, "..");
const sdkName = "sdkwork-drive-admin-storage-sdk";
const apiPrefix = "/backend/v3/api";
const languages = ["typescript", "rust", "java", "python", "go"];
const requiredOperations = [
  "storageProviderBindings.default.delete",
  "storageProviderBindings.default.retrieve",
  "storageProviderBindings.default.update",
  "storageProviderBindings.list",
  "storageProviders.activate",
  "storageProviders.bucket.delete",
  "storageProviders.bucket.retrieve",
  "storageProviders.bucket.update",
  "storageProviders.buckets.list",
  "storageProviders.capabilities.list",
  "storageProviders.create",
  "storageProviders.credentials.rotate",
  "storageProviders.deactivate",
  "storageProviders.delete",
  "storageProviders.list",
  "storageProviders.objects.copy",
  "storageProviders.objects.delete",
  "storageProviders.objects.list",
  "storageProviders.objects.retrieve",
  "storageProviders.retrieve",
  "storageProviders.test",
  "storageProviders.update",
];

test("sdkwork-drive-admin-storage-sdk uses sdkwork-drive-admin-storage-v3 profile", () => {
  const source = readFileSync(path.join(sdkRoot, "bin/generate-sdk.mjs"), "utf8");
  assert.match(source, /manifestStandardProfile:\s*"sdkwork-drive-admin-storage-v3"/);
});

test("sdkwork-drive-admin-storage-sdk records family metadata outside generated output for every official language", () => {
  const manifest = JSON.parse(readFileSync(path.join(sdkRoot, "sdk-manifest.json"), "utf8"));
  assert.equal(manifest.sdkName, sdkName);
  assert.equal(manifest.apiPrefix, apiPrefix);
  assert.equal(manifest.apiAuthority, "sdkwork-drive.admin.storage");
  assert.ok(
    manifest.ownerOnlyOperationCount >= 22,
    "family manifest should include the completed admin storage operation surface",
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
        new RegExp(`"operationId": "${operationId.replaceAll(".", "\\.")}"`),
        `${language} generated source OpenAPI should include ${operationId}`,
      );
    }
  }
});

test("sdkwork-drive-admin-storage-sdk composed TypeScript operations include completed admin storage operations", () => {
  const source = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-admin-storage-sdk-typescript",
      "composed/operations.ts",
    ),
    "utf8",
  );
  for (const operationId of requiredOperations) {
    assert.match(source, new RegExp(`"${operationId.replaceAll(".", "\\.")}"`));
  }
});

test("sdkwork-drive-admin-storage-sdk generated TypeScript does not carry ownership metadata", () => {
  const source = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-admin-storage-sdk-typescript",
      "generated/server-openapi/src/index.ts",
    ),
    "utf8",
  );
  assert.doesNotMatch(source, /sdkMetadata/);
  assert.doesNotMatch(source, /sdkDependencies/);
});

test("sdkwork-drive-admin-storage-sdk generated TypeScript exposes a real client surface", () => {
  const source = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-admin-storage-sdk-typescript",
      "generated/server-openapi/src/index.ts",
    ),
    "utf8",
  );
  assert.match(source, /createClient/);
  assert.match(source, /export \* from ['"]\.\/api['"]/);
});
