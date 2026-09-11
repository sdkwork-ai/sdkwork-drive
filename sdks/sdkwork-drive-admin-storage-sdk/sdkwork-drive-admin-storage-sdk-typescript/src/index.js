import { operations, sdkMetadata, } from "../composed/operations";
import { createClient as createGeneratedDriveAdminStorageClient, SdkworkCustomClient, } from "../generated/server-openapi/src/index";
export { SdkworkCustomClient, createGeneratedDriveAdminStorageClient, operations, sdkMetadata, };
export * from "../generated/server-openapi/src/types";
export * from "../generated/server-openapi/src/api";
export * from "../generated/server-openapi/src/http";
export * from "../generated/server-openapi/src/auth";
export function createDriveAdminStorageClient(config) {
    return createGeneratedDriveAdminStorageClient(config);
}
export function createClient(config) {
    return createDriveAdminStorageClient(config);
}
//# sourceMappingURL=index.js.map