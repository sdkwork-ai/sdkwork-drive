#!/usr/bin/env node

const message = [
  'tools/sdkwork_sdk_generator_stub.mjs is a fail-closed compatibility tombstone.',
  'Drive SDK generation must use canonical sdkgen at ../sdkwork-sdk-generator/bin/sdkgen.js through tools/drive_sdk_generator_runner.mjs.',
  'Local stub generators must not produce committed SDK family output.',
].join('\n');

process.stderr.write(`${message}\n`);
process.exit(1);
