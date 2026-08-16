import { describe, expect, it } from 'vitest';
import { supportsServerSideFileBrowserSort } from './fileBrowserSortSupport';

describe('supportsServerSideFileBrowserSort', () => {
  it('uses server sort for folder browsing, recent/trash, and starred/shared library roots', () => {
    expect(supportsServerSideFileBrowserSort('my-storage', '', 'folder-1')).toBe(true);
    expect(supportsServerSideFileBrowserSort('recent', '', null)).toBe(true);
    expect(supportsServerSideFileBrowserSort('trash', '', null)).toBe(true);
    expect(supportsServerSideFileBrowserSort('starred', '', null)).toBe(true);
    expect(supportsServerSideFileBrowserSort('shared', '', null)).toBe(true);
  });

  it('falls back to client sort for search and computers views', () => {
    expect(supportsServerSideFileBrowserSort('my-storage', 'report', null)).toBe(false);
    expect(supportsServerSideFileBrowserSort('computers', '', null)).toBe(false);
  });
});
