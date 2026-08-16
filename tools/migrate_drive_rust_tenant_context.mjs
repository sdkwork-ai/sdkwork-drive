#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

const rustDtoFiles = [
  'crates/sdkwork-routes-drive-app-api/src/dto.rs',
  'crates/sdkwork-routes-drive-backend-api/src/dto.rs',
  'crates/sdkwork-routes-storage-backend-api/src/dto.rs',
];

const rustHandlerFiles = [
  'crates/sdkwork-routes-drive-app-api/src/routes.rs',
  'crates/sdkwork-routes-drive-app-api/src/download_packages.rs',
  'crates/sdkwork-routes-storage-backend-api/src/binding_handlers.rs',
];

function stripOptionalTenantIdFields(source) {
  const lines = source.split('\n');
  const output = [];
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    const next = lines[index + 1] ?? '';
    if (
      line.trim() === '#[serde(default)]'
      && next.includes('tenant_id: Option<String>')
    ) {
      index += 1;
      continue;
    }
    if (line.includes('tenant_id: Option<String>')) {
      continue;
    }
    output.push(line);
  }
  return output.join('\n');
}

function normalizeResolveTenantCalls(source) {
  return source
    .replaceAll('ctx.resolve_tenant_id(query.tenant_id.clone())?', 'ctx.resolve_tenant_id()?')
    .replaceAll('ctx.resolve_tenant_id(payload.tenant_id.clone())?', 'ctx.resolve_tenant_id()?')
    .replaceAll('ctx.resolve_tenant_id(query.tenant_id)?', 'ctx.resolve_tenant_id()?')
    .replaceAll('ctx.resolve_tenant_id(payload.tenant_id)?', 'ctx.resolve_tenant_id()?')
    .replaceAll('ctx.resolve_tenant_id(None)?', 'ctx.resolve_tenant_id()?')
    .replaceAll('match ctx.resolve_tenant_id(query.tenant_id)', 'match ctx.resolve_tenant_id()')
    .replaceAll('match ctx.resolve_tenant_id(payload.tenant_id)', 'match ctx.resolve_tenant_id()');
}

function stripTenantIdFromTestSources(source) {
  let updated = source;
  updated = updated.replace(/[?&]tenantId=[^"&\s}]+/gu, '');
  updated = updated.replace(/\?&/gu, '?');
  updated = updated.replace(/\?"/gu, '"');
  updated = updated.replace(/,\s*"tenantId"\s*:\s*"[^"]*"/gu, '');
  updated = updated.replace(/"tenantId"\s*:\s*"[^"]*"\s*,\s*/gu, '');
  updated = updated.replace(/\[\s*,/gu, '[');
  return updated;
}

for (const relativePath of rustDtoFiles) {
  const absolutePath = path.join(repoRoot, relativePath);
  const before = fs.readFileSync(absolutePath, 'utf8');
  const after = stripOptionalTenantIdFields(before);
  if (before !== after) {
    fs.writeFileSync(absolutePath, after);
    process.stdout.write(`${relativePath}: removed optional tenant_id fields\n`);
  }
}

for (const relativePath of rustHandlerFiles) {
  const absolutePath = path.join(repoRoot, relativePath);
  const before = fs.readFileSync(absolutePath, 'utf8');
  const after = normalizeResolveTenantCalls(before);
  if (before !== after) {
    fs.writeFileSync(absolutePath, after);
    process.stdout.write(`${relativePath}: normalized resolve_tenant_id calls\n`);
  }
}

const testFiles = [];
function walkTests(directory) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const absolutePath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      walkTests(absolutePath);
      continue;
    }
    if (entry.name.endsWith('.rs')) {
      testFiles.push(absolutePath);
    }
  }
}

for (const directory of [
  path.join(repoRoot, 'crates/sdkwork-routes-drive-app-api/tests'),
  path.join(repoRoot, 'crates/sdkwork-routes-drive-backend-api/tests'),
  path.join(repoRoot, 'crates/sdkwork-routes-storage-backend-api/tests'),
]) {
  if (fs.existsSync(directory)) {
    walkTests(directory);
  }
}

for (const absolutePath of testFiles) {
  const before = fs.readFileSync(absolutePath, 'utf8');
  const after = stripTenantIdFromTestSources(before);
  if (before !== after) {
    fs.writeFileSync(absolutePath, after);
    process.stdout.write(`updated ${path.relative(repoRoot, absolutePath)}\n`);
  }
}
