#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const testFiles = [
  'crates/sdkwork-routes-drive-app-api/tests/command_routes.rs',
  'crates/sdkwork-routes-drive-app-api/tests/observability_routes.rs',
  'crates/sdkwork-routes-drive-app-api/tests/version_routes.rs',
  'crates/sdkwork-routes-drive-app-api/tests/drive_routes.rs',
];

const TENANT_LITERAL = /'((?:tenant-[a-z0-9-]+))'/gi;

function defaultUserForTenant(tenant) {
  if (tenant.startsWith('tenant-')) {
    return `user-${tenant.slice('tenant-'.length)}`;
  }
  return 'user-001';
}

function lastMatch(text, pattern) {
  const flags = pattern.flags.includes('g') ? pattern.flags : `${pattern.flags}g`;
  const re = new RegExp(pattern.source, flags);
  let match = null;
  let current = re.exec(text);
  while (current) {
    match = current;
    current = re.exec(text);
  }
  return match?.[1] ?? null;
}

function mostCommon(values) {
  if (!values.length) {
    return null;
  }
  const counts = new Map();
  for (const value of values) {
    counts.set(value, (counts.get(value) ?? 0) + 1);
  }
  return [...counts.entries()].sort((a, b) => b[1] - a[1])[0][0];
}

function tenantFromUri(uri) {
  return uri.match(/[?&]tenantId=([^&"']+)/)?.[1] ?? null;
}

function userFromUri(uri) {
  return (
    uri.match(/[?&]subjectId=([^&"']+)/)?.[1]
    ?? uri.match(/[?&]operatorId=([^&"']+)/)?.[1]
    ?? uri.match(/[?&]userId=([^&"']+)/)?.[1]
    ?? null
  );
}

function appIdFromUri(uri) {
  return uri.match(/[?&]appId=([^&"']+)/)?.[1] ?? null;
}

function tenantFromJson(jsonText) {
  return jsonText.match(/"tenantId"\s*:\s*"([^"]+)"/)?.[1] ?? null;
}

function userFromJson(jsonText) {
  return (
    jsonText.match(/"operatorId"\s*:\s*"([^"]+)"/)?.[1]
    ?? jsonText.match(/"subjectId"\s*:\s*"([^"]+)"/)?.[1]
    ?? jsonText.match(/"userId"\s*:\s*"([^"]+)"/)?.[1]
    ?? jsonText.match(/"ownerSubjectId"\s*:\s*"([^"]+)"/)?.[1]
    ?? null
  );
}

function appIdFromJson(jsonText) {
  return jsonText.match(/"appId"\s*:\s*"([^"]+)"/)?.[1] ?? null;
}

function tenantFromSql(body) {
  const explicit = lastMatch(body, /tenant_id='([^']+)'/gi);
  if (explicit) {
    return explicit;
  }
  const bindTenant = lastMatch(body, /\.bind\("((?:tenant-[a-z0-9-]+))"\)/gi);
  if (bindTenant) {
    return bindTenant;
  }
  const literals = [...body.matchAll(TENANT_LITERAL)].map((match) => match[1]);
  return mostCommon(literals);
}

function userFromSql(body) {
  const owner = lastMatch(body, /owner_subject_id='([^']+)'/gi);
  if (owner) {
    return owner;
  }
  const createdBy = lastMatch(body, /created_by='([^']+)'/gi);
  if (createdBy) {
    return createdBy;
  }
  return lastMatch(body, /'((?:user|admin|operator)-[a-z0-9-]+)'/gi);
}

function inferAuthFromScope(scope) {
  const uriMatches = [...scope.matchAll(/\.uri\(([^)\n]+)\)/g)].map((match) => match[1]);
  const jsonMatches = [
    ...scope.matchAll(/Body::from\(\s*r#"(.*?)"#/gs),
    ...scope.matchAll(/Body::from\(\s*"(.*?)"\s*\)/gs),
    ...scope.matchAll(/let request_body = r#"(.*?)"#/gs),
  ].map((match) => match[1]);

  let tenant = null;
  let user = null;
  let appId = null;

  for (const uriLiteral of uriMatches) {
    tenant ??= tenantFromUri(uriLiteral);
    user ??= userFromUri(uriLiteral);
    appId ??= appIdFromUri(uriLiteral);
  }
  for (const jsonText of jsonMatches) {
    tenant ??= tenantFromJson(jsonText);
    user ??= userFromJson(jsonText);
    appId ??= appIdFromJson(jsonText);
  }

  tenant ??= tenantFromSql(scope) ?? 'tenant-001';
  user ??= userFromSql(scope) ?? defaultUserForTenant(tenant);
  appId ??= 'appbase';

  return { tenant, user, appId };
}

function stripTenantIdFromUri(uri) {
  return uri
    .replace(/([?&])tenantId=[^&"']+&/g, '$1')
    .replace(/&tenantId=[^&"']+/g, '')
    .replace(/\?tenantId=[^&"']+(?=")/g, '');
}

function stripTenantIdFromJson(jsonText) {
  return jsonText
    .replace(/"tenantId"\s*:\s*"[^"]*"\s*,\s*/g, '')
    .replace(/,\s*"tenantId"\s*:\s*"[^"]*"/g, '')
    .replace(/"tenantId"\s*:\s*"[^"]*"/g, '');
}

function authHeaderBlock(indent, tenant, user, appId) {
  return `${indent}.header(
${indent}    "authorization",
${indent}    format!("Bearer {}", common::auth_token("${tenant}", "${user}", "${appId}")),
${indent})
${indent}.header("access-token", common::access_token("${tenant}", "${user}", "${appId}"))`;
}

function stripAuthHeaders(block) {
  let cleaned = block;
  for (let pass = 0; pass < 4; pass += 1) {
    const next = cleaned
      .replace(
        /\n\s*\.header\(\s*\n\s*"authorization",[\s\S]*?\)\s*\n\s*\.header\(\s*\n\s*"access-token",[\s\S]*?\)\s*/g,
        '\n',
      )
      .replace(
        /\n\s*\.header\(\s*"authorization",[\s\S]*?\)\s*\n\s*\.header\(\s*"access-token",[\s\S]*?\)\s*/g,
        '\n',
      );
    if (next === cleaned) {
      break;
    }
    cleaned = next;
  }
  return cleaned;
}

function migrateRequestBlock(rest, auth, builderIndent) {
  let cleanedRest = stripAuthHeaders(rest);
  cleanedRest = cleanedRest.replace(/\.uri\("([^"]*)"\)/g, (_, uri) => `.uri("${stripTenantIdFromUri(uri)}")`);
  cleanedRest = cleanedRest.replace(
    /\.uri\(\s*\n\s*"([^"]*)"\s*\n\s*\)/g,
    (_, uri) => `.uri(\n                "${stripTenantIdFromUri(uri)}"\n            )`,
  );
  cleanedRest = cleanedRest.replace(/Body::from\(\s*r#"(.*?)"#/gs, (full, raw) => {
    if (!raw.includes('tenantId')) {
      return full;
    }
    return `Body::from(r#"${stripTenantIdFromJson(raw)}"#`;
  });
  cleanedRest = cleanedRest.replace(
    /format!\(\s*\n?\s*"([^"]*tenantId=[^"]*)"\s*\)/g,
    (_, template) => `format!(\n            "${stripTenantIdFromUri(template)}"\n        )`,
  );
  return `${authHeaderBlock(builderIndent, auth.tenant, auth.user, auth.appId)}\n${cleanedRest}`;
}

function migrateSource(source) {
  const lines = source.split('\n');
  let output = '';
  let index = 0;
  let testScopeStart = 0;

  while (index < lines.length) {
    const line = lines[index];

    if (/^async fn /.test(line)) {
      testScopeStart = index;
    }

    if (
      line.includes('async fn fetch_paged_items(')
      || line.includes('async fn fetch_json(')
    ) {
      output += `${line}\n`;
      index += 1;
      while (index < lines.length && !/^}\s*$/.test(lines[index])) {
        output += `${lines[index]}\n`;
        index += 1;
      }
      if (index < lines.length) {
        output += `${lines[index]}\n`;
        index += 1;
      }
      continue;
    }

    if (!line.includes('Request::builder()')) {
      output += `${line}\n`;
      index += 1;
      continue;
    }

    const builderIndent = line.match(/^(\s*)/)?.[1] ?? '';
    output += `${line}\n`;
    index += 1;

    let rest = '';
    while (index < lines.length) {
      rest += `${lines[index]}\n`;
      if (/^\s*\.expect\([^\n]+\)\s*,?\s*$/.test(lines[index])) {
        index += 1;
        break;
      }
      index += 1;
    }

    const scope = lines.slice(testScopeStart, index).join('\n');
    const blockScope = `${scope}\n${rest}`;
    const blockAuth = inferAuthFromScope(blockScope);
    const testAuth = inferAuthFromScope(scope);
    const auth = {
      tenant: blockAuth.tenant !== 'tenant-001' ? blockAuth.tenant : testAuth.tenant,
      user: blockAuth.user !== defaultUserForTenant(blockAuth.tenant) ? blockAuth.user : testAuth.user,
      appId: blockAuth.appId !== 'appbase' ? blockAuth.appId : testAuth.appId,
    };
    output += migrateRequestBlock(rest, auth, builderIndent);
  }

  return output.replace(/\n$/, '');
}

function ensureCommonModule(source) {
  if (source.includes('mod common;')) {
    return source;
  }
  const useEnd = source.lastIndexOf('\nuse ');
  if (useEnd === -1) {
    return `mod common;\n\n${source}`;
  }
  const lineEnd = source.indexOf('\n', useEnd + 1);
  return `${source.slice(0, lineEnd + 1)}\nmod common;\n${source.slice(lineEnd + 1)}`;
}

function updateFetchHelpers(source) {
  return source
    .replace(
      /common::auth_token\(&tenant, &user\)/g,
      'common::auth_token(&tenant, &user, "appbase")',
    )
    .replace(
      /common::access_token\(&tenant, &user\)/g,
      'common::access_token(&tenant, &user, "appbase")',
    );
}

function removeOrphanClosingParens(source) {
  return source.replace(
    /(\.header\("access-token", common::access_token\([^\n]+\)\))\r?\n\s+\)\r?\n/g,
    '$1\n',
  );
}

for (const relativeFile of testFiles) {
  const absolutePath = path.join(repoRoot, relativeFile);
  if (!fs.existsSync(absolutePath)) {
    continue;
  }
  let source = fs.readFileSync(absolutePath, 'utf8');
  source = ensureCommonModule(source);
  source = source.replace(
    /use sdkwork_routes_drive_app_api::build_router_with_pool;\n?/g,
    '',
  );
  source = source.replaceAll('build_router_with_pool(', 'common::test_router_with_pool(');
  source = migrateSource(source);
  source = updateFetchHelpers(source);
  source = removeOrphanClosingParens(source);
  fs.writeFileSync(absolutePath, `${source.replace(/\n$/, '')}\n`);
  const count = (source.match(/common::auth_token\(/g) ?? []).length;
  process.stdout.write(`${relativeFile}: normalized ${count} authed requests\n`);
}
