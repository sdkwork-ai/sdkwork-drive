import type { DriveCoreRuntime } from 'sdkwork-drive-pc-core';
import type { DriveAdminStorageSdkClient } from '../sdk/driveAdminStorageSdkClient';
import type { DriveBackendSdkClient } from '../sdk/driveBackendSdkClient';

export interface DriveRuntime extends DriveCoreRuntime {
  admin: {
    adminStorage: DriveAdminStorageSdkClient;
    backend: DriveBackendSdkClient;
  };
}
