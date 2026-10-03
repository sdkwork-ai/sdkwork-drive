/* @vitest-environment jsdom */

import { describe, expect, it } from 'vitest';
import { formatDriveDate, formatDriveDateTime } from '../src/utils/formatDriveTimestamp';

const ISO = '2023-05-01T08:30:00.000Z';

describe('formatDriveDate', () => {
  it('formats the same instant in the language it is given', () => {
    const zh = formatDriveDate(ISO, 'zh-CN');
    const en = formatDriveDate(ISO, 'en-US');

    expect(zh).not.toBe(en);
    expect(en).toContain('2023');
    expect(en).toMatch(/May/);
  });

  it('never leaks a raw value or an empty cell', () => {
    // 缺失和不可解析都变成破折号：台账上宁可写"没有"，也不能写原始串。
    expect(formatDriveDate(undefined, 'zh-CN')).toBe('—');
    expect(formatDriveDate('', 'zh-CN')).toBe('—');
    expect(formatDriveDate('not-a-date', 'zh-CN')).toBe('—');
  });

  it('falls back to the runtime locale when the language tag is unusable', () => {
    // 格式化失败不能让时间消失：退回运行环境默认区域，断言只要求"有个日期"。
    expect(formatDriveDate(ISO, 'not a language')).toMatch(/\d/);
  });
});

describe('formatDriveDateTime', () => {
  it('carries the time of day as well as the date', () => {
    expect(formatDriveDateTime(ISO, 'en-US')).toMatch(/\d{1,2}:\d{2}/);
    expect(formatDriveDateTime(ISO, 'en-US')).not.toBe(formatDriveDate(ISO, 'en-US'));
  });

  it('uses the same empty answer as the date formatter', () => {
    expect(formatDriveDateTime(undefined, 'en-US')).toBe('—');
    expect(formatDriveDateTime('nonsense', 'en-US')).toBe('—');
  });
});
