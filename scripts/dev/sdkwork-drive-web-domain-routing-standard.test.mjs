import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const readJson = (relativePath) => JSON.parse(readFileSync(path.join(repoRoot, relativePath), 'utf8'));
const readText = (relativePath) => readFileSync(path.join(repoRoot, relativePath), 'utf8');

const deployment = readJson('etc/sdkwork.deployment.config.json');
const topology = readJson('specs/topology.spec.json');
const deployManifest = readText('deployments/deploy.yaml');

// APP_RUNTIME_TOPOLOGY_NAMING.md section 9.2: drive role host on sdkwork.com;
// application and platform surfaces are distinct origins (TECH-drive-topology-standard).
const expectedOrigins = {
  development: 'http://drive-dev.sdkwork.com:3900',
  test: 'https://drive-test.sdkwork.com',
  staging: 'https://drive-staging.sdkwork.com',
  production: 'https://drive.sdkwork.com',
};
const expectedCloudApiBaseUrls = {
  development: 'https://api-dev.sdkwork.com',
  test: 'https://api-test.sdkwork.com',
  staging: 'https://api-staging.sdkwork.com',
  production: 'https://api.sdkwork.com',
};

for (const [environment, expectedOrigin] of Object.entries(expectedOrigins)) {
  const canonical = deployment.environments?.[environment];
  assert.ok(canonical, `deployment config must declare ${environment}`);
  assert.equal(canonical.applicationOrigin, expectedOrigin);
  assert.equal(canonical.cloudApiBaseUrl, expectedCloudApiBaseUrls[environment]);
  const parsed = new URL(expectedOrigin);
  assert.doesNotMatch(parsed.hostname, /^api(?:-|\.)/u);
}

const publicHost = topology.cloudPublicHosts?.['application.public-ingress'];
assert.ok(publicHost, 'topology must register application.public-ingress');
assert.equal(publicHost.httpHost, 'drive.sdkwork.com');
assert.equal(publicHost.environments?.development?.httpHost, 'drive-dev.sdkwork.com');
assert.equal(publicHost.environments?.test?.httpHost, 'drive-test.sdkwork.com');
assert.equal(publicHost.environments?.staging?.httpHost, 'drive-staging.sdkwork.com');
assert.equal(
  topology.cloudPublicHosts?.['platform.api-gateway']?.environments?.test?.httpHost,
  'api-test.sdkwork.com',
);

// Drive-owned SDK keys resolve to the drive origin; platform keys stay on api.
const DRIVE_FACE_KEYS = [
  'SDKWORK_DRIVE_APPLICATION_PUBLIC_HTTP_URL',
  'VITE_DRIVE_PC_APPLICATION_PUBLIC_HTTP_URL',
  'VITE_DRIVE_PC_DRIVE_APP_API_BASE_URL',
  'VITE_DRIVE_PC_APP_API_BASE_URL',
  'VITE_DRIVE_PC_DRIVE_ADMIN_STORAGE_API_BASE_URL',
  'VITE_DRIVE_PC_BACKEND_API_BASE_URL',
];
const PLATFORM_KEYS = [
  'SDKWORK_DRIVE_PLATFORM_API_GATEWAY_HTTP_URL',
  'VITE_DRIVE_PC_PLATFORM_API_GATEWAY_HTTP_URL',
  'VITE_DRIVE_PC_APPBASE_APP_API_BASE_URL',
];

for (const environment of ['development', 'test', 'staging', 'production']) {
  const profileSource = readText(`etc/topology/cloud.${environment}.env`);
  const driveOrigin = environment === 'development'
    ? 'http://drive-dev.sdkwork.com:3900'
    : expectedOrigins[environment].replace(/\/$/u, '');
  const apiOrigin = expectedCloudApiBaseUrls[environment].replace(/\/$/u, '');
  for (const key of DRIVE_FACE_KEYS) {
    const line = profileSource.split('\n').find((l) => l.startsWith(`${key}=`));
    assert.ok(line, `cloud ${environment} must declare ${key}`);
    assert.ok(line.includes(driveOrigin), `cloud ${environment} ${key} must use ${driveOrigin}: ${line}`);
  }
  for (const key of PLATFORM_KEYS) {
    const line = profileSource.split('\n').find((l) => l.startsWith(`${key}=`));
    assert.ok(line, `cloud ${environment} must declare ${key}`);
    assert.ok(line.includes(apiOrigin), `cloud ${environment} ${key} must use ${apiOrigin}: ${line}`);
  }
}

// Standalone profiles fold SDK base URLs to loopback and must not reference
// cloud hostnames.
for (const environment of ['development', 'test', 'staging', 'production']) {
  const profileSource = readText(`etc/topology/standalone.${environment}.env`);
  assert.doesNotMatch(profileSource, /\.sdkwork\.com/u, `standalone ${environment} must not reference cloud hostnames`);
  assert.match(profileSource, /127\.0\.0\.1/u, `standalone ${environment} must fold to loopback URLs`);
}

// Retired placeholder domains must not appear in source config.
const topologyEnvFiles = [
  'cloud.development.env', 'cloud.test.env', 'cloud.staging.env', 'cloud.production.env',
  'standalone.development.env', 'standalone.test.env', 'standalone.staging.env', 'standalone.production.env',
];
const workspaceConfigText = [
  ...topologyEnvFiles.map((name) => readText(`etc/topology/${name}`)),
  readText('etc/sdkwork.deployment.config.json'),
  readText('specs/topology.spec.json'),
  deployManifest,
].join('\n');
assert.doesNotMatch(workspaceConfigText, /\.invalid/u, 'placeholder .invalid domains are retired');
assert.doesNotMatch(workspaceConfigText, /drive\.example\.com/u, 'drive.example.com is retired');

// deploy.yaml cloud expose domains must belong to the registered host set.
const cloudSection = deployManifest.split('standalone.production:')[0] ?? deployManifest;
const hostSets = new Set(Object.values(expectedOrigins).map((url) => new URL(url).hostname));
const exposeBlocks = [...cloudSection.matchAll(/domain:\s*([^\s]+)[\s\S]*?(?=\n\s{4}- domain:|\n\s{2}cloud\.|\n\s{2}standalone\.|$)/gu)];
assert.ok(exposeBlocks.length >= 3, 'deploy.yaml must declare cloud test/staging/production exposes');
for (const block of exposeBlocks) {
  assert.ok(hostSets.has(block[1]), `expose domain ${block[1]} must be registered in cloudPublicHosts`);
}

console.log('sdkwork-drive web domain routing standard passed');
