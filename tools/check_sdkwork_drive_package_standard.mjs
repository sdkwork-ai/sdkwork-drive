import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const failures = [];

const skippedDirectoryNames = new Set([
  '.git',
  'dist',
  'generated',
  'node_modules',
  'target',
  'target-test-migrate',
]);
const authoredPackageRoots = [
  'apps/sdkwork-drive-pc',
  'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-commons',
  'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-core',
  'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-desktop',
  'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-file',
  'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-transfer',
  'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-types',
  'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-drive',
  'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-admin-core',
  'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-admin-storage-providers',
];

function fail(message) {
  failures.push(message);
}

function toPosix(value) {
  return value.split(path.sep).join('/');
}

function relativePath(absolutePath) {
  return toPosix(path.relative(repoRoot, absolutePath));
}

function readJson(relativeFile) {
  const absolutePath = path.join(repoRoot, relativeFile);
  try {
    return JSON.parse(readFileSync(absolutePath, 'utf8'));
  } catch (error) {
    fail(`${relativeFile} is not valid JSON: ${error.message}`);
    return undefined;
  }
}

function walk(directory, visitor) {
  if (!existsSync(directory)) {
    return;
  }
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (entry.isDirectory() && skippedDirectoryNames.has(entry.name)) {
      continue;
    }
    const absolutePath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      walk(absolutePath, visitor);
      continue;
    }
    visitor(absolutePath);
  }
}

function cargoPackageName(cargoTomlPath) {
  const source = readFileSync(cargoTomlPath, 'utf8');
  const packageMatch = source.match(/^\[package\][\s\S]*?^name\s*=\s*"([^"]+)"/m);
  return packageMatch?.[1] ?? null;
}

function assertAuthoredNpmPackages() {
  for (const relativeRoot of authoredPackageRoots) {
    const packageJsonPath = path.join(relativeRoot, 'package.json');
    const packageJson = readJson(packageJsonPath);
    if (!packageJson) {
      continue;
    }
    const expectedName = path.basename(relativeRoot);
    if (packageJson.name !== expectedName) {
      fail(`${packageJsonPath} name must be ${expectedName}`);
    }
  }

  const pcPackagesRoot = path.join(repoRoot, 'apps/sdkwork-drive-pc/packages');
  for (const entry of readdirSync(pcPackagesRoot, { withFileTypes: true })) {
    if (!entry.isDirectory()) {
      continue;
    }
    if (!entry.name.startsWith('sdkwork-drive-pc-')) {
      fail(`apps/sdkwork-drive-pc/packages/${entry.name} must use sdkwork-drive-pc-*`);
    }
  }
}

function assertCargoPackages() {
  walk(repoRoot, (absolutePath) => {
    if (path.basename(absolutePath) !== 'Cargo.toml') {
      return;
    }

    const relative = relativePath(absolutePath);
    if (relative === 'Cargo.toml' || relative.includes('/generated/server-openapi/')) {
      return;
    }

    const name = cargoPackageName(absolutePath);
    if (!name) {
      return;
    }

    const directoryName = path.basename(path.dirname(absolutePath));
    const allowedTauriHost =
      relative === 'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-desktop/src-tauri/Cargo.toml' &&
      name === 'sdkwork-drive-pc-desktop';
    if (!allowedTauriHost && name !== directoryName) {
      fail(`${relative} package name ${name} must match directory ${directoryName}`);
    }
    if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(name)) {
      fail(`${relative} package name ${name} must use lowercase kebab-case`);
    }
  });
}

function assertComponentSpecs() {
  walk(repoRoot, (absolutePath) => {
    if (toPosix(absolutePath).includes('/generated/server-openapi/')) {
      return;
    }
    if (path.basename(absolutePath) !== 'component.spec.json') {
      return;
    }

    const relative = relativePath(absolutePath);
    const spec = readJson(relative);
    if (!spec) {
      return;
    }

    const componentName = spec.component?.name;
    const componentRoot = spec.component?.root;
    if (!componentName || !componentRoot) {
      fail(`${relative} must declare component.name and component.root`);
      return;
    }

    const specRoot = path.resolve(path.dirname(absolutePath), '..');
    if (componentRoot !== '.') {
      fail(`${relative} component.root must be "." per COMPONENT_SPEC.md (found ${componentRoot})`);
    }

    const packageJsonPath = path.join(specRoot, 'package.json');
    if (existsSync(packageJsonPath)) {
      const packageJson = readJson(relativePath(packageJsonPath));
      if (packageJson?.name && packageJson.name !== componentName) {
        fail(`${relative} component.name must match package.json name ${packageJson.name}`);
      }
    }

    const cargoTomlPath = path.join(specRoot, 'Cargo.toml');
    if (existsSync(cargoTomlPath)) {
      const packageName = cargoPackageName(cargoTomlPath);
      if (packageName && packageName !== componentName) {
        fail(`${relative} component.name must match Cargo package ${packageName}`);
      }
    }
  });
}

function assertRustLibEntrypoints() {
  walk(repoRoot, (absolutePath) => {
    if (path.basename(absolutePath) !== 'lib.rs') {
      return;
    }
    const relative = relativePath(absolutePath);
    if (relative.includes('/generated/server-openapi/')) {
      return;
    }
    const source = readFileSync(absolutePath, 'utf8');
    const lineCount = source.split(/\r?\n/).length;
    if (lineCount > 150) {
      fail(`${relative} must stay a module assembly file; found ${lineCount} lines`);
    }
    for (const forbiddenPattern of [/^async fn /m, /^pub async fn /m, /sqlx::query\(/]) {
      if (forbiddenPattern.test(source)) {
        fail(`${relative} must not contain handlers, async business logic, or SQL queries`);
      }
    }
  });
}

assertAuthoredNpmPackages();
assertCargoPackages();
assertComponentSpecs();
assertRustLibEntrypoints();

if (failures.length > 0) {
  console.error('SDKWork Drive package standard check failed:');
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log('SDKWork Drive package standard check passed.');
