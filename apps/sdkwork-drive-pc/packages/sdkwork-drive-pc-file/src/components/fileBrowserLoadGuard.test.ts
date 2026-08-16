import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { createLatestRequestGuard } from './fileBrowserLoadGuard';

describe('FileBrowser latest request guard', () => {
  it('allows only the most recent Drive backend request to commit UI state', () => {
    const guard = createLatestRequestGuard();

    const firstSequence = guard.begin();
    const secondSequence = guard.begin();

    expect(guard.isCurrent(firstSequence)).toBe(false);
    expect(guard.isCurrent(secondSequence)).toBe(true);
  });

  it('rejects stale Drive view refreshes after the user navigates elsewhere', () => {
    const guard = createLatestRequestGuard();

    guard.setCurrentScope('my-storage/root');
    const rootSequence = guard.begin('my-storage/root');
    guard.setCurrentScope('my-storage/folder-a');

    expect(guard.isCurrentScope('my-storage/root')).toBe(false);
    expect(guard.isCurrent(rootSequence, 'my-storage/root')).toBe(false);
  });

  it('keeps local load sequence terminology separate from SDKWork requestId fields', () => {
    const source = readFileSync(new URL('./fileBrowserLoadGuard.ts', import.meta.url), 'utf8');

    expect(source).not.toContain('requestId');
    expect(source).not.toContain('currentRequestId');
  });
});
