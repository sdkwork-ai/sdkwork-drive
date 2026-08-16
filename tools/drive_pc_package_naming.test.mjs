import assert from 'node:assert/strict';
import test from 'node:test';
import {
  containsLegacyPackageToken,
  expectedDrivePcPackageName,
} from './drive_pc_package_naming.mjs';

test('legacy token detection rejects exact retired names without rejecting canonical scoped packages', () => {
  assert.equal(containsLegacyPackageToken('"@sdkwork/drive-pc"', '@sdkwork/drive-pc'), true);
  assert.equal(
    containsLegacyPackageToken(
      '"@sdkwork/drive-pc-sandbox-explorer"',
      '@sdkwork/drive-pc',
    ),
    false,
  );
  assert.equal(
    containsLegacyPackageToken(
      'packages/sdkwork-drive-file/src/index.ts',
      'packages/sdkwork-drive-file',
    ),
    true,
  );
});

test('new sandbox packages use the scoped npm name required by NAMING_SPEC', () => {
  assert.equal(
    expectedDrivePcPackageName('sdkwork-drive-pc-sandbox-contracts'),
    '@sdkwork/drive-pc-sandbox-contracts',
  );
  assert.equal(
    expectedDrivePcPackageName('sdkwork-drive-pc-sandbox-explorer-sdk-adapter'),
    '@sdkwork/drive-pc-sandbox-explorer-sdk-adapter',
  );
  assert.equal(
    expectedDrivePcPackageName('sdkwork-drive-pc-core'),
    'sdkwork-drive-pc-core',
  );
});
