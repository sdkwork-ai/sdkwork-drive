#!/usr/bin/env node
/**
 * Build the Drive PC desktop bundle for the current host platform and stage release artifacts.
 *
 * Packaging is owned by `electron-builder` through
 * `apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-electron`:
 *
 *   - darwin  -> `.dmg`
 *   - linux   -> `.AppImage`
 *   - win32   -> `.msi` + `.exe` (NSIS)
 *
 * Why Electron rather than the Tauri host: `tauri.conf.json` declares
 * `bundle.targets = ["msi", "nsis"]`, which only exist on Windows. The Tauri CLI
 * refuses to emit a DMG or an AppImage from that configuration, so the previous
 * per-platform branches could never produce the `macos/.../app.dmg` and
 * `linux/.../app.AppImage` paths that `sdkwork.workflow.json` declared.
 * electron-builder ships all three target types and runs them from a
 * per-OS GitHub runner, so every declared artifact is reproducible.
 *
 * Installers are staged as-is. Wrapping a `.msi` back into `app.zip` (the
 * previous behaviour) produced an artifact whose format disagreed with the
 * `packageFormat` recorded in `sdkwork.app.config.json`, so downstream
 * consumers had no way to know what they were downloading.
 *
 * The renderer bundle (`apps/sdkwork-drive-pc/dist`) is built here because it is
 * an `extraResources` input of the Electron host. Building it inside this script
 * — rather than relying on an earlier lifecycle phase — keeps the packaging step
 * self-contained no matter which target the matrix selected.
 */

import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { copyFile, mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const pcRoot = path.join(repoRoot, 'apps', 'sdkwork-drive-pc');
const electronRoot = path.join(pcRoot, 'packages', 'sdkwork-drive-pc-electron');
// `electron-builder.config.mjs` points `directories.output` at `../../dist-electron`,
// which resolves to `apps/sdkwork-drive-pc/dist-electron` from the Electron package.
const electronOutRoot = path.join(pcRoot, 'dist-electron');

function pnpmCommand() {
  return process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? repoRoot,
    env: process.env,
    stdio: 'inherit',
    shell: process.platform === 'win32',
  });
  if (result.error) {
    throw new Error(`Failed to start ${command}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(' ')} exited with ${result.status ?? 1}`);
  }
}

async function resolveVersion() {
  const manifestPath = path.join(repoRoot, 'sdkwork.app.config.json');
  const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
  return manifest.release?.currentVersion ?? '0.1.0';
}

async function sha256File(filePath) {
  const digest = createHash('sha256');
  digest.update(await readFile(filePath));
  return digest.digest('hex');
}

async function findNewestFile(root, matcher) {
  const matches = [];
  async function walk(current) {
    const entries = await readdir(current, { withFileTypes: true });
    for (const entry of entries) {
      const entryPath = path.join(current, entry.name);
      if (entry.isDirectory()) {
        await walk(entryPath);
      } else if (matcher(entry.name)) {
        matches.push(entryPath);
      }
    }
  }
  if (existsSync(root)) {
    await walk(root);
  }
  matches.sort((left, right) => right.localeCompare(left));
  return matches[0] ?? null;
}

/**
 * Stage one installer into `dist/release/<channel>/<version>/...` under the
 * canonical name the workflow config declares, and return its manifest record.
 */
async function stageInstaller({ sourcePath, targetRelDir, targetName, packageId, platform, version, channel }) {
  const releaseDir = path.join(repoRoot, 'dist', 'release', channel, version, targetRelDir);
  await mkdir(releaseDir, { recursive: true });
  const stagedPath = path.join(releaseDir, targetName);
  await copyFile(sourcePath, stagedPath);
  const checksum = await sha256File(stagedPath);
  return {
    packageId,
    platform,
    stagedPath,
    archivePath: path.relative(repoRoot, stagedPath).split(path.sep).join('/'),
    checksum,
  };
}

/**
 * electron-builder writes every target into the same flat output directory.
 * Pick the newest per extension so a stale artifact from an earlier run can
 * never be staged as the current release.
 */
async function collectElectronInstaller(extension) {
  const pattern = new RegExp(`${extension.replace('.', '\\.')}$`, 'i');
  const found = await findNewestFile(electronOutRoot, (name) => pattern.test(name));
  if (!found) {
    throw new Error(
      `Missing ${extension} installer under ${electronOutRoot}; electron-builder did not produce it`,
    );
  }
  return found;
}

async function stageWindowsBundle({ version, channel }) {
  const msi = await collectElectronInstaller('.msi');
  const exe = await collectElectronInstaller('.exe');

  const msiRecord = await stageInstaller({
    sourcePath: msi,
    targetRelDir: path.join('windows', 'x64'),
    targetName: 'app.msi',
    packageId: 'windows-x64-standalone-desktop-msi',
    platform: 'DESKTOP_WINDOWS',
    version,
    channel,
  });
  const exeRecord = await stageInstaller({
    sourcePath: exe,
    targetRelDir: path.join('windows', 'x64'),
    targetName: 'app-setup.exe',
    packageId: 'windows-x64-standalone-desktop-nsis',
    platform: 'DESKTOP_WINDOWS',
    version,
    channel,
  });

  // `windows-x64-standalone-desktop-zip` stays the canonical Windows package id
  // (it is the id `sdkwork.app.config.json` and release notes reference), so the
  // MSI acts as its primary artifact.
  return {
    records: [msiRecord, exeRecord],
    primary: {
      packageId: 'windows-x64-standalone-desktop-zip',
      runtimeTarget: 'desktop',
      deploymentProfile: 'standalone',
      platform: 'DESKTOP_WINDOWS',
      archivePath: msiRecord.archivePath,
      installerPath: msiRecord.archivePath,
      checksum: msiRecord.checksum,
    },
  };
}

async function stageMacosBundle({ version, channel }) {
  const dmg = await collectElectronInstaller('.dmg');
  const record = await stageInstaller({
    sourcePath: dmg,
    targetRelDir: path.join('macos', 'universal'),
    targetName: 'app.dmg',
    packageId: 'macos-universal-standalone-desktop-dmg',
    platform: 'DESKTOP_MACOS',
    version,
    channel,
  });
  return {
    records: [record],
    primary: {
      packageId: 'macos-universal-standalone-desktop-dmg',
      runtimeTarget: 'desktop',
      deploymentProfile: 'standalone',
      platform: 'DESKTOP_MACOS',
      archivePath: record.archivePath,
      installerPath: record.archivePath,
      checksum: record.checksum,
    },
  };
}

async function stageLinuxBundle({ version, channel }) {
  const appImage = await collectElectronInstaller('.AppImage');
  const record = await stageInstaller({
    sourcePath: appImage,
    targetRelDir: path.join('linux', 'generic', 'x64'),
    targetName: 'app.AppImage',
    packageId: 'linux-x64-standalone-desktop-appimage',
    platform: 'DESKTOP_LINUX',
    version,
    channel,
  });
  return {
    records: [record],
    primary: {
      packageId: 'linux-x64-standalone-desktop-appimage',
      runtimeTarget: 'desktop',
      deploymentProfile: 'standalone',
      platform: 'DESKTOP_LINUX',
      archivePath: record.archivePath,
      installerPath: record.archivePath,
      checksum: record.checksum,
    },
  };
}

/** Maps `SDKWORK_TARGET_PLATFORM` (matrix metadata) to the host it must build on. */
const HOST_PLATFORM_BY_TARGET = {
  windows: 'win32',
  macos: 'darwin',
  linux: 'linux',
};

const STAGERS_BY_HOST = {
  win32: stageWindowsBundle,
  darwin: stageMacosBundle,
  linux: stageLinuxBundle,
};

/**
 * Fail loudly when the runner's OS does not match the target being packaged.
 * Without this guard the script would silently build the *host's* installer and
 * then report it as the requested platform — the exact class of mismatch that
 * made the previous matrix unreliable.
 */
function assertHostMatchesTarget(platform) {
  const expectedHost = HOST_PLATFORM_BY_TARGET[platform];
  if (!expectedHost) {
    return;
  }
  if (expectedHost !== process.platform) {
    throw new Error(
      `Target ${platform} must be packaged on ${expectedHost}, but this host is ${process.platform}`,
    );
  }
}

async function main() {
  const version = process.env.SDKWORK_PACKAGE_VERSION ?? (await resolveVersion());
  const channel = process.env.SDKWORK_RELEASE_CHANNEL ?? 'STABLE';
  const targetPlatform = process.env.SDKWORK_TARGET_PLATFORM ?? '';

  assertHostMatchesTarget(targetPlatform);

  // 1. Renderer bundle. electron-builder copies `apps/sdkwork-drive-pc/dist` into
  //    the app's `resources/renderer`, so it must exist before packaging.
  run(pnpmCommand(), ['build'], { cwd: pcRoot });

  // 2. Electron host: tray icon + typecheck + compile + electron-builder.
  run(pnpmCommand(), ['build:desktop'], { cwd: electronRoot });

  const stager = STAGERS_BY_HOST[process.platform];
  if (!stager) {
    throw new Error(`Unsupported desktop packaging host ${process.platform}`);
  }

  const { records, primary } = await stager({ version, channel });

  // 3. Manifest consumed by downstream evidence/verification tooling.
  const manifest = {
    schemaVersion: 1,
    packageId: primary.packageId,
    version,
    channel,
    archivePath: primary.archivePath,
    installerPath: primary.installerPath,
    checksumAlgorithm: 'SHA-256',
    checksum: primary.checksum,
    runtimeTarget: primary.runtimeTarget,
    deploymentProfile: primary.deploymentProfile,
    platform: primary.platform,
    installers: records.map((record) => ({
      packageId: record.packageId,
      archivePath: record.archivePath,
      checksumAlgorithm: 'SHA-256',
      checksum: record.checksum,
    })),
  };
  await writeFile(
    path.join(repoRoot, path.dirname(primary.archivePath), 'desktop-package-manifest.json'),
    `${JSON.stringify(manifest, null, 2)}\n`,
    'utf8',
  );

  for (const record of records) {
    console.log(
      `[release-desktop-bundle] staged ${record.archivePath} (${record.checksum.slice(0, 12)}...)`,
    );
  }
}

main().catch((error) => {
  console.error(`[release-desktop-bundle] ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
});
