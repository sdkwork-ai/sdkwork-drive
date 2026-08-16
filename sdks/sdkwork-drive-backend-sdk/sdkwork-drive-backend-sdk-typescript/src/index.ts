import {
  operations,
  sdkMetadata,
} from '../composed/operations';
import {
  createClient as createGeneratedDriveBackendClient,
  SdkworkBackendClient,
} from '../generated/server-openapi/src/index';
import type { SdkworkBackendConfig } from '../generated/server-openapi/src/types/common';

export {
  SdkworkBackendClient,
  createGeneratedDriveBackendClient,
  operations,
  sdkMetadata,
};
export * from '../generated/server-openapi/src/types';
export * from '../generated/server-openapi/src/api';
export * from '../generated/server-openapi/src/http';
export * from '../generated/server-openapi/src/auth';

export interface SdkworkDriveBackendClient extends SdkworkBackendClient {}

export function createDriveBackendClient(
  config: SdkworkBackendConfig,
): SdkworkDriveBackendClient {
  return createGeneratedDriveBackendClient(config) as SdkworkDriveBackendClient;
}

export function createClient(
  config: SdkworkBackendConfig,
): SdkworkDriveBackendClient {
  return createDriveBackendClient(config);
}
