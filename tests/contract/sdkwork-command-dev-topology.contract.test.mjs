#!/usr/bin/env node
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  loadProfile,
  resolveStandaloneGatewayConfigPath,
  resolveTemporaryDatabaseDriverEnv,
} from '../../scripts/lib/drive-topology.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const commandPath = path.join(repoRoot, 'scripts/sdkwork-command.mjs');
const appCommandPath = path.resolve(repoRoot, '..', 'sdkwork-app-topology', 'scripts', 'sdkwork-app.mjs');

const applicationManifestPaths = [
  path.join(repoRoot, 'sdkwork.app.config.json'),
  ...readdirSync(path.join(repoRoot, 'apps'), { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(repoRoot, 'apps', entry.name, 'sdkwork.app.config.json')),
];

for (const manifestPath of applicationManifestPaths) {
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  const permissionScope = manifest.backend?.accessTokenPermissionScope;
  assert.ok(
    Array.isArray(permissionScope) && permissionScope.length > 0,
    `${path.relative(repoRoot, manifestPath)} must declare backend.accessTokenPermissionScope`,
  );
  assert.ok(
    permissionScope.every(
      (permission) =>
        typeof permission === 'string' && /^[a-z][a-z0-9]*(?:\.[a-z][a-z0-9]*)+$/u.test(permission),
    ),
    `${path.relative(repoRoot, manifestPath)} must use canonical permission identifiers`,
  );
}

const standaloneDevelopmentProfile = loadProfile('standalone.development');
assert.equal(
  standaloneDevelopmentProfile.SDKWORK_DATABASE_TEMPORARY_ANY_POOL_EXCEPTION,
  'true',
);
assert.deepEqual(
  resolveTemporaryDatabaseDriverEnv({
    SDKWORK_DATABASE_TEMPORARY_ANY_POOL_EXCEPTION: ' true ',
    SDKWORK_DATABASE_TEMPORARY_DRIVER_POOL_COUNT: '2',
  }),
  {
    SDKWORK_DATABASE_TEMPORARY_ANY_POOL_EXCEPTION: 'true',
    SDKWORK_DATABASE_TEMPORARY_DRIVER_POOL_COUNT: '2',
  },
);
assert.match(
  resolveStandaloneGatewayConfigPath({
    SDKWORK_DRIVE_STANDALONE_GATEWAY_ENVIRONMENT: 'development',
  }),
  /sdkwork-api-drive-standalone-gateway\.development\.toml\.example$/u,
);

function runCommand(args) {
  return spawnSync(process.execPath, [commandPath, ...args], {
    cwd: repoRoot,
    encoding: 'utf8',
    env: {
      ...process.env,
      SDKWORK_DRIVE_PLATFORM_API_GATEWAY_AUTOSTART: 'false',
      SDKWORK_DRIVE_STANDALONE_GATEWAY_CONFIG:
        'etc/sdkwork-api-drive-standalone-gateway.development.toml.example',
    },
  });
}

{
  const result = spawnSync(process.execPath, [
    appCommandPath,
    'dev',
    '--runtime-target', 'browser',
    '--deployment-profile', 'cloud',
    '--environment', 'development',
    '--dry-run',
  ], { cwd: repoRoot, encoding: 'utf8', env: process.env });

  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stdout, /cloud\.development/);
  assert.match(result.stdout, /drive-browser/);
  assert.doesNotMatch(result.stdout, /"role": "(?:standalone-gateway|application-cloud-gateway|platform-gateway|api-listener|database|redis|migration|seed|worker)"/u);
}

{
  const result = runCommand([
    'dev',
    '--runtime-target',
    'browser',
    '--database',
    'postgres',
    '--deployment-profile',
    'standalone',
    '--service-layout',
    'split-services',
    '--dry-run',
  ]);

  assert.notEqual(result.status, 0, 'public sdkwork-command dispatcher must reject --service-layout');
  assert.match(result.stderr, /--service-layout is internal topology detail/);
}

{
  const result = runCommand(['release:plan']);

  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.doesNotMatch(result.stderr, /TypeError: scriptArgs is not iterable/);
}

{
  const commandSource = readFileSync(commandPath, 'utf8');
  const driveDevSource = readFileSync(path.join(repoRoot, 'scripts/drive-dev.mjs'), 'utf8');
  const gatewayRunnerSource = readFileSync(
    path.join(repoRoot, 'scripts/gateway-standalone-run.mjs'),
    'utf8',
  );

  assert.doesNotMatch(
    commandSource,
    /gateway-standalone-pack\.mjs",\s*\[\s*"package",\s*"--skip-build"\s*\]/,
    'release:package must build the standalone gateway binary instead of assuming a pre-existing local target/release binary',
  );
  assert.match(driveDevSource, /resolveTemporaryDatabaseDriverEnv\(env\)/u);
  assert.match(gatewayRunnerSource, /resolveTemporaryDatabaseDriverEnv\(runtimeEnv\)/u);
}

console.log('sdkwork-command-dev-topology.contract.test.mjs passed');
