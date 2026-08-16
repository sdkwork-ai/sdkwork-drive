#!/usr/bin/env node
/**
 * Build the Drive PC browser bundle and package it as web.zip for release evidence.
 */

import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const pcRoot = path.join(repoRoot, 'apps', 'sdkwork-drive-pc');

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
  if (!existsSync(manifestPath)) {
    return '0.1.0';
  }
  const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
  return manifest.release?.currentVersion ?? '0.1.0';
}

async function sha256File(filePath) {
  const digest = createHash('sha256');
  digest.update(await readFile(filePath));
  return digest.digest('hex');
}

async function main() {
  const version = process.env.SDKWORK_PACKAGE_VERSION ?? await resolveVersion();
  const channel = process.env.SDKWORK_RELEASE_CHANNEL ?? 'STABLE';
  const viteDist = path.join(pcRoot, 'dist');
  const releaseDir = path.join(repoRoot, 'dist', 'release', channel, version);
  const archivePath = path.join(releaseDir, 'web.zip');

  run(pnpmCommand(), ['build'], { cwd: pcRoot });

  if (!existsSync(viteDist)) {
    throw new Error(`Missing browser build output: ${viteDist}`);
  }

  await mkdir(releaseDir, { recursive: true });
  if (process.platform === 'win32') {
    run('powershell', [
      '-NoProfile',
      '-Command',
      `Compress-Archive -Path '${viteDist.replace(/'/g, "''")}\\*' -DestinationPath '${archivePath.replace(/'/g, "''")}' -Force`,
    ]);
  } else {
    run('zip', ['-qr', archivePath, '.'], { cwd: viteDist });
  }

  const checksum = await sha256File(archivePath);
  const manifest = {
    schemaVersion: 1,
    packageId: 'web-universal-cloud-browser-zip',
    version,
    channel,
    archivePath: path.relative(repoRoot, archivePath).split(path.sep).join('/'),
    checksumAlgorithm: 'SHA-256',
    checksum,
    runtimeTarget: 'browser',
    deploymentProfile: 'cloud',
  };
  await writeFile(
    path.join(releaseDir, 'web-package-manifest.json'),
    `${JSON.stringify(manifest, null, 2)}\n`,
    'utf8',
  );

  console.log(`[release-web-bundle] packaged ${manifest.archivePath} (${checksum.slice(0, 12)}...)`);
}

main().catch((error) => {
  console.error(`[release-web-bundle] ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
});
