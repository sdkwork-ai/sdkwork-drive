#!/usr/bin/env node
/**
 * Release and supply-chain readiness gate for SDKWork Drive.
 * Follows RELEASE_SPEC.md and SUPPLY_CHAIN_SECURITY_SPEC.md.
 *
 * Default mode reports placeholder gaps as warnings so development can continue.
 * Strict mode fails on production-blocking signing, checksum, and catalog media gaps.
 * Strict command: SDKWORK_RELEASE_VALIDATION=strict pnpm check:release-readiness
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptRepoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const rootFlagIndex = process.argv.indexOf('--root');
const workspaceRoot = rootFlagIndex >= 0
  ? path.resolve(process.argv[rootFlagIndex + 1])
  : scriptRepoRoot;
const strict = process.env.SDKWORK_RELEASE_VALIDATION === 'strict';
const failures = [];
const warnings = [];

function readJson(relativePath) {
  const absolutePath = path.join(workspaceRoot, relativePath);
  if (!fs.existsSync(absolutePath)) {
    failures.push(`${relativePath} must exist`);
    return null;
  }
  return JSON.parse(fs.readFileSync(absolutePath, 'utf8'));
}

function fail(message) {
  failures.push(message);
}

function warn(message) {
  warnings.push(message);
}

function strictFail(message) {
  if (strict) {
    failures.push(message);
  } else {
    warnings.push(message);
  }
}

function validateEnvironmentAuthority(manifest) {
  if (manifest.environments !== undefined) {
    if (!manifest.environments || typeof manifest.environments !== 'object' || Array.isArray(manifest.environments)) {
      fail('sdkwork.app.config.json environments must be an object when present');
    }
    return;
  }

  const deploymentConfig = manifest.metadata?.deploymentConfig;
  if (typeof deploymentConfig !== 'string' || deploymentConfig.trim() === '') {
    fail('sdkwork.app.config.json must include environments or metadata.deploymentConfig');
    return;
  }

  const absoluteConfigPath = path.resolve(workspaceRoot, deploymentConfig);
  const relativeConfigPath = path.relative(workspaceRoot, absoluteConfigPath);
  if (path.isAbsolute(deploymentConfig) || relativeConfigPath.startsWith('..') || path.isAbsolute(relativeConfigPath)) {
    fail('metadata.deploymentConfig must be a safe application-relative path');
    return;
  }
  if (!fs.existsSync(absoluteConfigPath)) {
    fail(`metadata.deploymentConfig must reference an existing file: ${deploymentConfig}`);
  }
}

function isPlaceholderChecksum(checksum) {
  if (typeof checksum !== 'string' || checksum.length < 16) {
    return true;
  }
  const sample = checksum.slice(0, 8);
  return checksum === sample.repeat(Math.ceil(checksum.length / sample.length)).slice(0, checksum.length);
}

function collectPlaceholderMedia(manifest) {
  const placeholders = [];
  const media = manifest.media ?? {};
  const isDeferred = (item) => item?.metadata?.catalogMediaDeferred === true;
  for (const icon of [media.icons?.primary, ...(media.icons?.platform ?? [])].filter(Boolean)) {
    if (icon.metadata?.generatedPlaceholder === true && !isDeferred(icon)) {
      placeholders.push(`media icon ${icon.id ?? icon.purpose ?? 'unknown'}`);
    }
  }
  for (const screenshot of media.screenshots ?? []) {
    if (screenshot.metadata?.generatedPlaceholder === true && !isDeferred(screenshot)) {
      placeholders.push(`screenshot ${screenshot.id ?? screenshot.platform ?? 'unknown'}`);
    }
  }
  for (const preview of media.previews ?? []) {
    if (preview.metadata?.generatedPlaceholder === true && !isDeferred(preview)) {
      placeholders.push(`preview ${preview.id ?? preview.purpose ?? 'unknown'}`);
    }
  }
  return placeholders;
}

function collectDeferredCatalogMedia(manifest, repoRoot) {
  const deferred = [];
  const media = manifest.media ?? {};
  const inspect = (item, label) => {
    if (item?.metadata?.generatedPlaceholder !== true || item?.metadata?.catalogMediaDeferred !== true) {
      return;
    }
    const stagedPath = item.metadata?.stagedArtifactPath;
    if (stagedPath && repoRoot && fs.existsSync(path.join(repoRoot, stagedPath))) {
      deferred.push(`${label} (staged locally; CDN upload pending before ACTIVE publication)`);
      return;
    }
    deferred.push(`${label} (staging required before CDN upload)`);
  };
  inspect(media.icons?.primary, `media icon ${media.icons?.primary?.id ?? 'primary'}`);
  for (const screenshot of media.screenshots ?? []) {
    inspect(screenshot, `screenshot ${screenshot.id ?? screenshot.platform ?? 'unknown'}`);
  }
  for (const preview of media.previews ?? []) {
    inspect(preview, `preview ${preview.id ?? preview.purpose ?? 'unknown'}`);
  }
  return deferred;
}

function main() {
  const manifest = readJson('sdkwork.app.config.json');
  if (!manifest) {
    reportAndExit();
  }

  if (manifest.schemaVersion !== 3 || manifest.kind !== 'sdkwork.app') {
    fail('sdkwork.app.config.json must use schemaVersion 3 and kind sdkwork.app');
  }

  for (const section of ['app', 'backend', 'runtime', 'media', 'publish', 'artifacts', 'release', 'security']) {
    if (!manifest[section] || typeof manifest[section] !== 'object') {
      fail(`sdkwork.app.config.json must include required section ${section}`);
    }
  }
  validateEnvironmentAuthority(manifest);

  if (!fs.existsSync(path.join(workspaceRoot, 'sdkwork.workflow.json'))) {
    fail('sdkwork.workflow.json must exist for release governance');
  }

  if (!fs.existsSync(path.join(workspaceRoot, 'tools/generate_release_sbom.mjs'))) {
    fail('tools/generate_release_sbom.mjs must exist when security.sbomRequired is enabled');
  }

  const security = manifest.security ?? {};
  if (security.sbomRequired !== true) {
    strictFail('security.sbomRequired must be true for commercial release evidence');
  }

  if (security.checksumRequired !== true) {
    strictFail('security.checksumRequired must be true before production artifact publication');
  }

  if (security.signatureRequired !== true) {
    if (security.signatureDeferred === true) {
      strictFail('security.signatureRequired must be true; protected CI signing credentials are still deferred');
    } else {
      strictFail('security.signatureRequired must be true before externally distributed desktop/web packages ship');
    }
  }

  const packages = manifest.artifacts?.installConfig?.packages ?? [];
  if (!Array.isArray(packages) || packages.length === 0) {
    fail('artifacts.installConfig.packages must define at least one install package');
  }

  for (const pkg of packages) {
    if (!pkg.id || !pkg.packageFormat || !pkg.platform) {
      fail('each install package must define id, packageFormat, and platform');
      continue;
    }
    if (pkg.enabled === false) {
      continue;
    }
    if (pkg.metadata?.releaseBuildDeferred === true) {
      if (pkg.checksum && isPlaceholderChecksum(pkg.checksum)) {
        fail(`package ${pkg.id} must not keep placeholder checksum fields while releaseBuildDeferred is true`);
      }
      if (!pkg.checksum || isPlaceholderChecksum(pkg.checksum)) {
        if (!pkg.metadata?.releaseBuildMaterializedAt) {
          const message = `package ${pkg.id} is deferred to cross-platform CI release builds; materialize checksum evidence on the target runner`;
          strictFail(message);
        }
      }
      continue;
    }
    if (!pkg.checksumAlgorithm) {
      fail(`package ${pkg.id} must declare checksumAlgorithm`);
    }
    if (!pkg.checksum) {
      strictFail(`package ${pkg.id} must declare an immutable checksum before production release`);
    } else if (isPlaceholderChecksum(pkg.checksum)) {
      strictFail(`package ${pkg.id} uses a placeholder checksum; replace with release-build SHA-256 evidence`);
    }
    if (!pkg.url || !/^https:\/\//.test(pkg.url)) {
      strictFail(`package ${pkg.id} must use an HTTPS artifact URL`);
    }
  }

  for (const placeholder of collectPlaceholderMedia(manifest)) {
    strictFail(`${placeholder} is still a generated placeholder; replace with Drive-hosted production media`);
  }

  for (const deferred of collectDeferredCatalogMedia(manifest, workspaceRoot)) {
    strictFail(deferred);
  }

  if (manifest.publish?.status === 'ACTIVE') {
    const hasPlaceholderMedia = collectPlaceholderMedia(manifest).length > 0;
    const hasPlaceholderChecksum = packages.some(
      (pkg) => pkg.enabled !== false && isPlaceholderChecksum(pkg.checksum),
    );
    if (hasPlaceholderMedia || hasPlaceholderChecksum) {
      strictFail('publish.status ACTIVE is incompatible with placeholder media or checksums');
    }
  }

  reportAndExit();
}

function reportAndExit() {
  for (const message of warnings) {
    console.warn(`[release-readiness] warning: ${message}`);
  }
  if (failures.length > 0) {
    for (const message of failures) {
      console.error(`[release-readiness] error: ${message}`);
    }
    console.error(`[release-readiness] failed (${failures.length} error(s), ${warnings.length} warning(s))`);
    process.exit(1);
  }
  const mode = strict ? 'strict' : 'development';
  console.log(`[release-readiness] passed (${mode} mode, ${warnings.length} warning(s))`);
}

main();
