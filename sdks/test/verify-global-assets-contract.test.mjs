import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const testDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(testDir, "..", "..");
const appOpenapiPath = path.join(
  repoRoot,
  "apis",
  "app-api",
  "drive",
  "drive-app-api.openapi.json",
);
const assetsSchemaPath = path.join(
  repoRoot,
  "docs",
  "schema-registry",
  "tables",
  "005-global-assets.yaml",
);

const forbiddenAssetPaths = [
  "/app/v3/api/generations/assets",
  "/app/v3/api/assets",
  "/app/v3/api/assets/upload",
  "/app/v3/api/assets/presign",
  "/app/v3/api/assets/upload_sessions",
  "/app/v3/api/assets/download_grants",
];

const forbiddenAssetTables = [
  "dr_asset_item",
  "dr_asset_resource_ref",
  "dr_asset_version",
  "dr_asset_relation",
  "dr_asset_collection",
  "dr_asset_collection_item",
  "dr_asset_event",
  "dr_asset_projection",
];

const expectedAssetIndexes = [
  "ix_dr_drive_node_asset_list",
  "ix_dr_drive_node_asset_scene_source",
  "ix_dr_drive_storage_object_node_latest",
];

function readJson(filePath) {
  assert.ok(existsSync(filePath), `${path.relative(repoRoot, filePath)} must exist`);
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function readText(filePath) {
  assert.ok(existsSync(filePath), `${path.relative(repoRoot, filePath)} must exist`);
  return readFileSync(filePath, "utf8");
}

function operationEntries(openapi) {
  const entries = [];
  for (const [pathKey, pathItem] of Object.entries(openapi.paths || {})) {
    for (const [method, operation] of Object.entries(pathItem || {})) {
      if (!["get", "put", "post", "patch", "delete"].includes(method)) {
        continue;
      }
      entries.push({ pathKey, method, operation });
    }
  }
  return entries;
}

test("drive app OpenAPI no longer exposes global assets routes", () => {
  const openapi = readJson(appOpenapiPath);

  for (const pathKey of forbiddenAssetPaths) {
    assert.ok(!openapi.paths?.[pathKey], `${pathKey} must not be exposed by drive app-api`);
  }

  for (const { pathKey } of operationEntries(openapi)) {
    assert.ok(
      !pathKey.startsWith("/app/v3/api/assets"),
      `global assets must be owned by sdkwork-assets: ${pathKey}`,
    );
    assert.ok(
      !pathKey.startsWith("/app/v3/api/generations/assets"),
      `global assets must not live under generations: ${pathKey}`,
    );
  }
});

test("drive app OpenAPI keeps MediaResource for drive-backed snapshots", () => {
  const openapi = readJson(appOpenapiPath);
  const schemas = openapi.components?.schemas || {};
  assert.ok(schemas.MediaResource, "MediaResource schema must remain for drive snapshots");
});

test("drive global assets schema registry keeps Drive node as the asset source of truth", () => {
  const schemaRegistry = readText(assetsSchemaPath);

  assert.ok(
    schemaRegistry.includes("canonical_asset_table: dr_drive_node"),
    "schema registry must declare dr_drive_node as canonical asset table",
  );
  assert.ok(
    schemaRegistry.includes("asset_id_alias: drive_node_id"),
    "schema registry must declare assetId as drive_node_id alias",
  );
  for (const tableName of forbiddenAssetTables) {
    assert.ok(!schemaRegistry.includes(`table: ${tableName}`), `schema registry must not add ${tableName}`);
  }
  for (const indexName of expectedAssetIndexes) {
    assert.ok(schemaRegistry.includes(indexName), `schema registry missing ${indexName}`);
  }
  for (const forbidden of ["bucket_name", "object_key", "presigned_url", "asset_upload_session"]) {
    assert.ok(!schemaRegistry.includes(forbidden), `asset schema must not include ${forbidden}`);
  }
});
