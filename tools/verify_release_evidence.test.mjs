#!/usr/bin/env node
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const toolPath = path.join(repoRoot, 'tools/verify_release_evidence.mjs');

function baseManifest(packagePatch) {
  return {
    schemaVersion: 3,
    kind: 'sdkwork.app',
    security: {
      checksumRequired: true,
      sbomRequired: true,
    },
    artifacts: {
      installConfig: {
        packages: [
          {
            id: 'macos-universal-standalone-desktop-dmg',
            enabled: true,
            packageFormat: 'DMG',
            platform: 'DESKTOP_MACOS',
            metadata: {
              releaseBuildDeferred: true,
            },
            ...packagePatch,
          },
        ],
      },
    },
  };
}

async function createTempWorkspace(manifest) {
  const tempRoot = await mkdtemp(path.join(os.tmpdir(), 'sdkwork-drive-release-verify-'));
  await mkdir(path.join(tempRoot, 'target', 'release'), { recursive: true });
  await writeFile(
    path.join(tempRoot, 'sdkwork.app.config.json'),
    `${JSON.stringify(manifest, null, 2)}\n`,
    'utf8',
  );
  await writeFile(
    path.join(tempRoot, 'target', 'release', 'release-evidence.json'),
    `${JSON.stringify({ schemaVersion: 1, packages: [], auxiliaryArtifacts: [], warnings: [] }, null, 2)}\n`,
    'utf8',
  );
  await writeFile(
    path.join(tempRoot, 'target', 'release', 'catalog-media-evidence.json'),
    `${JSON.stringify({ schemaVersion: 1, catalogMedia: [] }, null, 2)}\n`,
    'utf8',
  );
  await writeFile(
    path.join(tempRoot, 'target', 'release', 'sbom.sdkwork-drive.json'),
    `${JSON.stringify({ bomFormat: 'CycloneDX' }, null, 2)}\n`,
    'utf8',
  );
  return tempRoot;
}

function runVerify(tempRoot) {
  return spawnSync(process.execPath, [toolPath, '--root', tempRoot], {
    cwd: tempRoot,
    encoding: 'utf8',
  });
}

{
  const tempRoot = await createTempWorkspace(baseManifest({
    checksumAlgorithm: 'SHA-256',
    checksum: '5ceffa105ceffa105ceffa105ceffa105ceffa105ceffa105ceffa105ceffa10',
  }));
  const result = runVerify(tempRoot);
  assert.notEqual(result.status, 0, result.stdout);
  assert.match(result.stderr, /must not keep placeholder checksum fields while releaseBuildDeferred is true/);
}

{
  const tempRoot = await createTempWorkspace(baseManifest({}));
  const result = runVerify(tempRoot);
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stdout, /passed/);
}

console.log('verify_release_evidence.test.mjs passed');
