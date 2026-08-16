import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..', '..');
const iamRepoRoot = path.resolve(repoRoot, '..', 'sdkwork-iam');

function read(relativePath, root = repoRoot) {
  return fs.readFileSync(path.join(root, relativePath), 'utf8');
}

const bootstrapSource = read(
  'crates/sdkwork-api-drive-standalone-gateway/src/iam_application_bootstrap.rs',
);
const gatewayMain = read('crates/sdkwork-api-drive-standalone-gateway/src/main.rs');
const gatewayCargo = read('crates/sdkwork-api-drive-standalone-gateway/Cargo.toml');
const workspaceCargo = read('Cargo.toml');
const topologySource = read('scripts/lib/drive-topology.mjs');
const sharedBootstrapSource = read(
  'crates/sdkwork-iam-embedded-application-bootstrap/src/runtime.rs',
  iamRepoRoot,
);

assert.match(
  bootstrapSource,
  /ensure_tenant_application_from_app_root_with_env_and_fallback/u,
  'Drive IAM bootstrap must delegate to the shared embedded bootstrap crate.',
);

assert.match(
  gatewayMain,
  /ensure_drive_tenant_application_bootstrap/u,
  'Standalone gateway must provision the drive IAM tenant application before building the IAM router.',
);

assert.match(
  gatewayMain,
  /bootstrap_iam_database_from_env/u,
  'Standalone gateway must bootstrap IAM schema before tenant application provisioning.',
);

assert.match(
  gatewayCargo,
  /sdkwork-iam-embedded-application-bootstrap/u,
  'Standalone gateway must depend on sdkwork-iam-embedded-application-bootstrap.',
);

assert.match(
  workspaceCargo,
  /sdkwork-iam-embedded-application-bootstrap/u,
  'Workspace must include sdkwork-iam-embedded-application-bootstrap.',
);

assert.match(
  topologySource,
  /SDKWORK_APP_ROOT:\s*REPO_ROOT/u,
  'Dev topology must inject SDKWORK_APP_ROOT for embedded IAM bootstrap.',
);

assert.match(
  topologySource,
  /SDKWORK_IAM_APP_ROOT:\s*IAM_REPO_ROOT/u,
  'Dev topology must export SDKWORK_IAM_APP_ROOT at the sdkwork-iam repository root for IMF catalog materialization.',
);

assert.match(
  sharedBootstrapSource,
  /SDKWORK_DRIVE_APP_ROOT/u,
  'Shared embedded bootstrap must resolve SDKWORK_DRIVE_APP_ROOT.',
);

console.log('sdkwork-drive IAM application bootstrap standard passed.');
