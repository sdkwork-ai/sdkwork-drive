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

});
