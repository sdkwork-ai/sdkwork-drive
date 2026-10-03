function recordOf(value: unknown): Record<string, unknown> {
  return value && typeof value === 'object' ? (value as Record<string, unknown>) : {};
}

export function formatMutationError(error: unknown, fallback: string): string {
  if (error instanceof Error && error.message.trim()) {
    const message = error.message.trim();
    if (message !== 'Failed to fetch' && message.length > 0) {
      return message;
    }
  }

  const payload = recordOf(error);
  const detail = payload.detail ?? payload.message ?? payload.error;
  if (typeof detail === 'string' && detail.trim()) {
    return detail.trim();
  }

  const nested = recordOf(payload.body ?? payload.data ?? payload.response);
  const nestedDetail = nested.detail ?? nested.message;
  if (typeof nestedDetail === 'string' && nestedDetail.trim()) {
    return nestedDetail.trim();
  }

  return fallback;
}

/**
 * 这个失败是不是"请求体太大"。
 *
 * 对象内容以 base64 装在 JSON 里传输，所以单次写入的真实体积约是文件大小的 1.37 倍。
 * 网关的请求体上限（`limits.maxRequestBodyBytes`）比内容接口的业务上限更早生效，超限时
 * 返回的是一个不带业务细节的 413（Problem 的 title/detail 都是 "Payload too large"）。
 * 调用方需要把这种"看起来像对象超限、其实是请求体超限"的失败单独解释给用户，否则用户会
 * 一直以为对象本身超了 8 MiB。
 */
export function isPayloadTooLargeError(error: unknown): boolean {
  const asRecord = recordOf(error);
  const status = asRecord.status ?? asRecord.statusCode ?? recordOf(asRecord.response).status;
  if (typeof status === 'number' && status === 413) {
    return true;
  }
  const code = asRecord.code;
  if (typeof code === 'number' && code === 41301) {
    return true;
  }
  const text = [
    error instanceof Error ? error.message : undefined,
    asRecord.detail,
    asRecord.title,
    asRecord.message,
  ]
    .filter((value): value is string => typeof value === 'string')
    .join(' ')
    .toLowerCase();
  return text.includes('payload too large') || text.includes('413');
}
