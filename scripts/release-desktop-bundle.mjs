#!/usr/bin/env node
/**
 * Build the Drive PC desktop bundle for the current host platform and stage release artifacts.
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
const desktopRoot = path.join(pcRoot, 'packages', 'sdkwork-drive-pc-desktop');
const tauriRoot = path.join(desktopRoot, 'src-tauri');

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
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
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

async function stageMacosBundle(version, channel) {
  const bundleRoot = path.join(tauriRoot, 'target', 'release', 'bundle');
  const dmg = await findNewestFile(bundleRoot, (name) => /\.dmg$/i.test(name));
  if (!dmg) {
    throw new Error(`Missing macOS desktop bundle under ${bundleRoot}`);
  }

  const releaseDir = path.join(repoRoot, 'dist', 'release', channel, version, 'macos', 'universal');
  await mkdir(releaseDir, { recursive: true });
  const stagedPath = path.join(releaseDir, 'app.dmg');
  await copyFile(dmg, stagedPath);

  const checksum = await sha256File(stagedPath);
  return {
    packageId: 'macos-universal-standalone-desktop-dmg',
    archivePath: path.relative(repoRoot, stagedPath).split(path.sep).join('/'),
    checksum,
    platform: 'DESKTOP_MACOS',
  };
}

async function stageLinuxBundle(version, channel) {
  const bundleRoot = path.join(tauriRoot, 'target', 'release', 'bundle');
  const appImage = await findNewestFile(bundleRoot, (name) => /\.AppImage$/i.test(name));
  if (!appImage) {
    throw new Error(`Missing Linux desktop AppImage under ${bundleRoot}`);
  }

  const releaseDir = path.join(repoRoot, 'dist', 'release', channel, version, 'linux', 'generic', 'x64');
  await mkdir(releaseDir, { recursive: true });
  const stagedPath = path.join(releaseDir, 'app.AppImage');
  await copyFile(appImage, stagedPath);

  const checksum = await sha256File(stagedPath);
  return {
    packageId: 'linux-x64-standalone-desktop-appimage',
    archivePath: path.relative(repoRoot, stagedPath).split(path.sep).join('/'),
    checksum,
    platform: 'DESKTOP_LINUX',
  };
}

async function stageWindowsBundle(version, channel) {
  const bundleRoot = path.join(tauriRoot, 'target', 'release', 'bundle');
  const installer = await findNewestFile(bundleRoot, (name) => /\.(exe|msi)$/i.test(name));
  if (!installer) {
    throw new Error(`Missing Windows desktop installer under ${bundleRoot}`);
  }

  const releaseDir = path.join(repoRoot, 'dist', 'release', channel, version, 'windows', 'x64');
  await mkdir(releaseDir, { recursive: true });
  const archivePath = path.join(releaseDir, 'app.zip');
  const stagedInstaller = path.join(releaseDir, path.basename(installer));

  await copyFile(installer, stagedInstaller);
  if (process.platform === 'win32') {
    run('powershell', [
      '-NoProfile',
      '-Command',
      `Compress-Archive -Path '${stagedInstaller.replace(/'/g, "''")}' -DestinationPath '${archivePath.replace(/'/g, "''")}' -Force`,
    ]);
  } else {
    run('zip', ['-qj', archivePath, stagedInstaller]);
  }

  const checksum = await sha256File(archivePath);
  return {
    packageId: 'windows-x64-standalone-desktop-zip',
    archivePath: path.relative(repoRoot, archivePath).split(path.sep).join('/'),
    installerPath: path.relative(repoRoot, stagedInstaller).split(path.sep).join('/'),
    checksum,
  };
}

async function main() {
  const version = process.env.SDKWORK_PACKAGE_VERSION ?? await resolveVersion();
  const channel = process.env.SDKWORK_RELEASE_CHANNEL ?? 'STABLE';

  run(pnpmCommand(), ['build:desktop'], { cwd: pcRoot });

  let staged;
  if (process.platform === 'win32') {
    staged = await stageWindowsBundle(version, channel);
  } else if (process.platform === 'darwin') {
    staged = await stageMacosBundle(version, channel);
  } else if (process.platform === 'linux') {
    staged = await stageLinuxBundle(version, channel);
  } else {
    console.warn(`[release-desktop-bundle] skipping desktop artifact staging on unsupported host ${process.platform}`);
    return;
  }

  const manifest = {
    schemaVersion: 1,
    packageId: staged.packageId,
    version,
    channel,
    archivePath: staged.archivePath,
    installerPath: staged.installerPath ?? staged.archivePath,
    checksumAlgorithm: 'SHA-256',
    checksum: staged.checksum,
    runtimeTarget: 'desktop',
    deploymentProfile: 'standalone',
    platform: staged.platform ?? 'DESKTOP_WINDOWS',
  };
  const manifestDir = path.dirname(path.join(repoRoot, staged.archivePath));
  await writeFile(
    path.join(manifestDir, 'desktop-package-manifest.json'),
    `${JSON.stringify(manifest, null, 2)}\n`,
    'utf8',
  );
  console.log(`[release-desktop-bundle] packaged ${staged.archivePath} (${staged.checksum.slice(0, 12)}...)`);
}

main().catch((error) => {
  console.error(`[release-desktop-bundle] ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
});
