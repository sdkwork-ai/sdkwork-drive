#!/usr/bin/env node
/**
 * Materialize release checksum evidence into sdkwork.app.config.json from local build outputs.
 * Follows RELEASE_SPEC.md and SUPPLY_CHAIN_SECURITY_SPEC.md.
 */

import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { glob } from 'node:fs/promises';

const scriptRepoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function resolveWorkspaceRoot() {
  const rootFlagIndex = process.argv.indexOf('--root');
  if (rootFlagIndex >= 0) {
    return path.resolve(process.argv[rootFlagIndex + 1]);
  }
  const cwdManifest = path.join(process.cwd(), 'sdkwork.app.config.json');
  if (existsSync(cwdManifest)) {
    return process.cwd();
  }
  return scriptRepoRoot;
}

const PACKAGE_ARTIFACT_GLOBS = {
  'web-universal-cloud-browser-zip': [
    'dist/release/**/web.zip',
    'dist/browser/**/web.zip',
    'apps/sdkwork-drive-pc/dist/**/*.zip',
  ],
  'windows-x64-standalone-desktop-zip': [
    'dist/release/**/windows/**/app.zip',
    'dist/desktop/**/windows/**/app.zip',
    'apps/sdkwork-drive-pc/src-tauri/target/release/bundle/**/*.zip',
    'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-desktop/src-tauri/target/release/bundle/**/*.zip',
  ],
  'macos-universal-standalone-desktop-dmg': [
    'dist/release/**/macos/**/app.dmg',
    'dist/desktop/**/macos/**/app.dmg',
    'apps/sdkwork-drive-pc/src-tauri/target/release/bundle/**/*.dmg',
  ],
  'linux-x64-standalone-desktop-appimage': [
    'dist/release/**/linux/**/app.AppImage',
    'dist/desktop/**/linux/**/app.AppImage',
    'apps/sdkwork-drive-pc/src-tauri/target/release/bundle/**/*.AppImage',
  ],
};

function isPlaceholderChecksum(checksum) {
  if (typeof checksum !== 'string' || checksum.length < 16) {
    return true;
  }
  const sample = checksum.slice(0, 8);
  return checksum === sample.repeat(Math.ceil(checksum.length / sample.length)).slice(0, checksum.length);
}

async function sha256File(absolutePath) {
  const digest = createHash('sha256');
  digest.update(await readFile(absolutePath));
  return digest.digest('hex');
}

function loadExistingEvidence(evidencePath) {
  if (!existsSync(evidencePath)) {
    return null;
  }
  try {
    return JSON.parse(readFileSync(evidencePath, 'utf8'));
  } catch {
    return null;
  }
}

function mergeEvidencePackages(existingPackages, updatedPackages) {
  const merged = new Map();
  for (const entry of existingPackages ?? []) {
    if (entry?.id) {
      merged.set(entry.id, entry);
    }
  }
  for (const entry of updatedPackages) {
    merged.set(entry.id, entry);
  }
  return [...merged.values()].sort((left, right) => left.id.localeCompare(right.id));
}

function mergeAuxiliaryArtifacts(existingArtifacts, updatedArtifacts) {
  const merged = new Map();
  for (const entry of existingArtifacts ?? []) {
    const key = `${entry.packageId ?? 'unknown'}:${entry.artifactPath ?? ''}`;
    merged.set(key, entry);
  }
  for (const entry of updatedArtifacts) {
    const key = `${entry.packageId ?? 'unknown'}:${entry.artifactPath ?? ''}`;
    merged.set(key, entry);
  }
  return [...merged.values()].sort((left, right) =>
    `${left.packageId}:${left.artifactPath}`.localeCompare(`${right.packageId}:${right.artifactPath}`),
  );
}

function mergeWarnings(existingWarnings, updatedWarnings) {
  return [...new Set([...(existingWarnings ?? []), ...(updatedWarnings ?? [])])].sort();
}

async function resolvePackageArtifact(workspaceRoot, packageId, extraGlobs = []) {
  const patterns = [...(PACKAGE_ARTIFACT_GLOBS[packageId] ?? []), ...extraGlobs];
  const matches = [];
  for (const pattern of patterns) {
    for await (const match of glob(pattern, { cwd: workspaceRoot })) {
      const absolutePath = path.join(workspaceRoot, match);
      if (existsSync(absolutePath)) {
        matches.push(absolutePath);
      }
    }
  }
  matches.sort((left, right) => right.localeCompare(left));
  return matches[0] ?? null;
}

async function main() {
  const strict = process.argv.includes('--strict');
  const workspaceRoot = resolveWorkspaceRoot();
  const manifestPath = path.join(workspaceRoot, 'sdkwork.app.config.json');
  const evidencePath = path.join(workspaceRoot, 'target', 'release', 'release-evidence.json');
  const existingEvidence = loadExistingEvidence(evidencePath);
  const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
  const packages = manifest.artifacts?.installConfig?.packages ?? [];
  const evidence = {
    schemaVersion: 1,
    generatedAt: new Date().toISOString(),
    manifestPath: 'sdkwork.app.config.json',
    packages: [],
    auxiliaryArtifacts: [],
    warnings: [],
  };

  let updatedCount = 0;
  for (const pkg of packages) {
    if (pkg.enabled === false) {
      continue;
    }
    const metadataGlobs = Array.isArray(pkg.metadata?.releaseArtifactGlobs)
      ? pkg.metadata.releaseArtifactGlobs
      : [];
    const artifactPath = await resolvePackageArtifact(workspaceRoot, pkg.id, metadataGlobs);
    if (!artifactPath) {
      if (pkg.metadata?.releaseBuildDeferred === true && pkg.checksum && isPlaceholderChecksum(pkg.checksum)) {
        console.error(
          `[release-evidence] error: package ${pkg.id} must not keep placeholder checksum fields while releaseBuildDeferred is true`,
        );
        process.exit(1);
      }
      const message = pkg.metadata?.releaseBuildDeferred === true
        ? `package ${pkg.id} is deferred to cross-platform CI release builds`
        : `package ${pkg.id} has no local build artifact; run release packaging before materializing checksum evidence`;
      evidence.warnings.push(message);
      if (strict && pkg.metadata?.releaseBuildDeferred !== true) {
        console.error(`[release-evidence] error: ${message}`);
        process.exit(1);
      }
      if (pkg.metadata?.releaseBuildDeferred !== true) {
        console.warn(`[release-evidence] warning: ${message}`);
      }
      continue;
    }

    const checksum = await sha256File(artifactPath);
    const relativeArtifact = path.relative(workspaceRoot, artifactPath).split(path.sep).join('/');
    const previousChecksum = pkg.checksum;
    pkg.checksum = checksum;
    pkg.checksumAlgorithm = 'SHA-256';
    pkg.metadata = {
      ...(pkg.metadata ?? {}),
      releaseArtifactPath: relativeArtifact,
      releaseEvidenceMaterializedAt: evidence.generatedAt,
      releaseBuildMaterializedAt: evidence.generatedAt,
    };
    delete pkg.metadata.releaseBuildDeferred;
    evidence.packages.push({
      id: pkg.id,
      artifactPath: relativeArtifact,
      checksumAlgorithm: 'SHA-256',
      checksum,
      previousChecksum: previousChecksum ?? null,
      placeholderReplaced: isPlaceholderChecksum(previousChecksum),
    });
    updatedCount += 1;
    console.log(`[release-evidence] ${pkg.id} -> ${relativeArtifact} (${checksum.slice(0, 12)}...)`);
  }

  const enabledPackages = packages.filter((pkg) => pkg.enabled !== false);
  const requiredPackages = enabledPackages.filter((pkg) => pkg.metadata?.releaseBuildDeferred !== true);
  const allRequiredChecksumsMaterialized = requiredPackages.length > 0
    && requiredPackages.every((pkg) => typeof pkg.checksum === 'string' && !isPlaceholderChecksum(pkg.checksum));
  if (allRequiredChecksumsMaterialized) {
    manifest.security = {
      ...(manifest.security ?? {}),
      checksumRequired: true,
      sbomRequired: true,
    };
    evidence.checksumRequiredEnabled = true;
  }

  await mkdir(path.dirname(evidencePath), { recursive: true });

  for await (const match of glob('dist/standalone/*.tar.gz', { cwd: workspaceRoot })) {
    const absolutePath = path.join(workspaceRoot, match);
    if (!existsSync(absolutePath)) {
      continue;
    }
    const checksum = await sha256File(absolutePath);
    evidence.auxiliaryArtifacts.push({
      packageId: 'linux-x64-standalone-server-tar-gz',
      artifactPath: match.split(path.sep).join('/'),
      checksumAlgorithm: 'SHA-256',
      checksum,
      runtimeTarget: 'server',
      deploymentProfile: 'standalone',
    });
    console.log(`[release-evidence] auxiliary ${match} (${checksum.slice(0, 12)}...)`);
    break;
  }

  evidence.packages = mergeEvidencePackages(existingEvidence?.packages, evidence.packages);
  evidence.auxiliaryArtifacts = mergeAuxiliaryArtifacts(
    existingEvidence?.auxiliaryArtifacts,
    evidence.auxiliaryArtifacts,
  );
  evidence.warnings = mergeWarnings(existingEvidence?.warnings, evidence.warnings);
  if (existingEvidence?.generatedAt) {
    evidence.previousGeneratedAt = existingEvidence.generatedAt;
  }

  await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  await writeFile(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`, 'utf8');

  console.log(
    `[release-evidence] updated ${updatedCount} package checksum(s); evidence written to ${path.relative(workspaceRoot, evidencePath)}`,
  );

  if (strict && evidence.warnings.length > 0) {
    process.exit(1);
  }
}

main().catch((error) => {
  console.error(`[release-evidence] ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
});
