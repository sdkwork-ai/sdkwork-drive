import { describe, expect, it } from 'vitest';
import {
  countBucketObjectsByCategory,
  matchesBucketObjectCategory,
  objectExtension,
  resolveBucketObjectCategory,
} from '../src/utils/bucketObjectCategory';

describe('objectExtension', () => {
  it('reads the last segment extension in lower case', () => {
    expect(objectExtension('reports/2026/Q1.PDF')).toBe('pdf');
    expect(objectExtension('photo.jpeg')).toBe('jpeg');
  });

  it('answers empty for extensionless keys and dotfiles', () => {
    expect(objectExtension('docs/README')).toBe('');
    expect(objectExtension('docs/')).toBe('');
    expect(objectExtension('.gitignore')).toBe('');
    expect(objectExtension('archive.tar.')).toBe('');
  });
});

describe('resolveBucketObjectCategory', () => {
  it('classifies folders first, whatever the key looks like', () => {
    expect(resolveBucketObjectCategory({ key: 'photos/', isFolder: true })).toBe('folder');
  });

  it('classifies by extension', () => {
    expect(resolveBucketObjectCategory({ key: 'a/b.png', isFolder: false })).toBe('image');
    expect(resolveBucketObjectCategory({ key: 'a/b.mp4', isFolder: false })).toBe('video');
    expect(resolveBucketObjectCategory({ key: 'a/b.mp3', isFolder: false })).toBe('audio');
    expect(resolveBucketObjectCategory({ key: 'a/b.md', isFolder: false })).toBe('document');
    expect(resolveBucketObjectCategory({ key: 'a/b.zip', isFolder: false })).toBe('archive');
    expect(resolveBucketObjectCategory({ key: 'a/b.bin', isFolder: false })).toBe('other');
  });

  it('falls back to the content type when the key carries no extension', () => {
    expect(
      resolveBucketObjectCategory({ key: 'export', contentType: 'image/webp', isFolder: false }),
    ).toBe('image');
    expect(
      resolveBucketObjectCategory({
        key: 'export',
        contentType: 'application/pdf',
        isFolder: false,
      }),
    ).toBe('document');
    expect(
      resolveBucketObjectCategory({
        key: 'export',
        contentType: 'application/octet-stream',
        isFolder: false,
      }),
    ).toBe('other');
  });
});

describe('countBucketObjectsByCategory', () => {
  it('counts every category and the total', () => {
    const counts = countBucketObjectsByCategory([
      { key: 'docs/', isFolder: true },
      { key: 'a.png', isFolder: false },
      { key: 'b.png', isFolder: false },
      { key: 'c.zip', isFolder: false },
      { key: 'd', isFolder: false },
    ]);
    expect(counts).toMatchObject({
      all: 5,
      folder: 1,
      image: 2,
      archive: 1,
      other: 1,
      document: 0,
    });
  });
});

describe('matchesBucketObjectCategory', () => {
  it('treats all as unfiltered and otherwise compares the resolved category', () => {
    const image = { key: 'a.png', isFolder: false };
    const folder = { key: 'docs/', isFolder: true };
    expect(matchesBucketObjectCategory(image, 'all')).toBe(true);
    expect(matchesBucketObjectCategory(image, 'image')).toBe(true);
    expect(matchesBucketObjectCategory(image, 'folder')).toBe(false);
    expect(matchesBucketObjectCategory(folder, 'folder')).toBe(true);
  });
});
