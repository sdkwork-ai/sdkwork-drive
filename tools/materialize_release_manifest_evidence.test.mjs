#!/usr/bin/env node
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const toolPath = path.join(repoRoot, 'tools/materialize_release_manifest_evidence.mjs');

function isPlaceholderChecksum(checksum) {
  const sample = checksum.slice(0, 8);
  return checksum === sample.repeat(Math.ceil(checksum.length / sample.length)).slice(0, checksum.length);
}

async function withTempRepo(run) {
  const tempRoot = await mkdtemp(path.join(os.tmpdir(), 'sdkwork-drive-release-evidence-'));
  const manifestPath = path.join(tempRoot, 'sdkwork.app.config.json');
  const manifest = {
    schemaVersion: 3,
    kind: 'sdkwork.app',
    security: { checksumRequired: false, signatureRequired: false, sbomRequired: true },
    artifacts: {
      installConfig: {
        packages: [
          {
            id: 'web-universal-cloud-browser-zip',
            enabled: true,
            checksumAlgorithm: 'SHA-256',
            checksum: '615da704615da704615da704615da704615da704615da704615da704615da704',
            metadata: {},
          },
        ],
      },
    },
  };
  await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  const artifactDir = path.join(tempRoot, 'dist', 'release', 'STABLE', '0.1.0');
  await mkdir(artifactDir, { recursive: true });
  const artifactPath = path.join(artifactDir, 'web.zip');
  await writeFile(artifactPath, 'sdkwork-drive-web-bundle-test-content', 'utf8');
  await run({ tempRoot, manifestPath, artifactPath });
}

await withTempRepo(async ({ tempRoot, manifestPath }) => {
  const result = spawnSync(
    process.execPath,
    [toolPath],
    {
      cwd: tempRoot,
      env: {
        ...process.env,
      },
      encoding: 'utf8',
    },
  );
  assert.equal(result.status, 0, result.stderr || result.stdout);

  const updatedManifest = JSON.parse(await readFile(manifestPath, 'utf8'));
  const pkg = updatedManifest.artifacts.installConfig.packages[0];
  assert.equal(pkg.checksumAlgorithm, 'SHA-256');
  assert.ok(!isPlaceholderChecksum(pkg.checksum));
  assert.match(pkg.metadata.releaseArtifactPath, /web\.zip$/);

  const evidencePath = path.join(tempRoot, 'target', 'release', 'release-evidence.json');
  assert.equal(existsSync(evidencePath), true);
  const evidence = JSON.parse(await readFile(evidencePath, 'utf8'));
  assert.equal(evidence.packages.length, 1);
  assert.equal(evidence.packages[0].placeholderReplaced, true);
  assert.equal(updatedManifest.security.checksumRequired, true);
});

await withTempRepo(async ({ tempRoot, manifestPath }) => {
  const evidenceDir = path.join(tempRoot, 'target', 'release');
  await mkdir(evidenceDir, { recursive: true });
  await writeFile(
    path.join(evidenceDir, 'release-evidence.json'),
    `${JSON.stringify({
      schemaVersion: 1,
      generatedAt: '2026-06-23T00:00:00.000Z',
      packages: [
        {
          id: 'windows-x64-standalone-desktop-zip',
          artifactPath: 'dist/release/STABLE/0.1.0/windows/x64/app.zip',
          checksumAlgorithm: 'SHA-256',
          checksum: 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        },
      ],
      auxiliaryArtifacts: [],
      warnings: ['package macos-universal-standalone-desktop-dmg is deferred to cross-platform CI release builds'],
    }, null, 2)}\n`,
    'utf8',
  );

  const result = spawnSync(process.execPath, [toolPath], { cwd: tempRoot, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr || result.stdout);

  const evidence = JSON.parse(await readFile(path.join(evidenceDir, 'release-evidence.json'), 'utf8'));
  assert.equal(evidence.packages.length, 2);
  assert.ok(evidence.packages.some((entry) => entry.id === 'web-universal-cloud-browser-zip'));
  assert.ok(evidence.packages.some((entry) => entry.id === 'windows-x64-standalone-desktop-zip'));
  assert.ok(evidence.warnings.some((warning) => warning.includes('macos-universal')));
});

{
  const tempRoot = await mkdtemp(path.join(os.tmpdir(), 'sdkwork-drive-release-evidence-deferred-'));
  await writeFile(
    path.join(tempRoot, 'sdkwork.app.config.json'),
    `${JSON.stringify({
      schemaVersion: 3,
      kind: 'sdkwork.app',
      security: { checksumRequired: true, signatureRequired: false, sbomRequired: true },
      artifacts: {
        installConfig: {
          packages: [
            {
              id: 'macos-universal-standalone-desktop-dmg',
              enabled: true,
              checksumAlgorithm: 'SHA-256',
              checksum: '5ceffa105ceffa105ceffa105ceffa105ceffa105ceffa105ceffa105ceffa10',
              metadata: {
                releaseBuildDeferred: true,
              },
            },
          ],
        },
      },
    }, null, 2)}\n`,
    'utf8',
  );

  const result = spawnSync(process.execPath, [toolPath, '--root', tempRoot], { cwd: tempRoot, encoding: 'utf8' });
  assert.notEqual(result.status, 0, result.stdout);
  assert.match(result.stderr, /must not keep placeholder checksum fields while releaseBuildDeferred is true/);
}

console.log('materialize_release_manifest_evidence.test.mjs passed');
