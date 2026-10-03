import type { FilePreviewLabels } from '../i18n/filePreviewLabels';

/**
 * 把解析失败翻译成用户看得懂的本地化说明。
 *
 * 解析器抛出的 `Error.message` 是给开发者看的（中文技术描述、部件路径、字节偏移）。
 * 直接渲染它会让 en-US 的操作员读到中文，也会把内部结构泄露给用户。所以这里只认
 * 错误的 `code`：
 * - `dom-parser-unavailable` → 运行环境不支持（宿主问题，不是文件问题）；
 * - 其它已知 code（`not-a-zip`、`*-invalid`、`*-missing`、`corrupt-entry`…）→ 损坏/不符；
 * - 没有 code 的错误（宿主通道、网络、鉴权）→ 保留原始信息：那是调用方自己的错误，
 *   对运维有诊断价值。
 */
export function describePreviewError(cause: unknown, labels: FilePreviewLabels): string {
  const code = readErrorCode(cause);
  if (code === undefined) {
    return cause instanceof Error && cause.message.trim() !== ''
      ? cause.message
      : labels.loadFailed;
  }
  return code === 'dom-parser-unavailable' ? labels.parserUnavailableHint : labels.corruptFileHint;
}

function readErrorCode(cause: unknown): string | undefined {
  if (typeof cause !== 'object' || cause === null || !('code' in cause)) {
    return undefined;
  }
  const code = (cause as { code?: unknown }).code;
  return typeof code === 'string' && code !== '' ? code : undefined;
}
