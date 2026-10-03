/**
 * 列表里的修改时间。
 *
 * 内容接口返回的是 ISO 8601 字符串（`2026-09-30T10:00:00Z`）；直接摆到界面上既占宽度
 * 又难读，主流网盘都会转成本地时间。这里用 `Intl` 按宿主语言格式化，因此不需要新增
 * 任何 i18n 词条，也不会出现「英文界面里混着中文日期」。
 *
 * 规则贴近行业网盘：
 * - 今天 → 只显示时分（列表里最常看的就是"几点改的"）；
 * - 今年内 → 月/日 + 时分；
 * - 更早 → 完整短日期；
 * - 缺失或无法解析 → `—`（而不是把原始串漏出去）。
 */
export function formatModifiedTime(value: string | undefined, language: string): string {
  if (value === undefined || value.trim() === '') {
    return '—';
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return '—';
  }

  const now = new Date();
  const sameDay =
    date.getFullYear() === now.getFullYear()
    && date.getMonth() === now.getMonth()
    && date.getDate() === now.getDate();

  const options: Intl.DateTimeFormatOptions = sameDay
    ? { hour: '2-digit', minute: '2-digit' }
    : date.getFullYear() === now.getFullYear()
      ? { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }
      : { year: 'numeric', month: 'short', day: 'numeric' };

  try {
    return new Intl.DateTimeFormat(language, options).format(date);
  } catch {
    // 语言标签不合法时退回运行环境默认区域，绝不因为格式化失败而丢掉时间。
    return new Intl.DateTimeFormat(undefined, options).format(date);
  }
}

/** 供 `<time dateTime>` 使用的机器可读值；无法解析时返回 undefined。 */
export function modifiedTimeIso(value: string | undefined): string | undefined {
  if (value === undefined || value.trim() === '') {
    return undefined;
  }
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? undefined : date.toISOString();
}
