import { describe, expect, it } from 'vitest';
import { MAX_ARCHIVE_PREVIEW_ENTRIES, readArchivePreview } from '../../src/ooxml/archivePreview';
import { ZipArchiveError } from '../../src/ooxml/zipArchive';
import { buildZipArchive, textBytes } from './zipFixture';

const SAMPLE_ENTRIES = [
  { path: 'b.txt', text: 'b' },
  { path: 'z/', directory: true },
  { path: 'a/', directory: true },
  { path: 'a.txt', text: 'aa' },
  { path: 'nested/c.txt', text: 'ccc' },
];

describe('readArchivePreview', () => {
  it('lists directories first, then files, each alphabetically', () => {
    const model = readArchivePreview(buildZipArchive(SAMPLE_ENTRIES));

    expect(model.entries.map((entry) => entry.path)).toEqual([
      'a/',
      'z/',
      'a.txt',
      'b.txt',
      'nested/c.txt',
    ]);
    expect(model.entries.map((entry) => entry.isDirectory)).toEqual([
      true,
      true,
      false,
      false,
      false,
    ]);
    expect(model.truncated).toBe(false);
  });

  it('computes totals over every entry, before truncating', () => {
    const model = readArchivePreview(buildZipArchive(SAMPLE_ENTRIES), { maxEntries: 2 });

    expect(model.entries.map((entry) => entry.path)).toEqual(['a/', 'z/']);
    expect(model.truncated).toBe(true);
    // 1 + 2 + 3 字节：截断只影响返回的切片，不影响总量。
    expect(model.totalUncompressedBytes).toBe(textBytes('a').length + textBytes('aa').length + textBytes('ccc').length);
  });

  it('reports sizes per entry', () => {
    const payload = '<w:document>重复内容重复内容重复内容重复内容</w:document>';
    const model = readArchivePreview(buildZipArchive([
      { path: 'word/document.xml', text: payload, method: 8 },
    ]));
    const entry = model.entries[0];

    expect(entry?.path).toBe('word/document.xml');
    expect(entry?.uncompressedSize).toBe(textBytes(payload).length);
    // deflate 夹具确实被压缩过：目录里读到的是压缩后长度。
    expect(entry?.compressedSize).toBeLessThan(entry?.uncompressedSize ?? 0);
  });

  it('honours an explicit entry cap and defaults to the exported limit', () => {
    const bytes = buildZipArchive(SAMPLE_ENTRIES);

    expect(MAX_ARCHIVE_PREVIEW_ENTRIES).toBe(2000);
    expect(readArchivePreview(bytes, { maxEntries: 0 }).entries).toEqual([]);
    expect(readArchivePreview(bytes, { maxEntries: 0 }).truncated).toBe(true);
    expect(readArchivePreview(bytes, { maxEntries: 99 }).entries).toHaveLength(5);
    expect(readArchivePreview(bytes, { maxEntries: 99 }).truncated).toBe(false);
    expect(readArchivePreview(bytes, { maxEntries: -1 }).entries).toHaveLength(5);
  });

  it('rejects payloads that are not zip archives', () => {
    expect(() => readArchivePreview(textBytes('plain text'))).toThrowError(ZipArchiveError);
  });
});
