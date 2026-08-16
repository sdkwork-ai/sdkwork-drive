import { describe, expect, it } from 'vitest';
import type { SessionSnapshot } from 'sdkwork-drive-pc-core';
import {
  canAccessAdminSection,
  canAccessAnyDriveAdminSurface,
  canAccessDriveAdminStorage,
  canAccessDriveAuditAdmin,
  canAccessDriveQuotaAdmin,
  canAccessDriveStorageAdmin,
  DRIVE_AUDIT_ADMIN_PERMISSION,
  DRIVE_QUOTA_ADMIN_PERMISSION,
  DRIVE_STORAGE_ADMIN_PERMISSION,
  resolveDriveAdminSectionAccess,
} from './adminAccess';

const baseSession: SessionSnapshot = {
  authToken: 'auth',
  accessToken: 'access',
  context: {
    tenantId: 'tenant-001',
    userId: 'user-001',
    actorId: 'user-001',
    actorKind: 'user',
  },
};

describe('drive admin capability scopes', () => {
  it('allows umbrella drive.storage.admin for all admin sections', () => {
    const session = {
      ...baseSession,
      context: {
        ...baseSession.context!,
        permissionScope: [DRIVE_STORAGE_ADMIN_PERMISSION],
      },
    };
    const access = resolveDriveAdminSectionAccess(session);
    expect(access.storageProviders).toBe(true);
    expect(access.audit).toBe(true);
    expect(access.quotas).toBe(true);
  });

  it('allows audit-only scope for audit but not quota or storage', () => {
    const session = {
      ...baseSession,
      context: {
        ...baseSession.context!,
        permissionScope: [DRIVE_AUDIT_ADMIN_PERMISSION],
      },
    };
    expect(canAccessDriveAuditAdmin(session)).toBe(true);
    expect(canAccessDriveQuotaAdmin(session)).toBe(false);
    expect(canAccessDriveStorageAdmin(session)).toBe(false);
    expect(canAccessAdminSection(session, 'audit')).toBe(true);
    expect(canAccessAdminSection(session, 'quotas')).toBe(false);
    expect(canAccessAnyDriveAdminSurface(session)).toBe(true);
  });

  it('allows quota-only scope for quotas but not audit', () => {
    const session = {
      ...baseSession,
      context: {
        ...baseSession.context!,
        permissionScope: [DRIVE_QUOTA_ADMIN_PERMISSION],
      },
    };
    expect(canAccessDriveQuotaAdmin(session)).toBe(true);
    expect(canAccessDriveAuditAdmin(session)).toBe(false);
  });

  it('denies regular drive file permissions', () => {
    const session = {
      ...baseSession,
      context: {
        ...baseSession.context!,
        permissionScope: ['drive.nodes.read'],
      },
    };
    expect(canAccessAnyDriveAdminSurface(session)).toBe(false);
  });
});

describe('canAccessDriveAdminStorage', () => {
  it('matches storage gate for legacy alias', () => {
    const session = {
      ...baseSession,
      context: {
        ...baseSession.context!,
        permissionScope: [DRIVE_STORAGE_ADMIN_PERMISSION],
      },
    };
    expect(canAccessDriveAdminStorage(session)).toBe(canAccessDriveStorageAdmin(session));
  });
});
