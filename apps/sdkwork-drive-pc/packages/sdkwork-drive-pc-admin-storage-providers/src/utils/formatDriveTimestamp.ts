/**
 * 存储管理界面里的时间显示。
 *
 * 服务层只搬 ISO 8601 字符串，格式化一律发生在这一层，因为只有组件知道宿主语言：
 * `new Date(epoch).toLocaleDateString()` 读的是**浏览器**区域，于是英文控制台会显示出
 * 中文/系统格式的日期，再被列表里的 `Intl` 格式化一遭（同一行两种格式，甚至要先解析
 * 一次本地化字符串才拿得回时间）。一条链上只允许一个格式化者，就是这里。
 *
 * 两个函数共用一套兜底：无法解析的值返回 `—`，绝不把原始串漏到界面上。
 */

/** 把 ISO 时间解析成 `Date`；缺失或不可解析时返回 undefined。 */
function parseIso(value: string | undefined): Date | undefined {
  if (value === undefined || value.trim() === '') {
    return undefined;
  }
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? undefined : date;
}

/**
 * `Intl` 格式化，语言标签不合法时退回运行环境默认区域。
 *
 * 格式化失败绝不能让时间消失：宁可格式不够完美，也不能在一张运维台账上留空。
 */
function formatWith(
  date: Date,
  language: string,
  options: Intl.DateTimeFormatOptions,
): string {
  try {
    return new Intl.DateTimeFormat(language, options).format(date);
  } catch {
    return new Intl.DateTimeFormat(undefined, options).format(date);
  }
}

/** 只要日期（存储桶创建时间这类"哪一天"的事实）。 */
export function formatDriveDate(value: string | undefined, language: string): string {
  const date = parseIso(value);
  if (!date) {
    return '—';
  }
  return formatWith(date, language, { year: 'numeric', month: 'short', day: 'numeric' });
}

/** 日期 + 时分（对象最后修改时间这类"什么时候动的"）。 */
export function formatDriveDateTime(value: string | undefined, language: string): string {
  const date = parseIso(value);
  if (!date) {
    return '—';
  }
  return formatWith(date, language, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}
