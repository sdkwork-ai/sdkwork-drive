export function isDriveConflictError(error: unknown): boolean {
  if (typeof error !== 'object' || error === null) {
    return false;
  }
  const record = error as { status?: unknown; code?: unknown };
  if (record.status === 409) {
    return true;
  }
  if (typeof record.code === 'number') {
    const code = record.code;
    if (code === 40901 || (code >= 40900 && code < 41000)) {
      return true;
    }
  }
  return (
    typeof record.code === 'string' &&
    /(?:conflict|already_exists|duplicate|40901)/i.test(record.code)
  );
}
