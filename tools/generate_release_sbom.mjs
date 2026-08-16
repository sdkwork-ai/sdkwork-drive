#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const WORKSPACE_ROOT = path.resolve(__dirname, '..');

function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, 'utf8'));
}

function sha256File(filePath) {
  const digest = createHash('sha256');
  digest.update(readFileSync(filePath));
  return digest.digest('hex');
}

function collectCargoPackages(lockText) {
  const packages = [];
  const blocks = lockText.split('\n\n').slice(1);
  for (const block of blocks) {
    const nameMatch = block.match(/^name = "([^"]+)"/m);
    const versionMatch = block.match(/^version = "([^"]+)"/m);
    if (!nameMatch || !versionMatch) {
      continue;
    }
    packages.push({
      name: nameMatch[1],
      version: versionMatch[1],
      ecosystem: 'cargo',
    });
  }
  return packages;
}

function main() {
  const outputPath = path.resolve(
    WORKSPACE_ROOT,
    process.argv[2] ?? 'target/release/sbom.sdkwork-drive.json',
  );
  const packageJsonPath = path.join(WORKSPACE_ROOT, 'package.json');
  const cargoLockPath = path.join(WORKSPACE_ROOT, 'Cargo.lock');
  const appManifestPath = path.join(WORKSPACE_ROOT, 'sdkwork.app.config.json');

  if (!existsSync(packageJsonPath) || !existsSync(cargoLockPath)) {
    throw new Error('package.json and Cargo.lock are required to generate SBOM');
  }

  const packageJson = readJson(packageJsonPath);
  const appManifest = existsSync(appManifestPath) ? readJson(appManifestPath) : null;
  const cargoLock = readFileSync(cargoLockPath, 'utf8');
  const components = [
    {
      name: packageJson.name,
      version: appManifest?.release?.currentVersion ?? '0.0.0',
      type: 'application',
      ecosystem: 'node',
      purl: `pkg:npm/${packageJson.name}`,
      hashes: [{ alg: 'SHA-256', content: sha256File(packageJsonPath) }],
    },
    ...collectCargoPackages(cargoLock).map((pkg) => ({
      ...pkg,
      type: 'library',
      purl: `pkg:cargo/${pkg.name}@${pkg.version}`,
    })),
  ];

  const sbom = {
    bomFormat: 'CycloneDX',
    specVersion: '1.5',
    version: 1,
    metadata: {
      timestamp: new Date().toISOString(),
      component: {
        type: 'application',
        name: appManifest?.app?.key ?? packageJson.name,
        version: appManifest?.release?.currentVersion ?? '0.0.0',
      },
      tools: [{ vendor: 'SDKWork', name: 'generate_release_sbom.mjs', version: '1.0.0' }],
    },
    components,
  };

  mkdirSync(path.dirname(outputPath), { recursive: true });
  writeFileSync(outputPath, `${JSON.stringify(sbom, null, 2)}\n`, 'utf8');
  console.log(`[sdkwork-drive] SBOM written to ${outputPath}`);
}

main();
