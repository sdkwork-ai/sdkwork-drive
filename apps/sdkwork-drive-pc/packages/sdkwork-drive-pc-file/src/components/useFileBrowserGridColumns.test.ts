import { describe, expect, it } from 'vitest';
import {
  FILE_BROWSER_GRID_ROW_HEIGHT_PX,
  resolveFileBrowserGridColumnCount,
} from './useFileBrowserGridColumns';

describe('useFileBrowserGridColumns', () => {
  it('maps viewport widths to grid column counts', () => {
    expect(resolveFileBrowserGridColumnCount(400)).toBe(2);
    expect(resolveFileBrowserGridColumnCount(700)).toBe(3);
    expect(resolveFileBrowserGridColumnCount(900)).toBe(4);
    expect(resolveFileBrowserGridColumnCount(1100)).toBe(5);
    expect(resolveFileBrowserGridColumnCount(1400)).toBe(6);
  });

  it('exposes a stable grid row height for virtualization', () => {
    expect(FILE_BROWSER_GRID_ROW_HEIGHT_PX).toBe(161);
  });
});
