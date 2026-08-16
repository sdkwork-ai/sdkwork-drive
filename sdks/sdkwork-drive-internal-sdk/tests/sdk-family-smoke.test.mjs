import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const testDir = path.dirname(fileURLToPath(import.meta.url));
const sdkRoot = path.resolve(testDir, "..");
const sdkName = "sdkwork-drive-internal-sdk";
const languages = ["typescript", "rust", "java", "python", "go"];
const requiredOperations = [
  "rootScopeSubscriptions.create",
  "rootScopeSubscriptions.retrieve",
  "rootScopeEventDeliveries.replace",
  "websiteRoots.retrieve",
  "websiteRootEventDeliveries.replace",
  "driveResources.resolve",
  "driveResourceContent.retrieve",
];

function operationIds(openapi) {
  return Object.values(openapi.paths || {}).flatMap((pathItem) =>
    Object.entries(pathItem || {})
      .filter(([method]) => ["get", "post", "put", "patch", "delete"].includes(method))
      .map(([, operation]) => operation.operationId),
  );
}

function containsForbiddenPropertyName(value, forbiddenNames) {
  if (Array.isArray(value)) {
    return value.some((item) => containsForbiddenPropertyName(item, forbiddenNames));
  }
  if (!value || typeof value !== "object") {
    return false;
  }
  return Object.entries(value).some(
    ([key, child]) =>
      forbiddenNames.has(key) || containsForbiddenPropertyName(child, forbiddenNames),
  );
}

test("internal SDK family declares canonical authority and generated-only transport", () => {
  const manifest = JSON.parse(readFileSync(path.join(sdkRoot, "sdk-manifest.json"), "utf8"));
  assert.equal(manifest.sdkOwner, "sdkwork-drive");
  assert.equal(manifest.apiAuthority, "sdkwork-drive-internal-api");
  assert.equal(manifest.sdkSurface, "internal");
  assert.equal(manifest.sdkType, "custom");
  assert.equal(manifest.apiPrefix, "/internal/v3/api");
  assert.equal(manifest.standardProfile, "sdkwork-v3");
  assert.deepEqual(manifest.sdkDependencies, []);
  assert.equal(manifest.authoritySpec, "openapi/sdkwork-drive-internal-api.openapi.yaml");
  assert.equal(manifest.generationInputSpec, "openapi/sdkwork-drive-internal-api.sdkgen.yaml");
  assert(existsSync(path.join(sdkRoot, manifest.authoritySpec)));
  assert(existsSync(path.join(sdkRoot, manifest.generationInputSpec)));
});

test("every official generated language carries only the seven internal operations", () => {
  for (const language of languages) {
    const output = path.join(
      sdkRoot,
      `${sdkName}-${language}`,
      "generated/server-openapi",
    );
    const source = JSON.parse(readFileSync(path.join(output, "source-openapi.json"), "utf8"));
    assert.deepEqual(operationIds(source).sort(), [...requiredOperations].sort());
    assert.equal(existsSync(path.join(output, "sdk-manifest.json")), false);
    assert.equal(
      containsForbiddenPropertyName(
        source,
        new Set(["bucket", "objectKey", "credential", "presignedUrl"]),
      ),
      false,
    );
  }
});
