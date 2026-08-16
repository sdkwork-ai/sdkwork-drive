import { describe, expect, it } from 'vitest';
import { formatDriveBytes } from '../src/utils/formatDriveBytes';

describe('formatDriveBytes', () => {
  it('returns placeholders for missing values', () => {
    expect(formatDriveBytes(undefined)).toBe('--');
    expect(formatDriveBytes(null)).toBe('--');
  });

  it('formats zero and large byte counts through sdkwork utils', () => {
    expect(formatDriveBytes(0)).toBe('0 B');
    expect(formatDriveBytes(4_294_967_296)).toBe('4.0 GB');
  });
});
