#!/usr/bin/env node
/**
 * SDKWork Drive standard command dispatcher.
 *
 * Parses standard SDKWork command names and maps them to product implementation
 * scripts. Follows ../sdkwork-specs/PNPM_SCRIPT_SPEC.md section 9.
 *
 * Fails fast on unknown standard commands and retired axis values.
 */

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const repoRoot = resolve(__dirname, "..");

const ALLOWED_DATABASES = new Set(["postgres"]);
const ALLOWED_DEPLOYMENT_PROFILES = new Set(["standalone", "cloud"]);
const ALLOWED_RUNTIME_TARGETS = new Set([
  "browser", "desktop", "server", "container",
  "tablet-ipados", "tablet-android",
  "capacitor-ios", "capacitor-android",
  "flutter-ios", "flutter-android",
  "android-native", "ios-native", "harmony-native",
  "mini-program", "test-runner",
]);

const RETIRED_VALUES = new Set([
  "self-hosted", "cloud-hosted", "hosting",
  "web", "mobile", "native", "docker",
]);

function parseArgs(argv) {
  const args = { command: null, flags: {} };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg.startsWith("--")) {
      const key = arg.slice(2);
      const value = argv[i + 1] && !argv[i + 1].startsWith("--") ? argv[++i] : true;
      args.flags[key] = value;
    } else if (!args.command) {
      args.command = arg;
    }
  }
  return args;
}

function validateAxisValues(flags) {
  const { database, "deployment-profile": deploymentProfile, "runtime-target": runtimeTarget } = flags;

  if (database && !ALLOWED_DATABASES.has(database)) {
    console.error(`[sdkwork-drive] Invalid database: ${database}. Allowed: ${[...ALLOWED_DATABASES].join(", ")}`);
    process.exit(1);
  }
  if (Object.hasOwn(flags, "service-layout")) {
    console.error("[sdkwork-drive] --service-layout is internal topology detail; select --deployment-profile standalone|cloud instead.");
    process.exit(1);
  }
  if (deploymentProfile && !ALLOWED_DEPLOYMENT_PROFILES.has(deploymentProfile)) {
    console.error(`[sdkwork-drive] Invalid deployment-profile: ${deploymentProfile}. Allowed: ${[...ALLOWED_DEPLOYMENT_PROFILES].join(", ")}`);
    process.exit(1);
  }
  if (runtimeTarget && !ALLOWED_RUNTIME_TARGETS.has(runtimeTarget)) {
    console.error(`[sdkwork-drive] Invalid runtime-target: ${runtimeTarget}. Allowed: ${[...ALLOWED_RUNTIME_TARGETS].join(", ")}`);
    process.exit(1);
  }

  for (const [key, value] of Object.entries(flags)) {
    if (typeof value === "string" && RETIRED_VALUES.has(value)) {
      console.error(`[sdkwork-drive] Retired value '${value}' for --${key}. Use standard axis values only.`);
      process.exit(1);
    }
  }
}

function runNodeScript(scriptRelativePath, scriptArgs = [], cwd = repoRoot) {
  const scriptPath = resolve(repoRoot, scriptRelativePath);
  const result = spawnSync("node", [scriptPath, ...scriptArgs], {
    cwd,
    stdio: "inherit",
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function runShell(command, cwd = repoRoot, env = process.env) {
  const result = spawnSync(command, {
    cwd,
    env,
    shell: true,
    stdio: "inherit",
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function dispatch(args) {
  const { command, flags } = args;
  if (!command) {
    console.error("[sdkwork-drive] No command provided. Usage: node scripts/sdkwork-command.mjs <command> [flags]");
    process.exit(1);
  }

  validateAxisValues(flags);

  const runtimeTarget = flags["runtime-target"] || "browser";
  const database = flags.database || "postgres";
  const deploymentProfile = flags["deployment-profile"] || "standalone";

  switch (command) {
    case "dev": {
      // Dispatch to drive-dev.mjs with standard axis values.
      // Default dev args: ['--database', 'postgres', '--deployment-profile', 'standalone']
      const devArgs = [
        "--target", runtimeTarget,
        "--database", database,
        "--deployment-profile", deploymentProfile,
      ];
      if (flags["dry-run"] === true) {
        devArgs.push("--dry-run");
      }
      runNodeScript("scripts/drive-dev.mjs", devArgs);
      break;
    }
    case "build": {
      // Dispatch to drive-build.mjs with cloud deployment profile by default.
      // Default build args: ['--deployment-profile', 'cloud']
      const buildProfile = deploymentProfile === "standalone" ? "standalone" : "cloud";
      runNodeScript("scripts/drive-build.mjs", ["--deployment-profile", buildProfile]);
      break;
    }
    case "test": {
      runShell("cargo test --workspace");
      const pcDir = resolve(repoRoot, "apps/sdkwork-drive-pc");
      if (existsSync(pcDir)) {
        runShell("pnpm test", pcDir);
      }
      runShell("pnpm test:drive-integration", repoRoot);
      runShell("pnpm test:global-assets-contract", repoRoot);
      runShell("pnpm test:drive-share-link-integration", repoRoot);
      runShell("node --test tests/contract/sdk-generator-stub.contract.test.mjs", repoRoot);
      runShell("pnpm test:drive-e2e", repoRoot);
      runShell("pnpm test:e2e", repoRoot);
      break;
    }
    case "check": {
      runShell("cargo check --workspace");
      const pcDir = resolve(repoRoot, "apps/sdkwork-drive-pc");
      if (existsSync(pcDir)) {
        runShell("pnpm typecheck", pcDir);
      }
      break;
    }
    case "verify": {
      runNodeScript("scripts/sdkwork-verify.mjs", []);
      break;
    }
    case "clean": {
      runShell("cargo clean");
      const pcDir = resolve(repoRoot, "apps/sdkwork-drive-pc");
      if (existsSync(pcDir)) {
        runShell("pnpm clean || true", pcDir);
      }
      break;
    }
    case "db:plan":
    case "db:init":
    case "db:migrate":
    case "db:seed":
    case "db:status":
    case "db:validate":
    case "db:bootstrap":
    case "db:drift":
    case "db:drift:check": {
      runShell(`cargo run -p sdkwork-drive-install-worker -- ${command.replace("db:", "--db-")}`);
      break;
    }
    case "gateway:run":
    case "gateway:plan":
    case "gateway:build":
    case "gateway:validate":
    case "gateway:matrix": {
      const profileFlag = deploymentProfile ? ` --${deploymentProfile}` : "";
      runShell(`cargo run -p sdkwork-api-drive-standalone-gateway -- ${command.replace("gateway:", "--gateway-")}${profileFlag}`);
      break;
    }
    case "gateway:package:standalone": {
      // Dispatch to gateway-standalone-pack.mjs for standalone server packaging.
      runNodeScript("scripts/gateway-standalone-pack.mjs", []);
      break;
    }
    case "topology:plan":
    case "topology:validate": {
      console.log(`[sdkwork-drive] topology ${command} for ${deploymentProfile}/development/${database}`);
      break;
    }
    case "release:plan": {
      runNodeScript("scripts/print-gateway-package-matrix.mjs");
      break;
    }
    case "release:build": {
      runNodeScript("scripts/drive-build.mjs");
      runShell("cargo build --release -p sdkwork-api-drive-standalone-gateway");
      break;
    }
    case "release:package": {
      runNodeScript("scripts/release-web-bundle.mjs");
      if (process.platform === 'win32') {
        runNodeScript("scripts/release-desktop-bundle.mjs");
      }
      runNodeScript("scripts/release-catalog-media.mjs");
      runNodeScript("tools/materialize_catalog_media_evidence.mjs");
      runNodeScript("scripts/gateway-standalone-pack.mjs", ["package"]);
      runNodeScript("tools/materialize_release_manifest_evidence.mjs");
      runNodeScript("tools/generate_release_sbom.mjs");
      runNodeScript("tools/check_sdkwork_drive_release_readiness.mjs");
      runNodeScript("tools/verify_release_evidence.mjs");
      break;
    }
    case "release:validate": {
      runNodeScript("tools/check_sdkwork_drive_release_readiness.mjs");
      runNodeScript("tools/verify_release_evidence.mjs");
      break;
    }
    case "deploy:plan":
    case "deploy:apply":
    case "deploy:rollback":
    case "deploy:validate": {
      runNodeScript("tools/check_drive_deployments.mjs", []);
      break;
    }
    default:
      console.error(`[sdkwork-drive] Unknown command: ${command}`);
      console.error("Allowed commands: dev, build, test, check, verify, clean, db:*, gateway:*, topology:*, release:*, deploy:*");
      process.exit(1);
  }
}

const args = parseArgs(process.argv.slice(2));
dispatch(args);
