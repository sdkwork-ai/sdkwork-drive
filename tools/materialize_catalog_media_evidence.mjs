#!/usr/bin/env node
/**
 * Materialize staged catalog media checksum evidence into sdkwork.app.config.json.
 * CDN publication remains a separate protected upload step.
 */

import { readFile, stat, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const MANIFEST_RELATIVE_PATHS = [
  'sdkwork.app.config.json',
  'apps/sdkwork-drive-pc/sdkwork.app.config.json',
];
const MEDIA_ID_ALIASES = {
  'sdkwork-drive-pc-primary-icon': 'sdkwork-drive-primary-icon',
  'sdkwork-drive-pc-desktop-screenshot': 'sdkwork-drive-desktop_windows-screenshot',
  'sdkwork-drive-pc-catalog-preview': 'sdkwork-drive-catalog-preview',
};

function resolveEvidenceEntry(mediaId, catalogMedia) {
  const direct = catalogMedia.find((entry) => entry.id === mediaId);
  if (direct) {
    return direct;
  }
  const aliasSource = MEDIA_ID_ALIASES[mediaId];
  if (!aliasSource) {
    return null;
  }
  return catalogMedia.find((entry) => entry.id === aliasSource) ?? null;
}

async function applyCatalogMediaEvidence(manifestPath, evidence) {
  const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
  const catalogMedia = evidence.catalogMedia ?? [];
  let updatedCount = 0;

  const mediaItems = [
    manifest.media?.icons?.primary,
    ...(manifest.media?.icons?.platform ?? []),
    ...(manifest.media?.screenshots ?? []),
    ...(manifest.media?.previews ?? []),
  ].filter(Boolean);

  for (const item of mediaItems) {
    const entry = resolveEvidenceEntry(item.id, catalogMedia);
    if (!entry) {
      continue;
    }
    const stagedPath = path.join(repoRoot, entry.stagedPath);
    if (!existsSync(stagedPath)) {
      console.warn(`[catalog-media-evidence] warning: staged media missing for ${item.id}: ${entry.stagedPath}`);
      continue;
    }

    const fileStat = await stat(stagedPath);
    item.metadata = {
      ...(item.metadata ?? {}),
      stagedArtifactPath: entry.stagedPath,
      stagedChecksumAlgorithm: entry.checksumAlgorithm ?? 'SHA-256',
      stagedChecksum: entry.checksum,
      catalogMediaStagedAt: evidence.generatedAt,
      catalogMediaDeferred: true,
      generatedPlaceholder: true,
    };
    item.fileSizeBytes = fileStat.size;
    updatedCount += 1;
    console.log(`[catalog-media-evidence] ${item.id} -> ${entry.stagedPath}`);
  }

  await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  console.log(`[catalog-media-evidence] updated ${updatedCount} catalog media item(s) in ${path.relative(repoRoot, manifestPath)}`);
}

async function main() {
  const evidencePath = path.join(repoRoot, 'target', 'release', 'catalog-media-evidence.json');
  if (!existsSync(evidencePath)) {
    console.error('[catalog-media-evidence] error: target/release/catalog-media-evidence.json must exist');
    process.exit(1);
  }

  const evidence = JSON.parse(await readFile(evidencePath, 'utf8'));

  for (const relativePath of MANIFEST_RELATIVE_PATHS) {
    const manifestPath = path.join(repoRoot, relativePath);
    if (!existsSync(manifestPath)) {
      console.warn(`[catalog-media-evidence] warning: manifest missing at ${relativePath}`);
      continue;
    }
    await applyCatalogMediaEvidence(manifestPath, evidence);
  }
}

main().catch((error) => {
  console.error(`[catalog-media-evidence] ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
});
