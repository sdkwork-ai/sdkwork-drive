import { describe, expect, it } from 'vitest';
import { computeFileBrowserVirtualWindowRange } from './useFileBrowserVirtualWindow';

describe('computeFileBrowserVirtualWindowRange', () => {
  it('returns the first page when scroll position is reset for a new section', () => {
    const range = computeFileBrowserVirtualWindowRange(12, 48, 1, 8, {
      scrollTop: 0,
      viewportHeight: 480,
    });

    expect(range.startIndex).toBe(0);
    expect(range.endIndex).toBeGreaterThan(0);
  });

  it('does not hide all rows when stale scroll position exceeds the new item count', () => {
    const staleScrollRange = computeFileBrowserVirtualWindowRange(12, 48, 1, 8, {
      scrollTop: 4800,
      viewportHeight: 480,
    });

    expect(staleScrollRange.startIndex).toBeLessThan(12);
    expect(staleScrollRange.endIndex).toBe(12);
    expect(staleScrollRange.startIndex).toBeLessThan(staleScrollRange.endIndex);
  });
});
