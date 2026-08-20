import { createClient as createGeneratedDriveAppClient, SdkworkAppClient, } from "../generated/server-openapi/src/index";
import { operations, sdkMetadata, } from "../composed/operations";
import { createDriveUploaderClient, } from "../composed/uploader/index";
export { SdkworkAppClient, createGeneratedDriveAppClient, operations, sdkMetadata };
export * from "../generated/server-openapi/src/types";
export * from "../generated/server-openapi/src/api";
export * from "../generated/server-openapi/src/http";
export * from "../generated/server-openapi/src/auth";
export { DriveUploaderClient, createDriveUploaderClient, createInMemoryUploaderStateStore, DEFAULT_UPLOADER_CHUNK_SIZE_BYTES, inferUploaderContentType, inferUploaderFileName, planUploaderParts, } from "../composed/uploader/index";
function typedSdkResponse(response) {
    return response;
}
export function createDriveUploaderTransport(client) {
    return {
        drive: {
            uploader: {
                uploads: {
                    create: (body) => typedSdkResponse(client.drive.uploader.uploads.create(body)),
                    parts: {
                        update: (uploadItemId, partNo, body) => typedSdkResponse(client.drive.uploader.uploads.parts.update(uploadItemId, partNo, body)),
                    },
                },
            },
            uploadSessions: {
                create: (body) => typedSdkResponse(client.drive.uploadSessions.create(body)),
                parts: {
                    update: (uploadSessionId, partNo, body) => typedSdkResponse(client.drive.uploadSessions.parts.update(uploadSessionId, partNo, body)),
                },
                complete: (uploadSessionId, body) => typedSdkResponse(client.drive.uploadSessions.complete(uploadSessionId, body)),
                abort: (uploadSessionId, body) => typedSdkResponse(client.drive.uploadSessions.abort(uploadSessionId, body)),
            },
        },
    };
}
export function attachDriveUploader(client, options = {}) {
    const driveClient = client;
    driveClient.uploader = createDriveUploaderClient({
        ...(options.uploader ?? {}),
        transport: createDriveUploaderTransport(client),
    });
    return driveClient;
}
export function createDriveAppClient(config, options = {}) {
    return attachDriveUploader(createGeneratedDriveAppClient(config), options);
}
export function createClient(config, options = {}) {
    return createDriveAppClient(config, options);
}
//# sourceMappingURL=index.js.map