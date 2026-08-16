#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const target = path.join(
  path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..'),
  'crates/sdkwork-routes-drive-app-api/tests/command_routes.rs',
);
let source = fs.readFileSync(target, 'utf8');
const before = source;
source = source.replace(
  /,\r?\n(\s+)"tenant-[^"]+",\r?\n\1"user-[^"]+"\)/g,
  '\r\n$1)',
);
const count = (before.match(/"tenant-[^"]+",\r?\n\s+"user-[^"]+"\)/g) ?? []).length;
console.log(`removed ${count} extra fetch arg pairs`);
fs.writeFileSync(target, source);
