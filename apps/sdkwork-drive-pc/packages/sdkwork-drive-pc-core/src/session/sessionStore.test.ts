import { describe, expect, it } from 'vitest';
import { createSessionStore, type SessionStorageLike } from './sessionStore';

class MemoryStorage implements SessionStorageLike {
  private readonly items = new Map<string, string>();

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

describe('session store', () => {
  it('normalizes and persists the standard SDKWork IAM session shape', () => {
    const storage = new MemoryStorage();
    const store = createSessionStore(storage, 'drive-session');

    store.setSession({
      authToken: 'auth-token',
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
      sessionId: 'session-001',
      user: {
        id: 'user-001',
        displayName: 'Ada',
      },
      context: {
        tenantId: 'tenant-001',
        userId: 'user-001',
        actorId: 'user-001',
        actorKind: 'user',
        permissionScope: ['drive.nodes.read'],
      },
    });

    const persisted = JSON.parse(storage.getItem('drive-session') ?? '{}');
    expect(persisted).toMatchObject({
      authToken: 'auth-token',
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
      sessionId: 'session-001',
      user: {
        id: 'user-001',
      },
      context: {
        tenantId: 'tenant-001',
        userId: 'user-001',
        actorId: 'user-001',
        actorKind: 'user',
      },
    });
    expect(persisted.updatedAt).toEqual(expect.any(String));

    const restored = createSessionStore(storage, 'drive-session');
    expect(restored.getSnapshot().authToken).toBe('auth-token');
    expect(restored.getSnapshot().context?.tenantId).toBe('tenant-001');

    restored.clearSession();
    expect(storage.getItem('drive-session')).toBeNull();
  });

  it('refreshes the snapshot from persisted appbase session changes', () => {
    const storage = new MemoryStorage();
    const store = createSessionStore(storage, 'drive-session');

    storage.setItem(
      'drive-session',
      JSON.stringify({
        authToken: 'auth-token',
        accessToken: 'access-token',
        context: {
          tenantId: 'tenant-001',
          userId: 'user-001',
        },
      }),
    );

    const refreshed = store.refreshSession();

    expect(refreshed.authToken).toBe('auth-token');
    expect(refreshed.accessToken).toBe('access-token');
    expect(refreshed.context?.tenantId).toBe('tenant-001');
    expect(store.getSnapshot().context?.userId).toBe('user-001');
  });
});
