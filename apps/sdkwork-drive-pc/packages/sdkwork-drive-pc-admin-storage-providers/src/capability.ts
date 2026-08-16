import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';

/**
 * Capability key for Drive storage provider administration.
 */
export const STORAGE_PROVIDER_ADMIN_CAPABILITY = 'drive.storage.provider.admin';

/**
 * Bind the capability to the Drive admin storage SDK client.
 */
export function resolveStorageProviderAdminCapability(
  sdk: DriveAdminStorageSdkClient,
): { capability: string; sdk: DriveAdminStorageSdkClient } {
  return { capability: STORAGE_PROVIDER_ADMIN_CAPABILITY, sdk };
}
