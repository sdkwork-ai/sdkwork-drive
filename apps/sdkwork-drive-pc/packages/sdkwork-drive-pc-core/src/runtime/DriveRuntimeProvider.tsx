import React, { createContext, useContext } from 'react';
import type { DriveRuntimeConfig } from '../config/runtimeConfig';
import type { HostAdapter } from '../host/hostAdapter';
import type { DriveAppSdkClient } from '../sdk/driveAppSdkClient';
import type { DriveFileService } from '../services/driveFileService';
import type { SessionStore } from '../session/sessionStore';

export interface DriveCoreRuntime {
  config: DriveRuntimeConfig;
  auth?: {
    iamRuntime: unknown;
  };
  sdk: {
    app: DriveAppSdkClient;
  };
  session: SessionStore;
  host: HostAdapter;
  services: {
    fileService: DriveFileService;
  };
}

const DriveRuntimeContext = createContext<DriveCoreRuntime | null>(null);

export function DriveRuntimeProvider<T extends DriveCoreRuntime>({
  runtime,
  children,
}: {
  runtime: T;
  children: React.ReactNode;
}) {
  return (
    <DriveRuntimeContext.Provider value={runtime}>{children}</DriveRuntimeContext.Provider>
  );
}

export function useDriveRuntime(): DriveCoreRuntime {
  const runtime = useContext(DriveRuntimeContext);
  if (!runtime) {
    throw new Error('DriveRuntimeProvider is missing.');
  }
  return runtime;
}
