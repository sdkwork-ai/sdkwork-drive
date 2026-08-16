#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const target = path.join(
  path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..'),
  'crates/sdkwork-routes-drive-app-api/tests/command_routes.rs',
);

let source = fs.readFileSync(target, 'utf8');
let replacements = 0;

source = source.replace(
  /(\w+)\["nextPageToken"\]\.as_str\(\)\.is_some\(\)/g,
  (_match, varName) => {
    replacements += 1;
    return `common::envelope_next_page_token(&${varName}).is_some()`;
  },
);

source = source.replace(
  /assert!\((\w+)\["nextPageToken"\]\.is_null\(\)\)/g,
  (_match, varName) => {
    replacements += 1;
    return `assert!(common::envelope_next_page_token(&${varName}).is_none())`;
  },
);

source = source.replace(
  /assert_eq!\((\w+)\["nextPageToken"\]\.as_str\(\), Some\("([^"]+)"\)\)/g,
  (_match, varName, token) => {
    replacements += 1;
    return `assert_eq!(common::envelope_next_page_token(&${varName}).as_deref(), Some("${token}"))`;
  },
);

source = source.replace(
  /let next_page_token = (\w+)\["nextPageToken"\]\s*\n\s*\.as_str\(\)\s*\n\s*\.expect\("([^"]+)"\)\s*\n\s*\.to_string\(\)/g,
  (_match, varName, message) => {
    replacements += 1;
    return `let next_page_token = common::envelope_next_page_token(&${varName}).expect("${message}")`;
  },
);

source = source.replace(
  /let next_page_token = (\w+)\["nextPageToken"\]\s*\n\s*\.as_str\(\)\s*\n\s*\.expect\("([^"]+)"\)/g,
  (_match, varName, message) => {
    replacements += 1;
    return `let next_page_token = common::envelope_next_page_token(&${varName}).expect("${message}")`;
  },
);

source = source.replace(
  /assert_eq!\((\w+)\["nextCursor"\]\.as_i64\(\), Some\((\d+)\)\)/g,
  (_match, varName, value) => {
    replacements += 1;
    return `assert_eq!(common::envelope_next_page_token(&${varName}).as_deref(), Some("${value}"))`;
  },
);

fs.writeFileSync(target, source);
process.stdout.write(
  `migrate_command_routes_page_tokens: updated ${replacements} pagination token assertions\n`,
);
