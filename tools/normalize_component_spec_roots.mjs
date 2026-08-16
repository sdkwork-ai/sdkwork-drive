#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const checkOnly = process.argv.includes('--check');
const failures = [];
let updated = 0;

function walk(directory) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const absolutePath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === 'node_modules' || entry.name === 'target' || entry.name === 'generated') {
        continue;
      }
      walk(absolutePath);
      continue;
    }
    if (entry.name !== 'component.spec.json') {
      continue;
    }

    const relativePath = path.relative(repoRoot, absolutePath).split(path.sep).join('/');
    const expectedRoot = '.';
    const source = fs.readFileSync(absolutePath, 'utf8');
    const spec = JSON.parse(source);
    const currentRoot = spec.component?.root;
    if (currentRoot === expectedRoot) {
      continue;
    }

    if (checkOnly) {
      failures.push(`${relativePath} component.root must be "." (found ${currentRoot ?? 'missing'})`);
      continue;
    }

    spec.component.root = expectedRoot;
    fs.writeFileSync(absolutePath, `${JSON.stringify(spec, null, 2)}\n`, 'utf8');
    updated += 1;
    console.log(`[normalize] ${relativePath}: ${currentRoot} -> ${expectedRoot}`);
  }
}

walk(repoRoot);

if (checkOnly) {
  if (failures.length > 0) {
    process.stderr.write(`Component spec root standard failed:\n${failures.map((failure) => `- ${failure}`).join('\n')}\n`);
    process.exit(1);
  }
  process.stdout.write('Component spec root standard passed\n');
  process.exit(0);
}

console.log(`[normalize] updated ${updated} component.spec.json files`);
