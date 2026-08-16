import type { SessionSnapshot } from 'sdkwork-drive-pc-core';

/** Legacy umbrella scope for backend and admin-storage APIs. */
export const DRIVE_STORAGE_ADMIN_PERMISSION = 'drive.storage.admin';
export const DRIVE_BACKEND_ADMIN_WILDCARD = 'drive.*';

/** Granular backend admin scopes enforced per route operation. */
export const DRIVE_AUDIT_ADMIN_PERMISSION = 'drive.audit.admin';
export const DRIVE_MAINTENANCE_ADMIN_PERMISSION = 'drive.maintenance.admin';
export const DRIVE_QUOTA_ADMIN_PERMISSION = 'drive.quota.admin';
export const DRIVE_LABELS_ADMIN_PERMISSION = 'drive.labels.admin';
export const DRIVE_SPACES_ADMIN_PERMISSION = 'drive.spaces.admin';
export const DRIVE_DOWNLOAD_PACKAGES_ADMIN_PERMISSION = 'drive.download_packages.admin';

export interface DriveAdminSectionAccess {
  storageProviders: boolean;
  storageBindings: boolean;
  storageKinds: boolean;
  storageBuckets: boolean;
  audit: boolean;
  maintenance: boolean;
  quotas: boolean;
  labels: boolean;
  spaces: boolean;
  downloadPackages: boolean;
}

function hasDriveAdminPrivilege(scopes: string[]): boolean {
  return scopes.includes(DRIVE_STORAGE_ADMIN_PERMISSION) || scopes.includes(DRIVE_BACKEND_ADMIN_WILDCARD);
}

export function hasDriveAdminCapability(session: SessionSnapshot, capabilityPermission: string): boolean {
  const scopes = session.context?.permissionScope ?? [];
  if (hasDriveAdminPrivilege(scopes)) {
    return true;
  }
  return scopes.includes(capabilityPermission);
}

export function canAccessDriveStorageAdmin(session: SessionSnapshot): boolean {
  return hasDriveAdminPrivilege(session.context?.permissionScope ?? []);
}

export function canAccessDriveAuditAdmin(session: SessionSnapshot): boolean {
  return hasDriveAdminCapability(session, DRIVE_AUDIT_ADMIN_PERMISSION);
}

export function canAccessDriveMaintenanceAdmin(session: SessionSnapshot): boolean {
  return hasDriveAdminCapability(session, DRIVE_MAINTENANCE_ADMIN_PERMISSION);
}

export function canAccessDriveQuotaAdmin(session: SessionSnapshot): boolean {
  return hasDriveAdminCapability(session, DRIVE_QUOTA_ADMIN_PERMISSION);
}

export function canAccessDriveLabelsAdmin(session: SessionSnapshot): boolean {
  return hasDriveAdminCapability(session, DRIVE_LABELS_ADMIN_PERMISSION);
}

export function canAccessDriveSpacesAdmin(session: SessionSnapshot): boolean {
  return hasDriveAdminCapability(session, DRIVE_SPACES_ADMIN_PERMISSION);
}

export function canAccessDriveDownloadPackagesAdmin(session: SessionSnapshot): boolean {
  return hasDriveAdminCapability(session, DRIVE_DOWNLOAD_PACKAGES_ADMIN_PERMISSION);
}

export function canAccessAnyDriveAdminSurface(session: SessionSnapshot): boolean {
  return (
    canAccessDriveStorageAdmin(session)
    || canAccessDriveAuditAdmin(session)
    || canAccessDriveMaintenanceAdmin(session)
    || canAccessDriveQuotaAdmin(session)
    || canAccessDriveLabelsAdmin(session)
    || canAccessDriveSpacesAdmin(session)
    || canAccessDriveDownloadPackagesAdmin(session)
  );
}

export function resolveDriveAdminSectionAccess(session: SessionSnapshot): DriveAdminSectionAccess {
  return {
    storageProviders: canAccessDriveStorageAdmin(session),
    storageBindings: canAccessDriveStorageAdmin(session),
    storageKinds: canAccessDriveStorageAdmin(session),
    storageBuckets: canAccessDriveStorageAdmin(session),
    audit: canAccessDriveAuditAdmin(session),
    maintenance: canAccessDriveMaintenanceAdmin(session),
    quotas: canAccessDriveQuotaAdmin(session),
    labels: canAccessDriveLabelsAdmin(session),
    spaces: canAccessDriveSpacesAdmin(session),
    downloadPackages: canAccessDriveDownloadPackagesAdmin(session),
  };
}

export function canAccessDriveBackendAdmin(session: SessionSnapshot): boolean {
  return canAccessAnyDriveAdminSurface(session);
}

export function canAccessDriveAdminStorage(session: SessionSnapshot): boolean {
  return canAccessDriveStorageAdmin(session);
}

export function canAccessAdminSection(
  session: SessionSnapshot,
  section: keyof DriveAdminSectionAccess,
): boolean {
  return resolveDriveAdminSectionAccess(session)[section];
}
