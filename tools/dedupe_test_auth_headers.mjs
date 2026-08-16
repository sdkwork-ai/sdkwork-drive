#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const testFiles = [
  'crates/sdkwork-routes-drive-app-api/tests/command_routes.rs',
  'crates/sdkwork-routes-drive-app-api/tests/observability_routes.rs',
  'crates/sdkwork-routes-drive-app-api/tests/version_routes.rs',
  'crates/sdkwork-routes-drive-app-api/tests/drive_routes.rs',
];

function stripAllAuthHeaders(source) {
  let cleaned = source;
  const patterns = [
    /\.header\(\s*\n\s*"authorization",[\s\S]*?\)\s*\n\s*\.header\(\s*\n?\s*"access-token",[\s\S]*?\)\s*/g,
    /\.header\(\s*"authorization",[\s\S]*?\)\s*\n\s*\.header\(\s*"access-token",[\s\S]*?\)\s*/g,
  ];
  for (const pattern of patterns) {
    let next = cleaned.replace(pattern, '');
    while (next !== cleaned) {
      cleaned = next;
      next = cleaned.replace(pattern, '');
    }
  }
  return cleaned
    .replace(/\n\s*\)\s*\n(\s*\.method)/g, '\n$1')
    .replace(/(\.header\("access-token", common::access_token\([^\n]+\)\))\n\s+\)\n/g, '$1\n');
}

for (const relativeFile of testFiles) {
  const absolutePath = path.join(repoRoot, relativeFile);
  if (!fs.existsSync(absolutePath)) {
    continue;
  }
  const before = (fs.readFileSync(absolutePath, 'utf8').match(/auth_token/g) ?? []).length;
  const cleaned = stripAllAuthHeaders(fs.readFileSync(absolutePath, 'utf8'));
  fs.writeFileSync(absolutePath, cleaned);
  const after = (cleaned.match(/auth_token/g) ?? []).length;
  process.stdout.write(`${relativeFile}: removed auth headers (${before} -> ${after})\n`);
}
