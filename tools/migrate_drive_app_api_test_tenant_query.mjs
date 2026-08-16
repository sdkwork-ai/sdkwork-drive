#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const testRoots = [
  'crates/sdkwork-routes-drive-app-api/tests',
  'crates/sdkwork-routes-drive-backend-api/tests',
];

function stripTenantIdQuery(uri) {
  const [pathPart, queryPart] = uri.split('?');
  if (!queryPart) {
    return pathPart;
  }
  const filtered = queryPart
    .split('&')
    .filter((segment) => segment.length > 0 && !segment.startsWith('tenantId='))
    .join('&');
  return filtered.length > 0 ? `${pathPart}?${filtered}` : pathPart;
}

function migrateFile(relativePath) {
  const absolutePath = path.join(repoRoot, relativePath);
  const source = fs.readFileSync(absolutePath, 'utf8');
  let updated = 0;
  const next = source.replace(/(["'`])(\/[^"'`]*?)\1/g, (match, quote, uri) => {
    if (!uri.includes('tenantId=')) {
      return match;
    }
    const cleaned = stripTenantIdQuery(uri);
    if (cleaned === uri) {
      return match;
    }
    updated += 1;
    return `${quote}${cleaned}${quote}`;
  });
  if (updated > 0) {
    fs.writeFileSync(absolutePath, next);
  }
  return updated;
}

let total = 0;
for (const relativeRoot of testRoots) {
  const absoluteRoot = path.join(repoRoot, relativeRoot);
  for (const entry of fs.readdirSync(absoluteRoot, { withFileTypes: true })) {
    if (!entry.isFile() || !entry.name.endsWith('.rs')) {
      continue;
    }
    const relativePath = path.join(relativeRoot, entry.name).replace(/\\/g, '/');
    total += migrateFile(relativePath);
  }
}

process.stdout.write(`Removed tenantId query params from ${total} test URI literals\n`);
