import {
  createDriveAppSdkClient,
  createDriveSessionTokenManager,
  createDriveFileService,
  createHostAdapter,
  createOsSecureSessionStorage,
  createRuntimeConfig,
  createSessionStore,
  DEFAULT_SESSION_STORAGE_KEY,
  resolveDriveStorageScope,
  type SessionStorageLike,
} from 'sdkwork-drive-pc-core';
import {
  createDriveAdminStorageSdkClient,
  createDriveBackendSdkClient,
  type DriveRuntime,
} from 'sdkwork-drive-pc-admin-core';
import { createDriveIamRuntime } from './driveIamRuntime';
import { primePcReactRuntimeSessionCache } from './sdkworkCorePcReactShim';

export async function createDrivePcRuntime(): Promise<DriveRuntime> {
  const config = createRuntimeConfig(import.meta.env);
  const sessionPersistence = await resolveSessionStorage(config.auth.tokenStorage);
  const session = createSessionStore(sessionPersistence);
  const tokenManager = createDriveSessionTokenManager(session);
  const appSdkClient = createDriveAppSdkClient({
    config,
    tokenManager,
    uploaderStateStorage: sessionPersistence,
    // Uploader resume state names server-side upload sessions, so it is scoped
    // to the signed-in account and stays in memory while no account is known.
    resolveUploaderStorageScope: () => resolveDriveStorageScope(session.getSnapshot()),
  });
  const adminStorageSdkClient = createDriveAdminStorageSdkClient({
    config,
    tokenManager,
  });
  const backendSdkClient = createDriveBackendSdkClient({
    config,
    tokenManager,
  });
  const iamRuntime = createDriveIamRuntime({
    config,
    sdkClients: [appSdkClient],
    session,
    tokenManager,
  });
  adminStorageSdkClient.setTokenManager(tokenManager);
  backendSdkClient.setTokenManager(tokenManager);

  const host = createHostAdapter();
  primePcReactRuntimeSessionCache(session.getSnapshot());
  session.subscribe((snapshot) => {
    primePcReactRuntimeSessionCache(snapshot);
  });

  return {
    config,
    auth: {
      iamRuntime,
    },
    sdk: {
      app: appSdkClient,
    },
    admin: {
      adminStorage: adminStorageSdkClient,
      backend: backendSdkClient,
    },
    session,
    host,
    services: {
      fileService: createDriveFileService({
        appSdkClient,
        getSession: session.getSnapshot,
        hostAdapter: host,
      }),
    },
  };
}

async function resolveSessionStorage(
  tokenStorage: DriveRuntime['config']['auth']['tokenStorage'],
): Promise<SessionStorageLike | undefined> {
  if (typeof window === 'undefined') {
    return undefined;
  }
  if (tokenStorage === 'browser-local') {
    migrateLegacyBrowserSession('local');
    return window.localStorage;
  }
  if (tokenStorage === 'browser-session') {
    // Session-scoped tokens must not survive the browser session: a durable copy
    // would keep the previous account's credentials on a shared device.
    migrateLegacyBrowserSession('session');
    return window.sessionStorage;
  }
  if (tokenStorage === 'os-secure-storage') {
    return createOsSecureSessionStorage();
  }
  return undefined;
}

/**
 * Move a session that was persisted by the other browser-storage mode.
 *
 * Legacy builds always used localStorage, so a browser-session runtime adopts
 * the durable copy once and deletes it; a browser-local runtime adopts a
 * leftover sessionStorage copy. Storage may be partially unavailable (sandboxed
 * webviews, test harnesses): guard each handle independently.
 */
function migrateLegacyBrowserSession(target: 'local' | 'session'): void {
  const sessionStore = window.sessionStorage as SessionStorageLike | undefined;
  const localStore = window.localStorage as SessionStorageLike | undefined;
  const source = target === 'local' ? sessionStore : localStore;
  const destination = target === 'local' ? localStore : sessionStore;
  if (!source || !destination) {
    return;
  }
  try {
    const legacySession = source.getItem(DEFAULT_SESSION_STORAGE_KEY);
    if (!legacySession) {
      return;
    }
    if (!destination.getItem(DEFAULT_SESSION_STORAGE_KEY)) {
      destination.setItem(DEFAULT_SESSION_STORAGE_KEY, legacySession);
    }
    source.removeItem(DEFAULT_SESSION_STORAGE_KEY);
  } catch {
    // Best-effort migration; a failed move only costs one re-login.
  }
}
