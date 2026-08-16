#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const files = [
  'crates/sdkwork-routes-drive-app-api/tests/command_routes.rs',
  'crates/sdkwork-routes-drive-app-api/tests/observability_routes.rs',
  'crates/sdkwork-routes-drive-app-api/tests/version_routes.rs',
  'crates/sdkwork-routes-drive-app-api/tests/drive_routes.rs',
];

function stripTenantIdLiterals(source) {
  return source
    .replace(/\?tenantId=[^&"']+&/g, '?')
    .replace(/&tenantId=[^&"']+/g, '')
    .replace(/\?tenantId=[^&"']+(?=")/g, '');
}

function stripTenantIdFromJson(source) {
  return source
    .replace(/"tenantId"\s*:\s*"[^"]*"\s*,\s*\n/g, '')
    .replace(/"tenantId"\s*:\s*"[^"]*"\s*,\s*/g, '')
    .replace(/,\s*\n?\s*"tenantId"\s*:\s*"[^"]*"/g, '')
    .replace(/"tenantId"\s*:\s*"[^"]*"\s*\n/g, '');
}

for (const relativeFile of files) {
  const absolutePath = path.join(repoRoot, relativeFile);
  let source = fs.readFileSync(absolutePath, 'utf8');
  source = source.replace(/,\s*,/g, ',');
  source = source.replace(
    /(\n\s+"[^"]+",\n\s+"[^"]+"\))\n(\s+\.await;)/g,
    (match, ending, awaitLine) => `${ending.slice(0, -1)},\n    )${awaitLine}`,
  );
  source = stripTenantIdLiterals(source);
  source = stripTenantIdFromJson(source);
  fs.writeFileSync(absolutePath, source);
  process.stdout.write(`${relativeFile}: syntax and tenantId literals cleaned\n`);
}
