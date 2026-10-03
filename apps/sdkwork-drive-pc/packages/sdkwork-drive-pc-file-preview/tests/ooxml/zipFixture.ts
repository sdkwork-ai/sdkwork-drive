import { deflateRawSync } from 'node:zlib';

/**
 * 测试专用的极简 ZIP 打包器。
 *
 * 刻意不复用 `src/ooxml/zipArchive.ts` 的任何代码：用被测实现去构造夹具，一旦它把
 * 偏移算错，夹具会跟着一起错，测试就永远发现不了。这里的字段偏移按 ZIP 规范独立书写。
 */

const LOCAL_FILE_HEADER_SIGNATURE = 0x04034b50;
const CENTRAL_DIRECTORY_SIGNATURE = 0x02014b50;
const EOCD_SIGNATURE = 0x06054b50;
const ZIP64_LOCATOR_SIGNATURE = 0x07064b50;
const EOCD_FIXED_LENGTH = 22;

const COMPRESSION_METHOD_STORED = 0;
const COMPRESSION_METHOD_DEFLATE = 8;

/** 2.0，MS-DOS 宿主；与 Word/Excel 写出的归档一致。 */
const VERSION_MADE_BY = 20;

export const ZIP_FIXTURE_FLAG_UTF8_FILE_NAME = 0x0800;

export interface ZipFixtureEntrySpec {
  /** 归档内路径；`directory: true` 时自动补结尾斜杠。 */
  path?: string;
  /** 直接给出文件名原始字节，用于构造非法 UTF-8 名字。 */
  pathBytes?: Uint8Array;
  text?: string;
  data?: Uint8Array;
  /** 0 = stored（默认），8 = deflate；其它值用于「不支持的压缩方法」用例。 */
  method?: number;
  /** 只写一个目录条目（无数据）。 */
  directory?: boolean;
  /** 通用位标记，例如 `ZIP_FIXTURE_FLAG_UTF8_FILE_NAME`。 */
  flags?: number;
  /** 中央目录里声明的解压后大小，用于构造超过上限的「压缩炸弹」条目。 */
  declaredUncompressedSize?: number;
  /** 本地文件头的扩展字段长度；内容为零字节，用于验证数据偏移按本地头计算。 */
  localExtraLength?: number;
  /** CRC32 字段；默认 0，因为读取器不应校验它。 */
  crc32?: number;
}

interface PreparedEntry {
  pathBytes: Uint8Array;
  payload: Uint8Array;
  method: number;
  flags: number;
  localExtraLength: number;
  uncompressedSize: number;
  declaredUncompressedSize: number;
  crc32: number;
}

export function textBytes(text: string): Uint8Array {
  return new TextEncoder().encode(text);
}

export function textOf(bytes: Uint8Array): string {
  return new TextDecoder('utf-8').decode(bytes);
}

export function buildZipArchive(specs: readonly ZipFixtureEntrySpec[]): Uint8Array {
  const prepared = specs.map(prepareEntry);
  const localBytes = prepared.reduce(
    (total, entry) => total + 30 + entry.pathBytes.length + entry.localExtraLength + entry.payload.length,
    0,
  );
  const centralBytes = prepared.reduce((total, entry) => total + 46 + entry.pathBytes.length, 0);
  const bytes = new Uint8Array(localBytes + centralBytes + EOCD_FIXED_LENGTH);
  const view = new DataView(bytes.buffer);

  let offset = 0;
  const localOffsets: number[] = [];
  for (const entry of prepared) {
    localOffsets.push(offset);
    view.setUint32(offset, LOCAL_FILE_HEADER_SIGNATURE, true);
    view.setUint16(offset + 4, VERSION_MADE_BY, true);
    view.setUint16(offset + 6, entry.flags, true);
    view.setUint16(offset + 8, entry.method, true);
    view.setUint16(offset + 10, 0, true);
    view.setUint16(offset + 12, 0, true);
    view.setUint32(offset + 14, entry.crc32, true);
    view.setUint32(offset + 18, entry.payload.length, true);
    view.setUint32(offset + 22, entry.uncompressedSize, true);
    view.setUint16(offset + 26, entry.pathBytes.length, true);
    view.setUint16(offset + 28, entry.localExtraLength, true);
    offset += 30;
    bytes.set(entry.pathBytes, offset);
    offset += entry.pathBytes.length;
    offset += entry.localExtraLength;
    bytes.set(entry.payload, offset);
    offset += entry.payload.length;
  }

  const centralDirectoryOffset = offset;
  for (const [index, entry] of prepared.entries()) {
    view.setUint32(offset, CENTRAL_DIRECTORY_SIGNATURE, true);
    view.setUint16(offset + 4, VERSION_MADE_BY, true);
    view.setUint16(offset + 6, VERSION_MADE_BY, true);
    view.setUint16(offset + 8, entry.flags, true);
    view.setUint16(offset + 10, entry.method, true);
    view.setUint16(offset + 12, 0, true);
    view.setUint16(offset + 14, 0, true);
    view.setUint32(offset + 16, entry.crc32, true);
    view.setUint32(offset + 20, entry.payload.length, true);
    view.setUint32(offset + 24, entry.declaredUncompressedSize, true);
    view.setUint16(offset + 28, entry.pathBytes.length, true);
    view.setUint16(offset + 30, 0, true);
    view.setUint16(offset + 32, 0, true);
    view.setUint16(offset + 34, 0, true);
    view.setUint16(offset + 36, 0, true);
    view.setUint32(offset + 38, 0, true);
    view.setUint32(offset + 42, localOffsets[index] ?? 0, true);
    offset += 46;
    bytes.set(entry.pathBytes, offset);
    offset += entry.pathBytes.length;
  }

  view.setUint32(offset, EOCD_SIGNATURE, true);
  view.setUint16(offset + 4, 0, true);
  view.setUint16(offset + 6, 0, true);
  view.setUint16(offset + 8, prepared.length, true);
  view.setUint16(offset + 10, prepared.length, true);
  view.setUint32(offset + 12, offset - centralDirectoryOffset, true);
  view.setUint32(offset + 16, centralDirectoryOffset, true);
  view.setUint16(offset + 20, 0, true);

  return bytes;
}

function prepareEntry(spec: ZipFixtureEntrySpec): PreparedEntry {
  const path = spec.directory === true && spec.path !== undefined && !spec.path.endsWith('/')
    ? `${spec.path}/`
    : spec.path ?? '';
  const pathBytes = spec.pathBytes ?? textBytes(path);
  const data = spec.directory === true ? new Uint8Array(0) : spec.data ?? textBytes(spec.text ?? '');
  const method = spec.method ?? COMPRESSION_METHOD_STORED;
  const payload = method === COMPRESSION_METHOD_DEFLATE ? new Uint8Array(deflateRawSync(data)) : data;

  return {
    pathBytes,
    payload,
    method,
    flags: spec.flags ?? 0,
    localExtraLength: spec.localExtraLength ?? 0,
    uncompressedSize: data.length,
    declaredUncompressedSize: spec.declaredUncompressedSize ?? data.length,
    crc32: spec.crc32 ?? 0,
  };
}

/** 覆写 EOCD 的「条目总数」字段，模拟 ZIP64 归档声明溢出的场景。 */
export function patchEocdEntryCount(bytes: Uint8Array, value: number): Uint8Array {
  const patched = new Uint8Array(bytes);
  new DataView(patched.buffer).setUint16(findEocdOffset(patched) + 10, value, true);
  return patched;
}

/** 在 EOCD 之前插入 ZIP64 Locator，模拟「字段没溢出但确实是 ZIP64」的归档。 */
export function insertZip64Locator(bytes: Uint8Array): Uint8Array {
  const eocdOffset = findEocdOffset(bytes);
  const extended = new Uint8Array(bytes.length + 20);
  extended.set(bytes.subarray(0, eocdOffset), 0);
  const view = new DataView(extended.buffer);
  view.setUint32(eocdOffset, ZIP64_LOCATOR_SIGNATURE, true);
  view.setUint32(eocdOffset + 4, 0, true);
  view.setUint32(eocdOffset + 8, 0, true);
  view.setUint32(eocdOffset + 12, 0, true);
  view.setUint32(eocdOffset + 16, 1, true);
  extended.set(bytes.subarray(eocdOffset), eocdOffset + 20);
  return extended;
}

/** 破坏中央目录签名，用于构造「是 zip 但目录损坏」的样本。 */
export function corruptCentralDirectorySignature(bytes: Uint8Array): Uint8Array {
  const corrupted = new Uint8Array(bytes);
  const view = new DataView(corrupted.buffer, corrupted.byteOffset, corrupted.byteLength);
  const eocdOffset = findEocdOffset(corrupted);
  const centralDirectoryOffset = view.getUint32(eocdOffset + 16, true);
  view.setUint32(centralDirectoryOffset, 0x11223344, true);
  return corrupted;
}

function findEocdOffset(bytes: Uint8Array): number {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  for (let offset = bytes.length - EOCD_FIXED_LENGTH; offset >= 0; offset -= 1) {
    if (view.getUint32(offset, true) === EOCD_SIGNATURE) {
      return offset;
    }
  }
  throw new Error('夹具构建失败：找不到 EOCD');
}
