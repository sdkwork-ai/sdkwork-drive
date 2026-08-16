#!/usr/bin/env node
import assert from 'node:assert/strict';
import { mkdtempSync, existsSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

const repoRoot = resolve(import.meta.dirname, '../..');
const tempRoot = mkdtempSync(join(tmpdir(), 'sdkwork-drive-sdk-stub-'));

try {
  const inputPath = join(tempRoot, 'openapi.json');
  const outputPath = join(tempRoot, 'generated');
  writeFileSync(
    inputPath,
    JSON.stringify({
      openapi: '3.0.3',
      info: { title: 'Stub Fixture', version: '0.0.0' },
      paths: {
        '/fixture': {
          get: {
            operationId: 'fixture.list',
            responses: {
              200: {
                description: 'ok',
              },
            },
          },
        },
      },
    }),
    'utf8',
  );

  const result = spawnSync(
    process.execPath,
    [
      'tools/sdkwork_sdk_generator_stub.mjs',
      'generate',
      '--input',
      inputPath,
      '--output',
      outputPath,
      '--name',
      'sdkwork-drive-app-sdk',
      '--type',
      'app',
      '--language',
      'typescript',
      '--base-url',
      'https://example.test',
      '--api-prefix',
      '/app/v3/api',
      '--fixed-sdk-version',
      '0.1.0',
      '--sdk-root',
      tempRoot,
      '--sdk-name',
      'sdkwork-drive-app-sdk',
      '--package-name',
      'sdkwork-drive-app-sdk-generated-typescript',
      '--standard-profile',
      'sdkwork-v3',
    ],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );

  assert.notEqual(result.status, 0);
  assert.match(`${result.stderr}\n${result.stdout}`, /canonical sdkgen/i);
  assert.equal(existsSync(join(outputPath, 'src', 'index.ts')), false);
} finally {
  rmSync(tempRoot, { force: true, recursive: true });
}

console.log('sdk-generator-stub.contract.test.mjs passed');
