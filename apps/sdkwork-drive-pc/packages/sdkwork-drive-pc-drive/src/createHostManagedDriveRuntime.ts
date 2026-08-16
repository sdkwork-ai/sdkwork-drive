import {
  createDriveAppSdkClient,
  createDriveFileService,
  createDriveSessionTokenManager,
  createHostAdapter,
  createRuntimeConfig,
  createSessionStore,
  type DriveCoreRuntime,
} from 'sdkwork-drive-pc-core';

import { getConfiguredDriveAppSdkClient } from './sdkPorts';
import { syncHostSessionIntoDriveStore } from './sessionBridge';

export function createHostManagedDriveRuntime(): DriveCoreRuntime {
  const config = createRuntimeConfig({
    ...import.meta.env,
    VITE_DRIVE_PC_TOKEN_STORAGE: 'memory',
    VITE_DRIVE_PC_TOKEN_MANAGER_MODE: 'appbase-global',
  });
  const session = createSessionStore();
  const tokenManager = createDriveSessionTokenManager(session);
  const hostClient = getConfiguredDriveAppSdkClient();
  const appSdkClient = createDriveAppSdkClient({
    config,
    sdkClient: hostClient as never,
    tokenManager,
  });
  const host = createHostAdapter();

  syncHostSessionIntoDriveStore(session);

  return {
    config,
    sdk: {
      app: appSdkClient,
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
