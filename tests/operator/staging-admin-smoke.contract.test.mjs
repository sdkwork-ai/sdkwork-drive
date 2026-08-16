#!/usr/bin/env node
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const repoRoot = resolve(import.meta.dirname, '../..');
const smokeScript = readFileSync(
  resolve(repoRoot, 'scripts/operator/staging-admin-smoke.mjs'),
  'utf8',
);

assert.match(smokeScript, /SDKWORK_DRIVE_STAGING_BACKEND_BASE_URL/);
assert.match(smokeScript, /assertSdkWorkEnvelope/);
assert.match(smokeScript, /\/audit_events/);
assert.doesNotMatch(smokeScript, /pageSize=/);
assert.match(smokeScript, /page_size=20/);
assert.match(smokeScript, /\/quotas/);
assert.match(smokeScript, /\/maintenance\/object_sweep/);
assert.match(smokeScript, /expectForbidden/);
assert.match(smokeScript, /code !== 0/);
assert.match(smokeScript, /traceId/);

console.log('staging-admin-smoke.contract.test.mjs passed');
