#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { copyFile, mkdir, readdir, readFile, rm, stat, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const APP_ID = 'sdkwork-api-drive-standalone-gateway';
const BINARY_NAME = 'sdkwork-api-drive-standalone-gateway';
const CARGO_PACKAGE = 'sdkwork-api-drive-standalone-gateway';
const SERVER_PROFILE = 'standalone';
const SUPPORTED_FORMAT = 'tar.gz';
const CONFIG_PREFIX = 'sdkwork-api-drive-standalone-gateway';

const scriptPath = fileURLToPath(import.meta.url);
const repoRoot = path.resolve(path.dirname(scriptPath), '..');

async function main() {
  const { command, options } = parseArgs(process.argv.slice(2));
  const context = await createPackageContext(options);

  if (command === 'package') {
    await packageServer(context);
    return;
  }
  if (command === 'validate') {
    await validateArchive(context);
    return;
  }
  if (command === 'help') {
    printHelp();
    return;
  }
  throw new Error(`Unsupported command: ${command}`);
}

function printHelp() {
  console.log(`Usage: node scripts/gateway-standalone-pack.mjs <package|validate> [options]

Package sdkwork-api-drive-standalone-gateway as a release tar.gz archive for standalone deployment.

Options:
  --version <value>       Package version. Defaults to SDKWORK_PACKAGE_VERSION or app manifest.
  --platform <value>      Package platform. Defaults to SDKWORK_PACKAGE_PLATFORM or host platform.
  --arch <value>          Package architecture. Defaults to SDKWORK_PACKAGE_ARCHITECTURE or host arch.
  --format <value>        Package format. Only tar.gz is supported.
  --package-id <value>    Package id. Defaults to SDKWORK_PACKAGE_ID or canonical id.
  --rust-target <value>   Optional cargo --target triple for cross-builds.
  --skip-build            Reuse an existing release binary.
`);
}

function parseArgs(argv) {
  const hasExplicitCommand = argv[0] && !argv[0].startsWith('-');
  const command = hasExplicitCommand ? argv[0] : 'package';
  const options = {};
  const start = hasExplicitCommand ? 1 : 0;

  for (let index = start; index < argv.length; index += 1) {
    const arg = argv[index];
    switch (arg) {
      case '--version':
        options.version = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--platform':
        options.platform = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--arch':
      case '--architecture':
        options.architecture = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--format':
        options.format = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--package-id':
        options.packageId = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--rust-target':
        options.rustTarget = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--skip-build':
        options.skipBuild = true;
        break;
      case '--help':
      case '-h':
        options.help = true;
        break;
      default:
        throw new Error(`Unsupported option: ${arg}`);
    }
  }

  return {
    command: options.help ? 'help' : command,
    options,
  };
}

function requireValue(argv, index, flag) {
  const value = argv[index + 1];
  if (value === undefined || value.startsWith('--')) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

async function createPackageContext(options) {
  const appManifestPath = path.join(repoRoot, 'sdkwork.app.config.json');
  const appManifest = existsSync(appManifestPath)
    ? JSON.parse(await readFile(appManifestPath, 'utf8'))
    : null;
  const hostPlatform = normalizeHostPlatform(process.platform);
  const hostArchitecture = normalizeHostArchitecture(process.arch);
  const platform = normalizePackagePlatform(
    options.platform ?? process.env.SDKWORK_PACKAGE_PLATFORM ?? hostPlatform,
  );
  const architecture = normalizePackageArchitecture(
    options.architecture ?? process.env.SDKWORK_PACKAGE_ARCHITECTURE ?? hostArchitecture,
  );
  const format = options.format ?? process.env.SDKWORK_PACKAGE_FORMAT ?? SUPPORTED_FORMAT;
  const version = normalizeVersion(
    options.version
      ?? process.env.SDKWORK_PACKAGE_VERSION
      ?? appManifest?.release?.currentVersion
      ?? '0.1.0',
  );
  const packageId = options.packageId
    ?? process.env.SDKWORK_PACKAGE_ID
    ?? `${platform}-${architecture}-${SERVER_PROFILE}-tar-gz`;
  const rustTarget = options.rustTarget ?? process.env.SDKWORK_RUST_TARGET ?? '';

  if (format !== SUPPORTED_FORMAT) {
    throw new Error(`Unsupported server package format ${format}; expected ${SUPPORTED_FORMAT}`);
  }
  if (!rustTarget && (platform !== hostPlatform || architecture !== hostArchitecture)) {
    throw new Error(
      `Refusing to label a native ${hostPlatform}/${hostArchitecture} build as ${platform}/${architecture}; pass --rust-target for cross-builds.`,
    );
  }

  const distRoot = path.join(repoRoot, 'dist', SERVER_PROFILE);
  const stageName = `${APP_ID}-${version}-${platform}-${architecture}-${SERVER_PROFILE}`;
  const stageRoot = path.join(distRoot, stageName);
  const archivePath = path.join(distRoot, `${stageName}.${SUPPORTED_FORMAT}`);
  const binaryName = platform === 'windows' ? `${BINARY_NAME}.exe` : BINARY_NAME;
  const releaseDir = rustTarget
    ? path.join(repoRoot, 'target', rustTarget, 'release')
    : path.join(repoRoot, 'target', 'release');
  const binaryPath = path.join(releaseDir, binaryName);

  return {
    architecture,
    archivePath,
    binaryName,
    binaryPath,
    distRoot,
    format,
    hostArchitecture,
    hostPlatform,
    packageId,
    platform,
    rustTarget,
    skipBuild: options.skipBuild === true,
    stageName,
    stageRoot,
    version,
  };
}

function normalizeVersion(value) {
  const text = String(value ?? '').trim();
  const normalized = text.startsWith('refs/tags/') ? text.slice('refs/tags/'.length) : text;
  const withoutPrefix = normalized.startsWith('v') && /^[0-9]/u.test(normalized.slice(1))
    ? normalized.slice(1)
    : normalized;
  if (!/^[0-9A-Za-z][0-9A-Za-z._+-]*$/u.test(withoutPrefix)) {
    throw new Error(`Invalid package version: ${value}`);
  }
  return withoutPrefix;
}

function normalizeHostPlatform(value) {
  if (value === 'win32') {
    return 'windows';
  }
  if (value === 'darwin') {
    return 'macos';
  }
  if (value === 'linux') {
    return 'linux';
  }
  throw new Error(`Unsupported host platform: ${value}`);
}

function normalizePackagePlatform(value) {
  const text = String(value ?? '').trim().toLowerCase();
  if (['linux', 'windows', 'macos'].includes(text)) {
    return text;
  }
  throw new Error(`Unsupported server package platform: ${value}`);
}

function normalizeHostArchitecture(value) {
  if (value === 'x64') {
    return 'x64';
  }
  if (value === 'arm64') {
    return 'arm64';
  }
  if (value === 'arm') {
    return 'armv7';
  }
  throw new Error(`Unsupported host architecture: ${value}`);
}

function normalizePackageArchitecture(value) {
  const text = String(value ?? '').trim().toLowerCase();
  if (['x64', 'arm64', 'armv7'].includes(text)) {
    return text;
  }
  throw new Error(`Unsupported server package architecture: ${value}`);
}

async function packageServer(context) {
  if (!context.skipBuild) {
    runCargoBuild(context);
  }
  if (!existsSync(context.binaryPath)) {
    throw new Error(`Missing release binary: ${context.binaryPath}`);
  }

  await assertPathInside(context.stageRoot, context.distRoot);
  await rm(context.stageRoot, { recursive: true, force: true });
  await rm(context.archivePath, { force: true });
  await mkdir(path.join(context.stageRoot, 'bin'), { recursive: true });
  await mkdir(path.join(context.stageRoot, 'config'), { recursive: true });
  await mkdir(path.join(context.stageRoot, 'deploy', 'systemd'), { recursive: true });

  await copyFile(context.binaryPath, path.join(context.stageRoot, 'bin', context.binaryName));
  await copyConfigExamples(context.stageRoot);
  await copyIfExists(
    path.join('deployments', 'systemd', 'sdkwork-api-drive-standalone-gateway.service'),
    path.join(context.stageRoot, 'deploy', 'systemd', 'sdkwork-api-drive-standalone-gateway.service'),
  );
  await copyIfExists('README.md', path.join(context.stageRoot, 'README.md'));
  await writeFile(path.join(context.stageRoot, 'INSTALL.md'), renderInstallGuide(context), 'utf8');
  await writeFile(
    path.join(context.stageRoot, 'install-manifest.json'),
    `${JSON.stringify(createInstallManifest(context), null, 2)}\n`,
    'utf8',
  );
  await writeChecksums(context.stageRoot);
  await createArchive(context);
  await validateArchive(context);

  console.log(`[${APP_ID}] packaged ${path.relative(repoRoot, context.archivePath)}`);
}

function runCargoBuild(context) {
  const args = ['build', '--release', '-p', CARGO_PACKAGE, '--bin', BINARY_NAME];
  if (context.rustTarget) {
    args.push('--target', context.rustTarget);
  }
  run(cargoCommand(), args, { cwd: repoRoot });
}

function cargoCommand() {
  return process.platform === 'win32' ? 'cargo.exe' : 'cargo';
}

async function copyConfigExamples(stageRoot) {
  const configRoot = path.join(repoRoot, 'configs');
  const files = await readdir(configRoot);
  for (const file of files.filter((item) => item.startsWith(CONFIG_PREFIX) && item.endsWith('.toml.example')).sort()) {
    await copyFile(path.join(configRoot, file), path.join(stageRoot, 'config', file));
  }
}

async function copyIfExists(relativeSource, destination) {
  const source = path.join(repoRoot, relativeSource);
  if (existsSync(source)) {
    await mkdir(path.dirname(destination), { recursive: true });
    await copyFile(source, destination);
  }
}

function renderInstallGuide(context) {
  const binaryPath = context.platform === 'windows'
    ? `.\\bin\\${BINARY_NAME}.exe`
    : `./bin/${BINARY_NAME}`;
  return `# SDKWork Drive Standalone Gateway Package

Package: ${context.packageId}
Version: ${context.version}
Target: ${context.platform}/${context.architecture}

Standalone gateway embeds appbase IAM (OAuth/login) and the Drive application assembly.
Cloud composition consumes that assembly through the platform API gateway.

## Start (development profile)

\`\`\`sh
${binaryPath} --config config/${CONFIG_PREFIX}.development.toml.example
\`\`\`

## Start (production profile)

Copy the production example to a protected host path, set IAM/database env vars, then:

\`\`\`sh
${binaryPath} --config /etc/sdkwork-drive/${CONFIG_PREFIX}.production.toml
\`\`\`

## Health Check

\`\`\`sh
curl http://127.0.0.1:3900/healthz
\`\`\`

## systemd

See \`deploy/systemd/sdkwork-api-drive-standalone-gateway.service\`.
`;
}

function createInstallManifest(context) {
  return {
    schemaVersion: 1,
    appId: APP_ID,
    packageId: context.packageId,
    profile: SERVER_PROFILE,
    platform: context.platform,
    architecture: context.architecture,
    format: context.format,
    version: context.version,
    binary: `bin/${context.binaryName}`,
    configExamples: [
      `config/${CONFIG_PREFIX}.development.toml.example`,
      `config/${CONFIG_PREFIX}.production.toml.example`,
    ],
    systemdUnit: 'deploy/systemd/sdkwork-api-drive-standalone-gateway.service',
    healthPath: '/healthz',
    deploymentMode: 'standalone',
  };
}

async function writeChecksums(stageRoot) {
  const entries = [];
  for (const filePath of await listFiles(stageRoot)) {
    const relativePath = toPosixPath(path.relative(stageRoot, filePath));
    if (relativePath === 'checksums.sha256') {
      continue;
    }
    const digest = createHash('sha256').update(await readFile(filePath)).digest('hex');
    entries.push(`${digest}  ${relativePath}`);
  }
  await writeFile(path.join(stageRoot, 'checksums.sha256'), `${entries.sort().join('\n')}\n`, 'utf8');
}

async function listFiles(root) {
  const result = [];
  const entries = await readdir(root, { withFileTypes: true });
  for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name))) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      result.push(...await listFiles(entryPath));
    } else if (entry.isFile()) {
      result.push(entryPath);
    }
  }
  return result;
}

async function createArchive(context) {
  await mkdir(context.distRoot, { recursive: true });
  run('tar', ['-czf', context.archivePath, '-C', context.distRoot, context.stageName], { cwd: repoRoot });
}

async function validateArchive(context) {
  if (!existsSync(context.archivePath)) {
    throw new Error(`Missing server archive: ${context.archivePath}`);
  }
  const archiveStats = await stat(context.archivePath);
  if (archiveStats.size <= 0) {
    throw new Error(`Server archive is empty: ${context.archivePath}`);
  }

  const listing = run('tar', ['-tzf', context.archivePath], { cwd: repoRoot, capture: true });
  const requiredEntries = [
    `${context.stageName}/bin/${context.binaryName}`,
    `${context.stageName}/config/${CONFIG_PREFIX}.production.toml.example`,
    `${context.stageName}/install-manifest.json`,
    `${context.stageName}/checksums.sha256`,
    `${context.stageName}/deploy/systemd/sdkwork-api-drive-standalone-gateway.service`,
  ];

  for (const entry of requiredEntries) {
    if (!listing.includes(entry)) {
      throw new Error(`Server archive missing ${entry}`);
    }
  }
  console.log(`[${APP_ID}] validated ${path.relative(repoRoot, context.archivePath)}`);
}

function run(command, args, { cwd, capture = false } = {}) {
  const result = spawnSync(command, args, {
    cwd,
    encoding: 'utf8',
    shell: false,
    stdio: capture ? ['ignore', 'pipe', 'pipe'] : 'inherit',
  });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    const output = [result.stdout, result.stderr].filter(Boolean).join('\n').trim();
    throw new Error(`${command} ${args.join(' ')} failed with exit code ${result.status}${output ? `\n${output}` : ''}`);
  }
  return capture ? String(result.stdout ?? '') : '';
}

async function assertPathInside(targetPath, parentPath) {
  const relativePath = path.relative(parentPath, targetPath);
  if (relativePath.startsWith('..') || path.isAbsolute(relativePath)) {
    throw new Error(`Refusing to write outside ${parentPath}: ${targetPath}`);
  }
}

function toPosixPath(value) {
  return value.split(path.sep).join('/');
}

main().catch((error) => {
  console.error(`[${APP_ID}] ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
});
