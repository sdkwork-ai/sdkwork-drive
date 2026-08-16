import { execSync } from 'node:child_process';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  containsLegacyPackageToken,
  expectedDrivePcPackageName,
} from './drive_pc_package_naming.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const appRoot = path.join(repoRoot, 'apps', 'sdkwork-drive-pc');
const packageRoot = path.join(appRoot, 'packages');

const expectedPackageDirs = [
  'sdkwork-drive-pc-commons',
  'sdkwork-drive-pc-core',
  'sdkwork-drive-pc-desktop',
  'sdkwork-drive-pc-drive',
  'sdkwork-drive-pc-file',
  'sdkwork-drive-pc-transfer',
  'sdkwork-drive-pc-types',
  'sdkwork-drive-pc-admin-core',
  'sdkwork-drive-pc-admin-storage-providers',
  'sdkwork-drive-pc-admin-operations',
  'sdkwork-drive-pc-sandbox-contracts',
  'sdkwork-drive-pc-sandbox-explorer',
  'sdkwork-drive-pc-sandbox-explorer-sdk-adapter',
];

const oldPackageTokens = [
  '@sdkwork/drive-pc',
  '@sdkwork/drive-pc-core',
  '@sdkwork/drive-pc-desktop',
  '@sdkwork/drive-commons',
  '@sdkwork/drive-file',
  '@sdkwork/drive-transfer',
  '@sdkwork/drive-types',
  'packages/sdkwork-drive-commons',
  'packages/sdkwork-drive-file',
  'packages/sdkwork-drive-transfer',
  'packages/sdkwork-drive-types',
];

const oldRuntimeConfigTokens = [
  'DriveRuntimeMode',
  'VITE_DRIVE_RUNTIME_MODE',
  'VITE_DRIVE_APP_API_BASE_URL',
  'VITE_DRIVE_ADMIN_STORAGE_API_BASE_URL',
  'VITE_DRIVE_RELEASE_CHANNEL',
  'releaseChannel',
  'runtime.config.mode',
  'config.mode',
];

const oldRuntimeDocumentationTokens = oldRuntimeConfigTokens.filter(
  (token) => token !== 'releaseChannel',
);

const textFileExtensions = new Set([
  '.json',
  '.md',
  '.mjs',
  '.ts',
  '.tsx',
  '.toml',
  '.yaml',
  '.yml',
  '.conf',
  '.example',
]);

const failures = [];

function fail(message) {
  failures.push(message);
}

function readJson(relativePath) {
  const absolutePath = path.join(repoRoot, relativePath);
  if (!existsSync(absolutePath)) {
    fail(`${relativePath} is missing`);
    return undefined;
  }

  try {
    return JSON.parse(readFileSync(absolutePath, 'utf8'));
  } catch (error) {
    fail(`${relativePath} is not valid JSON: ${error.message}`);
    return undefined;
  }
}

function requirePath(relativePath) {
  if (!existsSync(path.join(repoRoot, relativePath))) {
    fail(`${relativePath} is missing`);
  }
}

function listTextFiles(root) {
  if (!existsSync(root)) {
    return [];
  }

  const files = [];
  for (const entry of readdirSync(root)) {
    const absolutePath = path.join(root, entry);
    const stat = statSync(absolutePath);
    if (stat.isDirectory()) {
      if (['node_modules', 'dist', 'target', 'gen'].includes(entry)) {
        continue;
      }
      files.push(...listTextFiles(absolutePath));
      continue;
    }

    const extension = path.extname(entry);
    if (textFileExtensions.has(extension) || entry.startsWith('.env')) {
      files.push(absolutePath);
    }
  }
  return files;
}

function assertExplicitSdkDependencies(relativePath) {
  const spec = readJson(relativePath);
  if (!spec) {
    return;
  }
  if (!spec.contracts || !Array.isArray(spec.contracts.sdkDependencies)) {
    fail(`${relativePath} must declare contracts.sdkDependencies as an explicit array`);
  }
}

function assertPackageSpec(packageDir) {
  const packageJson = readJson(`apps/sdkwork-drive-pc/packages/${packageDir}/package.json`);
  const componentSpec = readJson(
    `apps/sdkwork-drive-pc/packages/${packageDir}/specs/component.spec.json`,
  );
  if (!packageJson || !componentSpec) {
    return;
  }

  const expectedPackageName = expectedDrivePcPackageName(packageDir);
  if (packageJson.name !== expectedPackageName) {
    fail(`${packageDir}/package.json name must be ${expectedPackageName}`);
  }
  if (componentSpec.component?.name !== expectedPackageName) {
    fail(
      `${packageDir}/specs/component.spec.json component.name must be ${expectedPackageName}`,
    );
  }
  if (componentSpec.component?.root !== '.') {
    fail(`${packageDir}/specs/component.spec.json component.root must be "." per COMPONENT_SPEC.md`);
  }
  if (!Array.isArray(componentSpec.contracts?.sdkDependencies)) {
    fail(`${packageDir}/specs/component.spec.json must declare contracts.sdkDependencies`);
  }
}

function assertNoNestedAppLockfiles() {
  const nestedLock = path.join(appRoot, 'pnpm-lock.yaml');
  if (existsSync(nestedLock)) {
    fail('apps/sdkwork-drive-pc must not maintain pnpm-lock.yaml; use the repository root workspace lockfile');
  }
}

const forbiddenTrackedArtifactPrefixes = [
  'apps/sdkwork-drive-pc/pnpm-lock.yaml',
  'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-desktop/src-tauri/target-test-migrate/',
  'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-desktop/src-tauri/target/',
  'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-desktop/src-tauri/gen/',
];

function assertNoTrackedBuildArtifacts() {
  let trackedFiles = [];
  try {
    trackedFiles = execSync('git ls-files', { cwd: repoRoot, encoding: 'utf8' })
      .trim()
      .split('\n')
      .filter(Boolean);
  } catch {
    fail('unable to inspect git index for tracked build artifacts');
    return;
  }

  for (const file of trackedFiles) {
    for (const prefix of forbiddenTrackedArtifactPrefixes) {
      const normalizedPrefix = prefix.endsWith('/') ? prefix : `${prefix}`;
      if (file === normalizedPrefix.replace(/\/$/, '') || file.startsWith(normalizedPrefix)) {
        fail(`git must not track build artifact ${file}`);
      }
    }
  }
}

function assertDevAppBuildCommands() {
  const manifestPath = path.join(appRoot, 'sdkwork.app.config.json');
  if (!existsSync(manifestPath)) {
    fail('apps/sdkwork-drive-pc/sdkwork.app.config.json is required for devApp build alignment');
    return;
  }

  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  const appPackage = JSON.parse(readFileSync(path.join(appRoot, 'package.json'), 'utf8'));
  const availableScripts = new Set(Object.keys(appPackage.scripts ?? {}));
  const targets = manifest.devApp?.build?.targets ?? [];

  for (const target of targets) {
    const command = target?.command;
    if (typeof command !== 'string' || command.length === 0) {
      continue;
    }
    const scriptName = command.startsWith('pnpm ') ? command.slice('pnpm '.length).split(/\s+/)[0] : null;
    if (scriptName && !availableScripts.has(scriptName)) {
      fail(
        `apps/sdkwork-drive-pc/sdkwork.app.config.json devApp target ${target.id ?? 'unknown'} references missing script ${scriptName}`,
      );
    }
  }
}

function assertNoOldTokens() {
  for (const file of listTextFiles(appRoot)) {
    const relativePath = path.relative(repoRoot, file).replaceAll(path.sep, '/');
    const source = readFileSync(file, 'utf8');
    for (const token of oldPackageTokens) {
      if (containsLegacyPackageToken(source, token)) {
        fail(`${relativePath} still contains legacy package token ${token}`);
      }
    }
    for (const token of oldRuntimeDocumentationTokens) {
      if (source.includes(token)) {
        fail(`${relativePath} still contains legacy runtime config token ${token}`);
      }
    }
  }
}

function assertStandardRuntimeConfig() {
  const runtimeConfigPath = 'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-core/src/config/runtimeConfig.ts';
  const runtimeConfigSource = readFileSync(path.join(repoRoot, runtimeConfigPath), 'utf8');
  for (const requiredToken of [
    'SdkworkEnvironment',
    'configProfile',
    'buildMode',
    'deploymentMode',
    'runtimeTarget',
    'sdkBaseUrls',
    'dependencySdkBaseUrls',
    'VITE_DRIVE_PC_ENVIRONMENT',
    'VITE_DRIVE_PC_DEPLOYMENT_MODE',
    'VITE_DRIVE_PC_RUNTIME_TARGET',
  ]) {
    if (!runtimeConfigSource.includes(requiredToken)) {
      fail(`${runtimeConfigPath} must include standard runtime config token ${requiredToken}`);
    }
  }

  for (const file of [
    runtimeConfigPath,
    'apps/sdkwork-drive-pc/src/App.tsx',
    'apps/sdkwork-drive-pc/src/bootstrap/driveIamRuntime.ts',
    'apps/sdkwork-drive-pc/.env.example',
  ]) {
    const source = readFileSync(path.join(repoRoot, file), 'utf8');
    for (const token of oldRuntimeConfigTokens) {
      if (source.includes(token)) {
        fail(`${file} still contains legacy runtime config token ${token}`);
      }
    }
  }
}

function assertNoSdkTransportBypass() {
  const sdkSourceRoot = path.join(
    appRoot,
    'packages',
    'sdkwork-drive-pc-core',
    'src',
    'sdk',
  );
  for (const file of listTextFiles(sdkSourceRoot)) {
    const relativePath = path.relative(repoRoot, file).replaceAll(path.sep, '/');
    if (relativePath.endsWith('.test.ts')) {
      continue;
    }

    const source = readFileSync(file, 'utf8');
    for (const token of [
      'fetch(',
      'new Headers',
      "headers.set('Authorization'",
      "headers.set('Access-Token'",
      'X-Sdkwork-',
    ]) {
      if (source.includes(token)) {
        fail(`${relativePath} must use generated SDK transport and TokenManager, not ${token}`);
      }
    }
  }

  const runtimeSource = readFileSync(
    path.join(repoRoot, 'apps/sdkwork-drive-pc/src/bootstrap/createDrivePcRuntime.ts'),
    'utf8',
  );
  for (const requiredToken of [
    'createDriveSessionTokenManager',
    'tokenManager',
    'sdkClients',
  ]) {
    if (!runtimeSource.includes(requiredToken)) {
      fail(`apps/sdkwork-drive-pc/src/bootstrap/createDrivePcRuntime.ts must wire ${requiredToken}`);
    }
  }
}

function assertStandardIamRuntime() {
  const runtimePath = 'apps/sdkwork-drive-pc/src/bootstrap/driveIamRuntime.ts';
  const runtimeSource = readFileSync(path.join(repoRoot, runtimePath), 'utf8');
  for (const requiredToken of [
    '@sdkwork/auth-runtime-pc-react',
    'createSdkworkAppbasePcAuthRuntime',
    'sessionBridge',
    'tokenManager',
    'sdkClients',
  ]) {
    if (!runtimeSource.includes(requiredToken)) {
      fail(`${runtimePath} must use standard appbase PC auth runtime token ${requiredToken}`);
    }
  }

  for (const forbiddenPattern of [
    /@sdkwork\/iam-sdk-adapter/u,
    /\bcreateIamSdkAdapters\b/u,
    /\bcreateIamAppSdkAdapter\b/u,
    /\bcreateIamBackendSdkAdapter\b/u,
    /\bcreateIamRuntime\s*\(/u,
  ]) {
    if (forbiddenPattern.test(runtimeSource)) {
      fail(`${runtimePath} must not wire low-level appbase IAM adapters or createIamRuntime directly`);
    }
  }

  const tsconfig = readJson('apps/sdkwork-drive-pc/tsconfig.json');
  const tsPaths = tsconfig?.compilerOptions?.paths ?? {};
  const authRuntimePath = tsPaths['@sdkwork/auth-runtime-pc-react']?.[0] ?? '';
  if (!authRuntimePath.includes('sdkwork-auth-runtime-pc-react/src/index.ts')) {
    fail('apps/sdkwork-drive-pc/tsconfig.json must map @sdkwork/auth-runtime-pc-react to appbase high-level runtime');
  }
  const iamRuntimePath = tsPaths['@sdkwork/iam-runtime']?.[0] ?? '';
  if (!iamRuntimePath.includes('sdkwork-iam-runtime/src/index.ts')) {
    fail('apps/sdkwork-drive-pc/tsconfig.json must map @sdkwork/iam-runtime to appbase iam-runtime');
  }
  if (iamRuntimePath.includes('driveIamRuntimeShim.ts')) {
    fail('apps/sdkwork-drive-pc/tsconfig.json must not map @sdkwork/iam-runtime to Drive product-local shim');
  }
  if (Object.hasOwn(tsPaths, '@sdkwork/iam-sdk-adapter')) {
    fail('apps/sdkwork-drive-pc/tsconfig.json must not expose @sdkwork/iam-sdk-adapter to product code');
  }

  const vitePath = 'apps/sdkwork-drive-pc/vite.config.ts';
  const viteSource = readFileSync(path.join(repoRoot, vitePath), 'utf8');
  for (const requiredToken of [
    '@sdkwork/auth-runtime-pc-react',
    'sdkwork-auth-runtime-pc-react/src/index.ts',
    '@sdkwork/iam-runtime',
    'sdkwork-iam-runtime/src/index.ts',
  ]) {
    if (!viteSource.includes(requiredToken)) {
      fail(`${vitePath} must include standard IAM runtime alias token ${requiredToken}`);
    }
  }
  for (const forbiddenToken of [
    'driveIamRuntimeShim.ts',
    '@sdkwork/iam-sdk-adapter',
    'sdkwork-iam-sdk-adapter/src/index.ts',
  ]) {
    if (viteSource.includes(forbiddenToken)) {
      fail(`${vitePath} must not alias product-local low-level IAM wiring token ${forbiddenToken}`);
    }
  }
}

for (const relativePath of [
  'apps/sdkwork-drive-pc/AGENTS.md',
  'apps/sdkwork-drive-pc/CLAUDE.md',
  'apps/sdkwork-drive-pc/GEMINI.md',
  'apps/sdkwork-drive-pc/CODEX.md',
  'apps/sdkwork-drive-pc/sdkwork.app.config.json',
  'apps/sdkwork-drive-pc/.sdkwork/README.md',
  'apps/sdkwork-drive-pc/.sdkwork/.gitignore',
  'apps/sdkwork-drive-pc/.sdkwork/skills/README.md',
  'apps/sdkwork-drive-pc/.sdkwork/plugins/README.md',
  'apps/sdkwork-drive-pc/etc/browser/runtime-env.development.example.json',
  'apps/sdkwork-drive-pc/etc/browser/runtime-env.test.example.json',
  'apps/sdkwork-drive-pc/etc/browser/runtime-env.staging.example.json',
  'apps/sdkwork-drive-pc/etc/browser/runtime-env.production.example.json',
  'apps/sdkwork-drive-pc/etc/desktop/sdkwork-drive-pc.development.toml.example',
  'apps/sdkwork-drive-pc/etc/desktop/sdkwork-drive-pc.test.toml.example',
  'apps/sdkwork-drive-pc/etc/desktop/sdkwork-drive-pc.staging.toml.example',
  'apps/sdkwork-drive-pc/etc/desktop/sdkwork-drive-pc.production.toml.example',
  'apps/sdkwork-drive-pc/etc/server/sdkwork-drive-pc.development.toml.example',
  'apps/sdkwork-drive-pc/etc/server/sdkwork-drive-pc.test.toml.example',
  'apps/sdkwork-drive-pc/etc/server/sdkwork-drive-pc.staging.toml.example',
  'apps/sdkwork-drive-pc/etc/server/sdkwork-drive-pc.production.toml.example',
  'apps/sdkwork-drive-pc/etc/container/sdkwork-drive-pc.development.toml.example',
  'apps/sdkwork-drive-pc/etc/container/sdkwork-drive-pc.test.toml.example',
  'apps/sdkwork-drive-pc/etc/container/sdkwork-drive-pc.staging.toml.example',
  'apps/sdkwork-drive-pc/etc/container/sdkwork-drive-pc.production.toml.example',
]) {
  requirePath(relativePath);
}

const appPackage = readJson('apps/sdkwork-drive-pc/package.json');
if (appPackage?.name !== 'sdkwork-drive-pc') {
  fail('apps/sdkwork-drive-pc/package.json name must be sdkwork-drive-pc');
}

const packageDirs = existsSync(packageRoot)
  ? readdirSync(packageRoot).filter((entry) => statSync(path.join(packageRoot, entry)).isDirectory())
  : [];
for (const packageDir of packageDirs) {
  if (!packageDir.startsWith('sdkwork-drive-pc-')) {
    fail(`packages/${packageDir} must be renamed to sdkwork-drive-pc-*`);
  }
}
for (const packageDir of expectedPackageDirs) {
  if (!packageDirs.includes(packageDir)) {
    fail(`packages/${packageDir} is missing`);
  }
  assertPackageSpec(packageDir);
}

assertExplicitSdkDependencies('apps/sdkwork-drive-pc/specs/component.spec.json');
assertExplicitSdkDependencies('sdks/sdkwork-drive-sdk/specs/component.spec.json');

const openSdkManifest = readJson('sdks/sdkwork-drive-sdk/sdk-manifest.json');
if (openSdkManifest && !Array.isArray(openSdkManifest.sdkDependencies)) {
  fail('sdks/sdkwork-drive-sdk/sdk-manifest.json must declare sdkDependencies: []');
}

const gitignore = existsSync(path.join(appRoot, '.gitignore'))
  ? readFileSync(path.join(appRoot, '.gitignore'), 'utf8')
  : '';
for (const requiredIgnore of [
  '.env.development.local',
  '.env.test.local',
  '.env.staging.local',
  '.env.production.local',
  '.env.postgres',
  '.env.release.local',
  'config/*.local.toml',
]) {
  if (!gitignore.includes(requiredIgnore)) {
    fail(`apps/sdkwork-drive-pc/.gitignore must ignore ${requiredIgnore}`);
  }
}

function assertNoLocalByteFormatters() {
  const allowedPaths = new Set([
    'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-commons/src/utils/formatDriveBytes.ts',
  ]);
  const localByteFormatterPattern = /if\s*\([^)]*<\s*1024\s*\*\s*1024/;
  for (const file of listTextFiles(appRoot)) {
    const relativePath = path.relative(repoRoot, file).replaceAll(path.sep, '/');
    if (!relativePath.endsWith('.ts') && !relativePath.endsWith('.tsx')) {
      continue;
    }
    if (relativePath.endsWith('.test.ts') || relativePath.endsWith('.test.tsx')) {
      continue;
    }
    if (allowedPaths.has(relativePath)) {
      continue;
    }
    const source = readFileSync(file, 'utf8');
    if (localByteFormatterPattern.test(source)) {
      fail(
        `${relativePath} must use formatDriveBytes from sdkwork-drive-pc-commons instead of local byte formatting`,
      );
    }
  }
}

assertNoNestedAppLockfiles();
assertNoTrackedBuildArtifacts();
assertDevAppBuildCommands();
assertNoOldTokens();
assertNoLocalByteFormatters();
assertStandardRuntimeConfig();
assertNoSdkTransportBypass();
assertStandardIamRuntime();
assertNoDuplicateFormatBytesHelpers();

function assertNoDuplicateFormatBytesHelpers() {
  const forbiddenPattern = /export function formatBytes\s*\(/u;
  const appRootPath = path.join(repoRoot, 'apps', 'sdkwork-drive-pc');
  const allowedRelativePath = 'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-commons/src/utils/formatDriveBytes.ts';

  function walk(directory) {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (entry.name === 'node_modules' || entry.name === 'dist') {
        continue;
      }
      const absolutePath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        walk(absolutePath);
        continue;
      }
      if (!entry.name.endsWith('.ts') && !entry.name.endsWith('.tsx')) {
        continue;
      }
      const relativePath = path.relative(repoRoot, absolutePath).split(path.sep).join('/');
      if (relativePath === allowedRelativePath) {
        continue;
      }
      const source = readFileSync(absolutePath, 'utf8');
      if (forbiddenPattern.test(source)) {
        fail(
          `${relativePath} must not export formatBytes; use formatDriveBytes from sdkwork-drive-pc-commons (@sdkwork/utils)`,
        );
      }
    }
  }

  if (existsSync(appRootPath)) {
    walk(appRootPath);
  }
}

function assertRepositoryDocsStandard() {
  const docsChecker = path.join(repoRoot, '..', 'sdkwork-specs', 'tools', 'check-repository-docs-standard.mjs');
  if (!existsSync(docsChecker)) {
    fail('sdkwork-specs/tools/check-repository-docs-standard.mjs is missing');
    return;
  }
  try {
    execSync(`node "${docsChecker}" --root "${appRoot}"`, {
      cwd: repoRoot,
      encoding: 'utf8',
      stdio: 'pipe',
    });
  } catch (error) {
    const output = `${error.stdout ?? ''}${error.stderr ?? ''}`.trim();
    fail(output || 'apps/sdkwork-drive-pc documentation layout failed check-repository-docs-standard.mjs');
  }
}

assertRepositoryDocsStandard();

if (failures.length > 0) {
  console.error('SDKWork Drive PC standard check failed:');
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log('SDKWork Drive PC standard check passed.');
