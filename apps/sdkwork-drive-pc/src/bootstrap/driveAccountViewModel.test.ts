import { describe, expect, it } from 'vitest';
import {
  createSessionStore,
} from 'sdkwork-drive-pc-core';
import {
  createDriveAccountViewModel,
  signOutDriveAccount,
} from './driveAccountViewModel';

describe('drive account view model', () => {
  it('derives menu-safe account details from the IAM session snapshot', () => {
    const account = createDriveAccountViewModel({
      sessionId: 'session-001',
      user: {
        id: 'user-001',
        displayName: 'Ada Lovelace',
        email: 'ada@sdkwork.local',
        avatarUrl: 'https://example.test/avatar.png',
      },
      context: {
        tenantId: 'tenant-001',
        userId: 'user-001',
        organizationId: 'org-001',
        appId: 'sdkwork-drive-pc',
        environment: 'dev',
        deploymentMode: 'saas',
        authLevel: 'password',
      },
    });

    expect(account).toEqual({
      id: 'user-001',
      displayName: 'Ada Lovelace',
      email: 'ada@sdkwork.local',
      avatarUrl: 'https://example.test/avatar.png',
      initials: 'AL',
      tenantId: 'tenant-001',
      organizationId: 'org-001',
      sessionId: 'session-001',
      environmentLabel: 'dev / saas',
      authLevel: 'password',
    });
  });

  it('formats storage labels only from the real Drive quota summary', () => {
    const account = createDriveAccountViewModel(
      {
        user: {
          id: 'user-001',
          displayName: 'Ada Lovelace',
        },
        context: {
          tenantId: 'tenant-001',
          userId: 'user-001',
        },
      },
      {
        tenantId: 'tenant-001',
        usedBytes: 4_294_967_296,
        objectCount: 12,
      },
    );

    expect(account.storageUsedLabel).toBe('4.0 GB');
    expect(account.storageTotalLabel).toBeUndefined();
    expect(account.storageUsagePercent).toBeUndefined();
    expect(account.storageObjectCount).toBe(12);
    expect(account.planLabel).toBeUndefined();
  });

  it('clears the runtime IAM session when signing out from the profile menu', async () => {
    const session = createSessionStore();
    session.setSession({
      authToken: 'auth-token',
      accessToken: 'access-token',
      context: {
        tenantId: 'tenant-001',
        userId: 'user-001',
      },
    });

    await signOutDriveAccount(session);

    expect(session.getSnapshot()).toEqual({});
  });

  it('clears every account-scoped storage key and keeps device preferences', async () => {
    const originalWindow = globalThis.window;
    const backing = new Map<string, string>([
      ['sdkwork-drive-pc-session', '{"accessToken":"token"}'],
      ['sdkwork.drive.pc.transfer.jobs.v1', '[]'],
      ['sdkwork.drive.pc.transfer.jobs.v1.tenant-001.user-a', '[]'],
      ['sdkwork.drive.pc.uploader.state.v1.tenant-001.user-a', '{}'],
      ['sdkwork.drive.pc.preferences.v1', '{"compactMode":true}'],
      ['sdkwork-ui-theme', 'dark'],
    ]);
    const storage = {
      get length(): number {
        return backing.size;
      },
      key: (index: number): string | null => [...backing.keys()][index] ?? null,
      getItem: (key: string): string | null => backing.get(key) ?? null,
      setItem: (key: string, value: string): void => {
        backing.set(key, value);
      },
      removeItem: (key: string): void => {
        backing.delete(key);
      },
    };
    Object.defineProperty(globalThis, 'window', {
      configurable: true,
      value: { localStorage: storage, sessionStorage: storage },
    });

    try {
      await signOutDriveAccount(createSessionStore());

      expect([...backing.keys()].filter((key) => key.startsWith('sdkwork.drive.pc.'))).toEqual([
        'sdkwork.drive.pc.preferences.v1',
      ]);
      expect(backing.has('sdkwork-drive-pc-session')).toBe(false);
      expect(backing.get('sdkwork-ui-theme')).toBe('dark');
      expect(backing.get('sdkwork.drive.pc.preferences.v1')).toContain('compactMode');
    } finally {
      Object.defineProperty(globalThis, 'window', {
        configurable: true,
        value: originalWindow,
      });
    }
  });
});
