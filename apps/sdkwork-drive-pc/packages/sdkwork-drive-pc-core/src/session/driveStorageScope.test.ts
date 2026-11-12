import { describe, expect, it } from 'vitest';

import {
  driveStorageScopeKey,
  resolveDriveStorageScope,
  scopeDriveStorageKey,
} from './driveStorageScope';

describe('drive storage scope', () => {
  it('names the account by tenant and user from the session context', () => {
    const scope = resolveDriveStorageScope({
      context: { tenantId: ' tenant-001 ', userId: ' user-001 ' },
      user: { id: 'user-ignored' },
    });

    expect(scope).toEqual({ tenantId: 'tenant-001', userId: 'user-001' });
    expect(driveStorageScopeKey(scope)).toBe('tenant-001:user-001');
  });

  it('falls back to the session user id when the context omits it', () => {
    expect(
      resolveDriveStorageScope({
        context: { tenantId: 'tenant-001', userId: '' },
        user: { id: 'user-002' },
      }),
    ).toEqual({ tenantId: 'tenant-001', userId: 'user-002' });
  });

  it('reports no owner when either id is missing', () => {
    expect(resolveDriveStorageScope({})).toBeUndefined();
    expect(resolveDriveStorageScope({ context: { tenantId: 'tenant-001', userId: '' } })).toBeUndefined();
    expect(resolveDriveStorageScope({ context: { tenantId: '', userId: 'user-001' } })).toBeUndefined();
    expect(driveStorageScopeKey(undefined)).toBe('');
  });

  it('namespaces a device key by account', () => {
    expect(
      scopeDriveStorageKey('sdkwork.drive.pc.transfer.jobs.v1', {
        tenantId: 'tenant-001',
        userId: 'user-001',
      }),
    ).toBe('sdkwork.drive.pc.transfer.jobs.v1.tenant-001.user-001');
  });
});
