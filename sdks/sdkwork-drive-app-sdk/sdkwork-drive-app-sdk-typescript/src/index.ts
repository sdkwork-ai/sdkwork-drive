import type {
  DriveUploadSession,
  PrepareUploaderUploadRequest,
  PrepareUploaderUploadResponse,
  PresignedUploadPart,
  PresignUploadPartRequest,
  UploadSessionMutationResponse,
  UploaderUploadPart,
} from "../generated/server-openapi/src/types";
import {
  createClient as createGeneratedDriveAppClient,
  SdkworkAppClient,
} from "../generated/server-openapi/src/index";
import type { SdkworkAppConfig } from "../generated/server-openapi/src/types/common";
import {
  operations,
  sdkMetadata,
} from "../composed/operations";
import {
  createDriveUploaderClient,
  type DriveUploaderClient,
  type DriveUploaderClientOptions,
  type DriveUploaderTransport,
} from "../composed/uploader/index";

export { SdkworkAppClient, createGeneratedDriveAppClient, operations, sdkMetadata };
export * from "../generated/server-openapi/src/types";
export * from "../generated/server-openapi/src/api";
export * from "../generated/server-openapi/src/http";
export * from "../generated/server-openapi/src/auth";
export {
  DriveUploaderClient,
  createDriveUploaderClient,
  createInMemoryUploaderStateStore,
  DEFAULT_UPLOADER_CHUNK_SIZE_BYTES,
  inferUploaderContentType,
  inferUploaderFileName,
  planUploaderParts,
} from "../composed/uploader/index";
export type {
  DriveUploaderBlobLike,
  DriveUploaderClientOptions,
  DriveUploaderCompletedPart,
  DriveUploaderPartPlan,
  DriveUploaderProfile,
  DriveUploaderProgress,
  DriveUploaderReplaceNodeContentRequest,
  DriveUploaderReplaceNodeContentResult,
  DriveUploaderRequest,
  DriveUploaderStateSnapshot,
  DriveUploaderStateStore,
  DriveUploaderTransport,
  DriveUploaderTransportOptions,
  DriveUploaderUploadResult,
} from "../composed/uploader/index";

export interface SdkworkDriveAppClient extends SdkworkAppClient {
  uploader: DriveUploaderClient;
}

export interface DriveAppClientOptions {
  uploader?: Omit<DriveUploaderClientOptions, "transport">;
}

function typedSdkResponse<T>(response: Promise<unknown>): Promise<T> {
  return response as Promise<T>;
}

export function createDriveUploaderTransport(
  client: Pick<SdkworkAppClient, "drive">,
): DriveUploaderTransport {
  return {
    drive: {
      uploader: {
        uploads: {
          create: (body) =>
            typedSdkResponse<PrepareUploaderUploadResponse>(
              client.drive.uploader.uploads.create(body as PrepareUploaderUploadRequest),
            ),
          parts: {
            update: (uploadItemId, partNo, body) =>
              typedSdkResponse<UploaderUploadPart>(
                client.drive.uploader.uploads.parts.update(uploadItemId, partNo, body),
              ),
          },
        },
      },
      uploadSessions: {
        create: (body) =>
          typedSdkResponse<DriveUploadSession>(client.drive.uploadSessions.create(body)),
        parts: {
          update: (uploadSessionId, partNo, body) =>
            typedSdkResponse<PresignedUploadPart>(
              client.drive.uploadSessions.parts.update(
                uploadSessionId,
                partNo,
                body as PresignUploadPartRequest,
              ),
            ),
        },
        complete: (uploadSessionId, body) =>
          typedSdkResponse<UploadSessionMutationResponse>(
            client.drive.uploadSessions.complete(uploadSessionId, body),
          ),
        abort: (uploadSessionId, body) =>
          typedSdkResponse<UploadSessionMutationResponse>(
            client.drive.uploadSessions.abort(uploadSessionId, body),
          ),
      },
    },
  };
}

export function attachDriveUploader(
  client: SdkworkAppClient,
  options: DriveAppClientOptions = {},
): SdkworkDriveAppClient {
  const driveClient = client as SdkworkDriveAppClient;
  driveClient.uploader = createDriveUploaderClient({
    ...(options.uploader ?? {}),
    transport: createDriveUploaderTransport(client),
  });
  return driveClient;
}

export function createDriveAppClient(
  config: SdkworkAppConfig,
  options: DriveAppClientOptions = {},
): SdkworkDriveAppClient {
  return attachDriveUploader(createGeneratedDriveAppClient(config), options);
}

export function createClient(
  config: SdkworkAppConfig,
  options: DriveAppClientOptions = {},
): SdkworkDriveAppClient {
  return createDriveAppClient(config, options);
}
