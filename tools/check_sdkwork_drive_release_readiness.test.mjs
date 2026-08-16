#!/usr/bin/env node
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const toolPath = path.join(repoRoot, 'tools/check_sdkwork_drive_release_readiness.mjs');

function baseManifest(packagePatch, manifestPatch = {}) {
  return {
    schemaVersion: 3,
    kind: 'sdkwork.app',
    app: {},
    backend: {},
    runtime: {},
    media: {},
    publish: { releaseMaturity: 'BETA', status: 'DRAFT' },
    environments: {},
    artifacts: {
      installConfig: {
        packages: [
          {
            id: 'macos-universal-standalone-desktop-dmg',
            packageFormat: 'DMG',
            platform: 'DESKTOP_MACOS',
            enabled: true,
            metadata: {
              releaseBuildDeferred: true,
            },
            ...packagePatch,
          },
        ],
      },
    },
    release: {},
    security: {
      checksumRequired: true,
      signatureRequired: false,
      signatureDeferred: true,
      sbomRequired: true,
    },
    ...manifestPatch,
  };
}

async function createTempWorkspace(manifest) {
  const tempRoot = await mkdtemp(path.join(os.tmpdir(), 'sdkwork-drive-release-readiness-'));
  await mkdir(path.join(tempRoot, 'tools'), { recursive: true });
  if (manifest.metadata?.deploymentConfig) {
    const deploymentConfigPath = path.join(tempRoot, manifest.metadata.deploymentConfig);
    await mkdir(path.dirname(deploymentConfigPath), { recursive: true });
    await writeFile(deploymentConfigPath, '{}\n', 'utf8');
  }
  await writeFile(path.join(tempRoot, 'sdkwork.workflow.json'), '{}\n', 'utf8');
  await writeFile(path.join(tempRoot, 'tools/generate_release_sbom.mjs'), '#!/usr/bin/env node\n', 'utf8');
  await writeFile(
    path.join(tempRoot, 'sdkwork.app.config.json'),
    `${JSON.stringify(manifest, null, 2)}\n`,
    'utf8',
  );
  return tempRoot;
}

function runReadiness(tempRoot, env = {}) {
  return spawnSync(process.execPath, [toolPath, '--root', tempRoot], {
    cwd: tempRoot,
    env: {
      ...process.env,
      ...env,
    },
    encoding: 'utf8',
  });
}

{
  const manifest = baseManifest({});
  delete manifest.environments;
  manifest.metadata = {
    deploymentConfig: 'etc/sdkwork.deployment.config.json',
  };
  const tempRoot = await createTempWorkspace(manifest);
  const result = runReadiness(tempRoot);
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stdout, /passed \(development mode/);
}

{
  const manifest = baseManifest({});
  delete manifest.environments;
  const tempRoot = await createTempWorkspace(manifest);
  const result = runReadiness(tempRoot);
  assert.notEqual(result.status, 0, result.stdout);
  assert.match(result.stderr, /must include environments or metadata\.deploymentConfig/);
}

{
  const tempRoot = await createTempWorkspace(baseManifest({
    checksumAlgorithm: 'SHA-256',
    checksum: '5ceffa105ceffa105ceffa105ceffa105ceffa105ceffa105ceffa105ceffa10',
  }));
  const result = runReadiness(tempRoot);
  assert.notEqual(result.status, 0, result.stdout);
  assert.match(result.stderr, /must not keep placeholder checksum fields while releaseBuildDeferred is true/);
}

{
  const tempRoot = await createTempWorkspace(baseManifest({}));
  const result = runReadiness(tempRoot);
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stderr, /materialize checksum evidence on the target runner/);
  assert.match(result.stdout, /passed \(development mode/);
}

{
  const tempRoot = await createTempWorkspace(baseManifest({}));
  const result = runReadiness(tempRoot, { SDKWORK_RELEASE_VALIDATION: 'strict' });
  assert.notEqual(result.status, 0, result.stdout);
  assert.match(result.stderr, /security\.signatureRequired must be true/);
  assert.match(result.stderr, /materialize checksum evidence on the target runner/);
}

{
  const tempRoot = await createTempWorkspace(
    baseManifest(
      {
        checksumAlgorithm: 'SHA-256',
        checksum: '2f1ed640e515b714fa835c13854f830a7709d2081fc22dd3584d7af5bbda49be',
        metadata: {
          releaseBuildMaterializedAt: '2026-07-08T00:00:00.000Z',
        },
      },
      {
        media: {
          icons: {
            primary: {
              id: 'sdkwork-drive-primary-icon',
              metadata: {
                generatedPlaceholder: true,
                catalogMediaDeferred: true,
              },
            },
          },
        },
        security: {
          checksumRequired: true,
          signatureRequired: true,
          sbomRequired: true,
        },
      },
    ),
  );
  const result = runReadiness(tempRoot, { SDKWORK_RELEASE_VALIDATION: 'strict' });
  assert.notEqual(result.status, 0, result.stdout);
  assert.match(result.stderr, /media icon sdkwork-drive-primary-icon/);
}

console.log('check_sdkwork_drive_release_readiness.test.mjs passed');
