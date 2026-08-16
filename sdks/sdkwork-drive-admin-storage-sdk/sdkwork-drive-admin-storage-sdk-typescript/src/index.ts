import {
  operations,
  sdkMetadata,
} from "../composed/operations";
import {
  createClient as createGeneratedDriveAdminStorageClient,
  SdkworkCustomClient,
} from "../generated/server-openapi/src/index";
import type { SdkworkCustomConfig } from "../generated/server-openapi/src/types/common";

export {
  SdkworkCustomClient,
  createGeneratedDriveAdminStorageClient,
  operations,
  sdkMetadata,
};
export * from "../generated/server-openapi/src/types";
export * from "../generated/server-openapi/src/api";
export * from "../generated/server-openapi/src/http";
export * from "../generated/server-openapi/src/auth";

export interface SdkworkDriveAdminStorageClient extends SdkworkCustomClient {}

export function createDriveAdminStorageClient(
  config: SdkworkCustomConfig,
): SdkworkDriveAdminStorageClient {
  return createGeneratedDriveAdminStorageClient(config) as SdkworkDriveAdminStorageClient;
}

export function createClient(
  config: SdkworkCustomConfig,
): SdkworkDriveAdminStorageClient {
  return createDriveAdminStorageClient(config);
}
