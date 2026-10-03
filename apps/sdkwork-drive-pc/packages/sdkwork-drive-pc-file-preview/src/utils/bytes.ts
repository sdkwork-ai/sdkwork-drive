/**
 * 预览层自用的字节/文本工具。
 *
 * 这个包不能依赖宿主字典或格式化模块（否则管理端与 drive 控制台就得把它反向依赖
 * 进各自的 commons），所以尺寸、时间与编码转换在这里各留一份最小实现。
 */

const SIZE_UNITS = ['B', 'KB', 'MB', 'GB', 'TB'] as const;

/** 人类可读的字节数（1024 进制，保留一位小数）。 */
export function formatByteSize(bytes: number | undefined): string {
  if (bytes === undefined || !Number.isFinite(bytes) || bytes < 0) {
    return '—';
  }
  if (bytes < 1024) {
    return `${Math.round(bytes)} B`;
  }
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < SIZE_UNITS.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value >= 100 ? Math.round(value) : value.toFixed(1)} ${SIZE_UNITS[unitIndex]}`;
}

/** UTF-8 文本解码；去掉 BOM，避免首行多出一个不可见字符。 */
export function decodeUtf8Text(bytes: Uint8Array): string {
  const withoutBom =
    bytes.length >= 3 && bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf
      ? bytes.subarray(3)
      : bytes;
  return new TextDecoder('utf-8').decode(withoutBom);
}

export function encodeUtf8Text(text: string): Uint8Array {
  return new TextEncoder().encode(text);
}

/** base64 → 字节。宿主若只提供 base64 通道，用它落地。 */
export function base64ToBytes(base64: string): Uint8Array {
  const binary = globalThis.atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}

/** 字节 → base64（编辑器保存回去时用）。 */
export function bytesToBase64(bytes: Uint8Array): string {
  let binary = '';
  const chunk = 0x8000;
  for (let index = 0; index < bytes.length; index += chunk) {
    binary += String.fromCharCode(...bytes.subarray(index, index + chunk));
  }
  return globalThis.btoa(binary);
}

/**
 * 二进制探测：文本预览只在内容确实是文本时才解码。
 *
 * NUL 字节在 UTF-8 文本里不合法，而二进制格式（图片、压缩包、Office）前几百字节
 * 几乎必然出现它；用这个廉价判据避免把二进制当文本渲染成一片乱码。
 */
export function looksLikeBinary(bytes: Uint8Array, sampleSize = 512): boolean {
  const end = Math.min(bytes.length, sampleSize);
  for (let index = 0; index < end; index += 1) {
    if (bytes[index] === 0) {
      return true;
    }
  }
  return false;
}

/** 文本行数（空文本算 0 行）。 */
export function countTextLines(text: string): number {
  return text === '' ? 0 : text.split('\n').length;
}

/** 单词数：以空白切分，中英混排时中文按字计（与主流编辑器的粗算一致）。 */
export function countTextWords(text: string): number {
  const trimmed = text.trim();
  if (trimmed === '') {
    return 0;
  }
  return trimmed.split(/\s+/).length;
}
