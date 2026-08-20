#!/usr/bin/env node
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, "..");

const defaultOpenOpenapiPath = path.join(
  workspaceRoot,
  "apis",
  "open-api",
  "drive",
  "drive-open-api.openapi.json",
);
const defaultAppOpenapiPath = path.join(
  workspaceRoot,
  "apis",
  "app-api",
  "drive",
  "drive-app-api.openapi.json",
);
const defaultBackendOpenapiPath = path.join(
  workspaceRoot,
  "apis",
  "backend-api",
  "drive",
  "drive-backend-api.openapi.json",
);
const defaultAdminStorageOpenapiPath = path.join(
  workspaceRoot,
  "apis",
  "backend-api",
  "drive",
  "drive-admin-storage-api.openapi.json",
);
const defaultSpecialSpacesSchemaPath = path.join(
  workspaceRoot,
  "docs",
  "schema-registry",
  "tables",
  "002-drive-special-spaces.yaml",
);
const defaultSecurityAuditSchemaPath = path.join(
  workspaceRoot,
  "docs",
  "schema-registry",
  "tables",
  "004-drive-security-audit.yaml",
);
const defaultStorageSchemaPath = path.join(
  workspaceRoot,
  "docs",
  "schema-registry",
  "tables",
  "003-drive-storage.yaml",
);
const defaultGlobalAssetsSchemaPath = path.join(
  workspaceRoot,
  "docs",
  "schema-registry",
  "tables",
  "005-global-assets.yaml",
);
const STORAGE_PROVIDER_KIND_ENUM = [
  "local_filesystem",
  "s3_compatible",
  "google_cloud_storage",
  "aliyun_oss",
  "tencent_cos",
  "huawei_obs",
  "volcengine_tos",
];
const STORAGE_PROVIDER_KIND_PATTERN =
  "^(local_filesystem|s3_compatible|google_cloud_storage|aliyun_oss|tencent_cos|huawei_obs|volcengine_tos|custom:[a-z0-9_-]{2,32})$";
const STORAGE_CREDENTIAL_REF_MAX_LENGTH = 255;
const STORAGE_CREDENTIAL_REF_PATTERN = "^(plain|env|secret|kms|vault):.+$";
const OBJECT_KEY_MAX_LENGTH = 1024;
const OBJECT_KEY_PATTERN =
  "^(?!/)(?!.*//)(?!.*(?:^|/)\\.{1,2}(?:/|$))(?!.*\\u0000).*(?:[^/])$";
const OBJECT_LIST_ENTRY_KEY_PATTERN =
  "^(?!/)(?!.*//)(?!.*(?:^|/)\\.{1,2}(?:/|$))(?!.*\\u0000).+$";
const S3_BUCKET_NAME_MAX_LENGTH = 63;
const S3_BUCKET_NAME_PATTERN =
  "^(?!xn--)(?!sthree-)(?!.*\\.\\.)(?!.*\\.-)(?!.*-\\.)(?!\\d+\\.\\d+\\.\\d+\\.\\d+$)(?!.*(-s3alias|--ol-s3|\\.mrap|--x-s3)$)[a-z0-9][a-z0-9.-]{1,61}[a-z0-9]$";
const OPERATOR_ID_MAX_LENGTH = 128;
const OPERATOR_ID_PATTERN = "^[A-Za-z0-9._:@-]+$";
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
const SDK_OWNER = "sdkwork-drive";
const SDK_AUTHORITIES = {
  open: "sdkwork-drive.open",
  app: "sdkwork-drive-app-api",
  backend: "sdkwork-drive.backend",
  adminStorage: "sdkwork-drive.admin.storage",
};
const APPBASE_DEPENDENCY_PATH_PREFIXES = [
  "/app/v3/api/auth/",
  "/app/v3/api/iam/",
  "/app/v3/api/oauth/",
  "/app/v3/api/open_platform/",
  "/app/v3/api/system/iam/",
  "/backend/v3/api/auth/",
  "/backend/v3/api/iam/",
  "/backend/v3/api/open_platform/",
  "/backend/v3/api/system/iam/",
];
const APPBASE_BACKEND_DEPENDENCY_PATH_PREFIXES = [
  "/backend/v3/api/auth/",
  "/backend/v3/api/iam/",
  "/backend/v3/api/open_platform/",
  "/backend/v3/api/system/iam/",
];
const APPBASE_APP_FORBIDDEN_OPERATION_IDS = [
  "oauth.authorizationUrls.create",
  "oauth.sessions.create",
  "passwordResetRequests.create",
  "passwordResets.create",
  "registrations.create",
  "sessions.create",
  "sessions.current.delete",
  "sessions.current.retrieve",
  "sessions.current.update",
  "sessions.organizationSelection.create",
  "sessions.refresh",
  "oauth.deviceAuthorizations.create",
  "oauth.deviceAuthorizations.retrieve",
  "oauth.deviceAuthorizations.scans.create",
  "oauth.deviceAuthorizations.passwordCompletions.create",
  "iam.runtime.retrieve",
  "iam.verificationPolicy.retrieve",
  "users.current.retrieve",
];

function fail(message) {
  process.stderr.write(`[drive_schema_quality_gate] ${message}\n`);
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

function parseArgs(argv) {
  const parsed = {
    openOpenapiPath: defaultOpenOpenapiPath,
    appOpenapiPath: defaultAppOpenapiPath,
    backendOpenapiPath: defaultBackendOpenapiPath,
    adminStorageOpenapiPath: defaultAdminStorageOpenapiPath,
    specialSpacesSchemaPath: defaultSpecialSpacesSchemaPath,
    securityAuditSchemaPath: defaultSecurityAuditSchemaPath,
    storageSchemaPath: defaultStorageSchemaPath,
    globalAssetsSchemaPath: defaultGlobalAssetsSchemaPath,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const current = argv[index];
    if (current === "--open-openapi") {
      parsed.openOpenapiPath = resolveWorkspacePath(argv[index + 1] || "");
      index += 1;
      continue;
    }
    if (current === "--app-openapi") {
      parsed.appOpenapiPath = resolveWorkspacePath(argv[index + 1] || "");
      index += 1;
      continue;
    }
    if (current === "--backend-openapi") {
      parsed.backendOpenapiPath = resolveWorkspacePath(argv[index + 1] || "");
      index += 1;
      continue;
    }
    if (current === "--admin-storage-openapi") {
      parsed.adminStorageOpenapiPath = resolveWorkspacePath(argv[index + 1] || "");
      index += 1;
      continue;
    }
    if (current === "--special-spaces-schema") {
      parsed.specialSpacesSchemaPath = resolveWorkspacePath(argv[index + 1] || "");
      index += 1;
      continue;
    }
    if (current === "--security-audit-schema") {
      parsed.securityAuditSchemaPath = resolveWorkspacePath(argv[index + 1] || "");
      index += 1;
      continue;
    }
    if (current === "--storage-schema") {
      parsed.storageSchemaPath = resolveWorkspacePath(argv[index + 1] || "");
      index += 1;
      continue;
    }
    if (current === "--global-assets-schema") {
      parsed.globalAssetsSchemaPath = resolveWorkspacePath(argv[index + 1] || "");
      index += 1;
      continue;
    }
    fail(`unknown argument: ${current}`);
  }

  return parsed;
}

function readJson(filePath) {
  if (!existsSync(filePath)) {
    fail(`missing file: ${filePath}`);
  }
  try {
    return JSON.parse(readFileSync(filePath, "utf8"));
  } catch (error) {
    fail(`invalid json ${filePath}: ${error.message}`);
  }
}

function assertPathExists(document, pathKey, label) {
  if (!document.paths || !document.paths[pathKey]) {
    fail(`${label} missing path: ${pathKey}`);
  }
}

function assertPathMissing(document, pathKey, label) {
  if (document.paths && document.paths[pathKey]) {
    fail(`${label} must not expose path: ${pathKey}`);
  }
}

function assertNoDependencyOwnedPaths(document, label, pathPrefixes = APPBASE_DEPENDENCY_PATH_PREFIXES) {
  for (const pathKey of Object.keys(document.paths || {})) {
    const forbiddenPrefix = pathPrefixes.find((prefix) =>
      pathKey.startsWith(prefix),
    );
    if (forbiddenPrefix) {
      fail(`${label} must not include dependency-owned path ${pathKey}; consume ${forbiddenPrefix} routes through sdkDependencies`);
    }
  }
}

function assertOwnerMetadata(document, expectedAuthority, label) {
  if (document["x-sdkwork-owner"] !== SDK_OWNER) {
    fail(`${label} must declare x-sdkwork-owner=${SDK_OWNER}`);
  }
  if (document["x-sdkwork-api-authority"] !== expectedAuthority) {
    fail(`${label} must declare x-sdkwork-api-authority=${expectedAuthority}`);
  }
  for (const [pathKey, pathItem] of Object.entries(document.paths || {})) {
    for (const [method, operation] of Object.entries(pathItem || {})) {
      if (!["get", "put", "post", "delete", "patch", "options", "head", "trace"].includes(method)) {
        continue;
      }
      if (operation?.["x-sdkwork-owner"] !== SDK_OWNER) {
        fail(`${label} ${method.toUpperCase()} ${pathKey} must declare x-sdkwork-owner=${SDK_OWNER}`);
      }
      if (operation?.["x-sdkwork-api-authority"] !== expectedAuthority) {
        fail(`${label} ${method.toUpperCase()} ${pathKey} must declare x-sdkwork-api-authority=${expectedAuthority}`);
      }
    }
  }
}

function assertProblemDetailSchema(document, label) {
  if (!document.components || !document.components.schemas || !document.components.schemas.ProblemDetail) {
    fail(`${label} missing components.schemas.ProblemDetail`);
  }
  const properties = document.components.schemas.ProblemDetail.properties || {};
  const requiredProperties = ["type", "title", "status", "code", "traceId"];
  for (const propertyName of requiredProperties) {
    if (!properties[propertyName]) {
      fail(`${label} ProblemDetail missing property: ${propertyName}`);
    }
  }
  if (properties.requestId) {
    fail(`${label} ProblemDetail must not declare legacy requestId (use traceId per API_SPEC.md §15)`);
  }
  const codeSchema = properties.code;
  const codeType =
    codeSchema?.$ref?.endsWith("/SdkWorkPlatformErrorCode") ? "integer" : codeSchema?.type;
  if (codeType !== "integer") {
    fail(`${label} ProblemDetail.code must be numeric int32 (SdkWorkPlatformErrorCode)`);
  }
  const traceIdSchema = properties.traceId;
  if (traceIdSchema?.type !== "string") {
    fail(`${label} ProblemDetail.traceId must be a string`);
  }
}

function assertDualTokenSecurity(document, label, options = {}) {
  const pathFilter = options.pathFilter ?? (() => true);
  const schemes = document.components && document.components.securitySchemes;
  if (!schemes || typeof schemes !== "object") {
    fail(`${label} missing components.securitySchemes`);
  }
  if (schemes.AuthToken?.type !== "http" || schemes.AuthToken?.scheme !== "bearer") {
    fail(`${label} AuthToken must be an HTTP bearer security scheme`);
  }
  if (
    schemes.AccessToken?.type !== "apiKey" ||
    schemes.AccessToken?.in !== "header" ||
    schemes.AccessToken?.name !== "Access-Token"
  ) {
    fail(`${label} AccessToken must be the canonical Access-Token header security scheme`);
  }
  for (const [pathKey, pathItem] of Object.entries(document.paths || {})) {
    for (const [method, operation] of Object.entries(pathItem || {})) {
      if (!["get", "put", "post", "delete", "patch", "options", "head", "trace"].includes(method)) {
        continue;
      }
      if (!pathFilter(pathKey, operation)) {
        continue;
      }
      const security = operation.security;
      if (!Array.isArray(security)) {
        fail(`${label} ${method.toUpperCase()} ${pathKey} missing operation security`);
      }
      if (!hasDualTokenSecurity(security)) {
        fail(`${label} ${method.toUpperCase()} ${pathKey} must require AuthToken and AccessToken`);
      }
    }
  }
}

function hasDualTokenSecurity(security) {
  return Array.isArray(security) && security.some(
    (entry) =>
      Array.isArray(entry?.AuthToken) &&
      entry.AuthToken.length === 0 &&
      Array.isArray(entry?.AccessToken) &&
      entry.AccessToken.length === 0,
  );
}

function assertPublicSecurity(document, label) {
  for (const [pathKey, pathItem] of Object.entries(document.paths || {})) {
    for (const [method, operation] of Object.entries(pathItem || {})) {
      if (!["get", "put", "post", "delete", "patch", "options", "head", "trace"].includes(method)) {
        continue;
      }
      if (!Array.isArray(operation.security) || operation.security.length !== 0) {
        fail(`${label} ${method.toUpperCase()} ${pathKey} must declare security: []`);
      }
    }
  }
}

function assertSchemaHasProperties(document, schemaName, requiredProperties, label) {
  const schemas = document.components && document.components.schemas;
  if (!schemas || !schemas[schemaName]) {
    fail(`${label} missing components.schemas.${schemaName}`);
  }
  const properties = schemas[schemaName].properties || {};
  for (const propertyName of requiredProperties) {
    if (!properties[propertyName]) {
      fail(`${label} ${schemaName} missing property: ${propertyName}`);
    }
  }
}

function assertSchemaRequired(document, schemaName, propertyName, label) {
  const schemas = document.components && document.components.schemas;
  if (!schemas || !schemas[schemaName]) {
    fail(`${label} missing components.schemas.${schemaName}`);
  }
  const required = Array.isArray(schemas[schemaName].required) ? schemas[schemaName].required : [];
  if (!required.includes(propertyName)) {
    fail(`${label} ${schemaName}.${propertyName} must be required`);
  }
}

function assertSchemaOptional(document, schemaName, propertyName, label) {
  const schemas = document.components && document.components.schemas;
  if (!schemas || !schemas[schemaName]) {
    fail(`${label} missing components.schemas.${schemaName}`);
  }
  const required = Array.isArray(schemas[schemaName].required) ? schemas[schemaName].required : [];
  if (required.includes(propertyName)) {
    fail(`${label} ${schemaName}.${propertyName} must be optional`);
  }
}

function schemaProperties(document, schemaName, label) {
  const schemas = document.components && document.components.schemas;
  if (!schemas || !schemas[schemaName]) {
    fail(`${label} missing components.schemas.${schemaName}`);
  }
  return schemas[schemaName].properties || {};
}

function assertSchemaPropertyExists(document, schemaName, propertyName, label) {
  const properties = schemaProperties(document, schemaName, label);
  if (!properties[propertyName]) {
    fail(`${label} ${schemaName} missing property: ${propertyName}`);
  }
}

function assertSchemaPropertyAbsent(document, schemaName, propertyName, label) {
  const properties = schemaProperties(document, schemaName, label);
  if (properties[propertyName]) {
    fail(`${label} ${schemaName}.${propertyName} must not be exposed`);
  }
}

function assertSchemaMissing(document, schemaName, label) {
  if (document.components?.schemas?.[schemaName]) {
    fail(`${label} components.schemas.${schemaName} must not be exposed`);
  }
}

function assertSchemaPropertyFormat(document, schemaName, propertyName, expectedFormat, label) {
  const schemas = document.components && document.components.schemas;
  if (!schemas || !schemas[schemaName]) {
    fail(`${label} missing components.schemas.${schemaName}`);
  }
  const properties = schemas[schemaName].properties || {};
  const property = properties[propertyName];
  if (!property) {
    fail(`${label} ${schemaName} missing property: ${propertyName}`);
  }
  if (property.format !== expectedFormat) {
    fail(
      `${label} ${schemaName}.${propertyName} format must be ${expectedFormat}, got ${String(property.format)}`,
    );
  }
}

function assertSchemaPropertyEnum(document, schemaName, propertyName, expectedValues, label) {
  const schemas = document.components && document.components.schemas;
  if (!schemas || !schemas[schemaName]) {
    fail(`${label} missing components.schemas.${schemaName}`);
  }
  const properties = schemas[schemaName].properties || {};
  const property = properties[propertyName];
  if (!property) {
    fail(`${label} ${schemaName} missing property: ${propertyName}`);
  }
  if (!Array.isArray(property.enum)) {
    fail(`${label} ${schemaName}.${propertyName} enum must be an array`);
  }

  const actual = [...property.enum].map((value) => String(value)).sort();
  const expected = [...expectedValues].map((value) => String(value)).sort();
  if (actual.length !== expected.length) {
    fail(
      `${label} ${schemaName}.${propertyName} enum mismatch: expected ${expected.join(",")}, got ${actual.join(",")}`,
    );
  }
  for (let index = 0; index < expected.length; index += 1) {
    if (actual[index] !== expected[index]) {
      fail(
        `${label} ${schemaName}.${propertyName} enum mismatch: expected ${expected.join(",")}, got ${actual.join(",")}`,
      );
    }
  }
}

function assertSchemaPropertyStringConstraints(
  document,
  schemaName,
  propertyName,
  expectedMaxLength,
  expectedPattern,
  label,
) {
  const schemas = document.components && document.components.schemas;
  if (!schemas || !schemas[schemaName]) {
    fail(`${label} missing components.schemas.${schemaName}`);
  }
  const properties = schemas[schemaName].properties || {};
  const property = properties[propertyName];
  if (!property) {
    fail(`${label} ${schemaName} missing property: ${propertyName}`);
  }
  if (property.maxLength !== expectedMaxLength) {
    fail(
      `${label} ${schemaName}.${propertyName} maxLength must be ${expectedMaxLength}, got ${String(property.maxLength)}`,
    );
  }
  if (property.pattern !== expectedPattern) {
    fail(
      `${label} ${schemaName}.${propertyName} pattern must be ${expectedPattern}, got ${String(property.pattern)}`,
    );
  }
}

function assertStorageCredentialRefContract(document, schemaName, label) {
  assertSchemaPropertyStringConstraints(
    document,
    schemaName,
    "credentialRef",
    STORAGE_CREDENTIAL_REF_MAX_LENGTH,
    STORAGE_CREDENTIAL_REF_PATTERN,
    label,
  );
  const property =
    document.components?.schemas?.[schemaName]?.properties?.credentialRef;
  if (property?.type !== "string") {
    fail(`${label} ${schemaName}.credentialRef type must be string`);
  }
  if (property.minLength !== 1) {
    fail(
      `${label} ${schemaName}.credentialRef minLength must be 1, got ${String(property.minLength)}`,
    );
  }
  if (
    typeof property.description !== "string" ||
    !property.description.includes("SDKWORK_DRIVE_STORAGE_CREDENTIAL__")
  ) {
    fail(
      `${label} ${schemaName}.credentialRef description must document materialized external credential env variables`,
    );
  }
}

function assertSchemaPropertyEnumAndPattern(
  document,
  schemaName,
  propertyName,
  expectedValues,
  expectedPattern,
  label,
) {
  assertSchemaPropertyEnum(document, schemaName, propertyName, expectedValues, label);
  const schemas = document.components && document.components.schemas;
  if (!schemas || !schemas[schemaName]) {
    fail(`${label} missing components.schemas.${schemaName}`);
  }
  const properties = schemas[schemaName].properties || {};
  const property = properties[propertyName];
  if (!property) {
    fail(`${label} ${schemaName} missing property: ${propertyName}`);
  }
  if (property.pattern !== expectedPattern) {
    fail(
      `${label} ${schemaName}.${propertyName} pattern must be ${expectedPattern}, got ${String(property.pattern)}`,
    );
  }
}

function assertQueryParameterEnum(
  document,
  pathKey,
  method,
  parameterName,
  expectedValues,
  label,
) {
  const pathItem = document.paths && document.paths[pathKey];
  if (!pathItem || !pathItem[method]) {
    fail(`${label} missing ${method.toUpperCase()} ${pathKey}`);
  }
  const parameters = Array.isArray(pathItem[method].parameters) ? pathItem[method].parameters : [];
  const parameter = parameters.find(
    (item) => item && item.in === "query" && item.name === parameterName,
  );
  if (!parameter) {
    fail(`${label} missing query parameter ${parameterName} at ${method.toUpperCase()} ${pathKey}`);
  }
  const enumValues = parameter.schema && Array.isArray(parameter.schema.enum)
    ? parameter.schema.enum.map((value) => String(value)).sort()
    : null;
  if (!enumValues) {
    fail(
      `${label} query parameter ${parameterName} enum must be present at ${method.toUpperCase()} ${pathKey}`,
    );
  }

  const expected = expectedValues.map((value) => String(value)).sort();
  if (enumValues.length !== expected.length) {
    fail(
      `${label} query parameter ${parameterName} enum mismatch at ${method.toUpperCase()} ${pathKey}: expected ${expected.join(",")}, got ${enumValues.join(",")}`,
    );
  }
  for (let index = 0; index < expected.length; index += 1) {
    if (enumValues[index] !== expected[index]) {
      fail(
        `${label} query parameter ${parameterName} enum mismatch at ${method.toUpperCase()} ${pathKey}: expected ${expected.join(",")}, got ${enumValues.join(",")}`,
      );
    }
  }
}

function assertQueryParameterExists(document, pathKey, method, parameterName, label) {
  const pathItem = document.paths && document.paths[pathKey];
  if (!pathItem || !pathItem[method]) {
    fail(`${label} missing ${method.toUpperCase()} ${pathKey}`);
  }
  const parameters = Array.isArray(pathItem[method].parameters) ? pathItem[method].parameters : [];
  const parameter = parameters.find(
    (item) => item && item.in === "query" && item.name === parameterName,
  );
  if (!parameter) {
    fail(`${label} missing query parameter ${parameterName} at ${method.toUpperCase()} ${pathKey}`);
  }
}

function assertQueryParameterStringConstraints(
  document,
  pathKey,
  method,
  parameterName,
  expectedMaxLength,
  expectedPattern,
  label,
) {
  const pathItem = document.paths && document.paths[pathKey];
  if (!pathItem || !pathItem[method]) {
    fail(`${label} missing ${method.toUpperCase()} ${pathKey}`);
  }
  const parameters = Array.isArray(pathItem[method].parameters) ? pathItem[method].parameters : [];
  const parameter = parameters.find(
    (item) => item && item.in === "query" && item.name === parameterName,
  );
  if (!parameter) {
    fail(`${label} missing query parameter ${parameterName} at ${method.toUpperCase()} ${pathKey}`);
  }
  const schema = parameter.schema || {};
  if (schema.maxLength !== expectedMaxLength) {
    fail(
      `${label} query parameter ${parameterName} maxLength must be ${expectedMaxLength} at ${method.toUpperCase()} ${pathKey}, got ${String(schema.maxLength)}`,
    );
  }
  if (schema.pattern !== expectedPattern) {
    fail(
      `${label} query parameter ${parameterName} pattern must be ${expectedPattern} at ${method.toUpperCase()} ${pathKey}, got ${String(schema.pattern)}`,
    );
  }
}

function assertQueryParameterAbsent(document, pathKey, method, parameterName, label) {
  const operation = document.paths?.[pathKey]?.[method];
  if (!operation || typeof operation !== "object") {
    fail(`${label} missing operation ${method.toUpperCase()} ${pathKey}`);
  }
  const parameters = Array.isArray(operation.parameters) ? operation.parameters : [];
  if (parameters.some((parameter) => parameter?.name === parameterName)) {
    fail(
      `${label} ${method.toUpperCase()} ${pathKey} must not expose auth projection query parameter ${parameterName}`,
    );
  }
}

const REQUEST_CONTEXT_ONLY_INPUT_FIELDS = new Set([
  "tenantId",
  "organizationId",
  "userId",
  "appId",
  "operatorId",
  "requestId",
  "correlationId",
  "traceId",
  "subjectType",
  "subjectId",
  "anonymousId",
]);

const REQUEST_TARGET_FIELD_ALLOWLIST = new Map([
  ["CreatePermissionRequest", new Set(["subjectType", "subjectId"])],
]);

const HISTORICAL_READ_FILTER_ALLOWLIST = new Set([
  "GET /backend/v3/api/drive/audit_events query:correlationId",
  "GET /backend/v3/api/drive/audit_events query:traceId",
  "GET /backend/v3/api/drive/maintenance/jobs query:operatorId",
]);

function assertContextOnlyFieldsNotClientWritable(document, label) {
  for (const [schemaName, schema] of Object.entries(document.components?.schemas || {})) {
    if (!schemaName.endsWith("Request")) {
      continue;
    }
    const allowedTargetFields = REQUEST_TARGET_FIELD_ALLOWLIST.get(schemaName) || new Set();
    for (const propertyName of Object.keys(schema?.properties || {})) {
      if (
        REQUEST_CONTEXT_ONLY_INPUT_FIELDS.has(propertyName)
        && !allowedTargetFields.has(propertyName)
      ) {
        fail(`${label} ${schemaName}.${propertyName} must come from authenticated request context`);
      }
    }
  }

  for (const [pathKey, pathItem] of Object.entries(document.paths || {})) {
    for (const [method, operation] of Object.entries(pathItem || {})) {
      if (!["get", "put", "post", "delete", "patch", "options", "head", "trace"].includes(method)) {
        continue;
      }
      for (const parameter of operation?.parameters || []) {
        if (!REQUEST_CONTEXT_ONLY_INPUT_FIELDS.has(parameter?.name)) {
          continue;
        }
        const parameterKey = `${method.toUpperCase()} ${pathKey} ${parameter.in}:${parameter.name}`;
        if (HISTORICAL_READ_FILTER_ALLOWLIST.has(parameterKey)) {
          continue;
        }
        fail(
          `${label} ${parameterKey} must come from authenticated request context`,
        );
      }
    }
  }
}

function assertNoContentResponse(document, pathKey, method, label) {
  const operation = document.paths?.[pathKey]?.[method];
  if (!operation || typeof operation !== "object") {
    fail(`${label} missing operation ${method.toUpperCase()} ${pathKey}`);
  }
  const responses = operation.responses || {};
  const noContent = responses["204"];
  if (!noContent) {
    fail(`${label} ${method.toUpperCase()} ${pathKey} must declare 204 No Content`);
  }
  if (noContent.content) {
    fail(`${label} ${method.toUpperCase()} ${pathKey} 204 response must not declare content`);
  }
  if (responses["200"]?.content?.["application/json"]) {
    fail(`${label} ${method.toUpperCase()} ${pathKey} must not keep legacy 200 JSON delete content`);
  }
}

function assertPathParameterStringConstraints(
  document,
  pathKey,
  method,
  parameterName,
  expectedMaxLength,
  expectedPattern,
  label,
) {
  const pathItem = document.paths && document.paths[pathKey];
  if (!pathItem || !pathItem[method]) {
    fail(`${label} missing ${method.toUpperCase()} ${pathKey}`);
  }
  const parameters = Array.isArray(pathItem[method].parameters) ? pathItem[method].parameters : [];
  const parameter = parameters.find(
    (item) => item && item.in === "path" && item.name === parameterName,
  );
  if (!parameter) {
    fail(`${label} missing path parameter ${parameterName} at ${method.toUpperCase()} ${pathKey}`);
  }
  const schema = parameter.schema || {};
  if (schema.maxLength !== expectedMaxLength) {
    fail(
      `${label} path parameter ${parameterName} maxLength must be ${expectedMaxLength} at ${method.toUpperCase()} ${pathKey}, got ${String(schema.maxLength)}`,
    );
  }
  if (schema.pattern !== expectedPattern) {
    fail(
      `${label} path parameter ${parameterName} pattern must be ${expectedPattern} at ${method.toUpperCase()} ${pathKey}, got ${String(schema.pattern)}`,
    );
  }
}

function assertOpenapiVersion31(document, label) {
  if (!document.openapi || !String(document.openapi).startsWith("3.1")) {
    fail(`${label} must use OpenAPI 3.1.x`);
  }
}

function assertPathPrefix(document, prefix, label) {
  for (const pathKey of Object.keys(document.paths || {})) {
    if (!pathKey.startsWith(prefix)) {
      fail(`${label} path must start with ${prefix}: ${pathKey}`);
    }
  }
}

function collectOperationIds(document, label) {
  const httpMethods = new Set(["get", "post", "put", "patch", "delete", "head", "options", "trace"]);
  const ids = [];
  for (const [pathKey, methods] of Object.entries(document.paths || {})) {
    if (!methods || typeof methods !== "object") {
      continue;
    }
    for (const [methodName, operation] of Object.entries(methods)) {
      if (!httpMethods.has(methodName)) {
        continue;
      }
      const operationId = operation && operation.operationId ? String(operation.operationId) : "";
      if (!operationId) {
        fail(`${label} ${methodName.toUpperCase()} ${pathKey} missing operationId`);
      }
      if (!operationId.includes(".")) {
        fail(`${label} operationId must be dotted: ${operationId}`);
      }
      ids.push(operationId);
    }
  }
  return ids;
}

function assertUniqueOperationIds(operationIds, label) {
  const seen = new Set();
  for (const operationId of operationIds) {
    if (seen.has(operationId)) {
      fail(`${label} duplicated operationId: ${operationId}`);
    }
    seen.add(operationId);
  }
}

function assertOperationIdsInclude(operationIds, expectedOperationIds, label) {
  const actual = new Set(operationIds);
  for (const operationId of expectedOperationIds) {
    if (!actual.has(operationId)) {
      fail(`${label} missing operationId: ${operationId}`);
    }
  }
}

function assertOperationIdsExclude(operationIds, forbiddenOperationIds, label) {
  const actual = new Set(operationIds);
  for (const operationId of forbiddenOperationIds) {
    if (actual.has(operationId)) {
      fail(`${label} must not copy dependency-owned operationId: ${operationId}`);
    }
  }
}

function assertContains(source, marker, label) {
  if (!source.includes(marker)) {
    fail(`${label} missing marker: ${marker}`);
  }
}

function assertNotContains(source, marker, label) {
  if (source.includes(marker)) {
    fail(`${label} must not contain stale marker: ${marker}`);
  }
}

function tableBlock(source, tableName, label) {
  const needle = `  - table: ${tableName}`;
  const start = source.indexOf(needle);
  if (start < 0) {
    fail(`${label} missing table: ${tableName}`);
  }
  const rest = source.slice(start);
  const next = rest.indexOf("\n  - table: ", 1);
  return next < 0 ? rest : rest.slice(0, next);
}

const args = parseArgs(process.argv.slice(2));
const openOpenapi = readJson(args.openOpenapiPath);
const appOpenapi = readJson(args.appOpenapiPath);
const backendOpenapi = readJson(args.backendOpenapiPath);
const adminStorageOpenapi = readJson(args.adminStorageOpenapiPath);
const specialSpacesSchema = readFileSync(args.specialSpacesSchemaPath, "utf8");
const securityAuditSchema = readFileSync(args.securityAuditSchemaPath, "utf8");
const storageSchema = readFileSync(args.storageSchemaPath, "utf8");
const globalAssetsSchema = readFileSync(args.globalAssetsSchemaPath, "utf8");

assertOpenapiVersion31(openOpenapi, "open openapi");
assertOpenapiVersion31(appOpenapi, "app openapi");
assertOpenapiVersion31(backendOpenapi, "backend openapi");
assertOpenapiVersion31(adminStorageOpenapi, "admin storage openapi");
assertPathPrefix(openOpenapi, "/open/v3/api", "open openapi");
assertPathPrefix(appOpenapi, "/app/v3/api", "app openapi");
assertContextOnlyFieldsNotClientWritable(appOpenapi, "app openapi");
assertPathPrefix(backendOpenapi, "/backend/v3/api", "backend openapi");
assertContextOnlyFieldsNotClientWritable(backendOpenapi, "backend openapi");
assertPathPrefix(adminStorageOpenapi, "/backend/v3/api/drive/storage", "admin storage openapi");
assertContextOnlyFieldsNotClientWritable(adminStorageOpenapi, "admin storage openapi");

assertPathExists(openOpenapi, "/open/v3/api/drive/share_links/{token}", "open openapi");
assertPathExists(
  openOpenapi,
  "/open/v3/api/drive/share_links/{token}/download_url",
  "open openapi",
);
assertPathExists(appOpenapi, "/app/v3/api/drive/spaces", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/spaces/{spaceId}", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/spaces/{spaceId}/nodes", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/folders", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/files", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/shortcuts", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/{nodeId}", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/{nodeId}/path", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/{nodeId}/capabilities", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/{nodeId}/properties", "app openapi");
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/nodes/{nodeId}/properties/{propertyKey}",
  "app openapi",
);
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/{nodeId}/move", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/{nodeId}/copy", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/{nodeId}/trash", "app openapi");
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/nodes/{nodeId}/download_url",
  "app openapi",
);
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/nodes/{nodeId}/download_grants",
  "app openapi",
);
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/{nodeId}/labels", "app openapi");
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/nodes/{nodeId}/labels/{labelId}",
  "app openapi",
);
assertPathExists(appOpenapi, "/app/v3/api/drive/trash/{nodeId}/restore", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/trash", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/trash/empty", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/recent", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/shared_with_me", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/favorites", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/{nodeId}/favorite", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/{nodeId}/versions", "app openapi");
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/nodes/{nodeId}/versions/{versionId}/restore",
  "app openapi",
);
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/{nodeId}/permissions", "app openapi");
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/nodes/{nodeId}/permissions/effective",
  "app openapi",
);
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/nodes/{nodeId}/permissions/{permissionId}",
  "app openapi",
);
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/{nodeId}/share_links", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/share_links/{shareLinkId}", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/{nodeId}/comments", "app openapi");
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}",
  "app openapi",
);
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies",
  "app openapi",
);
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies/{replyId}",
  "app openapi",
);
assertPathExists(appOpenapi, "/app/v3/api/drive/search", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/changes", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/changes/start_page_token", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/changes/watch", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/nodes/{nodeId}/watch", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/drive/watch_channels", "app openapi");
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/watch_channels/{channelId}",
  "app openapi",
);
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/watch_channels/{channelId}/stop",
  "app openapi",
);
assertPathExists(appOpenapi, "/app/v3/api/drive/upload_sessions", "app openapi");
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/upload_sessions/{uploadSessionId}",
  "app openapi",
);
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/upload_sessions/{uploadSessionId}/parts/{partNo}",
  "app openapi",
);
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/upload_sessions/{uploadSessionId}/complete",
  "app openapi",
);
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/upload_sessions/{uploadSessionId}/abort",
  "app openapi",
);
assertPathExists(appOpenapi, "/app/v3/api/drive/download_urls", "app openapi");
assertPathExists(
  appOpenapi,
  "/app/v3/api/drive/download_tokens/{token}",
  "app openapi",
);
assertPathExists(appOpenapi, "/app/v3/api/assets", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/assets/{assetId}", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/assets/{assetId}/archive", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/assets/{assetId}/restore", "app openapi");
assertPathExists(appOpenapi, "/app/v3/api/assets/collections", "app openapi");
assertPathExists(
  appOpenapi,
  "/app/v3/api/assets/collections/{collectionId}/items",
  "app openapi",
);
assertPathExists(
  appOpenapi,
  "/app/v3/api/assets/collections/{collectionId}/items/{itemId}",
  "app openapi",
);
assertPathExists(appOpenapi, "/app/v3/api/assets/{assetId}/relations", "app openapi");
assertPathExists(
  appOpenapi,
  "/app/v3/api/assets/{assetId}/relations/{relationId}",
  "app openapi",
);
for (const forbiddenAssetPath of [
  "/app/v3/api/generations/assets",
  "/app/v3/api/assets/upload",
  "/app/v3/api/assets/presign",
  "/app/v3/api/assets/upload_sessions",
  "/app/v3/api/assets/download_grants",
]) {
  assertPathMissing(appOpenapi, forbiddenAssetPath, "app openapi");
}
assertPathMissing(
  appOpenapi,
  "/app/v3/api/drive/storage_provider_bindings/default",
  "app openapi",
);
for (const pathKey of [
  "/app/v3/api/drive/storage_providers",
  "/app/v3/api/drive/storage_providers/{providerId}",
  "/app/v3/api/drive/storage_providers/{providerId}/test",
  "/app/v3/api/drive/storage_providers/{providerId}/capabilities",
  "/app/v3/api/drive/storage_providers/{providerId}/activate",
  "/app/v3/api/drive/storage_providers/{providerId}/deactivate",
  "/app/v3/api/drive/storage_providers/{providerId}/credentials/rotate",
  "/app/v3/api/drive/storage_providers/{providerId}/bucket",
  "/app/v3/api/drive/storage_providers/{providerId}/objects",
  "/app/v3/api/drive/storage_providers/{providerId}/objects/{objectKey}",
  "/app/v3/api/drive/storage_providers/{providerId}/objects/copy",
]) {
  assertPathMissing(appOpenapi, pathKey, "app openapi");
}
for (const schemaName of [
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
]) {
  assertSchemaMissing(appOpenapi, schemaName, "app openapi");
}
assertPathMissing(
  backendOpenapi,
  "/backend/v3/api/drive/storage_providers",
  "backend openapi",
);
assertPathMissing(
  backendOpenapi,
  "/backend/v3/api/drive/storage_provider_bindings/default",
  "backend openapi",
);
assertPathExists(backendOpenapi, "/backend/v3/api/drive/labels", "backend openapi");
assertPathExists(
  backendOpenapi,
  "/backend/v3/api/drive/labels/{labelId}",
  "backend openapi",
);
assertPathExists(backendOpenapi, "/backend/v3/api/drive/audit_events", "backend openapi");
assertPathExists(
  backendOpenapi,
  "/backend/v3/api/drive/maintenance/object_sweep",
  "backend openapi",
);
assertPathExists(
  backendOpenapi,
  "/backend/v3/api/drive/maintenance/upload_session_sweep",
  "backend openapi",
);
assertPathExists(
  backendOpenapi,
  "/backend/v3/api/drive/maintenance/expired_upload_content_sweep",
  "backend openapi",
);
assertPathExists(
  backendOpenapi,
  "/backend/v3/api/drive/maintenance/abandoned_upload_task_sweep",
  "backend openapi",
);
assertPathExists(
  backendOpenapi,
  "/backend/v3/api/drive/maintenance/jobs",
  "backend openapi",
);
assertPathExists(backendOpenapi, "/backend/v3/api/drive/spaces", "backend openapi");
assertPathExists(backendOpenapi, "/backend/v3/api/drive/quotas", "backend openapi");
assertPathExists(
  adminStorageOpenapi,
  "/backend/v3/api/drive/storage/providers",
  "admin storage openapi",
);
assertPathExists(
  adminStorageOpenapi,
  "/backend/v3/api/drive/storage/providers/{providerId}",
  "admin storage openapi",
);
assertPathExists(
  adminStorageOpenapi,
  "/backend/v3/api/drive/storage/providers/{providerId}/test",
  "admin storage openapi",
);
assertPathExists(
  adminStorageOpenapi,
  "/backend/v3/api/drive/storage/providers/{providerId}/capabilities",
  "admin storage openapi",
);
assertPathExists(
  adminStorageOpenapi,
  "/backend/v3/api/drive/storage/providers/{providerId}/bucket",
  "admin storage openapi",
);
assertPathExists(
  adminStorageOpenapi,
  "/backend/v3/api/drive/storage/providers/{providerId}/buckets",
  "admin storage openapi",
);
assertPathExists(
  adminStorageOpenapi,
  "/backend/v3/api/drive/storage/providers/{providerId}/objects",
  "admin storage openapi",
);
assertPathExists(
  adminStorageOpenapi,
  "/backend/v3/api/drive/storage/providers/{providerId}/objects/{objectKey}",
  "admin storage openapi",
);
assertPathExists(
  adminStorageOpenapi,
  "/backend/v3/api/drive/storage/providers/{providerId}/objects/copy",
  "admin storage openapi",
);
assertPathExists(
  adminStorageOpenapi,
  "/backend/v3/api/drive/storage/bindings/default",
  "admin storage openapi",
);
assertPathExists(
  adminStorageOpenapi,
  "/backend/v3/api/drive/storage/bindings",
  "admin storage openapi",
);
assertProblemDetailSchema(openOpenapi, "open openapi");
assertProblemDetailSchema(appOpenapi, "app openapi");
assertProblemDetailSchema(backendOpenapi, "backend openapi");
assertProblemDetailSchema(adminStorageOpenapi, "admin storage openapi");
assertPublicSecurity(openOpenapi, "open openapi");
assertDualTokenSecurity(appOpenapi, "app openapi drive routes", {
  pathFilter: (pathKey) => pathKey.startsWith("/app/v3/api/drive/"),
});
assertDualTokenSecurity(backendOpenapi, "backend openapi");
assertDualTokenSecurity(adminStorageOpenapi, "admin storage openapi");
assertNoDependencyOwnedPaths(openOpenapi, "open openapi");
assertNoDependencyOwnedPaths(appOpenapi, "app openapi");
assertNoDependencyOwnedPaths(backendOpenapi, "backend openapi", APPBASE_BACKEND_DEPENDENCY_PATH_PREFIXES);
for (const [document, pathKey, label] of [
  [
    adminStorageOpenapi,
    "/backend/v3/api/drive/storage/providers/{providerId}/objects/{objectKey}",
    "admin storage openapi",
  ],
]) {
  for (const method of ["get", "delete"]) {
    assertPathParameterStringConstraints(
      document,
      pathKey,
      method,
      "objectKey",
      OBJECT_KEY_MAX_LENGTH,
      OBJECT_KEY_PATTERN,
      label,
    );
  }
}
assertOwnerMetadata(openOpenapi, SDK_AUTHORITIES.open, "open openapi");
assertOwnerMetadata(appOpenapi, SDK_AUTHORITIES.app, "app openapi");
assertOwnerMetadata(backendOpenapi, SDK_AUTHORITIES.backend, "backend openapi");
assertOwnerMetadata(
  adminStorageOpenapi,
  SDK_AUTHORITIES.adminStorage,
  "admin storage openapi",
);
assertSchemaPropertyExists(appOpenapi, "CreateShareLinkRequest", "token", "app openapi");
assertSchemaPropertyExists(appOpenapi, "FileVersion", "storageObjectId", "app openapi");
assertSchemaOptional(appOpenapi, "FileVersion", "storageObjectId", "app openapi");
assertSchemaHasProperties(
  appOpenapi,
  "UpdatePermissionRequest",
  [
  "role"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "UpdateShareLinkRequest",
  [
  "role",
  "expiresAtEpochMs",
  "downloadLimit"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "CreateCommentRequest",
  [
  "id",
  "content"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "UpdateCommentRequest",
  [
  "content",
  "resolved"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "DriveComment",
  [
    "id",
    "tenantId",
    "nodeId",
    "content",
    "resolved",
    "lifecycleStatus",
    "version",
    "createdBy",
    "updatedBy",
    "createdAt",
    "updatedAt",
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "CreateCommentReplyRequest",
  [
  "id",
  "content"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "UpdateCommentReplyRequest",
  [
  "content"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "DriveCommentReply",
  [
    "id",
    "tenantId",
    "nodeId",
    "commentId",
    "content",
    "lifecycleStatus",
    "version",
    "createdBy",
    "updatedBy",
    "createdAt",
    "updatedAt",
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "StartPageTokenResponse",
  ["startPageToken"],
  "app openapi",
);
for (const propertyName of ["subjectType", "subjectId", "operatorId"]) {
  assertSchemaPropertyAbsent(
    appOpenapi,
    "FavoriteNodeRequest",
    propertyName,
    "app openapi",
  );
}
assertSchemaHasProperties(
  appOpenapi,
  "CreatePermissionRequest",
  ["id", "subjectType", "subjectId", "role"],
  "app openapi",
);
assertSchemaRequired(appOpenapi, "CreatePermissionRequest", "subjectType", "app openapi");
assertSchemaRequired(appOpenapi, "CreatePermissionRequest", "subjectId", "app openapi");
assertSchemaHasProperties(
  appOpenapi,
  "FavoriteNodeResponse",
  ["favorited"],
  "app openapi",
);
assertSchemaPropertyAbsent(appOpenapi, "DriveShareLink", "token", "app openapi");
assertSchemaPropertyAbsent(appOpenapi, "DriveShareLink", "tokenHash", "app openapi");
assertSchemaHasProperties(
  appOpenapi,
  "CreateFileRequest",
  [
  "id",
  "spaceId",
  "nodeName",
  "uploadSessionId",
  "idempotencyKey",
  "expiresAtEpochMs"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "CreateFileResponse",
  ["node", "uploadSession"],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "EmptyTrashRequest",
  [
  "spaceId"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "EmptyTrashResponse",
  ["deletedCount"],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "MoveNodeRequest",
  [
  "targetParentNodeId"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "CopyNodeRequest",
  [
  "id",
  "targetSpaceId",
  "targetParentNodeId",
  "nodeName"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "DriveNode",
  ["id", "tenantId", "spaceId", "spaceType", "nodeType", "nodeName", "scene", "source"],
  "app openapi",
);
assertSchemaPropertyEnum(
  appOpenapi,
  "DriveNode",
  "spaceType",
  DRIVE_SPACE_TYPE_ENUM,
  "app openapi",
);
for (const propertyName of ["scene", "source"]) {
  assertSchemaPropertyStringConstraints(
    appOpenapi,
    "DriveNode",
    propertyName,
    OPERATOR_ID_MAX_LENGTH,
    OPERATOR_ID_PATTERN,
    "app openapi",
  );
}
assertSchemaHasProperties(
  appOpenapi,
  "NodePathResponse",
  ["items", "pathSegments"],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "PresignUploadPartRequest",
  [
  "uploadId",
  "requestedTtlSeconds"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "PresignedUploadPart",
  ["uploadUrl", "expiresAtEpochMs", "method", "headers", "partNo", "uploadId"],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "CompleteUploadSessionRequest",
  [
  "uploadId",
  "contentType",
  "contentLength",
  "checksumSha256Hex",
  "parts"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "CompletedUploadPart",
  ["partNo", "etag"],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "PrepareUploaderUploadRequest",
  [
  "id",
  "taskId",
  "appResourceType",
  "appResourceId",
  "scene",
  "source",
  "shareToken"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "UploaderUploadItem",
  [
    "id",
    "taskId",
    "tenantId",
    "actorType",
    "actorId",
    "appId",
    "appResourceType",
    "appResourceId",
    "scene",
    "source",
  ],
  "app openapi",
);
for (const schemaName of ["PrepareUploaderUploadRequest", "UploaderUploadItem"]) {
  for (const propertyName of ["scene", "source"]) {
    assertSchemaPropertyStringConstraints(
      appOpenapi,
      schemaName,
      propertyName,
      OPERATOR_ID_MAX_LENGTH,
      OPERATOR_ID_PATTERN,
      "app openapi",
    );
  }
}
assertSchemaPropertyStringConstraints(
  appOpenapi,
  "PrepareUploaderUploadRequest",
  "shareToken",
  512,
  undefined,
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "UploadSessionMutationResponse",
  ["id", "tenantId", "spaceId", "nodeId", "bucket", "objectKey", "state", "version"],
  "app openapi",
);
assertSchemaPropertyAbsent(openOpenapi, "DriveOpenShareLink", "token", "open openapi");
assertSchemaPropertyAbsent(openOpenapi, "DriveOpenShareLink", "tokenHash", "open openapi");
assertSchemaPropertyEnum(
  appOpenapi,
  "CreateSpaceRequest",
  "spaceType",
  DRIVE_SPACE_TYPE_ENUM,
  "app openapi",
);
assertSchemaPropertyEnum(
  appOpenapi,
  "DriveSpace",
  "spaceType",
  DRIVE_SPACE_TYPE_ENUM,
  "app openapi",
);
for (const schemaName of ["SweepObjectStoreRequest", "SweepUploadSessionsRequest"]) {
  for (const propertyName of ["operatorId", "requestId", "correlationId", "traceId"]) {
    assertSchemaPropertyAbsent(backendOpenapi, schemaName, propertyName, "backend openapi");
  }
}
for (const fieldName of ["startedAt", "finishedAt", "createdAt"]) {
  assertSchemaPropertyFormat(
    backendOpenapi,
    "MaintenanceJob",
    fieldName,
    "date-time",
    "backend openapi",
  );
}
assertSchemaPropertyEnum(
  backendOpenapi,
  "DriveSpace",
  "spaceType",
  DRIVE_SPACE_TYPE_ENUM,
  "backend openapi",
);
assertSchemaPropertyEnum(
  adminStorageOpenapi,
  "CreateStorageProviderRequest",
  "providerKind",
  STORAGE_PROVIDER_KIND_ENUM,
  "admin storage openapi",
);
assertSchemaPropertyEnumAndPattern(
  adminStorageOpenapi,
  "StorageProvider",
  "providerKind",
  STORAGE_PROVIDER_KIND_ENUM,
  STORAGE_PROVIDER_KIND_PATTERN,
  "admin storage openapi",
);
assertSchemaPropertyEnumAndPattern(
  adminStorageOpenapi,
  "CreateStorageProviderRequest",
  "providerKind",
  STORAGE_PROVIDER_KIND_ENUM,
  STORAGE_PROVIDER_KIND_PATTERN,
  "admin storage openapi",
);
assertSchemaHasProperties(
  adminStorageOpenapi,
  "CreateStorageProviderRequest",
  [
    "name",
    "region",
    "pathStyle",
    "strictTls",
    "serverSideEncryptionMode",
    "defaultStorageClass",
  ],
  "admin storage openapi",
);
assertSchemaHasProperties(
  adminStorageOpenapi,
  "UpdateStorageProviderRequest",
  [
    "name",
    "region",
    "pathStyle",
    "strictTls",
    "serverSideEncryptionMode",
    "defaultStorageClass",
  ],
  "admin storage openapi",
);
assertSchemaHasProperties(
  adminStorageOpenapi,
  "StorageProvider",
  [
    "name",
    "region",
    "pathStyle",
    "strictTls",
    "serverSideEncryptionMode",
    "defaultStorageClass",
    "credentialConfigured",
  ],
  "admin storage openapi",
);
assertSchemaHasProperties(
  adminStorageOpenapi,
  "StorageProviderCapabilities",
  [
    "providerId",
    "providerKind",
    "supportsMultipartUpload",
    "supportsPresignedUploadPart",
    "supportsPresignedDownload",
    "supportsServerSideEncryption",
    "supportsStorageClass",
    "supportsCredentialRotation",
    "supportedServerSideEncryptionModes",
    "supportedStorageClasses",
  ],
  "admin storage openapi",
);
assertSchemaHasProperties(
  adminStorageOpenapi,
  "ProviderBucketListItem",
  ["bucket", "configured", "creationDateEpochMs"],
  "admin storage openapi",
);
assertSchemaPropertyStringConstraints(
  adminStorageOpenapi,
  "ProviderObject",
  "objectKey",
  OBJECT_KEY_MAX_LENGTH,
  OBJECT_LIST_ENTRY_KEY_PATTERN,
  "admin storage openapi",
);
assertSchemaHasProperties(
  adminStorageOpenapi,
  "ProviderObject",
  ["objectKind"],
  "admin storage openapi",
);
assertSchemaPropertyEnum(
  adminStorageOpenapi,
  "ProviderObject",
  "objectKind",
  ["object", "prefix"],
  "admin storage openapi",
);
for (const propertyName of ["sourceObjectKey", "destinationObjectKey"]) {
  assertSchemaPropertyStringConstraints(
    adminStorageOpenapi,
    "CopyProviderObjectRequest",
    propertyName,
    OBJECT_KEY_MAX_LENGTH,
    OBJECT_KEY_PATTERN,
    "admin storage openapi",
  );
}
assertSchemaPropertyStringConstraints(
  adminStorageOpenapi,
  "CopyProviderObjectRequest",
  "destinationBucket",
  S3_BUCKET_NAME_MAX_LENGTH,
  S3_BUCKET_NAME_PATTERN,
  "admin storage openapi",
);
assertSchemaHasProperties(
  adminStorageOpenapi,
  "RotateStorageProviderCredentialRequest",
  [
  "credentialRef"
  ],
  "admin storage openapi",
);
for (const schemaName of [
  "CreateStorageProviderRequest",
  "UpdateStorageProviderRequest",
  "RotateStorageProviderCredentialRequest",
  "StorageProvider",
]) {
  assertStorageCredentialRefContract(
    adminStorageOpenapi,
    schemaName,
    "admin storage openapi",
  );
}
assertSchemaHasProperties(
  adminStorageOpenapi,
  "SetDefaultStorageProviderBindingRequest",
  [
  "providerId",
  "storageRootPrefix"
  ],
  "admin storage openapi",
);
assertSchemaHasProperties(
  adminStorageOpenapi,
  "CopyProviderObjectRequest",
  [
  "sourceObjectKey",
  "destinationObjectKey"
  ],
  "admin storage openapi",
);
assertSchemaPropertyAbsent(
  adminStorageOpenapi,
  "CopyProviderObjectRequest",
  "operatorId",
  "admin storage openapi",
);
for (const [pathKey, method] of [
  ["/backend/v3/api/drive/storage/providers/{providerId}/bucket", "put"],
  ["/backend/v3/api/drive/storage/providers/{providerId}/bucket", "delete"],
  ["/backend/v3/api/drive/storage/providers/{providerId}/objects/{objectKey}", "delete"],
  ["/backend/v3/api/drive/storage/bindings/default", "delete"],
]) {
  assertQueryParameterAbsent(
    adminStorageOpenapi,
    pathKey,
    method,
    "operatorId",
    "admin storage openapi",
  );
}
assertQueryParameterAbsent(
  adminStorageOpenapi,
  "/backend/v3/api/drive/storage/bindings",
  "get",
  "tenantId",
  "admin storage openapi",
);
assertSchemaHasProperties(
  adminStorageOpenapi,
  "StorageProviderBinding",
  [
    "id",
    "tenantId",
    "providerId",
    "bindingScope",
    "purpose",
    "storageRootPrefix",
    "lifecycleStatus",
    "version",
    "storageProvider",
  ],
  "admin storage openapi",
);
assertSchemaHasProperties(
  backendOpenapi,
  "DriveLabel",
  ["id", "tenantId", "labelKey", "displayName", "lifecycleStatus", "version"],
  "backend openapi",
);
assertSchemaHasProperties(
  backendOpenapi,
  "CreateLabelRequest",
  [
  "id",
  "labelKey",
  "displayName"
  ],
  "backend openapi",
);
assertSchemaHasProperties(
  backendOpenapi,
  "UpdateLabelRequest",
  [

  ],
  "backend openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "DriveLabelSummary",
  ["id", "tenantId", "labelKey", "displayName", "lifecycleStatus", "version"],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "NodeLabel",
  [
    "id",
    "tenantId",
    "nodeId",
    "labelId",
    "lifecycleStatus",
    "version",
    "label",
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "ApplyNodeLabelRequest",
  [

  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "DriveWatchChannel",
  [
    "id",
    "tenantId",
    "resourceType",
    "channelType",
    "address",
    "expirationEpochMs",
    "lifecycleStatus",
    "version",
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "CreateWatchChannelRequest",
  [
  "id",
  "address",
  "expirationEpochMs"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "StopWatchChannelRequest",
  [

  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "StopWatchChannelResponse",
  ["stopped", "channel"],
  "app openapi",
);
assertSchemaPropertyAbsent(appOpenapi, "DriveWatchChannel", "token", "app openapi");
assertSchemaPropertyAbsent(appOpenapi, "DriveWatchChannel", "tokenHash", "app openapi");
assertSchemaHasProperties(
  appOpenapi,
  "AssetItem",
  [
    "assetId",
    "driveSpaceId",
    "driveNodeId",
    "driveUri",
    "id",
    "tenantId",
    "assetKind",
    "title",
    "nodeType",
    "scene",
    "source",
    "resourceSnapshot",
    "lifecycleStatus",
  ],
  "app openapi",
);
assertSchemaMissing(appOpenapi, "AssetResourceRef", "app openapi");
assertSchemaHasProperties(
  appOpenapi,
  "CreateAssetRequest",
  [
  "driveNodeId",
  "virtualReference"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "UpdateAssetRequest",
  [
  "title",
  "description",
  "tags"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "AssetCollection",
  ["id", "tenantId", "userId", "title", "lifecycleStatus"],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "CreateAssetCollectionRequest",
  [
  "title"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "AssetRelation",
  ["id", "tenantId", "assetId", "relationType", "lifecycleStatus"],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "CreateAssetRelationRequest",
  [
  "relationType"
  ],
  "app openapi",
);
assertSchemaHasProperties(
  appOpenapi,
  "MediaResource",
  ["mediaResourceId", "mediaType", "contentType"],
  "app openapi",
);
const assetResourceRefProperties = schemaProperties(
  appOpenapi,
  "AssetItem",
  "app openapi",
);
if (
  assetResourceRefProperties.resourceSnapshot?.$ref !==
  "#/components/schemas/MediaResource"
) {
  fail("app openapi AssetItem.resourceSnapshot must reuse MediaResource");
}
for (const schemaName of ["AssetItem", "CreateAssetRequest"]) {
  for (const propertyName of [
    "bucket",
    "objectKey",
    "presignedUrl",
    "uploadSession",
  ]) {
    assertSchemaPropertyAbsent(appOpenapi, schemaName, propertyName, "app openapi");
  }
}
assertSchemaPropertyEnum(
  backendOpenapi,
  "MaintenanceJob",
  "jobType",
  [
    "object_sweep",
    "upload_session_sweep",
    "expired_upload_content_sweep",
    "abandoned_upload_task_sweep",
  ],
  "backend openapi",
);
assertSchemaPropertyEnum(
  backendOpenapi,
  "MaintenanceJob",
  "status",
  ["completed", "failed"],
  "backend openapi",
);
assertSchemaPropertyExists(
  backendOpenapi,
  "SandboxVolume",
  "providerRootRef",
  "backend openapi",
);
assertSchemaPropertyAbsent(
  appOpenapi,
  "DriveSandboxVolume",
  "providerRootRef",
  "app openapi",
);
assertSchemaPropertyStringConstraints(
  backendOpenapi,
  "MaintenanceJob",
  "operatorId",
  128,
  "^[A-Za-z0-9._:@-]+$",
  "backend openapi",
);
assertSchemaPropertyStringConstraints(
  backendOpenapi,
  "MaintenanceJob",
  "correlationId",
  64,
  "^[A-Za-z0-9._:@-]+$",
  "backend openapi",
);
assertSchemaPropertyStringConstraints(
  backendOpenapi,
  "MaintenanceJob",
  "traceId",
  128,
  "^[A-Za-z0-9._:@-]+$",
  "backend openapi",
);
assertQueryParameterEnum(
  backendOpenapi,
  "/backend/v3/api/drive/maintenance/jobs",
  "get",
  "jobType",
  [
    "object_sweep",
    "upload_session_sweep",
    "expired_upload_content_sweep",
    "abandoned_upload_task_sweep",
  ],
  "backend openapi",
);
assertQueryParameterEnum(
  backendOpenapi,
  "/backend/v3/api/drive/maintenance/jobs",
  "get",
  "status",
  ["completed", "failed"],
  "backend openapi",
);
assertQueryParameterStringConstraints(
  backendOpenapi,
  "/backend/v3/api/drive/maintenance/jobs",
  "get",
  "operatorId",
  128,
  "^[A-Za-z0-9._:@-]+$",
  "backend openapi",
);
assertQueryParameterStringConstraints(
  backendOpenapi,
  "/backend/v3/api/drive/audit_events",
  "get",
  "action",
  128,
  "^[A-Za-z0-9._:@-]+$",
  "backend openapi",
);
assertQueryParameterStringConstraints(
  backendOpenapi,
  "/backend/v3/api/drive/audit_events",
  "get",
  "resourceType",
  64,
  "^[A-Za-z0-9._:@-]+$",
  "backend openapi",
);
assertQueryParameterStringConstraints(
  backendOpenapi,
  "/backend/v3/api/drive/audit_events",
  "get",
  "resourceId",
  128,
  "^[A-Za-z0-9._:@-]+$",
  "backend openapi",
);
assertQueryParameterStringConstraints(
  backendOpenapi,
  "/backend/v3/api/drive/audit_events",
  "get",
  "correlationId",
  64,
  "^[A-Za-z0-9._:@-]+$",
  "backend openapi",
);
assertQueryParameterStringConstraints(
  backendOpenapi,
  "/backend/v3/api/drive/audit_events",
  "get",
  "traceId",
  128,
  "^[A-Za-z0-9._:@-]+$",
  "backend openapi",
);
for (const pathKey of [
  "/app/v3/api/drive/spaces/{spaceId}/nodes",
  "/app/v3/api/drive/trash",
  "/app/v3/api/drive/recent",
  "/app/v3/api/drive/shared_with_me",
  "/app/v3/api/drive/favorites",
  "/app/v3/api/drive/search",
  "/app/v3/api/drive/changes",
  "/app/v3/api/drive/nodes/{nodeId}/permissions",
  "/app/v3/api/drive/nodes/{nodeId}/permissions/effective",
  "/app/v3/api/drive/nodes/{nodeId}/properties",
  "/app/v3/api/drive/nodes/{nodeId}/labels",
  "/app/v3/api/drive/watch_channels",
  "/app/v3/api/drive/nodes/{nodeId}/share_links",
  "/app/v3/api/drive/nodes/{nodeId}/versions",
  "/app/v3/api/drive/nodes/{nodeId}/comments",
  "/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies",
]) {
  assertQueryParameterExists(appOpenapi, pathKey, "get", "page_size", "app openapi");
  assertQueryParameterExists(appOpenapi, pathKey, "get", "cursor", "app openapi");
}
for (const [pathKey, label] of [
  ["/app/v3/api/drive/nodes/{nodeId}", "app openapi"],
  ["/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}", "app openapi"],
  ["/app/v3/api/drive/nodes/{nodeId}/comments/{commentId}/replies/{replyId}", "app openapi"],
  ["/app/v3/api/drive/nodes/{nodeId}/favorite", "app openapi"],
  ["/app/v3/api/drive/nodes/{nodeId}/labels/{labelId}", "app openapi"],
  ["/app/v3/api/drive/nodes/{nodeId}/permissions/{permissionId}", "app openapi"],
  ["/app/v3/api/drive/nodes/{nodeId}/properties/{propertyKey}", "app openapi"],
  ["/app/v3/api/drive/nodes/{nodeId}/versions/{versionId}", "app openapi"],
  ["/app/v3/api/drive/share_links/{shareLinkId}", "app openapi"],
  ["/app/v3/api/drive/spaces/{spaceId}", "app openapi"],
  ["/app/v3/api/assets/collections/{collectionId}/items/{itemId}", "app openapi"],
  ["/app/v3/api/assets/{assetId}/relations/{relationId}", "app openapi"],
]) {
  assertNoContentResponse(appOpenapi, pathKey, "delete", label);
}
for (const pathKey of [
  "/backend/v3/api/drive/labels/{labelId}",
  "/backend/v3/api/drive/sandbox_volumes/{sandboxId}/grants/{grantId}",
]) {
  assertNoContentResponse(backendOpenapi, pathKey, "delete", "backend openapi");
}
for (const pathKey of [
  "/backend/v3/api/drive/storage/bindings/default",
  "/backend/v3/api/drive/storage/providers/{providerId}",
  "/backend/v3/api/drive/storage/providers/{providerId}/bucket",
  "/backend/v3/api/drive/storage/providers/{providerId}/objects/{objectKey}",
]) {
  assertNoContentResponse(adminStorageOpenapi, pathKey, "delete", "admin storage openapi");
}
for (const schemaName of [
  "ArchiveEntryListResponse",
  "AssetCollectionPage",
  "AssetPage",
  "ChangeListResponse",
  "CommentListResponse",
  "CommentReplyListResponse",
  "DeleteAssetCollectionItemHttpResponse",
  "DeleteAssetCollectionItemResponse",
  "DeleteAssetRelationHttpResponse",
  "DeleteAssetRelationResponse",
  "DeleteNodePropertyHttpResponse",
  "DeleteNodePropertyResponse",
  "DeleteNodeResponse",
  "DeleteSpaceHttpResponse",
  "DeleteSpaceResponse",
  "DeleteVersionHttpResponse",
  "DeleteVersionResponse",
  "DeletedCommandHttpResponse",
  "DriveWatchChannelListResponse",
  "EffectivePermissionListResponse",
  "ListSpacesResponse",
  "NodeLabelListResponse",
  "NodeListResponse",
  "NodePropertyListResponse",
  "PermissionListResponse",
  "RemoveNodeLabelResponse",
  "RevokeShareLinkResponse",
  "SdkWorkListResponse",
  "ShareLinkListResponse",
  "VersionListResponse",
]) {
  assertSchemaMissing(appOpenapi, schemaName, "app openapi");
}
for (const schemaName of [
  "AuditEventPage",
  "AuditEventPageSdkWorkEnvelope",
  "DeleteLabelResponse",
  "DownloadPackagePage",
  "DownloadPackagePageSdkWorkEnvelope",
  "LabelListResponse",
  "LabelListResponseSdkWorkEnvelope",
  "ListSpacesResponse",
  "ListSpacesResponseSdkWorkEnvelope",
  "MaintenanceJobPage",
  "MaintenanceJobPageSdkWorkEnvelope",
  "SdkWorkListResponse",
]) {
  assertSchemaMissing(backendOpenapi, schemaName, "backend openapi");
}
for (const schemaName of [
  "DeleteStorageProviderBindingResponse",
  "DeleteStorageProviderResponse",
  "ListStorageProvidersResponse",
  "ProviderBucketList",
  "ProviderObjectList",
  "SdkWorkListResponse",
  "StorageProviderBindingListResponse",
]) {
  assertSchemaMissing(adminStorageOpenapi, schemaName, "admin storage openapi");
}
const openOperationIds = collectOperationIds(openOpenapi, "open openapi");
const appOperationIds = collectOperationIds(appOpenapi, "app openapi");
const backendOperationIds = collectOperationIds(backendOpenapi, "backend openapi");
const adminStorageOperationIds = collectOperationIds(
  adminStorageOpenapi,
  "admin storage openapi",
);
assertUniqueOperationIds(openOperationIds, "open openapi");
assertUniqueOperationIds(appOperationIds, "app openapi");
assertUniqueOperationIds(backendOperationIds, "backend openapi");
assertUniqueOperationIds(adminStorageOperationIds, "admin storage openapi");
assertOperationIdsInclude(
  appOperationIds,
  [
    "spaces.list",
    "spaces.create",
    "spaces.retrieve",
    "spaces.update",
    "spaces.delete",
    "nodes.files.create",
    "nodes.shortcuts.create",
    "nodes.retrieve",
    "nodes.path.retrieve",
    "nodes.capabilities.list",
    "nodeProperties.list",
    "nodeProperties.update",
    "nodeProperties.delete",
    "nodeLabels.list",
    "nodeLabels.update",
    "nodeLabels.delete",
    "changes.watch",
    "nodes.watch",
    "watchChannels.list",
    "watchChannels.retrieve",
    "watchChannels.stop",
    "nodes.move",
    "nodes.copy",
    "nodes.delete",
    "nodes.downloadUrls.retrieve",
    "trash.empty",
    "trash.list",
    "recent.list",
    "sharedWithMe.list",
    "favorites.list",
    "favorites.update",
    "favorites.delete",
    "uploadSessions.retrieve",
    "uploadSessions.parts.update",
    "uploadSessions.complete",
    "uploadSessions.abort",
    "permissions.retrieve",
    "permissions.update",
    "permissions.effective.list",
    "shareLinks.retrieve",
    "shareLinks.list",
    "shareLinks.update",
    "comments.list",
    "comments.create",
    "comments.retrieve",
    "comments.update",
    "comments.delete",
    "changes.startPageToken.retrieve",
    "commentReplies.list",
    "commentReplies.create",
    "commentReplies.retrieve",
    "commentReplies.update",
    "commentReplies.delete",
    "versions.retrieve",
    "versions.delete",
    "assets.list",
    "assets.create",
    "assets.retrieve",
    "assets.update",
    "assets.archive",
    "assets.restore",
    "assetCollections.list",
    "assetCollections.create",
    "assetCollectionItems.create",
    "assetCollectionItems.delete",
    "assetRelations.create",
    "assetRelations.delete",
  ],
  "app openapi",
);
assertOperationIdsExclude(
  appOperationIds,
  [
    "storageProviders.list",
    "storageProviders.create",
    "storageProviders.retrieve",
    "storageProviders.update",
    "storageProviders.delete",
    "storageProviders.test",
    "storageProviders.capabilities.list",
    "storageProviders.activate",
    "storageProviders.deactivate",
    "storageProviders.credentials.rotate",
    "storageProviders.bucket.retrieve",
    "storageProviders.bucket.update",
    "storageProviders.bucket.delete",
    "storageProviders.objects.list",
    "storageProviders.objects.retrieve",
    "storageProviders.objects.delete",
    "storageProviders.objects.copy",
    "storageProviderBindings.default.retrieve",
    "storageProviderBindings.default.update",
  ],
  "app openapi",
);
assertOperationIdsExclude(
  appOperationIds,
  APPBASE_APP_FORBIDDEN_OPERATION_IDS,
  "app openapi",
);
assertOperationIdsInclude(
  backendOperationIds,
  [
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
    "sandboxVolumes.list",
    "sandboxVolumes.create",
    "sandboxVolumes.retrieve",
    "sandboxVolumes.update",
    "sandboxGrants.list",
    "sandboxGrants.create",
    "sandboxGrants.update",
    "sandboxGrants.delete",
  ],
  "backend openapi",
);
assertOperationIdsExclude(
  backendOperationIds,
  [
    "storageProviders.list",
    "storageProviders.retrieve",
    "storageProviderBindings.default.retrieve",
  ],
  "backend openapi",
);
assertOperationIdsInclude(
  adminStorageOperationIds,
  [
    "storageProviders.list",
    "storageProviders.create",
    "storageProviders.retrieve",
    "storageProviders.update",
    "storageProviders.delete",
    "storageProviders.test",
    "storageProviders.capabilities.list",
    "storageProviders.activate",
    "storageProviders.deactivate",
    "storageProviders.credentials.rotate",
    "storageProviders.buckets.list",
    "storageProviders.bucket.retrieve",
    "storageProviders.bucket.update",
    "storageProviders.bucket.delete",
    "storageProviders.objects.list",
    "storageProviders.objects.retrieve",
    "storageProviders.objects.delete",
    "storageProviders.objects.copy",
    "storageProviderBindings.default.retrieve",
    "storageProviderBindings.default.update",
  ],
  "admin storage openapi",
);

assertContains(
  specialSpacesSchema,
  "dr_drive_space_knowledge_profile",
  "special spaces schema",
);
assertContains(
  specialSpacesSchema,
  "dr_drive_space_ai_generation_profile",
  "special spaces schema",
);
assertContains(
  specialSpacesSchema,
  "dr_drive_space_app_upload_profile",
  "special spaces schema",
);
assertContains(
  specialSpacesSchema,
  "dr_drive_space_rtc_profile",
  "special spaces schema",
);
for (const indexName of [
  "ix_dr_drive_node_permission_resource",
  "ix_dr_drive_node_permission_subject",
  "ux_dr_drive_node_share_link_token_hash",
  "ix_dr_drive_node_share_link_resource",
  "ix_dr_drive_node_comment_node",
  "ix_dr_drive_node_comment_resolved",
  "ix_dr_drive_node_comment_reply_comment",
  "ix_dr_drive_node_comment_reply_node",
  "ux_dr_drive_node_favorite_subject_node",
  "ix_dr_drive_node_favorite_subject",
  "ux_dr_drive_label_key",
  "ix_dr_drive_label_tenant_status",
  "ux_dr_drive_node_label_node_label",
  "ix_dr_drive_node_label_node",
  "ix_dr_drive_node_label_label",
  "ix_dr_drive_watch_channel_tenant_status",
  "ix_dr_drive_watch_channel_resource",
  "ix_dr_drive_watch_channel_node",
  "ix_dr_drive_watch_channel_expires",
  "ux_dr_drive_change_log_space_sequence",
  "ix_dr_drive_change_log_tenant_space_created",
  "ix_dr_drive_domain_outbox_pending",
  "ix_dr_drive_audit_event_tenant_created",
  "ix_dr_drive_audit_event_resource",
  "ix_dr_drive_audit_event_action_created",
  "ix_dr_drive_audit_event_request_created",
  "ix_dr_drive_audit_event_trace_created",
]) {
  assertContains(securityAuditSchema, indexName, "security audit schema");
}
for (const marker of [
  "dr_drive_node_permission",
  "dr_drive_node_share_link",
  "dr_drive_node_comment",
  "dr_drive_node_comment_reply",
  "dr_drive_node_favorite",
  "dr_drive_label",
  "dr_drive_node_label",
  "dr_drive_watch_channel",
  "dr_drive_change_log",
  "dr_drive_domain_outbox",
  "node_id",
  "comment_id",
  "subject_id",
  "subject_type",
  "label_key",
  "display_name",
  "label_id",
  "color",
  "resource_type",
  "resource_id",
  "channel_type",
  "address",
  "expiration_epoch_ms",
  "anchor",
  "resolved",
  "token_hash",
  "algorithm: sha256_hex",
  "expires_at_epoch_ms",
  "download_limit",
  "download_count",
  "sequence_no",
  "event_type",
  "actor_id",
  "lifecycle_status",
]) {
  assertContains(securityAuditSchema, marker, "security audit schema");
}
assertContains(storageSchema, "dr_drive_storage_provider", "storage schema");
assertContains(storageSchema, "dr_drive_storage_provider_binding", "storage schema");
assertContains(storageSchema, "provider_kind", "storage schema");
for (const fieldName of [
  "name",
  "region",
  "path_style",
  "strict_tls",
  "credential_ref",
  "server_side_encryption_mode",
  "default_storage_class",
  "storage_root_prefix",
]) {
  assertContains(storageSchema, fieldName, "storage schema");
}
for (const marker of [
  STORAGE_CREDENTIAL_REF_PATTERN,
  "plain:<accessKeyId>:<secretAccessKey>[:<sessionToken>]",
  "env:<accessKeyEnv>:<secretKeyEnv>[:<sessionTokenEnv>]",
  "SDKWORK_DRIVE_STORAGE_CREDENTIAL__<sanitized_ref>__ACCESS_KEY_ID",
  "must be normalized before adapter construction",
  "object-store adapter input must be normalized before adapter construction",
]) {
  assertContains(storageSchema, marker, "storage schema");
}
for (const marker of [
  "dr_drive_storage_object",
  "node_id",
  "version_no",
  "content_type",
  "content_length",
  "checksum_sha256_hex",
  "lifecycle_status",
  "ux_dr_drive_storage_object_node_version",
  "ux_dr_drive_storage_object_active_locator",
  "ix_dr_drive_storage_object_node_latest",
  "dr_drive_upload_session",
  "bucket",
  "expires_at_epoch_ms",
  "created_by",
  "updated_by",
  "ux_dr_drive_upload_session_idempotency",
  "ix_dr_drive_upload_session_expires",
  "ix_dr_drive_storage_provider_binding_lookup",
  "ix_dr_drive_storage_provider_binding_provider",
]) {
  assertContains(storageSchema, marker, "storage schema");
}
assertNotContains(
  tableBlock(storageSchema, "dr_drive_storage_object", "storage schema"),
  "- name: size_bytes",
  "storage schema dr_drive_storage_object",
);
assertNotContains(
  tableBlock(storageSchema, "dr_drive_upload_session", "storage schema"),
  "- name: expires_at\n",
  "storage schema dr_drive_upload_session",
);
assertContains(
  storageSchema,
  "local_filesystem, s3_compatible, google_cloud_storage, aliyun_oss, tencent_cos, huawei_obs, volcengine_tos",
  "storage schema",
);
assertContains(storageSchema, STORAGE_PROVIDER_KIND_PATTERN, "storage schema");

for (const marker of [
  "canonical_asset_table: dr_drive_node",
  "asset_id_alias: drive_node_id",
  "canonical_storage_table: dr_drive_storage_object",
  "ix_dr_drive_node_asset_list",
  "ix_dr_drive_node_asset_scene_source",
  "ix_dr_drive_storage_object_node_latest",
  "sourceType",
  "sourceDomain",
  "assetId: dr_drive_node.id",
  "driveNodeId: dr_drive_node.id",
  "resourceSnapshot",
]) {
  assertContains(globalAssetsSchema, marker, "global assets schema");
}
for (const marker of [
  "table: dr_asset_item",
  "table: dr_asset_resource_ref",
  "table: dr_asset_version",
  "table: dr_asset_relation",
  "table: dr_asset_collection",
  "table: dr_asset_collection_item",
  "table: dr_asset_event",
  "table: dr_asset_projection",
  "bucket_name",
  "object_key",
  "presigned_url",
  "asset_upload_session",
]) {
  assertNotContains(globalAssetsSchema, marker, "global assets schema");
}

for (const [schemaName, schema] of Object.entries(appOpenapi.components?.schemas ?? {})) {
  if (!schemaName.endsWith("Request")) {
    continue;
  }
  assertSchemaPropertyAbsent(appOpenapi, schemaName, "tenantId", "app openapi");
}

for (const [pathKey, pathItem] of Object.entries(appOpenapi.paths ?? {})) {
  for (const [method, operation] of Object.entries(pathItem ?? {})) {
    if (!operation || typeof operation !== "object" || !Array.isArray(operation.parameters)) {
      continue;
    }
    for (const parameter of operation.parameters) {
      if (parameter?.in === "query" && parameter?.name === "tenantId") {
        throw new Error(
          `[drive_schema_quality_gate] app openapi ${method.toUpperCase()} ${pathKey} must not accept client tenantId query params`,
        );
      }
    }
  }
}

process.stdout.write("[drive_schema_quality_gate] passed\n");
