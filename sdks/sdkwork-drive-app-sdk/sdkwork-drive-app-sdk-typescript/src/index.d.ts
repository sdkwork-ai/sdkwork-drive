import { createClient as createGeneratedDriveAppClient, SdkworkAppClient } from "../generated/server-openapi/src/index";
import type { SdkworkAppConfig } from "../generated/server-openapi/src/types/common";
import { operations, sdkMetadata } from "../composed/operations";
import { type DriveUploaderClient, type DriveUploaderClientOptions, type DriveUploaderTransport } from "../composed/uploader/index";
export { SdkworkAppClient, createGeneratedDriveAppClient, operations, sdkMetadata };
export * from "../generated/server-openapi/src/types";
export * from "../generated/server-openapi/src/api";
export * from "../generated/server-openapi/src/http";
export * from "../generated/server-openapi/src/auth";
export { DriveUploaderClient, createDriveUploaderClient, createInMemoryUploaderStateStore, DEFAULT_UPLOADER_CHUNK_SIZE_BYTES, inferUploaderContentType, inferUploaderFileName, planUploaderParts, } from "../composed/uploader/index";
export type { DriveUploaderBlobLike, DriveUploaderClientOptions, DriveUploaderCompletedPart, DriveUploaderPartPlan, DriveUploaderProfile, DriveUploaderProgress, DriveUploaderReplaceNodeContentRequest, DriveUploaderReplaceNodeContentResult, DriveUploaderRequest, DriveUploaderStateSnapshot, DriveUploaderStateStore, DriveUploaderTransport, DriveUploaderTransportOptions, DriveUploaderUploadResult, } from "../composed/uploader/index";
export interface SdkworkDriveAppClient extends SdkworkAppClient {
    uploader: DriveUploaderClient;
}
export interface DriveAppClientOptions {
    uploader?: Omit<DriveUploaderClientOptions, "transport">;
}
export declare function createDriveUploaderTransport(client: Pick<SdkworkAppClient, "drive">): DriveUploaderTransport;
export declare function attachDriveUploader(client: SdkworkAppClient, options?: DriveAppClientOptions): SdkworkDriveAppClient;
export declare function createDriveAppClient(config: SdkworkAppConfig, options?: DriveAppClientOptions): SdkworkDriveAppClient;
export declare function createClient(config: SdkworkAppConfig, options?: DriveAppClientOptions): SdkworkDriveAppClient;
//# sourceMappingURL=index.d.ts.map