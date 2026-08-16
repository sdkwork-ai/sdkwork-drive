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
