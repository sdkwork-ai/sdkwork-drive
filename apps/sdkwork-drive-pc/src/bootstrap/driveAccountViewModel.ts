import { formatDriveBytes } from 'sdkwork-drive-pc-commons';
import {
  clearOsSecureSessionStorage,
  type DriveStorageSummary,
  type SdkworkAuthRuntimeConfig,
  type SessionSnapshot,
  type SessionStore,
} from 'sdkwork-drive-pc-core';

export interface DriveAccountViewModel {
  id: string;
  displayName: string;
  email?: string;
  avatarUrl?: string;
  initials: string;
  tenantId?: string;
  organizationId?: string;
  sessionId?: string;
  environmentLabel: string;
  authLevel: string;
  planLabel?: string;
  storageUsedLabel?: string;
  storageTotalLabel?: string;
  storageUsagePercent?: number;
  storageObjectCount?: number;
}

export function createDriveAccountViewModel(
  session: SessionSnapshot,
  storageSummary?: DriveStorageSummary,
): DriveAccountViewModel {
  const userId = session.user?.id ?? session.context?.userId ?? 'drive-user';
  const displayName = session.user?.displayName?.trim() || 'SDKWork Drive User';
  const environment = session.context?.environment?.trim();
  const deploymentMode = session.context?.deploymentMode?.trim();
  const storageUsedLabel = storageSummary ? formatDriveBytes(storageSummary.usedBytes) : undefined;
  const storageTotalLabel = storageSummary?.totalBytes
    ? formatDriveBytes(storageSummary.totalBytes)
    : undefined;

  const account: DriveAccountViewModel = {
    id: userId,
    displayName,
    email: session.user?.email,
    avatarUrl: session.user?.avatarUrl,
    initials: buildInitials(displayName, userId),
    tenantId: session.context?.tenantId,
    organizationId: session.context?.organizationId,
    sessionId: session.sessionId ?? session.context?.sessionId,
    environmentLabel:
      environment && deploymentMode
        ? `${environment} / ${deploymentMode}`
        : environment || deploymentMode || 'standard',
    authLevel: session.context?.authLevel ?? 'standard',
  };

  if (storageUsedLabel) {
    account.storageUsedLabel = storageUsedLabel;
  }
  if (storageTotalLabel) {
    account.storageTotalLabel = storageTotalLabel;
  }
  if (storageSummary?.usagePercent !== undefined) {
    account.storageUsagePercent = storageSummary.usagePercent;
  }
  if (storageSummary?.objectCount !== undefined) {
    account.storageObjectCount = storageSummary.objectCount;
  }

  return account;
}

export async function signOutDriveAccount(
  session: SessionStore,
  options?: {
    tokenStorage?: SdkworkAuthRuntimeConfig['tokenStorage'];
  },
): Promise<void> {
  session.clearSession();
  await clearPersistedDriveRuntimeState(options?.tokenStorage);
}

async function clearPersistedDriveRuntimeState(
  tokenStorage?: SdkworkAuthRuntimeConfig['tokenStorage'],
): Promise<void> {
  if (typeof window === 'undefined') {
    return;
  }
  if (tokenStorage === 'os-secure-storage') {
    await clearOsSecureSessionStorage();
  }
  const storages: Array<Storage | undefined> = [
    window.localStorage,
    window.sessionStorage,
  ];
  for (const storage of storages) {
    if (!storage) {
      continue;
    }
    clearDriveStorageKeys(storage);
  }
}

/** Device preferences that hold no account data and survive sign-out. */
const PRESERVED_STORAGE_KEYS = new Set<string>([
  'sdkwork.drive.pc.preferences.v1',
]);

/** Keys written before account scoping existed; removed even if enumeration fails. */
const LEGACY_DRIVE_STORAGE_KEYS = [
  'sdkwork-drive-pc-session',
  'sdkwork.drive.pc.transfer.jobs.v1',
  'sdkwork.drive.pc.uploader.state.v1',
];

/** Every account-scoped key this app writes is namespaced below this prefix. */
const ACCOUNT_SCOPED_STORAGE_KEY_PREFIX = 'sdkwork.drive.pc.';

function isAccountScopedStorageKey(key: string): boolean {
  if (PRESERVED_STORAGE_KEYS.has(key)) {
    return false;
  }
  return key === 'sdkwork-drive-pc-session' || key.startsWith(ACCOUNT_SCOPED_STORAGE_KEY_PREFIX);
}

/**
 * Remove every account-scoped key, including the `...<tenant>.<user>` variants.
 *
 * A sign-out has to be exhaustive: the scoped transfer/uploader keys name files,
 * space ids and server-side upload sessions, and a fixed key list silently misses
 * every key a later build adds. Device preferences are kept.
 */
function clearDriveStorageKeys(storage: Storage): void {
  const keys = new Set<string>(LEGACY_DRIVE_STORAGE_KEYS);
  try {
    for (let index = 0; index < storage.length; index += 1) {
      const key = storage.key(index);
      if (key && isAccountScopedStorageKey(key)) {
        keys.add(key);
      }
    }
  } catch {
    // Fall back to the known keys when the storage cannot be enumerated.
  }
  for (const key of keys) {
    if (!isAccountScopedStorageKey(key)) {
      continue;
    }
    try {
      storage.removeItem(key);
    } catch {
      // Ignore storage cleanup errors to keep logout resilient.
    }
  }
}

function buildInitials(displayName: string, fallbackId: string): string {
  const normalized = displayName.trim();
  if (normalized) {
    const asciiWords = normalized.match(/[A-Za-z0-9]+/g);
    if (asciiWords?.length) {
      return asciiWords
        .slice(0, 2)
        .map((word) => word.charAt(0).toUpperCase())
        .join('');
    }
    return Array.from(normalized).slice(0, 2).join('');
  }

  return fallbackId.charAt(0).toUpperCase() || 'S';
}
