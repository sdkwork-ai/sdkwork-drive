import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const testDir = path.dirname(fileURLToPath(import.meta.url));
const sdkRoot = path.resolve(testDir, "..");
const sdkName = "sdkwork-drive-sdk";
const apiPrefix = "/open/v3/api";
const languages = ["typescript", "rust", "java", "python", "go"];

test("sdkwork-drive-sdk uses standard generator custom profile for open API", () => {
  const source = readFileSync(path.join(sdkRoot, "bin/generate-sdk.mjs"), "utf8");
  assert.match(source, /runDriveSdkGenerator/);
  assert.match(source, /sdkType: "custom"/);
  assert.match(source, /\/open\/v3\/api/);
});

test("sdkwork-drive-sdk records family metadata outside generated output for every official language", () => {
  const manifest = JSON.parse(readFileSync(path.join(sdkRoot, "sdk-manifest.json"), "utf8"));
  assert.equal(manifest.sdkName, sdkName);
  assert.equal(manifest.apiPrefix, apiPrefix);
  assert.equal(manifest.sdkType, "custom");
  assert.equal(manifest.standardProfile, "sdkwork-drive-open-v3");

  for (const language of languages) {
    const generatedOutput = `${sdkName}-${language}/generated/server-openapi`;
    assert.deepEqual(manifest.generatedPackages?.[language], {
      language,
      packageName: `${sdkName}-generated-${language}`,
      generatedOutput,
    });
    assert.equal(
      existsSync(path.join(sdkRoot, generatedOutput, "sdk-manifest.json")),
      false,
      `${language} generated output must not carry SDK ownership manifest`,
    );
  }
});

test("sdkwork-drive-sdk generated TypeScript exposes a real client surface", () => {
  const source = readFileSync(
    path.join(
      sdkRoot,
      "sdkwork-drive-sdk-typescript",
      "generated/server-openapi/src/index.ts",
    ),
    "utf8",
  );
  assert.match(source, /createClient/);
  assert.match(source, /export \* from ['"]\.\/api['"]/);
});
