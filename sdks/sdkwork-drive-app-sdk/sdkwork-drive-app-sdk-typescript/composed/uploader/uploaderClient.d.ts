import type { DriveUploaderClientOptions, DriveUploaderReplaceNodeContentRequest, DriveUploaderReplaceNodeContentResult, DriveUploaderRequest, DriveUploaderUploadResult, DriveUploaderProfile } from "./types";
export declare class DriveUploaderClient {
    private readonly transport;
    private readonly stateStore;
    private readonly uploadFetch;
    private readonly defaultChunkSizeBytes;
    constructor({ transport, stateStore, uploadFetch, defaultChunkSizeBytes, }: DriveUploaderClientOptions);
    upload(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
    uploadVideo(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
    uploadImage(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
    uploadAudio(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
    uploadDocument(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
    uploadArchive(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
    uploadText(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
    uploadDataset(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
    uploadAttachment(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
    uploadAvatar(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
    uploadThumbnail(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
    uploadByProfile(profile: DriveUploaderProfile, request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
    replaceNodeContent(request: DriveUploaderReplaceNodeContentRequest): Promise<DriveUploaderReplaceNodeContentResult>;
    private normalizeRequest;
    private normalizeReplaceNodeContentRequest;
    private abortUploadSession;
}
export declare function createDriveUploaderClient(options: DriveUploaderClientOptions): DriveUploaderClient;
//# sourceMappingURL=uploaderClient.d.ts.map