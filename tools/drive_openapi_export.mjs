#!/usr/bin/env node
import { execSync } from "node:child_process";
import { createRequire } from "node:module";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, "..");

const YAML_PACKAGE_VERSION = "2.8.0";

function yamlModulePath(baseDir) {
  return path.join(baseDir, "node_modules", "yaml", "package.json");
}

function loadYamlModuleFrom(baseDir) {
  const packageJsonPath = yamlModulePath(baseDir);
  if (!existsSync(packageJsonPath)) {
    return null;
  }
  return createRequire(packageJsonPath)("yaml");
}

function ensureYamlModule() {
  const candidates = [
    workspaceRoot,
    path.join(workspaceRoot, ".pnpm-tooling"),
  ];
  for (const baseDir of candidates) {
    const yamlModule = loadYamlModuleFrom(baseDir);
    if (yamlModule) {
      return yamlModule;
    }
  }

  const toolingRoot = path.join(workspaceRoot, ".pnpm-tooling");
  mkdirSync(toolingRoot, { recursive: true });
  writeFileSync(
    path.join(toolingRoot, "package.json"),
    `${JSON.stringify(
      {
        name: "sdkwork-drive-tooling-deps",
        private: true,
        dependencies: {
          yaml: YAML_PACKAGE_VERSION,
        },
      },
      null,
      2,
    )}\n`,
  );
  writeFileSync(path.join(toolingRoot, ".npmrc"), "workspaces=false\n");
  execSync("npm install --no-package-lock --ignore-scripts", {
    cwd: toolingRoot,
    stdio: "inherit",
  });

  const yamlModule = loadYamlModuleFrom(toolingRoot);
  if (!yamlModule) {
    throw new Error(
      `unable to resolve yaml@${YAML_PACKAGE_VERSION}; run pnpm install at the repository root`,
    );
  }
  return yamlModule;
}

const HTTP_METHODS = new Set([
  "get",
  "post",
  "put",
  "patch",
  "delete",
  "head",
  "options",
  "trace",
]);
const SDK_OWNER = "sdkwork-drive";
const SDK_AUTHORITIES = {
  open: "sdkwork-drive.open",
  app: "sdkwork-drive-app-api",
  backend: "sdkwork-drive.backend",
  adminStorage: "sdkwork-drive.admin.storage",
};
const BACKEND_OPERATION_PERMISSIONS = new Map([
  ["auditEvents.list", "drive.audit.admin"],
  ["labels.list", "drive.labels.admin"],
  ["labels.create", "drive.labels.admin"],
  ["labels.retrieve", "drive.labels.admin"],
  ["labels.update", "drive.labels.admin"],
  ["labels.delete", "drive.labels.admin"],
  ["maintenance.jobs.list", "drive.maintenance.admin"],
  ["maintenance.objectSweep", "drive.maintenance.admin"],
  ["maintenance.uploadSessionSweep", "drive.maintenance.admin"],
  ["maintenance.expiredUploadContentSweep", "drive.maintenance.admin"],
  ["maintenance.abandonedUploadTaskSweep", "drive.maintenance.admin"],
  ["quotas.retrieve", "drive.quota.admin"],
  ["quotas.update", "drive.quota.admin"],
  ["spaces.admin.list", "drive.spaces.admin"],
  ["downloadPackages.list", "drive.download_packages.admin"],
  ["sandboxVolumes.list", "drive.sandboxes.admin"],
  ["sandboxVolumes.create", "drive.sandboxes.admin"],
  ["sandboxVolumes.retrieve", "drive.sandboxes.admin"],
  ["sandboxVolumes.update", "drive.sandboxes.admin"],
  ["sandboxVolumes.delete", "drive.sandboxes.admin"],
  ["sandboxGrants.list", "drive.sandboxes.admin"],
  ["sandboxGrants.create", "drive.sandboxes.admin"],
  ["sandboxGrants.retrieve", "drive.sandboxes.admin"],
  ["sandboxGrants.update", "drive.sandboxes.admin"],
  ["sandboxGrants.delete", "drive.sandboxes.admin"],
]);
const ADMIN_STORAGE_OPERATION_PERMISSION = "drive.storage.admin";
const APPBASE_APP_PATH_PREFIXES = [
  "/app/v3/api/auth/",
  "/app/v3/api/iam/",
  "/app/v3/api/oauth/",
  "/app/v3/api/open_platform/",
  "/app/v3/api/system/iam/",
];
const APPBASE_BACKEND_PATH_PREFIXES = [
  "/backend/v3/api/auth/",
  "/backend/v3/api/iam/",
  "/backend/v3/api/open_platform/",
  "/backend/v3/api/system/iam/",
];
const APPBASE_SCHEMA_PREFIXES = ["Iam"];
const APPBASE_TAGS = new Set(["auth", "iam", "oauth", "openPlatform", "system"]);
const APP_STORAGE_ADMIN_PATH_PREFIXES = [
  "/app/v3/api/drive/storage_providers",
  "/app/v3/api/drive/storage_provider_bindings",
];
const APP_STORAGE_ADMIN_SCHEMA_NAMES = new Set([
  "CopyProviderObjectRequest",
  "CreateStorageProviderRequest",
  "DeleteStorageProviderResponse",
  "ListStorageProvidersResponse",
  "ProviderBucket",
  "ProviderBucketMutation",
  "ProviderObject",
  "ProviderObjectList",
  "ProviderObjectMutation",
  "RotateStorageProviderCredentialRequest",
  "SetDefaultStorageProviderBindingRequest",
  "StorageProvider",
  "StorageProviderBinding",
  "StorageProviderCapabilities",
  "TestStorageProviderRequest",
  "TestStorageProviderResponse",
  "UpdateStorageProviderRequest",
]);
const STORAGE_PROVIDER_KIND_ENUM = [
  "local_filesystem",
  "s3_compatible",
  "google_cloud_storage",
  "aliyun_oss",
  "tencent_cos",
  "huawei_obs",
  "volcengine_tos",
];
const DRIVE_SPACE_TYPE_ENUM = [
  "personal",
  "team",
  "knowledge_base",
  "ai_generated",
  "git_repository",
  "deployment",
  "app_upload",
  "im",
  "rtc",
  "notary",
  "website",
];
const STORAGE_PROVIDER_KIND_PATTERN =
  "^(local_filesystem|s3_compatible|google_cloud_storage|aliyun_oss|tencent_cos|huawei_obs|volcengine_tos|custom:[a-z0-9_-]{2,32})$";
const OBJECT_KEY_PATTERN =
  "^(?!/)(?!.*//)(?!.*(?:^|/)\\.{1,2}(?:/|$))(?!.*\\u0000).*(?:[^/])$";
const OBJECT_LIST_ENTRY_KEY_PATTERN =
  "^(?!/)(?!.*//)(?!.*(?:^|/)\\.{1,2}(?:/|$))(?!.*\\u0000).+$";
const OBJECT_KEY_DESCRIPTION =
  "Drive object key. UTF-8 1-1024 bytes, trimmed relative key; no leading/trailing slash, double slash, NUL, or period-only path segments.";
const OBJECT_LIST_ENTRY_KEY_DESCRIPTION =
  "Drive object listing key. UTF-8 1-1024 bytes, trimmed relative key; prefixes may end with a slash; no leading slash, double slash, NUL, or period-only path segments.";
const OBJECT_KEY_PATH_DESCRIPTION =
  "Drive object key path tail. The runtime accepts slash-separated tail paths such as objects/file.bin and URL-encoded keys where / is encoded as %2F. UTF-8 1-1024 bytes, trimmed relative key; no leading/trailing slash, double slash, NUL, or period-only path segments.";
const S3_BUCKET_NAME_PATTERN =
  "^(?!xn--)(?!sthree-)(?!.*\\.\\.)(?!.*\\.-)(?!.*-\\.)(?!\\d+\\.\\d+\\.\\d+\\.\\d+$)(?!.*(-s3alias|--ol-s3|\\.mrap|--x-s3)$)[a-z0-9][a-z0-9.-]{1,61}[a-z0-9]$";
const S3_BUCKET_NAME_DESCRIPTION =
  "S3-compatible bucket name. DNS-compatible 3-63 characters; lowercase letters, digits, dots, and hyphens only; must start and end with a letter or digit; no IPv4-looking names, adjacent dots, dot-hyphen adjacency, or reserved S3 affixes.";
const STORAGE_CREDENTIAL_REF_PATTERN = "^(plain|env|secret|kms|vault):.+$";
const STORAGE_CREDENTIAL_REF_DESCRIPTION =
  "Drive storage credential reference. Supported forms: plain:<accessKeyId>:<secretAccessKey>[:<sessionToken>], env:<accessKeyEnv>:<secretKeyEnv>[:<sessionTokenEnv>], secret:<ref>, kms:<ref>, or vault:<ref>. secret/kms/vault refs are materialized at runtime from SDKWORK_DRIVE_STORAGE_CREDENTIAL__<sanitized_ref>__ACCESS_KEY_ID, __SECRET_ACCESS_KEY, and optional __SESSION_TOKEN environment variables.";
const OBJECT_KEY_SCHEMA = {
  type: "string",
  minLength: 1,
  maxLength: 1024,
  pattern: OBJECT_KEY_PATTERN,
  description: OBJECT_KEY_DESCRIPTION,
};
const OBJECT_LIST_ENTRY_KEY_SCHEMA = {
  type: "string",
  minLength: 1,
  maxLength: 1024,
  pattern: OBJECT_LIST_ENTRY_KEY_PATTERN,
  description: OBJECT_LIST_ENTRY_KEY_DESCRIPTION,
};
const STORAGE_ROOT_PREFIX_SCHEMA = {
  type: "string",
  minLength: 1,
  maxLength: 512,
  pattern: OBJECT_KEY_PATTERN,
  description:
    "Storage binding root prefix. UTF-8 1-512 bytes, trimmed relative prefix; no leading/trailing slash, double slash, NUL, or period-only path segments.",
};
const S3_BUCKET_NAME_SCHEMA = {
  type: "string",
  minLength: 3,
  maxLength: 63,
  pattern: S3_BUCKET_NAME_PATTERN,
  description: S3_BUCKET_NAME_DESCRIPTION,
};
const USAGE_CONTEXT_SCHEMA = {
  type: "string",
  minLength: 1,
  maxLength: 128,
  pattern: "^[A-Za-z0-9._:@-]+$",
  description:
    "Drive uploader usage context identifier. Optional semantic context for idempotency, ownership, and cleanup scoping.",
};
const UPLOADER_SHARE_TOKEN_SCHEMA = {
  type: "string",
  minLength: 1,
  maxLength: 512,
  description:
    "Optional Drive share token authorizing anonymous or external uploads into an explicit target folder. The raw token is accepted only on prepare requests and is never returned.",
};
const STORAGE_PROVIDER_STRICT_TLS_SCHEMA = {
  type: "boolean",
  description:
    "Provider-level TLS policy. HTTPS endpoints default to true, private HTTP endpoints default to false, and true requires an HTTPS endpoint.",
};
const STORAGE_CREDENTIAL_REF_SCHEMA = {
  type: "string",
  minLength: 1,
  maxLength: 255,
  pattern: STORAGE_CREDENTIAL_REF_PATTERN,
  description: STORAGE_CREDENTIAL_REF_DESCRIPTION,
};

const generatedOpenapiDir = path.join(workspaceRoot, "apis", "openapi");
const defaultOpenOpenapiPath = path.join(
  generatedOpenapiDir,
  "drive-open-api.openapi.json",
);
const defaultAppOpenapiPath = path.join(
  generatedOpenapiDir,
  "drive-app-api.openapi.json",
);
const defaultBackendOpenapiPath = path.join(
  generatedOpenapiDir,
  "drive-backend-api.openapi.json",
);
const defaultAdminStorageOpenapiPath = path.join(
  workspaceRoot,
  "apis",
  "backend-api",
  "drive",
  "drive-admin-storage-api.openapi.json",
);

function fail(message) {
  process.stderr.write(`[drive_openapi_export] ${message}\n`);
  process.exit(1);
}

function resolveWorkspacePath(inputPath) {
  if (!inputPath) {
    fail("path argument cannot be empty");
  }
  if (path.isAbsolute(inputPath)) {
    return inputPath;
  }
  return path.resolve(workspaceRoot, inputPath);
}

function readOpenapiDocument(filePath, label) {
  if (!existsSync(filePath)) {
    fail(`missing OpenAPI file: ${filePath}`);
  }
  const raw = readFileSync(filePath, "utf8");
  if (filePath.endsWith(".yaml") || filePath.endsWith(".yml")) {
    return parseYamlOpenapi(raw, label);
  }
  try {
    return JSON.parse(raw);
  } catch (error) {
    fail(`invalid JSON in ${label}: ${error.message}`);
  }
}

function parseYamlOpenapi(raw, label) {
  try {
    const { parse } = ensureYamlModule();
    const parsed = parse(raw);
    if (!parsed || typeof parsed !== "object") {
      fail(`invalid YAML in ${label}: expected an object document`);
    }
    return parsed;
  } catch (error) {
    fail(`invalid YAML in ${label}: ${error.message}`);
  }
}

function ensureReadableJson(filePath) {
  return readOpenapiDocument(filePath, filePath);
}

function maybeReadJson(filePath, label) {
  if (!filePath) {
    return null;
  }
  if (!existsSync(filePath)) {
    return null;
  }
  return readOpenapiDocument(filePath, label);
}

function cloneJson(value) {
  return JSON.parse(JSON.stringify(value));
}

function lowerInitial(value) {
  if (!value) {
    return value;
  }
  return `${value.charAt(0).toLowerCase()}${value.slice(1)}`;
}

function upperInitial(value) {
  if (!value) {
    return value;
  }
  return `${value.charAt(0).toUpperCase()}${value.slice(1)}`;
}

function normalizeTagName(tagName) {
  const raw = String(tagName || "").trim();
  if (!raw) {
    return raw;
  }

  const words = raw
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1 $2")
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .split(/[^A-Za-z0-9]+/)
    .map((word) => word.trim())
    .filter(Boolean);

  if (words.length === 0) {
    return raw;
  }

  return words
    .map((word, index) => (index === 0 ? lowerInitial(word) : upperInitial(lowerInitial(word))))
    .join("");
}

function normalizeOperationTags(document) {
  const tagNameMap = new Map();

  if (Array.isArray(document.tags)) {
    for (const tag of document.tags) {
      if (!tag || typeof tag !== "object" || typeof tag.name !== "string") {
        continue;
      }
      const normalized = normalizeTagName(tag.name);
      tagNameMap.set(tag.name, normalized);
      tag.name = normalized;
    }
  }

  for (const pathItem of Object.values(document.paths || {})) {
    if (!pathItem || typeof pathItem !== "object") {
      continue;
    }
    for (const [methodName, operation] of Object.entries(pathItem)) {
      if (!HTTP_METHODS.has(methodName) || !operation || typeof operation !== "object") {
        continue;
      }
      if (!Array.isArray(operation.tags)) {
        continue;
      }
      operation.tags = operation.tags.map((tag) => {
        const current = String(tag || "");
        return tagNameMap.get(current) || normalizeTagName(current);
      });
    }
  }
}

function problemDetailMediaType() {
  return {
    schema: {
      $ref: "#/components/schemas/ProblemDetail",
    },
  };
}

function findContentKey(content, expectedMediaType) {
  const expected = expectedMediaType.toLowerCase();
  return Object.keys(content).find((key) => key.toLowerCase() === expected);
}

function normalizeErrorResponseContent(document) {
  for (const pathItem of Object.values(document.paths || {})) {
    if (!pathItem || typeof pathItem !== "object") {
      continue;
    }
    for (const [methodName, operation] of Object.entries(pathItem)) {
      if (!HTTP_METHODS.has(methodName) || !operation || typeof operation !== "object") {
        continue;
      }
      for (const [statusCode, response] of Object.entries(operation.responses || {})) {
        const numericStatus = Number(statusCode);
        if (!Number.isFinite(numericStatus) || numericStatus < 400) {
          continue;
        }
        if (!response || typeof response !== "object") {
          continue;
        }

        const content =
          response.content && typeof response.content === "object" ? response.content : {};
        const problemKey = findContentKey(content, "application/problem+json");
        if (problemKey) {
          if (problemKey !== "application/problem+json") {
            response.content = {
              "application/problem+json": content[problemKey],
              ...content,
            };
          }
          continue;
        }

        const jsonKey = findContentKey(content, "application/json");
        response.content = {
          "application/problem+json": jsonKey ? content[jsonKey] : problemDetailMediaType(),
          ...content,
        };
      }
    }
  }
}

function operationExclusionKey(method, pathKey) {
  return `${method.toLowerCase()} ${pathKey}`;
}

function removeDependencyOwnedOperations(document, {
  dependencyWorkspace,
  dependencyAuthority,
  exactOperationKeys = new Set(),
  pathPrefixes = [],
  schemaPrefixes = [],
  tagNames = new Set(),
}) {
  const removed = [];
  const paths = document.paths && typeof document.paths === "object" ? document.paths : {};
  for (const [pathKey, pathItem] of Object.entries(paths)) {
    if (!pathItem || typeof pathItem !== "object") {
      continue;
    }
    for (const [methodName, operation] of Object.entries(pathItem)) {
      if (!HTTP_METHODS.has(methodName.toLowerCase())) {
        continue;
      }
      const key = operationExclusionKey(methodName, pathKey);
      const matched = exactOperationKeys.has(key)
        || pathPrefixes.some((prefix) => pathKey.startsWith(prefix));
      if (!matched) {
        continue;
      }
      removed.push({
        method: methodName.toUpperCase(),
        path: pathKey,
        operationId: operation?.operationId ? String(operation.operationId) : "",
        dependencyWorkspace,
        apiAuthority: dependencyAuthority,
      });
      delete pathItem[methodName];
    }
    const remainingMethods = Object.keys(pathItem).filter((methodName) =>
      HTTP_METHODS.has(methodName.toLowerCase()),
    );
    if (remainingMethods.length === 0) {
      delete paths[pathKey];
    }
  }

  if (document.components?.schemas && schemaPrefixes.length > 0) {
    for (const schemaName of Object.keys(document.components.schemas)) {
      if (schemaPrefixes.some((prefix) => schemaName.startsWith(prefix))) {
        delete document.components.schemas[schemaName];
      }
    }
  }

  if (Array.isArray(document.tags) && tagNames.size > 0) {
    document.tags = document.tags.filter((tag) => !tagNames.has(String(tag?.name || "")));
  }

  return removed;
}

function annotateOwnerOnlyOpenapi(document, { authority, dependencyExclusions = {} }) {
  document["x-sdkwork-owner"] = SDK_OWNER;
  document["x-sdkwork-api-authority"] = authority;
  document.info = document.info || {};
  document.info["x-sdkwork-owner"] = SDK_OWNER;
  document.info["x-sdkwork-api-authority"] = authority;

  const nonEmptyExclusions = Object.fromEntries(
    Object.entries(dependencyExclusions).filter(([, operations]) => operations.length > 0),
  );
  if (Object.keys(nonEmptyExclusions).length > 0) {
    document["x-sdkwork-dependency-exclusions"] = nonEmptyExclusions;
  } else {
    delete document["x-sdkwork-dependency-exclusions"];
  }

  for (const pathItem of Object.values(document.paths || {})) {
    if (!pathItem || typeof pathItem !== "object") {
      continue;
    }
    for (const [methodName, operation] of Object.entries(pathItem)) {
      if (!HTTP_METHODS.has(methodName.toLowerCase()) || !operation || typeof operation !== "object") {
        continue;
      }
      operation["x-sdkwork-owner"] = SDK_OWNER;
      operation["x-sdkwork-api-authority"] = authority;
    }
  }
}

function annotateBackendPermissions(document, authority) {
  if (authority !== SDK_AUTHORITIES.backend && authority !== SDK_AUTHORITIES.adminStorage) {
    return;
  }

  const observedOperationIds = new Set();
  for (const pathItem of Object.values(document.paths || {})) {
    if (!pathItem || typeof pathItem !== "object") {
      continue;
    }
    for (const [methodName, operation] of Object.entries(pathItem)) {
      if (!HTTP_METHODS.has(methodName.toLowerCase()) || !operation || typeof operation !== "object") {
        continue;
      }
      const operationId = String(operation.operationId || "").trim();
      if (!operationId) {
        fail(`${authority} operation ${methodName.toUpperCase()} is missing operationId`);
      }
      const permission = authority === SDK_AUTHORITIES.adminStorage
        ? ADMIN_STORAGE_OPERATION_PERMISSION
        : BACKEND_OPERATION_PERMISSIONS.get(operationId);
      if (!permission) {
        fail(`${authority} operation ${operationId} is missing a Drive IAM permission mapping`);
      }
      operation["x-sdkwork-permission"] = permission;
      observedOperationIds.add(operationId);
    }
  }

  if (authority === SDK_AUTHORITIES.backend) {
    for (const operationId of BACKEND_OPERATION_PERMISSIONS.keys()) {
      if (!observedOperationIds.has(operationId)) {
        fail(`${authority} permission mapping references missing operationId: ${operationId}`);
      }
    }
  }
}

function collectSchemaRefs(value, refs = new Set()) {
  if (!value || typeof value !== "object") {
    return refs;
  }
  if (typeof value.$ref === "string") {
    const prefix = "#/components/schemas/";
    if (value.$ref.startsWith(prefix)) {
      refs.add(value.$ref.slice(prefix.length));
    }
  }
  if (Array.isArray(value)) {
    for (const item of value) {
      collectSchemaRefs(item, refs);
    }
    return refs;
  }
  for (const nested of Object.values(value)) {
    collectSchemaRefs(nested, refs);
  }
  return refs;
}

function removePathsByPrefix(document, pathPrefixes) {
  if (!document.paths || typeof document.paths !== "object") {
    return [];
  }
  const removed = [];
  for (const pathKey of Object.keys(document.paths)) {
    if (pathPrefixes.some((prefix) => pathKey === prefix || pathKey.startsWith(`${prefix}/`))) {
      delete document.paths[pathKey];
      removed.push(pathKey);
    }
  }
  return removed;
}

function removeSchemasByName(document, schemaNames) {
  const schemas = document.components?.schemas;
  if (!schemas || typeof schemas !== "object") {
    return [];
  }
  const removed = [];
  for (const schemaName of schemaNames) {
    if (schemas[schemaName]) {
      delete schemas[schemaName];
      removed.push(schemaName);
    }
  }
  return removed;
}

function pruneUnreferencedSchemas(document) {
  const schemas = document.components?.schemas;
  if (!schemas || typeof schemas !== "object") {
    return [];
  }
  const referenced = collectSchemaRefs(document.paths || {});
  const queue = [...referenced];
  while (queue.length > 0) {
    const schemaName = queue.shift();
    const schema = schemas[schemaName];
    if (!schema) {
      continue;
    }
    for (const nestedName of collectSchemaRefs(schema)) {
      if (!referenced.has(nestedName)) {
        referenced.add(nestedName);
        queue.push(nestedName);
      }
    }
  }
  const removed = [];
  for (const schemaName of Object.keys(schemas)) {
    if (!referenced.has(schemaName)) {
      delete schemas[schemaName];
      removed.push(schemaName);
    }
  }
  return removed;
}

function removeAppStorageAdminSurface(document) {
  removePathsByPrefix(document, APP_STORAGE_ADMIN_PATH_PREFIXES);
  removeSchemasByName(document, APP_STORAGE_ADMIN_SCHEMA_NAMES);
  pruneUnreferencedSchemas(document);
}

function problemDetailResponse(description) {
  return {
    description,
    content: {
      "application/problem+json": problemDetailMediaType(),
    },
  };
}

function materializeOwnerOnlyOpenapi(document, {
  authority,
  dependencyExclusions = [],
}) {
  const normalized = normalizeOpenapiDocument(document, `${authority} openapi`);
  const exclusionsByWorkspace = {};
  for (const exclusion of dependencyExclusions) {
    const removed = removeDependencyOwnedOperations(normalized, exclusion);
    if (removed.length > 0) {
      exclusionsByWorkspace[exclusion.dependencyWorkspace] = [
        ...(exclusionsByWorkspace[exclusion.dependencyWorkspace] || []),
        ...removed,
      ];
    }
  }
  if (authority === SDK_AUTHORITIES.app) {
    removeAppStorageAdminSurface(normalized);
  }
  pruneUnreferencedSchemas(normalized);
  annotateOwnerOnlyOpenapi(normalized, {
    authority,
    dependencyExclusions: exclusionsByWorkspace,
  });
  annotateBackendPermissions(normalized, authority);
  return normalized;
}

function normalizeOpenapiDocument(document, label) {
  if (!document || typeof document !== "object") {
    fail(`${label} is not a JSON object`);
  }
  if (!document.openapi || !String(document.openapi).startsWith("3.1")) {
    fail(`${label} must use OpenAPI 3.1.x`);
  }
  if (!document.info || typeof document.info !== "object") {
    fail(`${label} missing info object`);
  }

  const normalized = cloneJson(document);
  normalizeOperationTags(normalized);
  normalizeErrorResponseContent(normalized);
  normalizeStorageProviderSchemas(normalized);
  normalizeStorageProviderBindingSchemas(normalized);
  normalizeDriveSpaceTypeSchemas(normalized);
  normalizeStorageCredentialRefSchemas(normalized);
  normalizeObjectKeyContract(normalized);
  normalizeUploaderUsageContextSchemas(normalized);
  return normalized;
}

function normalizeDriveSpaceTypeSchemas(document) {
  const schemas = document.components?.schemas;
  if (!schemas || typeof schemas !== "object") {
    return;
  }
  const driveNode = schemas.DriveNode;
  if (driveNode && typeof driveNode === "object") {
    driveNode.properties =
      driveNode.properties && typeof driveNode.properties === "object"
        ? driveNode.properties
        : {};
    driveNode.properties.spaceType =
      driveNode.properties.spaceType && typeof driveNode.properties.spaceType === "object"
        ? driveNode.properties.spaceType
        : {};
    driveNode.required = Array.isArray(driveNode.required)
      ? driveNode.required.filter((value, index, values) => values.indexOf(value) === index)
      : [];
    if (!driveNode.required.includes("spaceType")) {
      driveNode.required.push("spaceType");
    }
  }
  for (const schemaName of ["CreateSpaceRequest", "DriveSpace", "SpaceResponse", "DriveNode"]) {
    const spaceType = schemas[schemaName]?.properties?.spaceType;
    if (!spaceType || typeof spaceType !== "object") {
      continue;
    }
    spaceType.type = "string";
    spaceType.enum = [...DRIVE_SPACE_TYPE_ENUM];
  }
}

function normalizeStorageProviderSchemas(document) {
  const schemas = document.components?.schemas;
  if (!schemas || typeof schemas !== "object") {
    return;
  }
  for (const schemaName of ["CreateStorageProviderRequest", "StorageProvider"]) {
    const providerKind = schemas[schemaName]?.properties?.providerKind;
    if (!providerKind || typeof providerKind !== "object") {
      continue;
    }
    providerKind.type = "string";
    providerKind.enum = [...STORAGE_PROVIDER_KIND_ENUM];
    providerKind.pattern = STORAGE_PROVIDER_KIND_PATTERN;
  }
  for (const schemaName of [
    "CreateStorageProviderRequest",
    "UpdateStorageProviderRequest",
    "StorageProvider",
  ]) {
    const schema = schemas[schemaName];
    if (!schema || typeof schema !== "object") {
      continue;
    }
    schema.properties =
      schema.properties && typeof schema.properties === "object" ? schema.properties : {};
    schema.properties.strictTls = {
      ...cloneJson(STORAGE_PROVIDER_STRICT_TLS_SCHEMA),
      ...(schema.properties.strictTls || {}),
    };
  }
  const storageProvider = schemas.StorageProvider;
  if (storageProvider && typeof storageProvider === "object") {
    storageProvider.required = Array.isArray(storageProvider.required)
      ? storageProvider.required.filter((value, index, values) => values.indexOf(value) === index)
      : [];
    if (!storageProvider.required.includes("strictTls")) {
      storageProvider.required.push("strictTls");
    }
  }
}

function normalizeStorageCredentialRefSchemas(document) {
  const schemas = document.components?.schemas;
  if (!schemas || typeof schemas !== "object") {
    return;
  }
  for (const schemaName of [
    "CreateStorageProviderRequest",
    "UpdateStorageProviderRequest",
    "RotateStorageProviderCredentialRequest",
    "StorageProvider",
  ]) {
    const schema = schemas[schemaName];
    if (!schema || typeof schema !== "object") {
      continue;
    }
    schema.properties =
      schema.properties && typeof schema.properties === "object" ? schema.properties : {};
    if (!schema.properties.credentialRef) {
      continue;
    }
    schema.properties.credentialRef = {
      ...cloneJson(STORAGE_CREDENTIAL_REF_SCHEMA),
      ...(schema.properties.credentialRef || {}),
      description: STORAGE_CREDENTIAL_REF_DESCRIPTION,
      pattern: STORAGE_CREDENTIAL_REF_PATTERN,
      minLength: 1,
      maxLength: 255,
      type: "string",
    };
  }
}

function normalizeStorageProviderBindingSchemas(document) {
  const schemas = document.components?.schemas;
  if (!schemas || typeof schemas !== "object") {
    return;
  }
  const bindingRequest = schemas.SetDefaultStorageProviderBindingRequest;
  if (bindingRequest && typeof bindingRequest === "object") {
    bindingRequest.properties =
      bindingRequest.properties && typeof bindingRequest.properties === "object"
        ? bindingRequest.properties
        : {};
    bindingRequest.properties.storageRootPrefix = cloneJson(STORAGE_ROOT_PREFIX_SCHEMA);
  }

  const binding = schemas.StorageProviderBinding;
  if (binding && typeof binding === "object") {
    binding.properties =
      binding.properties && typeof binding.properties === "object"
        ? binding.properties
        : {};
    binding.properties.storageRootPrefix = cloneJson(STORAGE_ROOT_PREFIX_SCHEMA);
    binding.required = Array.isArray(binding.required)
      ? binding.required.filter((value, index, values) => values.indexOf(value) === index)
      : [];
    if (!binding.required.includes("storageRootPrefix")) {
      binding.required.push("storageRootPrefix");
    }
  }
}

function normalizeObjectKeyContract(document) {
  normalizeObjectKeySchemas(document);
  normalizeObjectKeyPathParameters(document);
}

function normalizeUploaderUsageContextSchemas(document) {
  const schemas = document.components?.schemas;
  if (!schemas || typeof schemas !== "object") {
    return;
  }
  for (const schemaName of ["PrepareUploaderUploadRequest", "UploaderUploadItem", "DriveNode"]) {
    const schema = schemas[schemaName];
    if (!schema || typeof schema !== "object") {
      continue;
    }
    schema.properties =
      schema.properties && typeof schema.properties === "object" ? schema.properties : {};
    for (const propertyName of ["scene", "source"]) {
      schema.properties[propertyName] = {
        ...cloneJson(USAGE_CONTEXT_SCHEMA),
        ...(schema.properties[propertyName] || {}),
      };
    }
    schema.required = Array.isArray(schema.required)
      ? schema.required.filter((value) => value !== "scene" && value !== "source")
      : [];
  }

  const prepareRequest = schemas.PrepareUploaderUploadRequest;
  if (prepareRequest && typeof prepareRequest === "object") {
    prepareRequest.properties =
      prepareRequest.properties && typeof prepareRequest.properties === "object"
        ? prepareRequest.properties
        : {};
    prepareRequest.properties.shareToken = {
      ...cloneJson(UPLOADER_SHARE_TOKEN_SCHEMA),
      ...(prepareRequest.properties.shareToken || {}),
    };
    prepareRequest.required = Array.isArray(prepareRequest.required)
      ? prepareRequest.required.filter((value) => value !== "shareToken")
      : [];
  }
}

function normalizeObjectKeySchemas(document) {
  const schemas = document.components?.schemas;
  if (!schemas || typeof schemas !== "object") {
    return;
  }

  const providerObjectKey = schemas.ProviderObject?.properties?.objectKey;
  if (providerObjectKey && typeof providerObjectKey === "object") {
    Object.assign(providerObjectKey, cloneJson(OBJECT_LIST_ENTRY_KEY_SCHEMA));
  }

  const copyRequestProperties = schemas.CopyProviderObjectRequest?.properties;
  if (copyRequestProperties && typeof copyRequestProperties === "object") {
    for (const propertyName of ["sourceObjectKey", "destinationObjectKey"]) {
      const property = copyRequestProperties[propertyName];
      if (property && typeof property === "object") {
        Object.assign(property, cloneJson(OBJECT_KEY_SCHEMA));
      }
    }
    if (
      copyRequestProperties.destinationBucket &&
      typeof copyRequestProperties.destinationBucket === "object"
    ) {
      Object.assign(copyRequestProperties.destinationBucket, cloneJson(S3_BUCKET_NAME_SCHEMA));
    }
  }
}

function normalizeObjectKeyPathParameters(document) {
  for (const [pathKey, pathItem] of Object.entries(document.paths || {})) {
    if (
      !pathKey.includes("/storage_providers/{providerId}/objects/{objectKey}") &&
      !pathKey.includes("/storage/providers/{providerId}/objects/{objectKey}")
    ) {
      continue;
    }
    for (const [methodName, operation] of Object.entries(pathItem || {})) {
      if (!HTTP_METHODS.has(methodName.toLowerCase()) || !operation || typeof operation !== "object") {
        continue;
      }
      for (const parameter of operation.parameters || []) {
        if (parameter?.name !== "objectKey" || parameter?.in !== "path") {
          continue;
        }
        parameter.description = OBJECT_KEY_PATH_DESCRIPTION;
        parameter.schema = cloneJson(OBJECT_KEY_SCHEMA);
        parameter.schema.description = OBJECT_KEY_DESCRIPTION;
      }
    }
  }
}

function parseArgs(argv) {
  const parsed = {
    check: false,
    outputDir: generatedOpenapiDir,
    openInput: defaultOpenOpenapiPath,
    appInput: defaultAppOpenapiPath,
    backendInput: defaultBackendOpenapiPath,
    adminStorageInput: defaultAdminStorageOpenapiPath,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const current = argv[index];
    if (current === "--check") {
      parsed.check = true;
      continue;
    }
    if (current === "--open-input") {
      parsed.openInput = resolveWorkspacePath(argv[index + 1] || "");
      index += 1;
      continue;
    }
    if (current === "--app-input") {
      parsed.appInput = resolveWorkspacePath(argv[index + 1] || "");
      index += 1;
      continue;
    }
    if (current === "--backend-input") {
      parsed.backendInput = resolveWorkspacePath(argv[index + 1] || "");
      index += 1;
      continue;
    }
    if (current === "--admin-storage-input") {
      parsed.adminStorageInput = resolveWorkspacePath(argv[index + 1] || "");
      index += 1;
      continue;
    }
    if (current === "--output-dir") {
      parsed.outputDir = resolveWorkspacePath(argv[index + 1] || "");
      index += 1;
      continue;
    }
    fail(`unknown argument: ${current}`);
  }
  return parsed;
}

const args = parseArgs(process.argv.slice(2));
const openOpenapi = materializeOwnerOnlyOpenapi(ensureReadableJson(args.openInput), {
  authority: SDK_AUTHORITIES.open,
});
const appOpenapi = materializeOwnerOnlyOpenapi(ensureReadableJson(args.appInput), {
  authority: SDK_AUTHORITIES.app,
  dependencyExclusions: [
    {
      dependencyWorkspace: "sdkwork-iam-app-sdk",
      dependencyAuthority: "sdkwork-iam-app-api",
      pathPrefixes: APPBASE_APP_PATH_PREFIXES,
      schemaPrefixes: APPBASE_SCHEMA_PREFIXES,
      tagNames: APPBASE_TAGS,
    },
  ],
});
const backendOpenapi = materializeOwnerOnlyOpenapi(ensureReadableJson(args.backendInput), {
  authority: SDK_AUTHORITIES.backend,
  dependencyExclusions: [
    {
      dependencyWorkspace: "sdkwork-iam-backend-sdk",
      dependencyAuthority: "sdkwork-iam-backend-api",
      pathPrefixes: APPBASE_BACKEND_PATH_PREFIXES,
      schemaPrefixes: APPBASE_SCHEMA_PREFIXES,
      tagNames: APPBASE_TAGS,
    },
  ],
});
const adminStorageInputOpenapi = maybeReadJson(
  args.adminStorageInput,
  "admin storage OpenAPI",
);
if (!adminStorageInputOpenapi) {
  fail(
    `admin storage OpenAPI is required; pass --admin-storage-input or ensure ${defaultAdminStorageOpenapiPath} exists`,
  );
}
const adminStorageOpenapi = materializeOwnerOnlyOpenapi(adminStorageInputOpenapi, {
  authority: SDK_AUTHORITIES.adminStorage,
});

if (!args.check) {
  mkdirSync(args.outputDir, { recursive: true });
  writeFileSync(
    path.join(args.outputDir, "drive-open-api.openapi.json"),
    `${JSON.stringify(openOpenapi, null, 2)}\n`,
    "utf8",
  );
  writeFileSync(
    path.join(args.outputDir, "drive-app-api.openapi.json"),
    `${JSON.stringify(appOpenapi, null, 2)}\n`,
    "utf8",
  );
  writeFileSync(
    path.join(args.outputDir, "drive-backend-api.openapi.json"),
    `${JSON.stringify(backendOpenapi, null, 2)}\n`,
    "utf8",
  );
  writeFileSync(
    path.join(args.outputDir, "drive-admin-storage-api.openapi.json"),
    `${JSON.stringify(adminStorageOpenapi, null, 2)}\n`,
    "utf8",
  );
}

process.stdout.write("[drive_openapi_export] ok\n");
