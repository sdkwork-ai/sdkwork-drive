/**
 * Drive HTTP baseline load scaffold.
 * Run against a staging gateway with explicit operator approval.
 *
 * Example:
 *   DRIVE_LOAD_BASE_URL=https://api-staging.sdkwork.com \
 *   node tests/load/drive-http-baseline.mjs
 */
import assert from 'node:assert/strict';

const baseUrl = process.env.DRIVE_LOAD_BASE_URL?.replace(/\/$/, '');
const iterations = Number(process.env.DRIVE_LOAD_ITERATIONS ?? '50');

if (!baseUrl) {
  console.log('[drive-load] skipped: set DRIVE_LOAD_BASE_URL to run baseline');
  process.exit(0);
}

const targets = ['/healthz', '/readyz'];

let failures = 0;
const started = Date.now();

for (let index = 0; index < iterations; index += 1) {
  for (const path of targets) {
    const response = await fetch(`${baseUrl}${path}`);
    if (!response.ok) {
      failures += 1;
    }
  }
}

const elapsedMs = Date.now() - started;
const total = iterations * targets.length;
const successRate = (total - failures) / total;

console.log(
  `[drive-load] requests=${total} failures=${failures} successRate=${successRate.toFixed(4)} elapsedMs=${elapsedMs}`,
);

assert.ok(successRate >= 0.99, 'baseline success rate must be >= 99%');
