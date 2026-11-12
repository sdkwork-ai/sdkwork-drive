import { afterEach, describe, expect, it } from 'vitest';

import { createScopedDriveUploaderStateStore } from './driveAppSdkClient';
import type { DriveStorageScope } from '../session/driveStorageScope';
import type { SessionStorageLike } from '../session/sessionStore';

const STORAGE_KEY = 'sdkwork.drive.pc.uploader.state.v1';

class MemoryStorage implements SessionStorageLike {
  readonly items = new Map<string, string>();

  getItem(key: string): string | null {
    return this.items.get(key) ?? null;
  }

  setItem(key: string, value: string): void {
    this.items.set(key, value);
  }

  removeItem(key: string): void {
    this.items.delete(key);
  }
}

function snapshot(taskId: string, uploadSessionId: string) {
  return {
    taskId,
    uploadItemId: `item-${taskId}`,
    uploadSessionId,
    uploadedParts: [{ partNo: 1, etag: 'etag-1', offsetBytes: 0, sizeBytes: 1024 }],
    updatedAtEpochMs: 1_700_000_000_000,
  };
}

afterEach(() => {
  Reflect.deleteProperty(globalThis, 'window');
});

afterEach(() => {
  Reflect.deleteProperty(globalThis, 'window');
});

describe('scoped drive uploader state store', () => {
  it('keeps each account resume state under its own key', async () => {
    const storage = new MemoryStorage();
    let scope: DriveStorageScope | undefined = { tenantId: 'tenant-1', userId: 'user-a' };
    const store = createScopedDriveUploaderStateStore(storage, () => scope);

    await store.put(snapshot('task-1', 'session-a'));

    expect(storage.items.get(`${STORAGE_KEY}.tenant-1.user-a`)).toContain('session-a');
    await expect(store.get('task-1')).resolves.toMatchObject({ uploadSessionId: 'session-a' });

    scope = { tenantId: 'tenant-1', userId: 'user-b' };

    // The next account must not resume (or even see) the previous one's session.
    expect(storage.items.has(`${STORAGE_KEY}.tenant-1.user-b`)).toBe(false);
    await expect(store.get('task-1')).resolves.toBeUndefined();
  });

  it('drops the pre-scope global record instead of handing it to an account', async () => {
    const storage = new MemoryStorage();
    storage.setItem(STORAGE_KEY, JSON.stringify({ 'task-legacy': snapshot('task-legacy', 'legacy') }));
    const store = createScopedDriveUploaderStateStore(storage, () => ({
      tenantId: 'tenant-1',
      userId: 'user-b',
    }));

    await expect(store.get('task-legacy')).resolves.toBeUndefined();
    expect(storage.items.has(STORAGE_KEY)).toBe(false);
  });

  it('stays in memory while no account owns the state', async () => {
    const storage = new MemoryStorage();
    const store = createScopedDriveUploaderStateStore(storage, () => undefined);

    await store.put(snapshot('task-1', 'session-unscoped'));

    expect([...storage.items.keys()]).toEqual([]);
    await expect(store.get('task-1')).resolves.toMatchObject({
      uploadSessionId: 'session-unscoped',
    });
  });

  it('forgets in-memory sessions when the account changes', async () => {
    const storage = new MemoryStorage();
    let scope: DriveStorageScope | undefined;
    const store = createScopedDriveUploaderStateStore(storage, () => scope);

    await store.put(snapshot('task-1', 'session-unscoped'));
    await expect(store.get('task-1')).resolves.toBeDefined();

    scope = { tenantId: 'tenant-1', userId: 'user-a' };

    await expect(store.get('task-1')).resolves.toBeUndefined();
  });

  it('keeps working without any browser storage', async () => {
    let scope: DriveStorageScope | undefined = { tenantId: 'tenant-1', userId: 'user-a' };
    const store = createScopedDriveUploaderStateStore(undefined, () => scope);

    await store.put(snapshot('task-1', 'session-a'));
    await expect(store.get('task-1')).resolves.toMatchObject({ uploadSessionId: 'session-a' });

    await store.clear('task-1');
    await expect(store.get('task-1')).resolves.toBeUndefined();
  });
});
