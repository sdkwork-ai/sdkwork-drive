#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const expectedDependencyIds = [
  'sdkwork-appbase',
  'sdkwork-iam',
  'sdkwork-database',
  'sdkwork-id',
  'sdkwork-ui',
  'sdkwork-sdk-commons',
  'sdkwork-sdk-generator',
  'sdkwork-web-framework',
  'sdkwork-utils',
  'sdkwork-app-topology',
];
const sourceDependencyFiles = [
  'package.json',
  'Cargo.toml',
  'sdkwork.workflow.json',
  '.github/workflows/package.yml',
  'AGENTS.md',
  'apps/sdkwork-drive-pc/tsconfig.json',
  'apps/sdkwork-drive-pc/vite.config.ts',
  'tools/drive_sdk_generator_runner.mjs',
  'tools/drive_openapi_export.mjs',
  'crates/sdkwork-drive-contract/tests/tooling_scripts_smoke.rs',
];
const activeDocumentationFiles = [
  'README.md',
  'docs/architecture/tech/TECH-drive-iam-integration-standard.md',
  'docs/architecture/tech/TECH-drive-sdk-integration-standard.md',
  'docs/architecture/tech/TECH-storage-s3-architecture.md',
];
const retiredDependencyRoot = ['.sdkwork', 'dependencies'].join('/');
const retiredLocalScript = ['prepare-local', 'dependencies.mjs'].join('-');
const retiredDepsLocalPrefix = ['deps', 'local'].join(':');
const failures = [];

function readText(relativePath) {
  const absolutePath = path.join(repoRoot, relativePath);
  if (!fs.existsSync(absolutePath)) {
    failures.push(`${relativePath} must exist`);
    return '';
  }
  return fs.readFileSync(absolutePath, 'utf8');
}

function readJson(relativePath) {
  const text = readText(relativePath);
  if (!text) {
    return {};
  }
  return JSON.parse(text);
}

function assert(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}

function assertNoRetiredDependencyModel(relativePath) {
  const text = readText(relativePath);
  assert(!text.includes(retiredDependencyRoot), `${relativePath} must not reference the retired SDKWork dependency root`);
  assert(!text.includes(retiredLocalScript.replace(/\.mjs$/u, '')), `${relativePath} must not reference the retired local dependency script`);
  assert(!text.includes(retiredDepsLocalPrefix), `${relativePath} must not reference retired local dependency scripts`);
}

function assertSiblingDependencyPathsAreKnown(relativePath) {
  const text = readText(relativePath);
  const sanitized = text.replace(/\$schema"\s*:\s*"[^"]+"/g, '"$schema":"__schema__"');
  const matches = [...sanitized.matchAll(/(?:\.\.\/|\.{2}\\)+(sdkwork-(?!specs|drive-)[A-Za-z0-9-]*)/g)];
  for (const match of matches) {
    assert(
      expectedDependencyIds.includes(match[1]),
      `${relativePath} uses undeclared SDKWork sibling dependency path ${match[0]}`,
    );
  }
}

function assertNativeDependencyFile(relativePath) {
  assertNoRetiredDependencyModel(relativePath);
  assertSiblingDependencyPathsAreKnown(relativePath);
}

function assertPnpmWorkspaceOnlyProtocol(relativePath) {
  if (!relativePath.endsWith('package.json') || relativePath === 'package.json') {
    return;
  }
  const text = readText(relativePath);
  const linkMatches = [...text.matchAll(/['"](link:[^'"]+)['"]/g)];
  for (const match of linkMatches) {
    const specifier = match[1];
    assert(
      !specifier.includes('sdkwork-'),
      `${relativePath} must not use ${specifier}; SDKWork cross-workspace sources must use the workspace: protocol declared in pnpm-workspace.yaml packages:`,
    );
  }
}

function collectWorkspaceCrateCargoTomlFiles() {
  const files = [];
  const cratesRoot = path.join(repoRoot, 'crates');
  if (!fs.existsSync(cratesRoot)) {
    return files;
  }
  for (const entry of fs.readdirSync(cratesRoot, { withFileTypes: true })) {
    if (!entry.isDirectory()) {
      continue;
    }
    const cargoToml = path.join('crates', entry.name, 'Cargo.toml');
    if (fs.existsSync(path.join(repoRoot, cargoToml))) {
      files.push(cargoToml.replace(/\\/g, '/'));
    }
  }
  return files;
}

function assertCargoWorkspaceOnlyProtocol(relativePath) {
  if (!relativePath.endsWith('Cargo.toml')) {
    return;
  }
  if (relativePath === 'Cargo.toml') {
    return;
  }
  const text = readText(relativePath);
  const pathMatches = [...text.matchAll(/path\s*=\s*"((?:\.\.\/)+sdkwork-[A-Za-z0-9-]+[^"]*)"/g)];
  for (const match of pathMatches) {
    const dependencyPath = match[1];
    assert(
      false,
      `${relativePath} must not redeclare SDKWork source path "${dependencyPath}"; declare it in root [workspace.dependencies] and consume with \`crate_name.workspace = true\``,
    );
  }
}

function assertRootCargoWorkspaceDependencies() {
  const text = readText('Cargo.toml');
  assert(
    !text.includes('../../../sdkwork-'),
    'Cargo.toml must use ../sdkwork-* sibling paths from repository root, not ../../../sdkwork-*',
  );
  const section = text.split('[workspace.dependencies]')[1];
  if (!section) {
    failures.push('Cargo.toml must declare [workspace.dependencies]');
    return;
  }
  const keys = [];
  for (const line of section.split('\n')) {
    const match = line.match(/^([A-Za-z0-9_-]+)\s*=/);
    if (match) {
      keys.push(match[1]);
    }
  }
  const seen = new Set();
  for (const key of keys) {
    assert(!seen.has(key), `Cargo.toml [workspace.dependencies] has duplicate key ${key}`);
    seen.add(key);
  }
}

function assertDependencyDeclaration() {
  const workflow = readJson('sdkwork.workflow.json');
  assert(Array.isArray(workflow.dependencies), 'sdkwork.workflow.json must declare dependencies[]');
  const dependencyIds = new Set((workflow.dependencies || []).map((dependency) => dependency.id));
  for (const expectedId of expectedDependencyIds) {
    assert(dependencyIds.has(expectedId), `sdkwork.workflow.json must declare ${expectedId}`);
  }
  for (const dependency of workflow.dependencies || []) {
    assert(typeof dependency.id === 'string' && expectedDependencyIds.includes(dependency.id), `unexpected dependency id ${dependency.id}`);
    assert(/^sdkwork-ai\/sdkwork-[a-z0-9-]+$/.test(dependency.repository || ''), `${dependency.id} must use owner/repo repository form`);
    assert(Boolean(dependency.ref || dependency.refInput), `${dependency.id} must declare ref or refInput`);
    assert(dependency.tokenSecret === 'SDKWORK_RELEASE_TOKEN', `${dependency.id} must use SDKWORK_RELEASE_TOKEN`);
    assert(!Object.hasOwn(dependency, 'path'), `${dependency.id} must omit dependencies[].path`);
  }
}

function assertNoLocalMaterializer() {
  const packageJson = readJson('package.json');
  assert(packageJson.scripts?.[[retiredDepsLocalPrefix, 'link'].join(':')] === undefined, 'package.json must not expose retired local link script');
  assert(packageJson.scripts?.[[retiredDepsLocalPrefix, 'check'].join(':')] === undefined, 'package.json must not expose retired local check script');
  assert(!readText('.gitignore').includes(retiredDependencyRoot), 'gitignore must not keep the retired SDKWork dependency ignore rule');
  assert(!fs.existsSync(path.join(repoRoot, 'scripts', retiredLocalScript)), 'retired local dependency script must not exist');
  assert(!fs.existsSync(path.join(repoRoot, ...retiredDependencyRoot.split('/'))), 'retired SDKWork dependency directory must not exist');
}

function assertLocalSdkGeneratorStubIsFailClosed() {
  const relativePath = 'tools/sdkwork_sdk_generator_stub.mjs';
  if (!fs.existsSync(path.join(repoRoot, relativePath))) {
    return;
  }
  const text = readText(relativePath);
  assert(
    text.includes('fail-closed compatibility tombstone'),
    `${relativePath} must be fail-closed if retained for compatibility`,
  );
  assert(
    text.includes('../sdkwork-sdk-generator/bin/sdkgen.js'),
    `${relativePath} must point callers to the canonical sdkgen entrypoint`,
  );
  for (const forbiddenToken of [
    'writeFileSync',
    'mkdirSync',
    'readFileSync',
    'collectOperations',
    'Generated by sdkwork_sdk_generator_stub',
  ]) {
    assert(
      !text.includes(forbiddenToken),
      `${relativePath} must not contain SDK generation behavior (${forbiddenToken})`,
    );
  }
}

function assertWorkflowRefs() {
  const workflowYaml = readText('.github/workflows/package.yml');
  assert(!workflowYaml.includes("dependency_refs_json: '{}'"), 'package workflow must not pass an empty dependency_refs_json');
  for (const dependencyId of expectedDependencyIds) {
    const inputName = `${dependencyId.replaceAll('-', '_')}_ref`;
    const envName = dependencyId.replaceAll('-', '_').toUpperCase();
    assert(workflowYaml.includes(inputName), `.github/workflows/package.yml must expose ${inputName}`);
    assert(workflowYaml.includes(envName), `.github/workflows/package.yml dependency_refs_json must include ${envName}`);
  }
}

function assertDocumentation() {
  for (const relativePath of activeDocumentationFiles) {
    assertNativeDependencyFile(relativePath);
  }
  const agents = readText('AGENTS.md');
  assert(!agents.includes('No `sdkwork.app.config.json` is present at this root'), 'AGENTS.md must reflect root sdkwork.app.config.json presence');
}

assertDependencyDeclaration();
assertRootCargoWorkspaceDependencies();
assertNoLocalMaterializer();
assertLocalSdkGeneratorStubIsFailClosed();
assertWorkflowRefs();
for (const relativePath of sourceDependencyFiles) {
  assertNativeDependencyFile(relativePath);
  assertPnpmWorkspaceOnlyProtocol(relativePath);
  assertCargoWorkspaceOnlyProtocol(relativePath);
}
for (const relativePath of collectWorkspaceCrateCargoTomlFiles()) {
  assertCargoWorkspaceOnlyProtocol(relativePath);
}
assertDocumentation();

if (failures.length > 0) {
  process.stderr.write(`Dependency management standard failed:\n${failures.map((failure) => `- ${failure}`).join('\n')}\n`);
  process.exit(1);
}

process.stdout.write('Dependency management standard passed\n');
