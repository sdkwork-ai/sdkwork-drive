import { beforeEach, describe, expect, it } from 'vitest';
import type { DriveFile } from 'sdkwork-drive-pc-types';

import {
  clearDriveCutSelection,
  getDriveCutClipboardScopeKey,
  getDriveCutSelection,
  setDriveCutClipboardScope,
  setDriveCutSelection,
} from './driveCutClipboard';

function makeFile(id: string, ownerId: string): DriveFile {
  return {
    id,
    name: `${id}.pdf`,
    type: 'file',
    ownerId,
    updatedAt: '2026-01-01T00:00:00.000Z',
  };
}

function cut(ownerId: string) {
  setDriveCutSelection({
    section: 'my-storage',
    parentId: null,
    files: [makeFile(`node-${ownerId}`, ownerId)],
  });
}

describe('drive cut clipboard account scope', () => {
  beforeEach(() => {
    setDriveCutClipboardScope(undefined);
    clearDriveCutSelection();
    setDriveCutClipboardScope('tenant-1:user-a');
  });

  it('keeps the pending cut for the account that created it', () => {
    cut('user-a');

    expect(getDriveCutSelection()?.files).toHaveLength(1);
    expect(getDriveCutClipboardScopeKey()).toBe('tenant-1:user-a');
  });

  it('drops the pending cut when another account signs in', () => {
    cut('user-a');

    setDriveCutClipboardScope('tenant-1:user-b');

    expect(getDriveCutSelection()).toBeNull();
    expect(getDriveCutClipboardScopeKey()).toBeUndefined();
  });

  it('never returns a cut recorded under another account key', () => {
    cut('user-a');

    // Even without the scope setter running, a reader must not see the cut.
    setDriveCutClipboardScope('tenant-1:user-b');
    setDriveCutClipboardScope('tenant-1:user-a');

    expect(getDriveCutSelection()).toBeNull();
  });

  it('keeps the cut across an unchanged account key', () => {
    cut('user-a');

    setDriveCutClipboardScope('tenant-1:user-a');

    expect(getDriveCutSelection()).not.toBeNull();
  });

  it('keeps a signed-out clipboard separate from any account', () => {
    cut('user-a');

    setDriveCutClipboardScope('');

    expect(getDriveCutSelection()).toBeNull();
  });
});
