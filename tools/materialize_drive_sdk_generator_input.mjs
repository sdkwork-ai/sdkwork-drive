#!/usr/bin/env node
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const sdkFamilies = [
  {
    sdkName: "sdkwork-drive-sdk",
    sourceOpenapiPath: path.join(
      repoRoot,
      "apis",
      "open-api",
      "drive",
      "drive-open-api.openapi.json",
    ),
  },
  {
    sdkName: "sdkwork-drive-app-sdk",
    sourceOpenapiPath: path.join(
      repoRoot,
      "apis",
      "app-api",
      "drive",
      "drive-app-api.openapi.json",
    ),
  },
  {
    sdkName: "sdkwork-drive-backend-sdk",
    sourceOpenapiPath: path.join(
      repoRoot,
      "apis",
      "backend-api",
      "drive",
      "drive-backend-api.openapi.json",
    ),
  },
  {
    sdkName: "sdkwork-drive-admin-storage-sdk",
    sourceOpenapiPath: path.join(
      repoRoot,
      "apis",
      "backend-api",
      "drive",
      "drive-admin-storage-api.openapi.json",
    ),
  },
];

const { toOwnerOnlyOpenApi } = await import(
  pathToFileURL(path.join(repoRoot, "tools", "drive_sdk_generator_runner.mjs")).href
);

for (const family of sdkFamilies) {
  if (!existsSync(family.sourceOpenapiPath)) {
    console.error(
      `materialize_drive_sdk_generator_input: missing ${family.sourceOpenapiPath}`,
    );
    process.exit(1);
  }

  const openapiDocument = JSON.parse(readFileSync(family.sourceOpenapiPath, "utf8"));
  const ownerOnlyDocument = toOwnerOnlyOpenApi(openapiDocument);
  const outputDir = path.join(
    repoRoot,
    "target",
    "drive-sdk-generator-input",
    family.sdkName,
  );
  mkdirSync(outputDir, { recursive: true });
  const outputPath = path.join(outputDir, path.basename(family.sourceOpenapiPath));
  writeFileSync(outputPath, `${JSON.stringify(ownerOnlyDocument, null, 2)}\n`, "utf8");

  if (!/SdkWorkApiResponse/.test(JSON.stringify(ownerOnlyDocument))) {
    console.error(
      `materialize_drive_sdk_generator_input: ${family.sdkName} owner-only OpenAPI is missing SdkWorkApiResponse envelope schemas`,
    );
    process.exit(1);
  }

  console.log(
    `materialize_drive_sdk_generator_input: wrote ${path.relative(repoRoot, outputPath)}`,
  );
}
