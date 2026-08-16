#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const target = path.join(
  path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..'),
  'crates/sdkwork-routes-drive-app-api/tests/command_routes.rs',
);
let source = fs.readFileSync(target, 'utf8');
const sections = source.split(/(?=#\[tokio::test\])/g);
let updatedCalls = 0;

function inferTenant(section) {
  const match = section.match(/'((?:tenant|org)-[a-z0-9-]+)'/);
  return match?.[1] ?? null;
}

function addMissingTenant(section, tenant) {
  return section.replace(
    /fetch_(paged_items|json)\(([\s\S]*?)\)\s*\n(\s*)\.await/g,
    (match, fn, body, indent) => {
      if (body.includes(`"${tenant}"`) || body.includes(`'${tenant}'`)) {
        return match;
      }
      if (!body.includes('app')) {
        return match;
      }
      updatedCalls += 1;
      const trimmed = body.trimEnd();
      return `fetch_${fn}(${trimmed},\n${indent}"${tenant}",\n${indent})\n${indent}.await`;
    },
  );
}

source = sections
  .map((section) => {
    const tenant = inferTenant(section);
    if (!tenant) {
      return section;
    }
    return addMissingTenant(section, tenant);
  })
  .join('');
fs.writeFileSync(target, source);
process.stdout.write(`Added missing tenant argument to ${updatedCalls} fetch helper calls\n`);
