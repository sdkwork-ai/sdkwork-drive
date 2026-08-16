#!/usr/bin/env node
import fs from 'node:fs';

const files = [
  'crates/sdkwork-routes-drive-app-api/tests/command_routes.rs',
  'crates/sdkwork-routes-drive-app-api/tests/observability_routes.rs',
  'crates/sdkwork-routes-drive-app-api/tests/version_routes.rs',
  'crates/sdkwork-routes-drive-app-api/tests/drive_routes.rs',
];

for (const file of files) {
  let source = fs.readFileSync(file, 'utf8');
  const before = source;
  source = source.replace(
    /(\.header\("access-token", common::access_token\([^\n]+\)\))\r?\n\s+\)\r?\n/g,
    '$1\n',
  );
  if (source !== before) {
    fs.writeFileSync(file, source);
    console.log(`fixed orphan parens in ${file}`);
  }
}
