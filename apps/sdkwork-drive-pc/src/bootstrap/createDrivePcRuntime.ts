import {
  createDriveAppSdkClient,
  createDriveSessionTokenManager,
  createDriveFileService,
  createHostAdapter,
  createOsSecureSessionStorage,
  createRuntimeConfig,
  createSessionStore,
  DEFAULT_SESSION_STORAGE_KEY,
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
  const sessionStorage = await resolveSessionStorage(config.auth.tokenStorage);
  const session = createSessionStore(sessionStorage);
  const tokenManager = createDriveSessionTokenManager(session);
  const appSdkClient = createDriveAppSdkClient({
    config,
    tokenManager,
    uploaderStateStorage: sessionStorage,
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
  if (tokenStorage === 'browser-local' || tokenStorage === 'browser-session') {
    migrateLegacyBrowserSession();
    return window.localStorage;
  }
  if (tokenStorage === 'os-secure-storage') {
    return createOsSecureSessionStorage();
  }
  return undefined;
}

function migrateLegacyBrowserSession(): void {
  // Storage may be partially unavailable (sandboxed webviews, test harnesses):
  // guard each handle independently instead of assuming the full pair exists.
  const sessionStorage = window.sessionStorage as SessionStorageLike | undefined;
  const localStorage = window.localStorage as SessionStorageLike | undefined;
  if (!sessionStorage && !localStorage) {
    return;
  }
  const legacySession = sessionStorage?.getItem(DEFAULT_SESSION_STORAGE_KEY) ?? null;
  if (legacySession && localStorage && !localStorage.getItem(DEFAULT_SESSION_STORAGE_KEY)) {
    localStorage.setItem(DEFAULT_SESSION_STORAGE_KEY, legacySession);
  }
  if (legacySession && sessionStorage) {
    sessionStorage.removeItem(DEFAULT_SESSION_STORAGE_KEY);
  }
}
