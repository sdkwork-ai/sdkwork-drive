import type { SessionSnapshot } from './sessionStore';

/**
 * Account identity that owns one piece of device-persisted Drive state.
 *
 * A browser profile or Electron user directory is shared by every account that
 * signs in on that device, so any key holding user data or server-side ids must
 * name both ids.
 */
export interface DriveStorageScope {
  tenantId: string;
  userId: string;
}

/**
 * Resolve the account that owns device-persisted Drive state.
 *
 * Transfer lists, uploader sessions and account-scoped caches carry file names,
 * space ids, node ids and local paths, so a shared device must not hand them to
 * the next signed-in account. A snapshot without both ids names no owner:
 * callers keep such state in memory instead of writing a key any account reads.
 */
export function resolveDriveStorageScope(session: SessionSnapshot): DriveStorageScope | undefined {
  const tenantId = session.context?.tenantId?.trim() ?? '';
  // An empty context id is as unusable as a missing one, so fall back to the user record.
  const userId = session.context?.userId?.trim()
    || session.user?.id?.trim()
    || '';
  if (!tenantId || !userId) {
    return undefined;
  }
  return { tenantId, userId };
}

/** Stable `tenant:user` fragment that identifies one account, or '' when unowned. */
export function driveStorageScopeKey(scope: DriveStorageScope | undefined): string {
  return scope ? `${scope.tenantId}:${scope.userId}` : '';
}

/** Namespace a device storage key by account, e.g. `base` -> `base.tenant.user`. */
export function scopeDriveStorageKey(baseKey: string, scope: DriveStorageScope): string {
  return `${baseKey}.${scope.tenantId}.${scope.userId}`;
}
