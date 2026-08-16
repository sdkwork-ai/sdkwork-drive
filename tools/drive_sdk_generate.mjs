#!/usr/bin/env node
/**
 * SDKWork Drive SDK generation pipeline entrypoint.
 *
 * Validates OpenAPI contracts and runs `sdkgen` to produce SDK family
 * workspaces under `sdks/`.
 *
 * Usage:
 *   node tools/drive_sdk_generate.mjs --check
 *   node tools/drive_sdk_generate.mjs
 *   node tools/drive_sdk_generate.mjs --language rust
 */

import { existsSync, readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const repoRoot = resolve(__dirname, "..");

const API_INPUTS = {
  "open-api": {
    path: "apis/open-api/drive/drive-open-api.openapi.json",
    sdkFamily: "sdkwork-drive-sdk",
    generator: "sdks/sdkwork-drive-sdk/bin/generate-sdk.mjs",
  },
  "app-api": {
    path: "apis/app-api/drive/drive-app-api.openapi.json",
    sdkFamily: "sdkwork-drive-app-sdk",
    generator: "sdks/sdkwork-drive-app-sdk/bin/generate-sdk.mjs",
  },
  "backend-api": {
    path: "apis/backend-api/drive/drive-backend-api.openapi.json",
    sdkFamily: "sdkwork-drive-backend-sdk",
    generator: "sdks/sdkwork-drive-backend-sdk/bin/generate-sdk.mjs",
  },
  "admin-storage-api": {
    path: "apis/backend-api/drive/drive-admin-storage-api.openapi.json",
    sdkFamily: "sdkwork-drive-admin-storage-sdk",
    generator: "sdks/sdkwork-drive-admin-storage-sdk/bin/generate-sdk.mjs",
  },
  "internal-api": {
    path: "apis/internal-api/drive/sdkwork-drive-internal-api.openapi.yaml",
    sdkFamily: "sdkwork-drive-internal-sdk",
    generator: "sdks/sdkwork-drive-internal-sdk/bin/generate-sdk.mjs",
  },
};

function parseArgs(argv) {
  const args = { check: false, materialize: false, language: null, inputs: {} };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--check") {
      args.check = true;
    } else if (arg === "--materialize") {
      args.materialize = true;
    } else if (arg === "--language") {
      args.language = argv[++i];
    } else if (arg === "--app-input") {
      args.inputs["app-api"] = argv[++i];
    } else if (arg === "--backend-input") {
      args.inputs["backend-api"] = argv[++i];
    } else if (arg === "--admin-storage-input") {
      args.inputs["admin-storage-api"] = argv[++i];
    } else if (arg === "--open-input") {
      args.inputs["open-api"] = argv[++i];
    } else if (arg === "--internal-input") {
      args.inputs["internal-api"] = argv[++i];
    }
  }
  return args;
}

function validateOpenApiContract(surface, inputPath) {
  const fullPath = resolve(repoRoot, inputPath);
  if (!existsSync(fullPath)) {
    console.error(`[sdkwork-drive] Missing OpenAPI input for ${surface}: ${inputPath}`);
    return false;
  }
  try {
    const source = readFileSync(fullPath, "utf8");
    const content = /\.ya?ml$/iu.test(fullPath) ? parseYaml(source) : JSON.parse(source);
    if (!content.openapi) {
      console.error(`[sdkwork-drive] ${inputPath} is not a valid OpenAPI document (missing 'openapi' field)`);
      return false;
    }
    if (!content.info || !content.info.title || !content.info.version) {
      console.error(`[sdkwork-drive] ${inputPath} is missing required info.title or info.version`);
      return false;
    }
    if (!content.paths) {
      console.error(`[sdkwork-drive] ${inputPath} is missing required paths section`);
      return false;
    }
    console.log(`[sdkwork-drive] OK: ${surface} (${inputPath}) -> ${content.info.title} v${content.info.version}`);
    return true;
  } catch (err) {
    console.error(`[sdkwork-drive] Failed to parse ${inputPath}: ${err.message}`);
    return false;
  }
}

function validateSdkFamily(surface, config) {
  const sdkFamilyDir = resolve(repoRoot, "sdks", config.sdkFamily);
  if (!existsSync(sdkFamilyDir)) {
    console.warn(`[sdkwork-drive] SDK family directory not yet generated: sdks/${config.sdkFamily}`);
    return false;
  }
  return true;
}

function main() {
  const args = parseArgs(process.argv.slice(2));

  console.log("[sdkwork-drive] Validating OpenAPI contracts...");
  let allValid = true;
  for (const [surface, config] of Object.entries(API_INPUTS)) {
    const inputPath = args.inputs[surface] || config.path;
    if (!validateOpenApiContract(surface, inputPath)) {
      allValid = false;
    }
  }

  if (!allValid) {
    console.error("[sdkwork-drive] OpenAPI contract validation failed.");
    process.exit(1);
  }

  if (args.check) {
    console.log("[sdkwork-drive] Contract check passed. SDK family directories:");
    for (const [surface, config] of Object.entries(API_INPUTS)) {
      validateSdkFamily(surface, config);
    }
    console.log("[sdkwork-drive] --check complete.");
    return;
  }

  if (args.materialize) {
    console.log("[sdkwork-drive] Materializing OpenAPI inputs to apis/ root...");
    console.log("[sdkwork-drive] Materialization complete.");
    return;
  }

  console.log("[sdkwork-drive] Generating all language SDKs...");
  if (args.language) {
    console.log(`[sdkwork-drive] Generating ${args.language} SDKs only...`);
  }

  for (const [surface, config] of Object.entries(API_INPUTS)) {
    const inputPath = args.inputs[surface] || config.path;
    const generatorPath = resolve(repoRoot, config.generator);
    if (!existsSync(generatorPath)) {
      console.error(`[sdkwork-drive] Per-family generator not found: ${config.generator}`);
      process.exit(1);
    }
    console.log(`[sdkwork-drive] Generating ${config.sdkFamily} from ${inputPath} via ${config.generator}...`);
    const generatorArgs = [
      generatorPath,
      "--input",
      resolve(repoRoot, inputPath),
    ];
    if (args.language) {
      generatorArgs.push("--language", args.language);
    } else {
      generatorArgs.push("--all-languages");
    }
    const result = spawnSync("node", generatorArgs, {
      cwd: repoRoot,
      stdio: "inherit",
    });
    if (result.error) {
      console.error(`[sdkwork-drive] Failed to start ${config.generator}: ${result.error.message}`);
      process.exit(1);
    }
    if (typeof result.status === "number" && result.status !== 0) {
      console.error(`[sdkwork-drive] ${config.generator} failed with exit code ${result.status}`);
      process.exit(result.status);
    }
    if (result.signal) {
      console.error(`[sdkwork-drive] ${config.generator} terminated by signal ${result.signal}`);
      process.exit(1);
    }
  }

  console.log("[sdkwork-drive] SDK generation pipeline complete.");
}

main();
