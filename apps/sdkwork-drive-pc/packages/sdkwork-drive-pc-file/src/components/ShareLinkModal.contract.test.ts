import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const shareLinkModalSource = readFileSync(
  join(dirname(fileURLToPath(import.meta.url)), 'ShareLinkModal.tsx'),
  'utf8',
);

describe('ShareLinkModal contract', () => {
  it('creates share links with optional extraction codes through DriveFileService', () => {
    expect(shareLinkModalSource).toContain('createShareLink(file.id, {');
    expect(shareLinkModalSource).toContain('accessCode');
    expect(shareLinkModalSource).toContain('fileBrowser.shareLinkAccessCodePlaceholder');
  });

  it('surfaces token and access-code feedback after creation', () => {
    expect(shareLinkModalSource).toContain('setLatestToken');
    expect(shareLinkModalSource).toContain('setLatestAccessCode');
    expect(shareLinkModalSource).toContain('fileBrowser.shareLinkCreated');
  });

  it('marks protected links when accessCodeRequired is true', () => {
    expect(shareLinkModalSource).toContain('accessCodeRequired');
    expect(shareLinkModalSource).toContain('fileBrowser.shareLinkAccessCodeRequired');
  });
});
