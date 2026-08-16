#!/usr/bin/env node
/**
 * Stage catalog media bytes locally and record checksum evidence for release governance.
 * CDN publication remains a separate Drive-hosted upload step.
 */

import { createHash } from 'node:crypto';
import { copyFile, mkdir, readFile, stat, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { createSolidColorPng } from '../tools/catalog_png.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const iconSource = path.join(
  repoRoot,
  'apps',
  'sdkwork-drive-pc',
  'packages',
  'sdkwork-drive-pc-desktop',
  'src-tauri',
  'icons',
  'icon.ico',
);

const CATALOG_MEDIA = [
  {
    id: 'sdkwork-drive-primary-icon',
    fileName: 'icon-1024.png',
    width: 1024,
    height: 1024,
    format: 'PNG',
    source: 'generated',
  },
  {
    id: 'sdkwork-drive-desktop_windows-screenshot',
    fileName: 'desktop_windows-screenshot.png',
    width: 1920,
    height: 1080,
    format: 'PNG',
    source: 'generated',
  },
  {
    id: 'sdkwork-drive-catalog-preview',
    fileName: 'preview-cover.png',
    width: 1600,
    height: 900,
    format: 'PNG',
    source: 'generated',
  },
];

async function sha256File(filePath) {
  const digest = createHash('sha256');
  digest.update(await readFile(filePath));
  return digest.digest('hex');
}

async function resolveVersion() {
  const manifestPath = path.join(repoRoot, 'sdkwork.app.config.json');
  const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
  return manifest.release?.currentVersion ?? '0.1.0';
}

async function stageMediaAsset(mediaDir, asset) {
  const stagedPath = path.join(mediaDir, asset.fileName);
  if (asset.id === 'sdkwork-drive-primary-icon' && existsSync(iconSource)) {
    await copyFile(iconSource, path.join(mediaDir, 'icon-1024.ico'));
  }
  const png = createSolidColorPng(asset.width, asset.height, [37, 99, 235]);
  await writeFile(stagedPath, png);
  const checksum = await sha256File(stagedPath);
  const fileStat = await stat(stagedPath);
  return {
    id: asset.id,
    stagedPath: path.relative(repoRoot, stagedPath).split(path.sep).join('/'),
    checksumAlgorithm: 'SHA-256',
    checksum,
    fileSizeBytes: fileStat.size,
    width: asset.width,
    height: asset.height,
    format: asset.format,
    publicationStatus: 'deferred_to_drive_upload',
    source: asset.source,
  };
}

async function main() {
  const version = process.env.SDKWORK_PACKAGE_VERSION ?? await resolveVersion();
  const channel = process.env.SDKWORK_RELEASE_CHANNEL ?? 'STABLE';
  const mediaDir = path.join(repoRoot, 'dist', 'release', channel, version, 'media');
  await mkdir(mediaDir, { recursive: true });

  const catalogMedia = [];
  for (const asset of CATALOG_MEDIA) {
    catalogMedia.push(await stageMediaAsset(mediaDir, asset));
  }

  const evidence = {
    schemaVersion: 1,
    generatedAt: new Date().toISOString(),
    catalogMedia,
  };

  const evidencePath = path.join(repoRoot, 'target', 'release', 'catalog-media-evidence.json');
  await mkdir(path.dirname(evidencePath), { recursive: true });
  await writeFile(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`, 'utf8');
  for (const entry of catalogMedia) {
    console.log(`[release-catalog-media] staged ${entry.stagedPath} (${entry.checksum.slice(0, 12)}...)`);
  }
}

main().catch((error) => {
  console.error(`[release-catalog-media] ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
});
