#!/usr/bin/env node
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse as parseYaml, stringify as stringifyYaml } from "yaml";

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

const OFFICIAL_LANGUAGE_ORDER = ["typescript", "rust", "java", "python", "go"];
const DEFAULT_LANGUAGE = "typescript";
const FIXED_SDK_VERSION = "0.1.0";
const STANDARD_PROFILE = "sdkwork-v3";

function readOpenApiDocument(filePath) {
  const source = readFileSync(filePath, "utf8");
  if (/\.ya?ml$/iu.test(filePath)) {
    return parseYaml(source);
  }
  return JSON.parse(source);
}

function writeOpenApiDocument(filePath, document) {
  const source = /\.ya?ml$/iu.test(filePath)
    ? stringifyYaml(document, { lineWidth: 0 })
    : `${JSON.stringify(document, null, 2)}\n`;
  writeFileSync(filePath, source.endsWith("\n") ? source : `${source}\n`, "utf8");
}

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(scriptDir, "..");
const STANDARD_SDK_GENERATOR_ROOT = path.resolve(
  workspaceRoot,
  "..",
  "sdkwork-sdk-generator",
);
const STANDARD_SDK_GENERATOR_BIN = path.join(
  STANDARD_SDK_GENERATOR_ROOT,
  "bin",
  "sdkgen.js",
);

function fail(sdkName, message) {
  process.stderr.write(`[${sdkName}] ${message}\n`);
  process.exit(1);
}

function parseLanguages(raw, sdkName) {
  const values = raw.flatMap((item) => String(item || "").split(","));
  const normalized = [];
  for (const value of values) {
    const language = value.trim().toLowerCase();
    if (!language) {
      continue;
    }
    if (!OFFICIAL_LANGUAGE_ORDER.includes(language)) {
      fail(sdkName, `unsupported language: ${language}`);
    }
    if (!normalized.includes(language)) {
      normalized.push(language);
    }
  }
  return OFFICIAL_LANGUAGE_ORDER.filter((language) => normalized.includes(language));
}

function parseArgs(argv, defaultBaseUrl, sdkName) {
  const result = {
    allLanguages: false,
    languages: [],
    baseUrl: defaultBaseUrl,
    input: null,
    passthrough: [],
  };

  for (let index = 0; index < argv.length; index += 1) {
    const current = argv[index];
    if (current === "--all-languages") {
      result.allLanguages = true;
      continue;
    }
    if (current === "--language") {
      result.languages.push(argv[index + 1] || "");
      index += 1;
      continue;
    }
    if (current.startsWith("--language=")) {
      result.languages.push(current.slice("--language=".length));
      continue;
    }
    if (current === "--base-url") {
      result.baseUrl = argv[index + 1] || defaultBaseUrl;
      index += 1;
      continue;
    }
    if (current === "--input") {
      result.input = argv[index + 1] || "";
      index += 1;
      continue;
    }
    if (current.startsWith("--input=")) {
      result.input = current.slice("--input=".length);
      continue;
    }
    if (current === "--") {
      result.passthrough.push(...argv.slice(index + 1));
      break;
    }
    result.passthrough.push(current);
  }

  if (!result.baseUrl || !String(result.baseUrl).trim()) {
    fail(sdkName, "base URL cannot be empty");
  }

  return result;
}

function executableFileExists(candidate) {
  return candidate && existsSync(candidate);
}

function configuredGeneratorInvocation(configuredPath) {
  if (!configuredPath) {
    return null;
  }
  const trimmed = configuredPath.trim();
  if (!trimmed) {
    return null;
  }
  if (trimmed.endsWith(".js") || trimmed.endsWith(".mjs") || trimmed.endsWith(".cjs")) {
    return {
      command: "node",
      prefixArgs: [trimmed],
      shell: false,
      generatorName: "sdkwork-sdk-generator",
      official: true,
    };
  }
  return {
    command: trimmed,
    prefixArgs: [],
    shell: process.platform === "win32",
    generatorName: "sdkwork-sdk-generator",
    official: true,
  };
}

function isOfficialGeneratorPath(candidate) {
  const normalized = path.resolve(candidate).toLowerCase();
  const standardRoot = path.resolve(STANDARD_SDK_GENERATOR_ROOT).toLowerCase();
  return normalized.startsWith(`${standardRoot}${path.sep}`) || normalized === standardRoot;
}

export function resolveSdkGeneratorInvocation() {
  const configured = configuredGeneratorInvocation(process.env.SDKWORK_SDK_GENERATOR_BIN);
  if (configured) {
    const configuredBin = configured.prefixArgs[0] || configured.command;
    if (!isOfficialGeneratorPath(configuredBin)) {
      throw new Error(
        `SDKWORK_SDK_GENERATOR_BIN must point to ${STANDARD_SDK_GENERATOR_ROOT}; received ${configuredBin}`,
      );
    }
    return configured;
  }

  if (executableFileExists(STANDARD_SDK_GENERATOR_BIN)) {
    return {
      command: "node",
      prefixArgs: [STANDARD_SDK_GENERATOR_BIN],
      shell: false,
      generatorName: "sdkwork-sdk-generator",
      official: true,
    };
  }

  throw new Error(
    `standard SDK generator not found: ${STANDARD_SDK_GENERATOR_BIN}. Drive SDK family generation must use ${STANDARD_SDK_GENERATOR_ROOT}.`,
  );
}

function collectOperations(openapiDocument) {
  const operations = [];
  for (const [pathKey, pathItem] of Object.entries(openapiDocument.paths || {})) {
    if (!pathItem || typeof pathItem !== "object") {
      continue;
    }
    for (const [methodName, operation] of Object.entries(pathItem)) {
      if (!HTTP_METHODS.has(methodName)) {
        continue;
      }
      operations.push({
        operationId: operation?.operationId
          ? String(operation.operationId)
          : `${methodName}.${pathKey}`,
        method: methodName.toUpperCase(),
        path: pathKey,
      });
    }
  }
  operations.sort((left, right) => left.operationId.localeCompare(right.operationId));
  return operations;
}

function cloneJson(value) {
  return JSON.parse(JSON.stringify(value));
}

function isDependencyComposedOperation(operation) {
  return Boolean(
    operation?.["x-sdkwork-composed-from-owner"] ||
      operation?.["x-sdkwork-composed-from-api-authority"],
  );
}

function hasHttpOperation(pathItem) {
  return Object.entries(pathItem || {}).some(([methodName]) => HTTP_METHODS.has(methodName));
}

function localComponentRef(value) {
  if (typeof value !== "string" || !value.startsWith("#/components/")) {
    return null;
  }
  const parts = value.slice("#/components/".length).split("/");
  if (parts.length < 2) {
    return null;
  }
  return {
    kind: parts[0],
    name: parts.slice(1).join("/"),
  };
}

function componentRefKey(ref) {
  return `${ref.kind}/${ref.name}`;
}

function collectLocalComponentRefs(value, refs = []) {
  if (!value || typeof value !== "object") {
    return refs;
  }
  if (typeof value.$ref === "string") {
    const ref = localComponentRef(value.$ref);
    if (ref) {
      refs.push(ref);
    }
  }
  if (Array.isArray(value)) {
    for (const item of value) {
      collectLocalComponentRefs(item, refs);
    }
    return refs;
  }
  for (const nested of Object.values(value)) {
    collectLocalComponentRefs(nested, refs);
  }
  return refs;
}

function collectReachableComponentRefs(openapiDocument) {
  const queue = [];
  for (const pathItem of Object.values(openapiDocument.paths || {})) {
    collectLocalComponentRefs(pathItem, queue);
  }

  const reachable = new Set();
  for (let index = 0; index < queue.length; index += 1) {
    const ref = queue[index];
    const key = componentRefKey(ref);
    if (reachable.has(key)) {
      continue;
    }
    reachable.add(key);
    const component = openapiDocument.components?.[ref.kind]?.[ref.name];
    if (component) {
      collectLocalComponentRefs(component, queue);
    }
  }
  return reachable;
}

const OPENAPI_ENVELOPE_SCHEMA_NAMES = [
  "ProblemDetail",
  "SdkWorkApiResponse",
  "SdkWorkResourceData",
  "SdkWorkPageData",
  "SdkWorkCommandData",
  "SdkWorkPlatformErrorCode",
  "SdkWorkResourceResponse",
  "SdkWorkListResponse",
  "SdkWorkCommandResponse",
  "PageInfo",
];

function preserveOpenApiEnvelopeSchemas(sourceDocument, ownerDocument) {
  const sourceSchemas = sourceDocument.components?.schemas;
  if (!sourceSchemas || typeof sourceSchemas !== "object") {
    return;
  }
  ownerDocument.components = ownerDocument.components || {};
  ownerDocument.components.schemas = ownerDocument.components.schemas || {};
  for (const schemaName of OPENAPI_ENVELOPE_SCHEMA_NAMES) {
    if (sourceSchemas[schemaName]) {
      ownerDocument.components.schemas[schemaName] = cloneJson(sourceSchemas[schemaName]);
    }
  }
}

function pruneUnusedSchemas(openapiDocument) {
  const schemas = openapiDocument.components?.schemas;
  if (!schemas || typeof schemas !== "object") {
    return;
  }
  const reachableRefs = collectReachableComponentRefs(openapiDocument);
  for (const schemaName of Object.keys(schemas)) {
    if (!reachableRefs.has(componentRefKey({ kind: "schemas", name: schemaName }))) {
      delete schemas[schemaName];
    }
  }
}

function collectUsedOperationTags(openapiDocument) {
  const tags = new Set();
  for (const pathItem of Object.values(openapiDocument.paths || {})) {
    if (!pathItem || typeof pathItem !== "object") {
      continue;
    }
    for (const [methodName, operation] of Object.entries(pathItem)) {
      if (!HTTP_METHODS.has(methodName) || !operation || typeof operation !== "object") {
        continue;
      }
      for (const tagName of operation.tags || []) {
        if (tagName) {
          tags.add(String(tagName));
        }
      }
    }
  }
  return tags;
}

function pruneUnusedTags(openapiDocument) {
  if (!Array.isArray(openapiDocument.tags)) {
    return;
  }
  const usedTags = collectUsedOperationTags(openapiDocument);
  openapiDocument.tags = openapiDocument.tags.filter((tag) =>
    usedTags.has(String(tag?.name || "")),
  );
}

export function toOwnerOnlyOpenApi(openapiDocument) {
  const ownerOnlyDocument = cloneJson(openapiDocument);
  preserveOpenApiEnvelopeSchemas(openapiDocument, ownerOnlyDocument);
  for (const [pathKey, pathItem] of Object.entries(ownerOnlyDocument.paths || {})) {
    if (!pathItem || typeof pathItem !== "object") {
      continue;
    }
    for (const [methodName, operation] of Object.entries(pathItem)) {
      if (!HTTP_METHODS.has(methodName)) {
        continue;
      }
      if (isDependencyComposedOperation(operation)) {
        delete pathItem[methodName];
      }
    }
    if (!hasHttpOperation(pathItem)) {
      delete ownerOnlyDocument.paths[pathKey];
    }
  }
  pruneUnusedTags(ownerOnlyDocument);
  pruneUnusedSchemas(ownerOnlyDocument);
  preserveOpenApiEnvelopeSchemas(openapiDocument, ownerOnlyDocument);
  return ownerOnlyDocument;
}

function writeOwnerOnlyOpenApiInput({ sourceOpenapiPath, sdkName, family }) {
  const openapiDocument = readOpenApiDocument(sourceOpenapiPath);
  const ownerOnlyDocument = toOwnerOnlyOpenApi(openapiDocument);
  const outputDir = family.generationInputFile
    ? path.join(family.sdkRoot, "openapi")
    : path.join(workspaceRoot, "target", "drive-sdk-generator-input", sdkName);
  mkdirSync(outputDir, { recursive: true });
  const outputPath = path.join(
    outputDir,
    family.generationInputFile || path.basename(sourceOpenapiPath),
  );
  writeOpenApiDocument(outputPath, ownerOnlyDocument);
  return outputPath;
}

function syncAuthorityMirror(family, sourceOpenapiPath) {
  if (!family.authorityMirrorFile) {
    return sourceOpenapiPath;
  }
  const outputDir = path.join(family.sdkRoot, "openapi");
  const outputPath = path.join(outputDir, family.authorityMirrorFile);
  mkdirSync(outputDir, { recursive: true });
  writeOpenApiDocument(outputPath, readOpenApiDocument(sourceOpenapiPath));
  return outputPath;
}

function writeTypeScriptComposedOperations(family, manifest, operations) {
  const composedRoot = path.join(
    family.sdkRoot,
    `${family.sdkName}-typescript`,
    "composed",
  );
  const operationLines = operations
    .map(
      (item) =>
        `  "${item.operationId}": { method: "${item.method}", path: "${item.path}" },`,
    )
    .join("\n");
  const source = `// Generated by tools/drive_sdk_generator_runner.mjs from the owner-only Drive OpenAPI input.
// This file is outside generated/server-openapi so SDK ownership metadata does not pollute sdkgen output.

export const sdkMetadata = {
  name: "${manifest.sdkName}",
  packageName: "${manifest.generatedPackages.typescript.packageName}",
  sdkOwner: "${manifest.sdkOwner}",
  apiAuthority: "${manifest.apiAuthority}",
  language: "typescript",
  standardProfile: "${manifest.standardProfile}",
  baseUrl: "${manifest.baseUrl}",
  apiPrefix: "${manifest.apiPrefix}",
  sdkDependencies: ${JSON.stringify(manifest.sdkDependencies ?? [])},
};

export const operations = {
${operationLines}
} as const;
`;

  mkdirSync(composedRoot, { recursive: true });
  writeFileSync(path.join(composedRoot, "operations.ts"), source, "utf8");
}

function writeDriveFamilyMetadata({
  openapiPath,
  family,
  generatorName,
  baseUrl,
}) {
  const openapiDocument = readOpenApiDocument(openapiPath);
  const operations = collectOperations(openapiDocument);
  const relativeOpenapiPath = toPosixPath(path.relative(family.sdkRoot, openapiPath));
  const generatedPackages = Object.fromEntries(OFFICIAL_LANGUAGE_ORDER.map((language) => [
    language,
    {
      language,
      packageName: `${family.sdkName}-generated-${language}`,
      generatedOutput: `${family.sdkName}-${language}/generated/server-openapi`,
    },
  ]));
  const manifestPath = path.join(family.sdkRoot, "sdk-manifest.json");
  const currentManifest = existsSync(manifestPath)
    ? JSON.parse(readFileSync(manifestPath, "utf8"))
    : {};
  const manifest = {
    ...currentManifest,
    schemaVersion: 1,
    sdkName: family.sdkName,
    sdkOwner: family.sdkOwner,
    apiAuthority: family.apiAuthority,
    sdkFamily: family.sdkName,
    sdkType: family.sdkType,
    ...(family.sdkSurface ? { sdkSurface: family.sdkSurface } : {}),
    apiPrefix: family.apiPrefix,
    generationInputSpec: relativeOpenapiPath,
    generatedPackages,
    sdkDependencies: family.sdkDependencies || [],
    generatorName,
    baseUrl,
    standardProfile: family.manifestStandardProfile,
    fixedSdkVersion: FIXED_SDK_VERSION,
    ownerOnlyOperationCount: operations.length,
    managedBy: "tools/drive_sdk_generator_runner.mjs",
  };

  mkdirSync(family.sdkRoot, { recursive: true });
  writeFileSync(
    manifestPath,
    `${JSON.stringify(manifest, null, 2)}\n`,
    "utf8",
  );

  writeTypeScriptComposedOperations(family, manifest, operations);
}

function removeStaleGeneratedTrackingFiles(outputPath) {
  for (const fileName of ["sdk-manifest.json", "source-openapi.json"]) {
    const filePath = path.join(outputPath, fileName);
    if (existsSync(filePath)) {
      rmSync(filePath, { force: true });
    }
  }
}

function writeGeneratedSourceOpenApi({ openapiPath, outputPath }) {
  const openapiDocument = readOpenApiDocument(openapiPath);
  writeFileSync(
    path.join(outputPath, "source-openapi.json"),
    `${JSON.stringify(openapiDocument, null, 2)}\n`,
    "utf8",
  );
}

function toPosixPath(value) {
  return value.replace(/\\/g, "/");
}

function defaultLanguageAssemblyEntries(family) {
  return OFFICIAL_LANGUAGE_ORDER.map((language) => ({
    language,
    workspace: `${family.sdkName}-${language}`,
    generationState: "declared",
    releaseState: "not_published",
    generatedPath: `${family.sdkName}-${language}/generated/server-openapi`,
    manifestPath: `${family.sdkName}-${language}/generated/server-openapi/${languageManifestFile(language)}`,
    name: languagePackageCoordinate(family, language),
    version: FIXED_SDK_VERSION,
    description: `Generator-owned ${languageDisplayName(language)} transport SDK for ${family.sdkName}.`,
  }));
}

function languageDisplayName(language) {
  return {
    typescript: "TypeScript",
    rust: "Rust",
    java: "Java",
    python: "Python",
    go: "Go",
  }[language] || language;
}

function languageManifestFile(language) {
  return {
    typescript: "package.json",
    rust: "Cargo.toml",
    java: "pom.xml",
    python: "pyproject.toml",
    go: "go.mod",
  }[language] || "sdkwork-sdk.json";
}

function languagePackageCoordinate(family, language) {
  const base = `${family.sdkName}-generated`;
  if (language === "typescript") {
    return `@sdkwork-internal/${base}`;
  }
  if (language === "java") {
    return `com.sdkwork:${base}`;
  }
  if (language === "go") {
    return `github.com/sdkwork/${base}`;
  }
  return base;
}

function syncLanguageManifestEntries(family, existingLanguages) {
  const existingByLanguage = new Map(
    Array.isArray(existingLanguages)
      ? existingLanguages
          .filter((entry) => entry && typeof entry === "object" && entry.language)
          .map((entry) => [String(entry.language), entry])
      : [],
  );
  return defaultLanguageAssemblyEntries(family).map((defaults) => ({
    ...defaults,
    ...(existingByLanguage.get(defaults.language) || {}),
  }));
}

function syncDriveFamilyManifest(family, authorityOpenapiPath, generationInputPath) {
  const manifestPath = path.join(family.sdkRoot, "sdk-manifest.json");
  let manifest = {};
  if (existsSync(manifestPath)) {
    manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  }

  const relativeAuthorityPath = toPosixPath(
    path.relative(family.sdkRoot, authorityOpenapiPath),
  );
  const relativeGenerationPath = toPosixPath(
    path.relative(family.sdkRoot, generationInputPath),
  );
  manifest.schemaVersion = manifest.schemaVersion || 1;
  manifest.workspace = manifest.workspace || family.sdkName;
  manifest.sdkFamily = manifest.sdkFamily || family.sdkName;
  manifest.sdkName = manifest.sdkName || family.sdkName;
  manifest.sdkOwner = family.sdkOwner;
  manifest.apiAuthority = family.apiAuthority;
  if (family.sdkSurface) {
    manifest.sdkSurface = family.sdkSurface;
  }
  manifest.authoritySpec = relativeAuthorityPath;
  manifest.generationInputSpec = relativeGenerationPath;
  manifest.derivedSpecs = {
    ...(manifest.derivedSpecs || {}),
    default: relativeGenerationPath,
  };
  manifest.discoverySurface = {
    ...(manifest.discoverySurface || {}),
    sdkTarget: family.sdkSurface || family.sdkType,
    apiPrefix: family.apiPrefix,
    generatedProtocols: ["http-openapi"],
    manualTransports: [],
  };
  manifest.languages = syncLanguageManifestEntries(family, manifest.languages);
  manifest.sdkDependencies = family.sdkDependencies || [];

  writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
}

function normalizeGeneratedSdkMetadata(outputPath, family, language, packageName) {
  const sdkMetadataPath = path.join(outputPath, "sdkwork-sdk.json");
  if (!existsSync(sdkMetadataPath)) {
    return;
  }
  const metadata = JSON.parse(readFileSync(sdkMetadataPath, "utf8"));
  metadata.name = family.sdkName;
  metadata.language = language;
  metadata.sdkType = family.sdkType;
  metadata.packageName = packageName;
  metadata.transportPackageName = packageName;
  writeFileSync(sdkMetadataPath, `${JSON.stringify(metadata, null, 2)}\n`, "utf8");
}

export function runDriveSdkGenerator(family, argv) {
  const sdkName = family.sdkName;
  const sdkRoot = family.sdkRoot;
  const args = parseArgs(argv, family.defaultBaseUrl, sdkName);
  const openapiPath = args.input
    ? path.isAbsolute(args.input)
      ? args.input
      : path.resolve(workspaceRoot, args.input)
    : resolveDefaultOpenApiPath(family);

  if (!existsSync(openapiPath)) {
    fail(sdkName, `openapi file not found: ${openapiPath}`);
  }
  const authorityOpenapiPath = syncAuthorityMirror(family, openapiPath);
  const ownerOnlyOpenapiPath = writeOwnerOnlyOpenApiInput({
    sourceOpenapiPath: authorityOpenapiPath,
    sdkName,
    family,
  });
  syncDriveFamilyManifest(family, authorityOpenapiPath, ownerOnlyOpenapiPath);

  const languages = args.allLanguages
    ? OFFICIAL_LANGUAGE_ORDER
    : parseLanguages(args.languages.length > 0 ? args.languages : [DEFAULT_LANGUAGE], sdkName);
  let generator;
  try {
    generator = resolveSdkGeneratorInvocation();
  } catch (error) {
    fail(sdkName, error instanceof Error ? error.message : String(error));
  }

  for (const language of languages) {
    const outputPath = path.join(
      sdkRoot,
      `${sdkName}-${language}`,
      "generated",
      "server-openapi",
    );
    const packageName = `${sdkName}-generated-${language}`;
    const commandArgs = [
      "generate",
      "--input",
      ownerOnlyOpenapiPath,
      "--output",
      outputPath,
      "--name",
      sdkName,
      "--type",
      family.sdkType,
      "--language",
      language,
      "--base-url",
      args.baseUrl,
      "--api-prefix",
      family.apiPrefix,
      "--fixed-sdk-version",
      FIXED_SDK_VERSION,
      "--sdk-root",
      sdkRoot,
      "--sdk-name",
      sdkName,
      "--package-name",
      packageName,
      ...family.standardProfileArgs,
      ...args.passthrough,
    ];

    const result = spawnSync(generator.command, [...generator.prefixArgs, ...commandArgs], {
      cwd: sdkRoot,
      stdio: "inherit",
      shell: generator.shell,
    });

    if (result.error) {
      fail(sdkName, `failed to start generator for ${language}: ${result.error.message}`);
    }
    if (typeof result.status === "number" && result.status !== 0) {
      fail(sdkName, `generator failed for ${language} with exit code ${result.status}`);
    }
    if (result.signal) {
      fail(sdkName, `generator terminated by signal ${result.signal}`);
    }

    normalizeGeneratedSdkMetadata(outputPath, family, language, packageName);
    removeStaleGeneratedTrackingFiles(outputPath);
    writeGeneratedSourceOpenApi({ openapiPath: ownerOnlyOpenapiPath, outputPath });
  }

  writeDriveFamilyMetadata({
    openapiPath: ownerOnlyOpenapiPath,
    family,
    generatorName: generator.generatorName,
    baseUrl: args.baseUrl,
  });
}

export function resolveDefaultOpenApiPath(family) {
  if (family.defaultOpenapiPath) {
    return path.resolve(workspaceRoot, family.defaultOpenapiPath);
  }
  return path.join(workspaceRoot, "apis", "openapi", family.defaultOpenapiFile);
}

export function resolveFamilySdkRoot(importMetaUrl) {
  return path.resolve(path.dirname(fileURLToPath(importMetaUrl)), "..");
}

export { STANDARD_PROFILE };
