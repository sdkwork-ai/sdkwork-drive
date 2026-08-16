const TOKEN_CONTINUATION = /[a-z0-9-]/iu;

const SCOPED_PACKAGE_NAMES = new Map([
  ['sdkwork-drive-pc-sandbox-contracts', '@sdkwork/drive-pc-sandbox-contracts'],
  ['sdkwork-drive-pc-sandbox-explorer', '@sdkwork/drive-pc-sandbox-explorer'],
  [
    'sdkwork-drive-pc-sandbox-explorer-sdk-adapter',
    '@sdkwork/drive-pc-sandbox-explorer-sdk-adapter',
  ],
]);

export function expectedDrivePcPackageName(packageDirectoryName) {
  return SCOPED_PACKAGE_NAMES.get(packageDirectoryName) ?? packageDirectoryName;
}

export function containsLegacyPackageToken(source, token) {
  let searchFrom = 0;
  while (searchFrom < source.length) {
    const index = source.indexOf(token, searchFrom);
    if (index < 0) return false;
    const nextCharacter = source[index + token.length];
    if (!nextCharacter || !TOKEN_CONTINUATION.test(nextCharacter)) {
      return true;
    }
    searchFrom = index + token.length;
  }
  return false;
}
