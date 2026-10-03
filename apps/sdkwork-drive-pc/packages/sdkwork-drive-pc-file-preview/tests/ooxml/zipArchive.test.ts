import { describe, expect, it } from 'vitest';
import {
  MAX_ZIP_ENTRY_BYTES,
  ZipArchiveError,
  isZipArchive,
  readZipArchive,
  type ZipArchive,
  type ZipEntry,
} from '../../src/ooxml/zipArchive';
import {
  ZIP_FIXTURE_FLAG_UTF8_FILE_NAME,
  buildZipArchive,
  corruptCentralDirectorySignature,
  insertZip64Locator,
  patchEocdEntryCount,
  textBytes,
  textOf,
} from './zipFixture';

function entryOf(archive: ZipArchive, path: string): ZipEntry {
  const entry = archive.byPath.get(path);
  if (!entry) {
    throw new Error(`夹具里没有条目 ${path}`);
  }
  return entry;
}

function captureError(action: () => unknown): unknown {
  try {
    action();
    return undefined;
  } catch (error) {
    return error;
  }
}

async function captureRejection(action: () => Promise<unknown>): Promise<unknown> {
  try {
    await action();
    return undefined;
  } catch (error) {
    return error;
  }
}

describe('readZipArchive', () => {
  it('walks the central directory and indexes entries by path', () => {
    const archive = readZipArchive(buildZipArchive([
      { path: '[Content_Types].xml', text: '<Types/>' },
      { path: 'docProps/', directory: true },
      { path: 'word/document.xml', text: '<w:document/>' },
    ]));

    expect(archive.entries.map((entry) => entry.path)).toEqual([
      '[Content_Types].xml',
      'docProps/',
      'word/document.xml',
    ]);
    expect(archive.byPath.size).toBe(3);

    const document = entryOf(archive, 'word/document.xml');
    expect(document.compressionMethod).toBe(0);
    expect(document.isDirectory).toBe(false);
    expect(document.uncompressedSize).toBe(textBytes('<w:document/>').length);
    expect(document.compressedSize).toBe(textBytes('<w:document/>').length);

    const directory = entryOf(archive, 'docProps/');
    expect(directory.isDirectory).toBe(true);
  });

  it('reads text parts and strips a leading byte-order mark', async () => {
    const archive = readZipArchive(buildZipArchive([
      { path: 'word/document.xml', text: '\uFEFF<w:document/>' },
    ]));

    await expect(archive.readText('word/document.xml')).resolves.toBe('<w:document/>');
  });

  it('returns undefined for a missing entry', async () => {
    const archive = readZipArchive(buildZipArchive([{ path: 'word/document.xml', text: '<w:document/>' }]));

    await expect(archive.readText('word/styles.xml')).resolves.toBeUndefined();
    expect(archive.byPath.get('word/styles.xml')).toBeUndefined();
  });

  it('reads an entry once and returns the same bytes afterwards', async () => {
    const archive = readZipArchive(buildZipArchive([
      { path: 'word/document.xml', text: '<w:document>cached</w:document>' },
    ]));
    const entry = entryOf(archive, 'word/document.xml');

    const [concurrentA, concurrentB] = await Promise.all([entry.read(), entry.read()]);
    expect(concurrentB).toBe(concurrentA);
    await expect(entry.read()).resolves.toBe(concurrentA);
    expect(textOf(concurrentA)).toBe('<w:document>cached</w:document>');
  });

  it('does not depend on CRC32 values', async () => {
    const archive = readZipArchive(buildZipArchive([
      { path: 'a.txt', text: 'zero-crc', crc32: 0 },
      { path: 'b.txt', text: 'bogus-crc', crc32: 0xdeadbeef },
    ]));

    await expect(archive.readText('a.txt')).resolves.toBe('zero-crc');
    await expect(archive.readText('b.txt')).resolves.toBe('bogus-crc');
  });

  it('computes the data offset from the local file header, not the central directory', async () => {
    // 本地文件头多出 12 字节扩展字段、中央目录里没有：若按中央目录的长度定位，
    // 数据起点会前移 12 字节，读到的是扩展字段的零字节。
    const archive = readZipArchive(buildZipArchive([
      { path: 'word/document.xml', text: '<w:document/>', localExtraLength: 12 },
    ]));

    await expect(archive.readText('word/document.xml')).resolves.toBe('<w:document/>');
  });

  it('rejects input that is not a zip archive', () => {
    const bytes = textBytes('这不是一个 zip 文件，只是一段普通文本');
    expect(isZipArchive(bytes)).toBe(false);

    const error = captureError(() => readZipArchive(bytes));
    expect(error).toBeInstanceOf(ZipArchiveError);
    expect(error).toMatchObject({ code: 'not-a-zip' });
  });

  it('recognises a zip archive by its local file header', () => {
    expect(isZipArchive(buildZipArchive([{ path: 'a.txt', text: 'a' }]))).toBe(true);
  });

  it('rejects archives whose end of central directory record is truncated', () => {
    const valid = buildZipArchive([{ path: 'a.txt', text: 'a' }]);
    const error = captureError(() => readZipArchive(valid.subarray(0, valid.length - 8)));
    expect(error).toMatchObject({ code: 'not-a-zip' });
  });

  it('rejects a damaged central directory', () => {
    const bytes = corruptCentralDirectorySignature(buildZipArchive([{ path: 'a.txt', text: 'a' }]));
    const error = captureError(() => readZipArchive(bytes));
    expect(error).toBeInstanceOf(ZipArchiveError);
    expect(error).toMatchObject({ code: 'corrupt-entry' });
  });

  it('rejects ZIP64 archives rather than parsing 32-bit fields as garbage', () => {
    const entries = [{ path: 'a.txt', text: 'a' }];
    const overflowed = captureError(() => readZipArchive(patchEocdEntryCount(buildZipArchive(entries), 0xffff)));
    expect(overflowed).toMatchObject({ code: 'too-large' });

    const located = captureError(() => readZipArchive(insertZip64Locator(buildZipArchive(entries))));
    expect(located).toMatchObject({ code: 'too-large' });
  });

  it('rejects compression methods other than stored and deflate', async () => {
    const archive = readZipArchive(buildZipArchive([
      { path: 'a.bin', text: 'payload', method: 12 },
    ]));
    const error = await captureRejection(() => entryOf(archive, 'a.bin').read());
    expect(error).toBeInstanceOf(ZipArchiveError);
    expect(error).toMatchObject({ code: 'unsupported-compression' });
  });

  it('rejects entries whose declared uncompressed size exceeds the entry limit', async () => {
    const archive = readZipArchive(buildZipArchive([
      { path: 'bomb.xml', text: 'tiny', declaredUncompressedSize: MAX_ZIP_ENTRY_BYTES + 1 },
    ]));

    const error = await captureRejection(() => entryOf(archive, 'bomb.xml').read());
    expect(error).toBeInstanceOf(ZipArchiveError);
    expect(error).toMatchObject({ code: 'too-large' });
    expect(MAX_ZIP_ENTRY_BYTES).toBe(64 * 1024 * 1024);
  });

  it('rejects entries whose inflated length disagrees with the declared size', async () => {
    const archive = readZipArchive(buildZipArchive([
      { path: 'a.txt', text: 'actual', declaredUncompressedSize: 99 },
    ]));

    const error = await captureRejection(() => entryOf(archive, 'a.txt').read());
    expect(error).toMatchObject({ code: 'corrupt-entry' });
  });

  it('decodes file names as UTF-8 even without the bit 11 flag', () => {
    const archive = readZipArchive(buildZipArchive([
      { path: 'word/文档.xml', text: 'x' },
      { path: 'word/备注.xml', text: 'y', flags: ZIP_FIXTURE_FLAG_UTF8_FILE_NAME },
    ]));

    expect(archive.entries.map((entry) => entry.path)).toEqual(['word/文档.xml', 'word/备注.xml']);
  });

  it('falls back to latin-1 for file name bytes that are not valid UTF-8', () => {
    // 0x80 单独出现不是合法 UTF-8：未声明 UTF-8 时按单字节码位保留，声明了则替换。
    const invalidName = new Uint8Array([0x66, 0x6f, 0x80, 0x2e, 0x74, 0x78, 0x74]);
    const archive = readZipArchive(buildZipArchive([
      { pathBytes: invalidName, text: 'latin' },
      { pathBytes: invalidName, text: 'replaced', flags: ZIP_FIXTURE_FLAG_UTF8_FILE_NAME },
    ]));

    expect(archive.entries.map((entry) => entry.path)).toEqual(['fo\u0080.txt', 'fo\uFFFD.txt']);
  });

  it.skipIf(typeof DecompressionStream !== 'function')(
    'inflates deflate entries with DecompressionStream',
    async () => {
      const payload = '压缩内容 '.repeat(200);
      const archive = readZipArchive(buildZipArchive([
        { path: 'word/document.xml', text: payload, method: 8 },
      ]));
      const entry = entryOf(archive, 'word/document.xml');

      expect(entry.compressionMethod).toBe(8);
      expect(entry.compressedSize).toBeLessThan(entry.uncompressedSize);
      await expect(archive.readText('word/document.xml')).resolves.toBe(payload);
    },
  );

  it('reports decompression as unavailable when the runtime cannot inflate raw deflate', async () => {
    const archive = readZipArchive(buildZipArchive([
      { path: 'a.txt', text: 'x', method: 8 },
    ]));
    const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'DecompressionStream');
    if (!descriptor || descriptor.configurable !== true) {
      // 运行时本来就没有 DecompressionStream（或不允许删掉它），直接断言即可。
      const error = await captureRejection(() => entryOf(archive, 'a.txt').read());
      expect(error).toMatchObject({ code: 'decompression-unavailable' });
      return;
    }

    Reflect.deleteProperty(globalThis, 'DecompressionStream');
    try {
      const error = await captureRejection(() => entryOf(archive, 'a.txt').read());
      expect(error).toBeInstanceOf(ZipArchiveError);
      expect(error).toMatchObject({ code: 'decompression-unavailable' });
    } finally {
      Object.defineProperty(globalThis, 'DecompressionStream', descriptor);
    }
  });
});
