/* @vitest-environment jsdom */

import { describe, expect, it } from 'vitest';
import { formatModifiedTime, modifiedTimeIso } from '../src/utils/formatModifiedTime';

/**
 * 列表里的修改时间必须被格式化：原始 ISO 串既难读又占宽度，而内容是接口原样返回的。
 */
describe('formatModifiedTime', () => {
  const now = new Date();
  const isoToday = new Date(now.getFullYear(), now.getMonth(), now.getDate(), 9, 5).toISOString();
  const isoThisYear = new Date(now.getFullYear(), 0, 15, 8, 30).toISOString();
  const isoLastYear = new Date(now.getFullYear() - 1, 5, 3, 12, 0).toISOString();

  it('shows only the time for today', () => {
    const label = formatModifiedTime(isoToday, 'en-US');

    expect(label).not.toContain(isoToday);
    expect(label).toMatch(/\d{1,2}:\d{2}/);
    // 今天的条目不该出现年份或月份。
    expect(label).not.toMatch(/[A-Za-z]{3,}/);
  });

  it('adds the month and day within the same year', () => {
    const label = formatModifiedTime(isoThisYear, 'en-US');

    expect(label).toMatch(/\d{1,2}:\d{2}/);
    expect(label).not.toContain(String(now.getFullYear()));
  });

  it('adds the year for older entries', () => {
    expect(formatModifiedTime(isoLastYear, 'en-US')).toContain(String(now.getFullYear() - 1));
  });

  it('formats according to the host language', () => {
    const english = formatModifiedTime(isoLastYear, 'en-US');
    const chinese = formatModifiedTime(isoLastYear, 'zh-CN');

    expect(english).not.toBe(chinese);
    expect(chinese).toMatch(/\d{4}/);
  });

  it('never leaks an unparseable value', () => {
    expect(formatModifiedTime(undefined, 'en-US')).toBe('—');
    expect(formatModifiedTime('', 'en-US')).toBe('—');
    expect(formatModifiedTime('not-a-date', 'en-US')).toBe('—');
  });

  it('survives an invalid language tag', () => {
    expect(formatModifiedTime(isoThisYear, 'not a language')).toMatch(/\d/);
  });

  it('exposes a machine-readable value for <time dateTime>', () => {
    expect(modifiedTimeIso(isoToday)).toBe(new Date(isoToday).toISOString());
    expect(modifiedTimeIso('nonsense')).toBeUndefined();
    expect(modifiedTimeIso(undefined)).toBeUndefined();
  });
});
