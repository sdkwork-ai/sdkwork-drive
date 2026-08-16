import { describe, expect, it } from 'vitest';
import type { DriveFile } from 'sdkwork-drive-pc-types';
import {
  isDefaultFileBrowserSort,
  mergeUniqueDriveFiles,
} from './fileBrowserPageFetcher';

function makeFile(id: string): DriveFile {
  return {
    id,
    name: id,
    type: 'file',
    ownerId: 'owner',
    updatedAt: '2026-01-01T00:00:00.000Z',
    size: 1,
  };
}

describe('fileBrowserPageFetcher', () => {
  it('detects default name ascending sort', () => {
    expect(isDefaultFileBrowserSort('name', 'asc')).toBe(true);
    expect(isDefaultFileBrowserSort('size', 'asc')).toBe(false);
  });

  it('merges pages without duplicate ids', () => {
    const merged = mergeUniqueDriveFiles(
      [makeFile('a')],
      [makeFile('a'), makeFile('b')],
    );
    expect(merged.map((file) => file.id)).toEqual(['a', 'b']);
  });

  it('deduplicates repeated resources within the first page', () => {
    const firstOccurrence = {
      ...makeFile('recent-file'),
      name: 'Newest recent entry',
    };
    const duplicateOccurrence = {
      ...makeFile('recent-file'),
      name: 'Older duplicate entry',
    };

    const merged = mergeUniqueDriveFiles(
      [],
      [firstOccurrence, duplicateOccurrence, makeFile('another-file')],
    );

    expect(merged.map((file) => file.id)).toEqual([
      'recent-file',
      'another-file',
    ]);
    expect(merged[0]?.name).toBe('Newest recent entry');
  });
});
