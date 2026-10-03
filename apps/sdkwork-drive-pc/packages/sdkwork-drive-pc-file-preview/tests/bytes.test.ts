import { describe, expect, it } from 'vitest';
import {
  base64ToBytes,
  bytesToBase64,
  countTextLines,
  countTextWords,
  decodeUtf8Text,
  encodeUtf8Text,
  formatByteSize,
  looksLikeBinary,
} from '../src/utils/bytes';

describe('formatByteSize', () => {
  it('formats bytes, kilobytes and megabytes the way a file list needs', () => {
    expect(formatByteSize(0)).toBe('0 B');
    expect(formatByteSize(512)).toBe('512 B');
    expect(formatByteSize(1024)).toBe('1.0 KB');
    expect(formatByteSize(1536)).toBe('1.5 KB');
    expect(formatByteSize(25_600)).toBe('25.0 KB');
    expect(formatByteSize(8 * 1024 * 1024)).toBe('8.0 MB');
  });

  it('answers a dash for unknown sizes rather than NaN', () => {
    expect(formatByteSize(undefined)).toBe('—');
    expect(formatByteSize(Number.NaN)).toBe('—');
    expect(formatByteSize(-1)).toBe('—');
  });
});

describe('text and bytes conversion', () => {
  it('round-trips UTF-8 text through base64', () => {
    const bytes = encodeUtf8Text('存储桶文件 hello');
    expect(decodeUtf8Text(base64ToBytes(bytesToBase64(bytes)))).toBe('存储桶文件 hello');
  });

  it('strips a UTF-8 BOM so the first line is not polluted', () => {
    const withBom = new Uint8Array([0xef, 0xbb, 0xbf, 0x61, 0x62]);
    expect(decodeUtf8Text(withBom)).toBe('ab');
  });
});

describe('looksLikeBinary', () => {
  it('detects NUL bytes that mark a binary payload', () => {
    expect(looksLikeBinary(new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0x00]))).toBe(true);
    expect(looksLikeBinary(encodeUtf8Text('plain text'))).toBe(false);
  });
});

describe('text statistics', () => {
  it('counts lines and words', () => {
    expect(countTextLines('a\nb\nc')).toBe(3);
    expect(countTextLines('')).toBe(0);
    expect(countTextWords('  hello   world  ')).toBe(2);
    expect(countTextWords('   ')).toBe(0);
  });
});
