/**
 * Builds (and optionally packages) the SDKWork Drive Electron desktop host.
 *
 * The renderer is produced by the app's Vite build; this script owns only the
 * Electron side. It:
 *   1. regenerates the tray icon asset,
 *   2. typechecks the host (`src/` + `src-electron/`),
 *   3. verifies the renderer bundle the main process will load exists,
 *   4. compiles the main/preload TypeScript tree,
 *   5. packages with electron-builder unless `--no-package` is passed.
 *
 * Usage:
 *   node ./scripts/build-electron.mjs [--debug] [--no-package]
 */

import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const PACKAGE_DIR = resolve(SCRIPT_DIR, "..");
const APP_DIR = resolve(PACKAGE_DIR, "../..");
const REPO_DIR = resolve(APP_DIR, "../..");

const argv = process.argv.slice(2);
const isDebug = argv.includes("--debug");
const skipPackage = argv.includes("--no-package");

function run(label, command, args, cwd) {
  // eslint-disable-next-line no-console
  console.log(`\n[sdkwork-drive-pc-electron] ${label}\n  > ${command} ${args.join(" ")}`);
  const result = spawnSync(command, args, { cwd, stdio: "inherit", shell: process.platform === "win32" });
  if (result.error) {
    throw new Error(`${label} failed to start: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(`${label} failed with exit code ${result.status}`);
  }
}

function resolveBin(name, fromDirs) {
  for (const dir of fromDirs) {
    const candidate = resolve(dir, "node_modules/.bin", name);
    if (existsSync(candidate) || existsSync(`${candidate}.cmd`)) {
      return { dir, candidate: existsSync(candidate) ? candidate : `${candidate}.cmd` };
    }
  }
  return null;
}

// 1. Tray icon asset. The tray controller treats a missing icon as
//    `unavailable`, so this must exist before packaging.
mkdirSync(resolve(PACKAGE_DIR, "resources/icons"), { recursive: true });
run("generate tray icon", process.execPath, [resolve(SCRIPT_DIR, "generate-tray-icon.mjs")], PACKAGE_DIR);

// 2. Typecheck the Electron host in isolation (it has its own tsconfig).
const tsc = resolveBin("tsc", [PACKAGE_DIR, APP_DIR, REPO_DIR]);
if (tsc) {
  run("typecheck electron host", tsc.candidate, ["--noEmit", "--project", "./tsconfig.json"], PACKAGE_DIR);
} else {
  // eslint-disable-next-line no-console
  console.warn("[sdkwork-drive-pc-electron] tsc not found; skipping typecheck");
}

// 3. The main process loads the renderer from `extraResources` (`../../../dist`
//    -> `renderer`), which electron-builder only copies during packaging. Fail
//    loudly rather than shipping a white window.
const rendererIndex = resolve(APP_DIR, "dist/index.html");
if (!existsSync(rendererIndex)) {
  // eslint-disable-next-line no-console
  console.warn(
    `[sdkwork-drive-pc-electron] renderer bundle missing at ${rendererIndex}; run the app build first (pnpm build).`,
  );
}

// 4. Compile main/preload using the Electron host's own tsconfig output rules.
//    The host is authored as ESM TypeScript and consumed via `electron .`, which
//    resolves `main` to `./src-electron/main/index.js`, so emit is required here.
const buildTsconfig = resolve(PACKAGE_DIR, "tsconfig.build.json");
if (tsc && existsSync(buildTsconfig)) {
  run("compile electron host", tsc.candidate, ["--project", "./tsconfig.build.json"], PACKAGE_DIR);
  // tsc emits extensionless relative specifiers; Node's ESM loader (used by
  // `electron .`) rejects those, so normalize them before packaging.
  run(
    "normalize esm specifiers",
    process.execPath,
    [resolve(SCRIPT_DIR, "fix-esm-specifiers.mjs"), resolve(PACKAGE_DIR, "dist-electron")],
    PACKAGE_DIR,
  );
} else if (!existsSync(buildTsconfig)) {
  // eslint-disable-next-line no-console
  console.warn("[sdkwork-drive-pc-electron] tsconfig.build.json missing; skipping compile");
}

// 5. Package.
if (skipPackage) {
  // eslint-disable-next-line no-console
  console.log("\n[sdkwork-drive-pc-electron] --no-package passed; skipping electron-builder");
} else {
  const eb = resolveBin("electron-builder", [PACKAGE_DIR, APP_DIR, REPO_DIR]);
  if (!eb) {
    // eslint-disable-next-line no-console
    console.warn("[sdkwork-drive-pc-electron] electron-builder not found; skipping packaging");
  } else {
    const ebArgs = ["--config", "./electron-builder.config.mjs"];
    if (isDebug) {
      ebArgs.push("--dir");
    }
    run("package desktop host", eb.candidate, ebArgs, PACKAGE_DIR);
  }
}

// eslint-disable-next-line no-console
console.log(`\n[sdkwork-drive-pc-electron] build complete${isDebug ? " (debug)" : ""}`);
