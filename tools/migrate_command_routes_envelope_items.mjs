#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const target = path.join(
  repoRoot,
  'crates/sdkwork-routes-drive-app-api/tests/command_routes.rs',
);

let source = fs.readFileSync(target, 'utf8');
let replacements = 0;

source = source.replace(
  /(\w+)\["items"\]/g,
  (match, varName) => {
    replacements += 1;
    return `common::envelope_items(&${varName})`;
  },
);

// Restore accidental double-wrapping for already-envelope-aware paths.
source = source.replace(
  /common::envelope_items\(&(\w+)\)\["data"\]\["items"\]/g,
  '$1["data"]["items"]',
);

fs.writeFileSync(target, source);
process.stdout.write(
  `migrate_command_routes_envelope_items: updated ${replacements} list item accessors\n`,
);
