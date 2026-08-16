#!/usr/bin/env node
/**
 * Validates deployment descriptor presence for SDKWork Drive.
 * Governed by DEPLOYMENT_SPEC.md and SDKWORK_DEPLOY_SPEC.md.
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadDeployManifest, loadTopology } from '../../sdkwork-specs/tools/deploy/load-manifest.mjs';
import { validateDeploy } from '../../sdkwork-specs/tools/deploy/validate.mjs';

const defaultRepoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function parseArgs(argv) {
  const rootFlagIndex = argv.indexOf('--root');
  if (rootFlagIndex >= 0 && !argv[rootFlagIndex + 1]) {
    console.error('[deploy-validate] --root requires a workspace path');
    process.exit(1);
  }
  return {
    repoRoot: rootFlagIndex >= 0 ? path.resolve(argv[rootFlagIndex + 1]) : defaultRepoRoot,
  };
}

const { repoRoot } = parseArgs(process.argv.slice(2));
const strictDeployValidation =
  process.env.SDKWORK_DEPLOY_VALIDATION === 'strict' ||
  process.env.SDKWORK_RELEASE_VALIDATION === 'strict';
const failures = [];
const warnings = [];
const releaseDigestPlaceholder = 'REPLACE_WITH_RELEASE_DIGEST';
const sha256ImageDigestPattern = /@sha256:[a-f0-9]{64}$/iu;

function repoRelativeSlash(relativePath) {
  return String(relativePath ?? '').replace(/\\/gu, '/').replace(/^\.\//u, '');
}

function deploymentBlock(manifest, deploymentName) {
  const start = manifest.indexOf(`name: ${deploymentName}`);
  if (start < 0) {
    return null;
  }
  const end = manifest.indexOf('\n---', start);
  return end < 0 ? manifest.slice(start) : manifest.slice(start, end);
}

function requirePath(relativePath, reason) {
  const absolutePath = path.join(repoRoot, relativePath);
  if (!fs.existsSync(absolutePath)) {
    failures.push(`${relativePath} is required (${reason})`);
  }
}

function envFileExists(relativePath) {
  return fs.existsSync(path.join(repoRoot, relativePath));
}

function validateTopologyProfileId(profileId, topology, location) {
  const allowedDeploymentProfiles = new Set(
    topology.vocabulary?.deploymentProfile?.allowed ?? ['standalone', 'cloud'],
  );
  const allowedEnvironments = new Set(
    topology.vocabulary?.environment?.allowed ?? ['development', 'test', 'staging', 'production'],
  );
  const segments = String(profileId ?? '').split('.');
  if (
    segments.length !== 2 ||
    !allowedDeploymentProfiles.has(segments[0]) ||
    !allowedEnvironments.has(segments[1])
  ) {
    failures.push(
      `${location}: profile "${profileId}" must use <deploymentProfile>.<environment> with deploymentProfile in ` +
        `${[...allowedDeploymentProfiles].join(', ')} and environment in ${[...allowedEnvironments].join(', ')}`,
    );
  }
}

function validateDeployTopologyContract() {
  let doc;
  let topology;
  try {
    ({ doc } = loadDeployManifest(repoRoot));
    topology = loadTopology(repoRoot);
  } catch (error) {
    failures.push(`deployments/deploy.yaml topology validation failed: ${error.message}`);
    return;
  }

  const profileFiles = topology.profileFiles;
  if (!profileFiles || typeof profileFiles !== 'object') {
    failures.push('specs/topology.spec.json must declare profileFiles for deploy profile validation');
    return;
  }

  const profiles =
    doc.profiles && typeof doc.profiles === 'object'
      ? doc.profiles
      : {
          [doc.profile]: {
            install: doc.install,
            expose: doc.expose,
            packages: doc.packages,
            overrides: doc.overrides,
          },
        };

  if (doc.profiles && !Object.hasOwn(profiles, doc.defaultProfile)) {
    failures.push(`defaultProfile "${doc.defaultProfile}" must exist in deployments/deploy.yaml profiles`);
  }

  for (const [profileId, block] of Object.entries(profiles)) {
    validateTopologyProfileId(profileId, topology, `deployments/deploy.yaml profiles.${profileId}`);

    const expectedEnv = profileFiles[profileId];
    if (!expectedEnv) {
      failures.push(`profile "${profileId}" is not listed in topology.profileFiles`);
    } else if (!envFileExists(expectedEnv)) {
      failures.push(`${repoRelativeSlash(expectedEnv)} does not exist`);
    }

    const topologyOverride = block?.overrides?.topology;
    if (!topologyOverride || typeof topologyOverride !== 'object') {
      failures.push(`profile "${profileId}" must declare overrides.topology with spec, profile, and env`);
      continue;
    }

    const topologySpec = repoRelativeSlash(topologyOverride.spec);
    if (topologySpec !== 'specs/topology.spec.json') {
      failures.push(
        `profile "${profileId}" overrides.topology.spec must be specs/topology.spec.json, got "${topologyOverride.spec}"`,
      );
    }

    if (topologyOverride.profile !== profileId) {
      failures.push(
        `profile "${profileId}" overrides.topology.profile must match profile key, got "${topologyOverride.profile}"`,
      );
    }

    const declaredEnv = repoRelativeSlash(topologyOverride.env);
    if (!declaredEnv) {
      failures.push(`profile "${profileId}" must declare overrides.topology.env`);
    } else {
      if (expectedEnv && declaredEnv !== repoRelativeSlash(expectedEnv)) {
        failures.push(
          `profile "${profileId}" overrides.topology.env must be ${repoRelativeSlash(expectedEnv)}, got ${declaredEnv}`,
        );
      }
      if (!envFileExists(declaredEnv)) {
        failures.push(`${declaredEnv} does not exist`);
      }
    }
  }
}

function extractImageReferences(manifest) {
  return [...manifest.matchAll(/^\s*image:\s*["']?([^"'\s#]+)["']?/gmu)].map(
    (match) => match[1],
  );
}

function validateKubernetesImageDigests(manifest) {
  const imageReferences = extractImageReferences(manifest);
  if (imageReferences.length === 0) {
    failures.push('deployments/kubernetes/drive-services.yaml must declare container image references');
    return;
  }

  const placeholderReferences = imageReferences.filter((imageRef) =>
    imageRef.includes(releaseDigestPlaceholder),
  );
  if (placeholderReferences.length > 0) {
    const message =
      `deployments/kubernetes/drive-services.yaml contains ${releaseDigestPlaceholder}; ` +
      'replace placeholders with immutable release evidence digests before production deployment';
    if (strictDeployValidation) {
      failures.push(`${message} (strict deployment validation)`);
    } else {
      warnings.push(message);
    }
  }

  if (!strictDeployValidation) {
    return;
  }

  for (const imageRef of imageReferences) {
    if (!sha256ImageDigestPattern.test(imageRef)) {
      failures.push(
        `Kubernetes image "${imageRef}" must use an immutable @sha256:<64 hex> digest in strict deployment validation`,
      );
    }
  }
}

function envValuePattern(envName, expectedValue) {
  return new RegExp(
    `name:\\s*${envName}[\\s\\S]*?value:\\s*["']?${expectedValue}["']?`,
    'u',
  );
}

function validateRedisRateLimit(deploymentName) {
  const block = deploymentBlock(kubernetesManifest, deploymentName);
  if (!block) {
    return;
  }
  if (!envValuePattern('SDKWORK_DRIVE_RATE_LIMIT_BACKEND', 'redis').test(block)) {
    failures.push(
      `${deploymentName} Deployment must set SDKWORK_DRIVE_RATE_LIMIT_BACKEND=redis for production multi-instance rate limiting`,
    );
  }
  if (
    !/name:\s*SDKWORK_DRIVE_RATE_LIMIT_REDIS_URL[\s\S]*?secretKeyRef:[\s\S]*?name:\s*sdkwork-drive-rate-limit[\s\S]*?key:\s*SDKWORK_DRIVE_RATE_LIMIT_REDIS_URL/u.test(
      block,
    )
  ) {
    failures.push(
      `${deploymentName} Deployment must source SDKWORK_DRIVE_RATE_LIMIT_REDIS_URL from sdkwork-drive-rate-limit secret`,
    );
  }
  if (!envValuePattern('SDKWORK_DRIVE_RATE_LIMIT_FAIL_CLOSED', 'true').test(block)) {
    failures.push(
      `${deploymentName} Deployment must set SDKWORK_DRIVE_RATE_LIMIT_FAIL_CLOSED=true for production multi-instance rate limiting`,
    );
  }
}

requirePath('deployments/deploy.yaml', 'SDKWORK_DEPLOY_SPEC.md deployctl contract');
validateDeployTopologyContract();

const deployResult = validateDeploy(
  repoRoot,
  process.env.SDKWORK_DRIVE_PROFILE_ID ?? process.env.SDKWORK_DEPLOY_PROFILE,
);
for (const warning of deployResult.warnings ?? []) {
  warnings.push(warning);
}
if (!deployResult.ok) {
  for (const error of deployResult.errors ?? []) {
    failures.push(`deploy.yaml: ${error}`);
  }
}

requirePath('deployments/kubernetes/drive-services.yaml', 'cloud HA topology');

const kubernetesManifest = fs.readFileSync(
  path.join(repoRoot, 'deployments/kubernetes/drive-services.yaml'),
  'utf8',
);
if (!/kind:\s*Ingress/u.test(kubernetesManifest)) {
  failures.push('deployments/kubernetes/drive-services.yaml must declare an Ingress resource');
}
if (!/nginx\.ingress\.kubernetes\.io\/limit-rps/u.test(kubernetesManifest)) {
  failures.push('drive Kubernetes Ingress must configure nginx limit-rps edge rate limiting');
}
validateKubernetesImageDigests(kubernetesManifest);
for (const deploymentName of [
  'sdkwork-drive-app-api',
  'sdkwork-drive-backend-api',
  'sdkwork-drive-open-api',
]) {
  const block = deploymentBlock(kubernetesManifest, deploymentName);
  if (!block) {
    failures.push(`deployments/kubernetes/drive-services.yaml must declare Deployment ${deploymentName}`);
    continue;
  }
  if (!/OTEL_EXPORTER_OTLP_ENDPOINT/u.test(block)) {
    failures.push(`${deploymentName} Deployment must configure OTEL_EXPORTER_OTLP_ENDPOINT`);
  }
  if (!/OTEL_SERVICE_NAME/u.test(block)) {
    failures.push(`${deploymentName} Deployment must configure OTEL_SERVICE_NAME`);
  }
}
for (const deploymentName of ['sdkwork-drive-app-api', 'sdkwork-drive-backend-api']) {
  const block = deploymentBlock(kubernetesManifest, deploymentName);
  if (!block) {
    continue;
  }
  if (!/sdkwork-drive-iam/u.test(block)) {
    failures.push(
      `${deploymentName} Deployment must mount sdkwork-drive-iam secrets for production JWT validation`,
    );
  }
}
for (const deploymentName of ['sdkwork-drive-open-api', 'sdkwork-drive-admin-storage-api']) {
  const block = deploymentBlock(kubernetesManifest, deploymentName);
  if (!block) {
    continue;
  }
  if (!/sdkwork-drive-iam/u.test(block)) {
    failures.push(
      `${deploymentName} Deployment must mount sdkwork-drive-iam secrets for IAM database session resolution`,
    );
  }
}
for (const [deploymentName, envName] of [
  ['sdkwork-drive-app-api', 'SDKWORK_DRIVE_APP_API_RATE_LIMIT_MAX_REQUESTS'],
  ['sdkwork-drive-backend-api', 'SDKWORK_DRIVE_BACKEND_API_RATE_LIMIT_MAX_REQUESTS'],
  ['sdkwork-drive-open-api', 'SDKWORK_DRIVE_OPEN_API_RATE_LIMIT_MAX_REQUESTS'],
  ['sdkwork-drive-admin-storage-api', 'SDKWORK_DRIVE_ADMIN_STORAGE_API_RATE_LIMIT_MAX_REQUESTS'],
]) {
  const block = deploymentBlock(kubernetesManifest, deploymentName);
  if (!block) {
    continue;
  }
  if (!block.includes(envName)) {
    failures.push(`${deploymentName} Deployment must configure ${envName}`);
  }
  validateRedisRateLimit(deploymentName);
}
const appApiBlock = deploymentBlock(kubernetesManifest, 'sdkwork-drive-app-api');
if (appApiBlock && !/SDKWORK_DRIVE_UPLOAD_CONTENT_POLICY_MODE/u.test(appApiBlock)) {
  failures.push('sdkwork-drive-app-api Deployment must enforce upload content policy in production');
}
requirePath('deployments/nginx/drive-edge-rate-limit.conf.example', 'edge rate limiting');
requirePath('deployments/docker-compose.minio-test.yml', 'object storage dev profile');
requirePath('docs/runbooks/drive-production-operations.md', 'production runbook');
requirePath('docs/runbooks/drive-backup-disaster-recovery.md', 'backup and DR runbook');
requirePath('docs/guides/operator/pre-launch-checklist.md', 'pre-launch operator checklist');
requirePath('deployments/docker/Dockerfile.app-api', 'container build descriptor');
requirePath('deployments/container/README.md', 'container packaging notes');

function readText(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

for (const service of [
  'sdkwork-drive-app-api.service',
  'sdkwork-drive-backend-api.service',
  'sdkwork-drive-open-api.service',
  'sdkwork-drive-admin-storage-api.service',
  'sdkwork-drive-install-worker.service',
  'sdkwork-api-drive-standalone-gateway.service',
]) {
  requirePath(`deployments/systemd/${service}`, 'systemd deployment');
  const unitPath = path.join(repoRoot, `deployments/systemd/${service}`);
  if (!fs.existsSync(unitPath)) {
    continue;
  }
  const unit = readText(`deployments/systemd/${service}`);
  if (!unit.includes('SDKWORK_DRIVE_DEPLOYMENT_PROFILE=')) {
    failures.push(
      `${service} must set SDKWORK_DRIVE_DEPLOYMENT_PROFILE for metrics and tracing labels`,
    );
  }
  if (unit.includes('SDKWORK_DRIVE_DEPLOYMENT_MODE')) {
    failures.push(`${service} must not use deprecated SDKWORK_DRIVE_DEPLOYMENT_MODE`);
  }
}

for (const warning of warnings) {
  console.warn(`[deploy-validate] warning: ${warning}`);
}

if (failures.length > 0) {
  console.error('[deploy-validate] failures:');
  for (const failure of failures) {
    console.error(`  - ${failure}`);
  }
  process.exit(1);
}

console.log('[deploy-validate] deployment descriptors ok');
