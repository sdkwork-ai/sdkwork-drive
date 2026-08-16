import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { analyzeDriveAppSdkConsumerIntegration } from "./check_drive_app_sdk_consumer_integration.mjs";

function writeJson(filePath, value) {
  writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function createWorkspace() {
  const workspaceRoot = mkdtempSync(path.join(os.tmpdir(), "drive-sdk-consumers-"));
  const driveRoot = path.join(workspaceRoot, "sdkwork-drive");
  mkdirSync(path.join(workspaceRoot, "sdkwork-specs"), { recursive: true });
  mkdirSync(path.join(driveRoot, "sdks", "sdkwork-drive-app-sdk"), { recursive: true });
  writeFileSync(path.join(workspaceRoot, "sdkwork-specs", "DRIVE_SPEC.md"), "# Drive\n");
  return { workspaceRoot, driveRoot };
}

function createApp(workspaceRoot, relativeRoot, { packageJson = {}, source = "" } = {}) {
  const appRoot = path.join(workspaceRoot, relativeRoot);
  mkdirSync(path.join(appRoot, "src"), { recursive: true });
  writeJson(path.join(appRoot, "sdkwork.app.config.json"), {
    schemaVersion: 3,
    app: {
      key: relativeRoot.replaceAll(/[\\/]/g, "-"),
    },
  });
  writeJson(path.join(appRoot, "package.json"), {
    name: relativeRoot.replaceAll(/[\\/]/g, "-"),
    ...packageJson,
  });
  writeFileSync(path.join(appRoot, "src", "index.ts"), source);
  return appRoot;
}

test("Drive consumer integration check ignores apps without Drive upload or SDK usage", () => {
  const { workspaceRoot, driveRoot } = createWorkspace();
  createApp(workspaceRoot, "no-drive-app", {
    source: "export const app = 'no drive upload usage';\n",
  });
  createApp(workspaceRoot, "raw-drive-uploader-app", {
    source: "export const path = '/app/v3/api/drive/upload_sessions';\n",
  });
  createApp(workspaceRoot, "declared-drive-sdk-app", {
    packageJson: {
      dependencies: {
        "@sdkwork/drive-app-sdk": "workspace:*",
      },
    },
    source: "import { createDriveUploaderClient } from '@sdkwork/drive-app-sdk/uploader';\n",
  });
  const sdkMetadataOnlyRoot = createApp(workspaceRoot, "sdk-metadata-only-app", {
    source: "export const app = 'no runtime drive upload usage';\n",
  });
  mkdirSync(path.join(sdkMetadataOnlyRoot, "sdks", "app-sdk"), { recursive: true });
  writeFileSync(
    path.join(sdkMetadataOnlyRoot, "sdks", "app-sdk", "standardize.mjs"),
    "export const dependency = '@sdkwork/drive-app-sdk';\n",
  );

  const result = analyzeDriveAppSdkConsumerIntegration({ workspaceRoot, driveRoot });

  assert.equal(result.apps.length, 4);
  assert.deepEqual(
    result.failures.map((failure) => failure.app.relativeRoot),
    ["raw-drive-uploader-app"],
  );
  assert.equal(result.failures[0].missingDependency, true);
  assert.equal(result.failures[0].forbidden.length, 1);
});
