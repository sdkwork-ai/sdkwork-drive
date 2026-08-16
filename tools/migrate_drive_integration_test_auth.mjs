#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const testRoot = path.join(repoRoot, 'crates/sdkwork-routes-drive-app-api/tests');

function defaultTenantUser(source) {
  if (source.includes('tenant-git-repository-delete-guard')) {
    return ['tenant-git-repository-delete-guard', 'user-owner'];
  }
  if (source.includes('tenant-list-guard')) {
    return ['tenant-list-guard', 'user-owner'];
  }
  if (source.includes('tenant-visibility')) {
    return ['tenant-visibility', 'user-owner'];
  }
  if (source.includes('tenant-quota')) {
    return ['tenant-quota', 'user-owner'];
  }
  if (source.includes('tenant-app')) {
    return ['tenant-app', 'admin-app'];
  }
  if (source.includes('tenant-upload-id-conflict')) {
    return ['tenant-upload-id-conflict', 'user-owner'];
  }
  if (source.includes('tenant-keygen')) {
    return ['tenant-keygen', 'user-owner'];
  }
  return ['tenant-001', 'user-001'];
}

function injectAuthHeaders(source) {
  if (!source.includes('Request::builder()')) {
    return source;
  }

  const [tenant, user] = defaultTenantUser(source);
  const headerBlock = `                .header(
                    "authorization",
                    format!("Bearer {}", common::auth_token("${tenant}", "${user}")),
                )
                .header("access-token", common::access_token("${tenant}", "${user}"))`;

  let updated = source;
  if (!updated.includes('mod common;')) {
    updated = updated.replace(
      /use axum::body::\{to_bytes, Body\};/u,
      'mod common;\n\nuse axum::body::{to_bytes, Body};',
    );
  }

  updated = updated.replace(
    /Request::builder\(\)\s*\n(\s*)\.method\((Method::[A-Z]+)\)\s*\n\s*\.uri\(([^)]+)\)\s*\n(\s*)(\.header\("content-type", "application\/json"\)\s*\n)?/gu,
    (match, indent, method, uri, nextIndent, contentTypeHeader) => {
      if (match.includes('"authorization"')) {
        return match;
      }
      const contentType = contentTypeHeader ?? '';
      return `Request::builder()
${indent}.method(${method})
${indent}.uri(${uri})
${indent}${headerBlock}
${contentType}${nextIndent}`;
    },
  );

  return updated;
}

for (const entry of fs.readdirSync(testRoot, { withFileTypes: true })) {
  if (!entry.isFile() || !entry.name.endsWith('.rs')) {
    continue;
  }
  if (entry.name === 'iam_auth_guard.rs') {
    continue;
  }
  const absolutePath = path.join(testRoot, entry.name);
  const before = fs.readFileSync(absolutePath, 'utf8');
  const after = injectAuthHeaders(before);
  if (before !== after) {
    fs.writeFileSync(absolutePath, after);
    process.stdout.write(`injected auth headers in ${entry.name}\n`);
  }
}
