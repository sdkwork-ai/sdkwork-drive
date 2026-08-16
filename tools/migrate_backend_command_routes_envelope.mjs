#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const target = path.join(
  path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..'),
  'crates/sdkwork-routes-drive-backend-api/tests/command_routes.rs',
);

const helpers = `fn envelope_items(payload: &serde_json::Value) -> &serde_json::Value {
    if let Some(items) = payload.pointer("/data/items") {
        return items;
    }
    payload
        .get("items")
        .expect("response should expose list items in data.items or items")
}

fn envelope_next_page_token(payload: &serde_json::Value) -> Option<String> {
    payload
        .pointer("/data/pageInfo/nextCursor")
        .or_else(|| payload.get("pageInfo").and_then(|info| info.get("nextCursor")))
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .or_else(|| {
            payload
                .get("nextPageToken")
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
}

`;

let source = fs.readFileSync(target, 'utf8');
if (!source.includes('fn envelope_items(')) {
  source = source.replace(
    'use tower::util::ServiceExt;\n\nasync fn fetch_backend_paged_items',
    `use tower::util::ServiceExt;\n\n${helpers}async fn fetch_backend_paged_items`,
  );
}

source = source.replace(
  /(\w+)\["items"\]/g,
  (match, varName) => `envelope_items(&${varName})`,
);

source = source.replace(
  /let next_page_token = payload\["nextPageToken"\]\.as_str\(\)\.map\(ToString::to_string\)/,
  'let next_page_token = envelope_next_page_token(&payload)',
);

fs.writeFileSync(target, source);
process.stdout.write('migrate_backend_command_routes_envelope: updated backend test helpers\n');
