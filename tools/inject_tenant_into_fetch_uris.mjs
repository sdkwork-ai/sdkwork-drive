#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const target = path.join(
  path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..'),
  'crates/sdkwork-routes-drive-app-api/tests/command_routes.rs',
);
const lines = fs.readFileSync(target, 'utf8').split(/\r?\n/);
let currentTenant = 'tenant-001';
let inFetchCall = false;
let parenDepth = 0;

function injectTenant(uri, tenant) {
  if (!tenant) {
    return uri;
  }
  const withoutTenant = uri
    .replace(/([?&])tenantId=[^&"']+&/g, '$1')
    .replace(/&tenantId=[^&"']+/g, '')
    .replace(/\?tenantId=[^&"']+(?=$)/g, '');
  if (withoutTenant.includes('?')) {
    return withoutTenant.replace('?', `?tenantId=${tenant}&`);
  }
  return `${withoutTenant}?tenantId=${tenant}`;
}

for (let index = 0; index < lines.length; index += 1) {
  const line = lines[index];
  if (/^async fn .+\(\) \{$/.test(line.trim())) {
    currentTenant = 'tenant-001';
  }
  const tenantMatches = [...line.matchAll(/'(tenant-[a-z0-9-]+)'/g)];
  if (tenantMatches.length > 0) {
    currentTenant = tenantMatches.at(-1)[1];
  }

  if (/fetch_(json|paged_items)\(/.test(line)) {
    inFetchCall = true;
    parenDepth = 0;
  }
  if (!inFetchCall) {
    continue;
  }
  parenDepth += (line.match(/\(/g) ?? []).length;
  parenDepth -= (line.match(/\)/g) ?? []).length;

  const literalMatch = line.match(/^(\s*)"(.+)"(,)?\s*$/);
  if (literalMatch && literalMatch[2].startsWith('/app/v3/api/')) {
    const uri = injectTenant(literalMatch[2], currentTenant);
    lines[index] = `${literalMatch[1]}"${uri}"${literalMatch[3] ?? ','}`;
  }
  const formatLineMatch = line.match(/^(\s*)"(\/app\/v3\/api\/[^"]+)"\s*$/);
  if (formatLineMatch) {
    lines[index] = `${formatLineMatch[1]}"${injectTenant(formatLineMatch[2], currentTenant)}"`;
  }

  if (inFetchCall && parenDepth <= 0 && line.includes(')')) {
    inFetchCall = false;
  }
}

fs.writeFileSync(target, `${lines.join('\n')}\n`);
process.stdout.write('injected tenantId into fetch helper URIs\n');
