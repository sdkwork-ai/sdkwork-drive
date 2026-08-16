#!/usr/bin/env node
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const repoRoot = resolve(import.meta.dirname, '../..');

function read(relativePath) {
  return readFileSync(resolve(repoRoot, relativePath), 'utf8');
}

const appOpenApi = read('apis/app-api/drive/drive-app-api.openapi.json');
const openOpenApi = read('apis/open-api/drive/drive-open-api.openapi.json');
const createShareLinkRoute = read('crates/sdkwork-routes-drive-app-api/src/share_link_handlers.rs');
const openHandlers = read('crates/sdkwork-routes-drive-open-api/src/handlers.rs');
const openRepository = read('crates/sdkwork-routes-drive-open-api/src/repository.rs');
const shareLinkModal = read(
  'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-file/src/components/ShareLinkModal.tsx',
);
const driveFileService = read(
  'apps/sdkwork-drive-pc/packages/sdkwork-drive-pc-core/src/services/driveFileService.ts',
);
const postgresSchema = read(
  'crates/sdkwork-drive-workspace-service/src/infrastructure/sql/postgres_core.sql',
);
const workspaceLib = read('crates/sdkwork-drive-workspace-service/src/lib.rs');
const metricsMiddleware = read('crates/sdkwork-drive-http/src/metrics.rs');
const rateLimitMiddleware = read('crates/sdkwork-drive-http/src/middleware/rate_limit.rs');
const runbook = read('docs/runbooks/drive-production-operations.md');

assert.match(appOpenApi, /"accessCode"/);
assert.match(appOpenApi, /"accessCodeRequired"/);
assert.match(openOpenApi, /"accessCodeRequired"/);
assert.match(openOpenApi, /"name": "accessCode"/);
assert.match(createShareLinkRoute, /access_code_hash/);
assert.match(createShareLinkRoute, /drive_share_access_code_hash/);
assert.match(openHandlers, /payload\.access_code\.as_deref\(\)/);
assert.match(openRepository, /share_link_access_code_matches/);
assert.match(shareLinkModal, /shareLinkAccessCode/);
assert.match(shareLinkModal, /accessCodeRequired/);
assert.match(driveFileService, /accessCodeRequired/);
assert.match(driveFileService, /assignDefined\(body, 'accessCode'/);
assert.match(postgresSchema, /access_code_hash/);
assert.match(workspaceLib, /share_link_access_code_matches/);
assert.match(metricsMiddleware, /record_http_request_duration/);
assert.match(rateLimitMiddleware, /record_http_rate_limited/);
assert.match(runbook, /drive_http_request_duration_seconds/);
assert.match(runbook, /X-SdkWork-Trace-Id/);
assert.doesNotMatch(runbook, /x-trace-id/);
assert.match(runbook, /drive-edge-rate-limit/);

assert.match(runbook, /\/share\/\{token\}/);
assert.match(runbook, /DRIVE_E2E_PC_SESSION_JSON/);
assert.match(runbook, /sdkwork-drive-iam/);

process.stdout.write('drive-share-link-flow.integration.test.mjs passed\n');
