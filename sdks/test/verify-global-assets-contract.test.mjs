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

const expectedAssetPaths = [
  ["/app/v3/api/assets", "get"],
  ["/app/v3/api/assets", "post"],
  ["/app/v3/api/assets/{assetId}", "get"],
  ["/app/v3/api/assets/{assetId}", "patch"],
  ["/app/v3/api/assets/{assetId}/archive", "post"],
  ["/app/v3/api/assets/{assetId}/restore", "post"],
  ["/app/v3/api/assets/collections", "get"],
  ["/app/v3/api/assets/collections", "post"],
  ["/app/v3/api/assets/collections/{collectionId}/items", "post"],
  ["/app/v3/api/assets/collections/{collectionId}/items/{itemId}", "delete"],
  ["/app/v3/api/assets/{assetId}/relations", "post"],
  ["/app/v3/api/assets/{assetId}/relations/{relationId}", "delete"],
];

const forbiddenAssetPaths = [
  "/app/v3/api/generations/assets",
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

function assertPathMethod(openapi, pathKey, method) {
  assert.ok(openapi.paths?.[pathKey]?.[method], `${method.toUpperCase()} ${pathKey} must exist`);
  return openapi.paths[pathKey][method];
}

function hasDualTokenSecurity(operation) {
  return Array.isArray(operation.security) && operation.security.some((entry) =>
    Array.isArray(entry.AuthToken) &&
    entry.AuthToken.length === 0 &&
    Array.isArray(entry.AccessToken) &&
    entry.AccessToken.length === 0
  );
}

test("drive app OpenAPI exposes global assets and no generation-scoped assets", () => {
  const openapi = readJson(appOpenapiPath);

  for (const [pathKey, method] of expectedAssetPaths) {
    const operation = assertPathMethod(openapi, pathKey, method);
    assert.ok(hasDualTokenSecurity(operation), `${method.toUpperCase()} ${pathKey} must be protected`);
    assert.equal(operation["x-sdkwork-owner"], "sdkwork-drive");
    assert.equal(operation["x-sdkwork-api-authority"], "sdkwork-drive-app-api");
  }

  for (const pathKey of forbiddenAssetPaths) {
    assert.ok(!openapi.paths?.[pathKey], `${pathKey} must not be exposed`);
  }

  for (const { pathKey } of operationEntries(openapi)) {
    assert.ok(
      !pathKey.startsWith("/app/v3/api/generations/assets"),
      `global assets must not live under generations: ${pathKey}`,
    );
  }
});

test("drive app asset schemas use Drive nodes as canonical global assets", () => {
  const openapi = readJson(appOpenapiPath);
  const schemas = openapi.components?.schemas || {};

  for (const schemaName of [
    "AssetItem",
    "AssetListData",
    "AssetListHttpResponse",
    "CreateAssetRequest",
    "UpdateAssetRequest",
    "AssetCollection",
    "AssetCollectionListData",
    "AssetCollectionListHttpResponse",
    "CreateAssetCollectionRequest",
    "AssetRelation",
    "CreateAssetRelationRequest",
    "MediaResource",
  ]) {
    assert.ok(schemas[schemaName], `missing schema ${schemaName}`);
  }

  assert.equal(
    schemas.AssetListHttpResponse?.allOf?.[1]?.properties?.data?.$ref,
    "#/components/schemas/AssetListData",
    "AssetListHttpResponse.data must use AssetListData",
  );
  const assetListPayload = schemas.AssetListData?.properties || {};
  assert.equal(
    assetListPayload.items?.items?.$ref,
    "#/components/schemas/AssetItem",
    "AssetListHttpResponse.data.items must contain AssetItem",
  );
  assert.equal(
    assetListPayload.pageInfo?.$ref,
    "#/components/schemas/PageInfo",
    "AssetListHttpResponse.data.pageInfo must use PageInfo",
  );

  assert.equal(
    schemas.AssetCollectionListHttpResponse?.allOf?.[1]?.properties?.data?.$ref,
    "#/components/schemas/AssetCollectionListData",
    "AssetCollectionListHttpResponse.data must use AssetCollectionListData",
  );
  const assetCollectionListPayload = schemas.AssetCollectionListData?.properties || {};
  assert.equal(
    assetCollectionListPayload.items?.items?.$ref,
    "#/components/schemas/AssetCollection",
    "AssetCollectionListHttpResponse.data.items must contain AssetCollection",
  );
  assert.equal(
    assetCollectionListPayload.pageInfo?.$ref,
    "#/components/schemas/PageInfo",
    "AssetCollectionListHttpResponse.data.pageInfo must use PageInfo",
  );

  assert.ok(!schemas.AssetPage, "assets must not keep legacy AssetPage list schema");
  assert.ok(!schemas.AssetResourceRef, "assets must not introduce a second resource-ref entity");

  const assetItem = schemas.AssetItem.properties || {};
  for (const propertyName of [
    "assetId",
    "driveSpaceId",
    "driveNodeId",
    "driveUri",
    "resourceSnapshot",
    "nodeType",
    "scene",
    "source",
  ]) {
    assert.ok(assetItem[propertyName], `AssetItem.${propertyName} must exist`);
  }
  assert.equal(
    assetItem.resourceSnapshot.$ref,
    "#/components/schemas/MediaResource",
    "AssetItem.resourceSnapshot must reuse MediaResource",
  );

  const createAssetRequest = schemas.CreateAssetRequest.properties || {};
  assert.ok(
    createAssetRequest.driveNodeId || createAssetRequest.virtualReference,
    "CreateAssetRequest must bind to an existing Drive node or create a virtual reference",
  );
  assert.ok(!createAssetRequest.resourceRefs, "CreateAssetRequest must not duplicate Drive resources");

  for (const schemaName of ["AssetItem", "CreateAssetRequest"]) {
    const serialized = JSON.stringify(schemas[schemaName]);
    for (const forbidden of ["bucket", "objectKey", "presignedUrl", "uploadSession"]) {
      assert.ok(
        !serialized.includes(forbidden),
        `${schemaName} must not expose ${forbidden}`,
      );
    }
  }
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
