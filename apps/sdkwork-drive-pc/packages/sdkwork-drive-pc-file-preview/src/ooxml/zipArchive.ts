/**
 * 无依赖的 ZIP 读取层，供 OOXML（docx/xlsx/pptx）预览取出归档内的 XML 部件。
 *
 * 之所以不引第三方 zip 库：预览只需要「按路径读一段字节」，而 Office 打包器写出的
 * 归档只用到 stored 与 deflate 两种方法，Node 24 与现代浏览器自带的
 * `DecompressionStream('deflate-raw')` 足以覆盖，为一次预览多背一个解析器依赖不划算。
 * 代价是这里不实现 ZIP64：字段一旦溢出就明确报错，而不是按 32 位字段解析出错误偏移后
 * 把别的文件内容当成部件。
 */

/** End of Central Directory 记录签名，小端写作 `PK\x05\x06`。 */
const EOCD_SIGNATURE = 0x06054b50;

/** ZIP64 End of Central Directory Locator 签名，小端写作 `PK\x06\x07`。 */
const ZIP64_LOCATOR_SIGNATURE = 0x07064b50;

/** Central Directory 文件头签名，小端写作 `PK\x01\x02`。 */
const CENTRAL_DIRECTORY_SIGNATURE = 0x02014b50;

/** Local File Header 签名，小端写作 `PK\x03\x04`。 */
const LOCAL_FILE_HEADER_SIGNATURE = 0x04034b50;

/** EOCD 定长部分长度（不含可变长度注释）。 */
const EOCD_FIXED_LENGTH = 22;

/** EOCD 注释长度是 16 位，所以向后回扫范围就是 64 KiB 注释再加上定长部分。 */
const EOCD_MAX_SCAN_LENGTH = 0xffff + EOCD_FIXED_LENGTH;

/** 32 位字段溢出到 ZIP64 扩展记录的占位值。 */
const ZIP64_PLACEHOLDER_32 = 0xffffffff;

/** 16 位字段溢出到 ZIP64 扩展记录的占位值。 */
const ZIP64_PLACEHOLDER_16 = 0xffff;

/** 通用位标记第 11 位：归档作者声明文件名按 UTF-8 编码。 */
const FLAG_UTF8_FILE_NAME = 0x0800;

/** 压缩方法 0 = stored，原样存放。 */
const COMPRESSION_METHOD_STORED = 0;

/** 压缩方法 8 = deflate（裸 deflate 流，不带 zlib 头）。 */
const COMPRESSION_METHOD_DEFLATE = 8;

/**
 * 归档内单个条目的解压上限（64 MiB）：压缩炸弹防护。
 *
 * 这是**硬上限**而不是"默认值"：`readZipArchive(bytes)` 不接受选项，任何宿主都改不了它。
 * 名字与文档必须一致，否则下一个维护者会以为可以调。可配置的同类参数是
 * `MAX_ARCHIVE_PREVIEW_ENTRIES` 配合 `readArchivePreview(bytes, { maxEntries })`。
 */
export const MAX_ZIP_ENTRY_BYTES = 64 * 1024 * 1024;

/** 归档内的单个条目。 */
export interface ZipEntry {
  /** 归档内完整路径，例如 `word/document.xml`。 */
  readonly path: string;
  readonly uncompressedSize: number;
  readonly compressedSize: number;
  /** 0 = stored，8 = deflate。 */
  readonly compressionMethod: number;
  readonly isDirectory: boolean;
  /** 按需解压该条目；重复调用应返回同样的字节。 */
  read(): Promise<Uint8Array>;
}

/** 解析完成的归档；只暴露按路径取内容所需要的最小面。 */
export interface ZipArchive {
  readonly entries: readonly ZipEntry[];
  readonly byPath: ReadonlyMap<string, ZipEntry>;
  readText(path: string): Promise<string | undefined>;
}

export type ZipArchiveErrorCode =
  | 'not-a-zip'
  | 'corrupt-entry'
  | 'unsupported-compression'
  | 'decompression-unavailable'
  | 'too-large';

/**
 * ZIP 读取失败的统一错误类型。
 *
 * `code` 是给调用方分支用的：预览层要能区分「这根本不是 zip（可以回退成文本预览）」与
 * 「是 zip 但内容坏了（应当报错）」，只看 message 做字符串匹配太脆。
 */
export class ZipArchiveError extends Error {
  readonly code: ZipArchiveErrorCode;

  constructor(code: ZipArchiveErrorCode, message: string) {
    super(message);
    this.name = 'ZipArchiveError';
    this.code = code;
  }
}

/** 单个条目的惰性读取器：构造阶段只记偏移，真正解压发生在 `read()`。 */
class ZipEntryReader implements ZipEntry {
  private cachedBytes?: Uint8Array;

  private pendingRead?: Promise<Uint8Array>;

  constructor(
    private readonly source: Uint8Array,
    private readonly view: DataView,
    readonly path: string,
    readonly uncompressedSize: number,
    readonly compressedSize: number,
    readonly compressionMethod: number,
    readonly isDirectory: boolean,
    private readonly localHeaderOffset: number,
  ) {}

  read(): Promise<Uint8Array> {
    if (this.cachedBytes) {
      return Promise.resolve(this.cachedBytes);
    }
    // 同一部件被并发读取（例如标题与正文同时取 core.xml）时只解压一次：
    // 缓存的是 Promise 而不是结果，后到的调用者直接挂到同一次解压上。
    if (!this.pendingRead) {
      this.pendingRead = this.inflate().then(
        (bytes) => {
          this.cachedBytes = bytes;
          return bytes;
        },
        (error: unknown) => {
          // 失败不能留在缓存里，否则同一次预览里换个时机重试会永远拿到旧错误。
          this.pendingRead = undefined;
          throw error;
        },
      );
    }
    return this.pendingRead;
  }

  private async inflate(): Promise<Uint8Array> {
    // 体积校验放在解压之前：压缩炸弹的「炸弹」部分正是声明的解压后大小，
    // 先看声明值就能在分配内存之前拒绝。
    if (this.uncompressedSize > MAX_ZIP_ENTRY_BYTES) {
      throw new ZipArchiveError(
        'too-large',
        `条目 ${this.path} 声明解压后 ${this.uncompressedSize} 字节，超过 ${MAX_ZIP_ENTRY_BYTES} 字节上限`,
      );
    }

    const dataOffset = this.resolveDataOffset();
    const dataEnd = dataOffset + this.compressedSize;
    if (dataEnd > this.source.length) {
      throw new ZipArchiveError(
        'corrupt-entry',
        `条目 ${this.path} 的数据区间 [${dataOffset}, ${dataEnd}) 超出归档长度 ${this.source.length}`,
      );
    }
    const compressed = this.source.subarray(dataOffset, dataEnd);

    let bytes: Uint8Array;
    if (this.compressionMethod === COMPRESSION_METHOD_STORED) {
      // stored 条目直接给出内容副本，避免返回值与调用方传入的归档缓冲区共享内存。
      bytes = compressed.slice();
    } else if (this.compressionMethod === COMPRESSION_METHOD_DEFLATE) {
      bytes = await inflateRaw(compressed, this.path);
    } else {
      throw new ZipArchiveError(
        'unsupported-compression',
        `条目 ${this.path} 使用压缩方法 ${this.compressionMethod}，只支持 0（stored）与 8（deflate）`,
      );
    }

    if (bytes.length !== this.uncompressedSize) {
      throw new ZipArchiveError(
        'corrupt-entry',
        `条目 ${this.path} 实际解出 ${bytes.length} 字节，与中央目录声明的 ${this.uncompressedSize} 字节不符`,
      );
    }
    return bytes;
  }

  /**
   * 本地文件头里的名字/扩展字段长度与中央目录的那份可能不同（中央目录常带 ZIP64、
   * Unix 权限等本地头没有的扩展字段），所以数据起点必须按本地头自己算。
   */
  private resolveDataOffset(): number {
    if (this.localHeaderOffset + 30 > this.source.length) {
      throw new ZipArchiveError(
        'corrupt-entry',
        `条目 ${this.path} 的本地文件头偏移 ${this.localHeaderOffset} 超出归档长度 ${this.source.length}`,
      );
    }
    if (this.view.getUint32(this.localHeaderOffset, true) !== LOCAL_FILE_HEADER_SIGNATURE) {
      throw new ZipArchiveError('corrupt-entry', `条目 ${this.path} 的本地文件头签名无效`);
    }
    const nameLength = this.view.getUint16(this.localHeaderOffset + 26, true);
    const extraLength = this.view.getUint16(this.localHeaderOffset + 28, true);
    return this.localHeaderOffset + 30 + nameLength + extraLength;
  }
}

/** 解析 End of Central Directory + Central Directory；失败抛 ZipArchiveError。 */
export function readZipArchive(bytes: Uint8Array): ZipArchive {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const eocdOffset = findEndOfCentralDirectory(view, bytes.length);
  if (eocdOffset < 0) {
    throw new ZipArchiveError('not-a-zip', '未找到 ZIP End of Central Directory 记录');
  }

  const totalEntries = view.getUint16(eocdOffset + 10, true);
  const centralDirectorySize = view.getUint32(eocdOffset + 12, true);
  const centralDirectoryOffset = view.getUint32(eocdOffset + 16, true);

  if (
    totalEntries === ZIP64_PLACEHOLDER_16
    || centralDirectorySize === ZIP64_PLACEHOLDER_32
    || centralDirectoryOffset === ZIP64_PLACEHOLDER_32
    || hasZip64Locator(view, eocdOffset)
  ) {
    throw new ZipArchiveError(
      'too-large',
      '归档使用 ZIP64 扩展记录（条目数或偏移超出 32 位），当前预览层不支持 ZIP64',
    );
  }

  if (centralDirectoryOffset + centralDirectorySize > bytes.length) {
    throw new ZipArchiveError(
      'corrupt-entry',
      `中央目录区间 [${centralDirectoryOffset}, ${centralDirectoryOffset + centralDirectorySize}) 超出归档长度 ${bytes.length}`,
    );
  }

  const entries: ZipEntryReader[] = [];
  const byPath = new Map<string, ZipEntry>();
  let offset = centralDirectoryOffset;

  for (let index = 0; index < totalEntries; index += 1) {
    if (offset + 46 > bytes.length) {
      throw new ZipArchiveError('corrupt-entry', `第 ${index + 1} 个中央目录条目被截断`);
    }
    if (view.getUint32(offset, true) !== CENTRAL_DIRECTORY_SIGNATURE) {
      throw new ZipArchiveError('corrupt-entry', `第 ${index + 1} 个中央目录条目签名无效`);
    }

    const flags = view.getUint16(offset + 8, true);
    const compressionMethod = view.getUint16(offset + 10, true);
    const compressedSize = view.getUint32(offset + 20, true);
    const uncompressedSize = view.getUint32(offset + 24, true);
    const nameLength = view.getUint16(offset + 28, true);
    const extraLength = view.getUint16(offset + 30, true);
    const commentLength = view.getUint16(offset + 32, true);
    const localHeaderOffset = view.getUint32(offset + 42, true);

    if (
      compressedSize === ZIP64_PLACEHOLDER_32
      || uncompressedSize === ZIP64_PLACEHOLDER_32
      || localHeaderOffset === ZIP64_PLACEHOLDER_32
    ) {
      throw new ZipArchiveError('too-large', `第 ${index + 1} 个条目的尺寸或偏移溢出到 ZIP64 扩展字段`);
    }

    const nameStart = offset + 46;
    const nameEnd = nameStart + nameLength;
    if (nameEnd > bytes.length) {
      throw new ZipArchiveError('corrupt-entry', `第 ${index + 1} 个条目的文件名超出归档长度`);
    }
    const path = decodeEntryPath(bytes.subarray(nameStart, nameEnd), (flags & FLAG_UTF8_FILE_NAME) !== 0);

    const entry = new ZipEntryReader(
      bytes,
      view,
      path,
      uncompressedSize,
      compressedSize,
      compressionMethod,
      // 目录条目按约定以 `/` 结尾；不看 external attributes 的 DOS 目录位，因为各家
      // 打包器对它的写法并不统一，误判会把真实文件当成目录而从列表里消失。
      path.endsWith('/'),
      localHeaderOffset,
    );
    entries.push(entry);
    // 归档里不应出现重复路径；保留首个可避免后写入的同名条目顶掉真实部件。
    if (!byPath.has(path)) {
      byPath.set(path, entry);
    }

    offset = nameEnd + extraLength + commentLength;
  }

  return {
    entries,
    byPath,
    readText: (path: string) => readArchiveText(byPath, path),
  };
}

/** 是否是 ZIP 归档：既认「以本地文件头开头」，也认只有 EOCD 的空归档。 */
export function isZipArchive(bytes: Uint8Array): boolean {
  if (bytes.length < 4) {
    return false;
  }
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  if (view.getUint32(0, true) === LOCAL_FILE_HEADER_SIGNATURE) {
    return true;
  }
  // 空归档（只有 EOCD）与带自解压头的数据没有开头的本地文件头，靠 EOCD 兜底。
  return findEndOfCentralDirectory(view, bytes.length) >= 0;
}

async function readArchiveText(
  byPath: ReadonlyMap<string, ZipEntry>,
  path: string,
): Promise<string | undefined> {
  const entry = byPath.get(path);
  if (!entry) {
    return undefined;
  }
  return decodeUtf8Text(stripByteOrderMark(await entry.read()));
}

/**
 * 从文件末尾向前找回扫 End of Central Directory。
 *
 * 回扫时先记下最后一个签名位置作为兜底，再优先返回「注释长度与到文件末尾的距离一致」
 * 的那个：注释里完全可能出现 `PK\x05\x06` 字节，但注释长度自洽的才是真记录。
 */
function findEndOfCentralDirectory(view: DataView, length: number): number {
  const scanStart = Math.max(0, length - EOCD_MAX_SCAN_LENGTH);
  let lastSignatureOffset = -1;
  for (let offset = length - EOCD_FIXED_LENGTH; offset >= scanStart; offset -= 1) {
    if (view.getUint32(offset, true) !== EOCD_SIGNATURE) {
      continue;
    }
    if (lastSignatureOffset < 0) {
      lastSignatureOffset = offset;
    }
    if (offset + EOCD_FIXED_LENGTH + view.getUint16(offset + 20, true) === length) {
      return offset;
    }
  }
  return lastSignatureOffset;
}

/** ZIP64 定位器紧邻 EOCD 之前；它的存在说明真正的计数在 ZIP64 记录里。 */
function hasZip64Locator(view: DataView, eocdOffset: number): boolean {
  if (eocdOffset < 20) {
    return false;
  }
  return view.getUint32(eocdOffset - 20, true) === ZIP64_LOCATOR_SIGNATURE;
}

function inflateRaw(compressed: Uint8Array, path: string): Promise<Uint8Array> {
  const source = typeof DecompressionStream === 'function' ? createCompressedStream(compressed) : undefined;
  if (!source || typeof Response !== 'function') {
    return Promise.reject(
      new ZipArchiveError(
        'decompression-unavailable',
        `当前运行时缺少 DecompressionStream('deflate-raw') 或可用的字节流构造方式，无法解压条目 ${path}`,
      ),
    );
  }
  const stream = source.pipeThrough(new DecompressionStream('deflate-raw'));
  return new Response(stream)
    .arrayBuffer()
    .then((buffer) => new Uint8Array(buffer))
    .catch((error: unknown) => {
      throw new ZipArchiveError('corrupt-entry', `条目 ${path} 的 deflate 数据无法解压：${describeError(error)}`);
    });
}

/**
 * 把压缩字节包成流。
 *
 * 首选 `Blob.stream()`；jsdom 提供了 Blob 却没有 `stream()`，所以补一条 `Response.body`
 * 退路——否则同一份压缩归档在浏览器里能预览、在 jsdom 测试环境里却报「运行时不支持解压」。
 */
function createCompressedStream(compressed: Uint8Array): ReadableStream<Uint8Array<ArrayBuffer>> | undefined {
  if (typeof Blob === 'function' && typeof Blob.prototype.stream === 'function') {
    return new Blob([toBlobPart(compressed)]).stream();
  }
  if (typeof Response === 'function') {
    return new Response(toBlobPart(compressed)).body ?? undefined;
  }
  return undefined;
}

/**
 * 转成 `Blob` / `Response` 能接受的字节视图。
 *
 * `BlobPart` 只接受由 `ArrayBuffer` 支撑的视图，而这里拿到的是
 * `Uint8Array<ArrayBufferLike>`（可能背后是 SharedArrayBuffer），用运行时判断而不是
 * 类型断言来收敛：普通 ArrayBuffer 零拷贝复用原内存，共享缓冲才复制一份。
 */
function toBlobPart(bytes: Uint8Array): Uint8Array<ArrayBuffer> {
  if (bytes.buffer instanceof ArrayBuffer) {
    return new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  }
  return new Uint8Array(bytes);
}

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function stripByteOrderMark(bytes: Uint8Array): Uint8Array {
  if (bytes.length >= 3 && bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf) {
    return bytes.subarray(3);
  }
  return bytes;
}

/** 部件内容都是 XML 文本，按 UTF-8 解码并丢掉 BOM。 */
function decodeUtf8Text(bytes: Uint8Array): string {
  return new TextDecoder('utf-8').decode(bytes).replace(/^\uFEFF/, '');
}

/**
 * 解码条目文件名。
 *
 * 位 11 是作者对「文件名是 UTF-8」的显式声明；未声明时仍然先按 UTF-8 解，因为 Word /
 * Excel 写中文部件名时普遍不置这一位却确实是 UTF-8，只有真正解不出来的字节才退回
 * latin-1 逐字节映射（GBK 归档至少还能显示出可辨认的单字节部分）。
 */
function decodeEntryPath(nameBytes: Uint8Array, utf8Declared: boolean): string {
  return utf8Declared
    ? decodeUtf8Bytes(nameBytes, false)
    : decodeUtf8Bytes(nameBytes, true);
}

/**
 * UTF-8 解码；`latin1Fallback` 为真时把非法字节按单字节码位原样保留。
 *
 * 自己实现而不用 `TextDecoder`：需要的是「逐个非法字节退化」而不是 `TextDecoder` 的
 * 整体替换或整体失败语义。
 */
function decodeUtf8Bytes(bytes: Uint8Array, latin1Fallback: boolean): string {
  let result = '';
  let index = 0;
  while (index < bytes.length) {
    const first = bytes[index] ?? 0;
    let codePoint = -1;
    let width = 1;

    if (first < 0x80) {
      codePoint = first;
    } else if (first >= 0xc2 && first <= 0xdf) {
      const second = bytes[index + 1];
      if (second !== undefined && isContinuationByte(second)) {
        codePoint = ((first & 0x1f) << 6) | (second & 0x3f);
        width = 2;
      }
    } else if (first >= 0xe0 && first <= 0xef) {
      const second = bytes[index + 1];
      const third = bytes[index + 2];
      if (
        second !== undefined
        && third !== undefined
        && isContinuationByte(second)
        && isContinuationByte(third)
        // 排除过长编码与代理区码位，它们不是合法 UTF-8。
        && !(first === 0xe0 && second < 0xa0)
        && !(first === 0xed && second > 0x9f)
      ) {
        codePoint = ((first & 0x0f) << 12) | ((second & 0x3f) << 6) | (third & 0x3f);
        width = 3;
      }
    } else if (first >= 0xf0 && first <= 0xf4) {
      const second = bytes[index + 1];
      const third = bytes[index + 2];
      const fourth = bytes[index + 3];
      if (
        second !== undefined
        && third !== undefined
        && fourth !== undefined
        && isContinuationByte(second)
        && isContinuationByte(third)
        && isContinuationByte(fourth)
        && !(first === 0xf0 && second < 0x90)
        && !(first === 0xf4 && second > 0x8f)
      ) {
        codePoint = ((first & 0x07) << 18)
          | ((second & 0x3f) << 12)
          | ((third & 0x3f) << 6)
          | (fourth & 0x3f);
        width = 4;
      }
    }

    if (codePoint < 0) {
      result += latin1Fallback ? String.fromCharCode(first) : '\uFFFD';
      index += 1;
      continue;
    }
    result += String.fromCodePoint(codePoint);
    index += width;
  }
  return result;
}

function isContinuationByte(byte: number): boolean {
  return (byte & 0xc0) === 0x80;
}
