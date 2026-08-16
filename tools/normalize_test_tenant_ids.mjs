#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const TENANT_PATTERN = /tenant-[a-z0-9-]+/g;

/** Scenarios that intentionally use non-numeric tenant strings (negative tests). */
const SKIP_LITERALS = new Set([
  'tenant' + '-bootstrap',
]);

/** Stable first-party defaults from SUBJECT_ID_SPEC bootstrap. */
const PRESET_MAP = new Map([
  ['tenant-user', '100001'],
  ['tenant-002', '100002'],
  ['tenant-quota', '100001'],
]);

function walk(dir, files = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === 'node_modules' || entry.name === 'target') {
      continue;
    }
    const absolute = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walk(absolute, files);
      continue;
    }
    if (entry.name.endsWith('.rs')) {
      files.push(absolute);
    }
  }
  return files;
}

function collectTenantLiterals(files) {
  const literals = new Set();
  for (const file of files) {
    const text = fs.readFileSync(file, 'utf8');
    for (const match of text.matchAll(TENANT_PATTERN)) {
      if (!SKIP_LITERALS.has(match[0])) {
        literals.add(match[0]);
      }
    }
  }
  return [...literals].sort();
}

function buildMap(literals) {
  const map = new Map(PRESET_MAP);
  let next = 100010;
  for (const literal of literals) {
    if (map.has(literal)) {
      continue;
    }
    map.set(literal, String(next));
    next += 1;
  }
  return map;
}

function replaceInFile(file, map) {
  const original = fs.readFileSync(file, 'utf8');
  let updated = original;
  const sortedKeys = [...map.keys()].sort((a, b) => b.length - a.length);
  for (const literal of sortedKeys) {
    const numeric = map.get(literal);
    updated = updated.replaceAll(literal, numeric);
  }
  if (updated !== original) {
    fs.writeFileSync(file, updated);
    return true;
  }
  return false;
}

const files = walk(repoRoot);
const literals = collectTenantLiterals(files);
const map = buildMap(literals);
let changed = 0;
for (const file of files) {
  if (replaceInFile(file, map)) {
    changed += 1;
    console.log(path.relative(repoRoot, file));
  }
}
console.log(`normalized ${literals.length} tenant literals across ${changed} files`);
