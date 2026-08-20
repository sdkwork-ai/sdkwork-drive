import type { DriveUploaderBlobLike, DriveUploaderPartPlan } from "./types";
export declare const DEFAULT_UPLOADER_CHUNK_SIZE_BYTES: number;
export declare function planUploaderParts(file: Pick<DriveUploaderBlobLike, "size">, chunkSizeBytes?: number): DriveUploaderPartPlan[];
export declare function inferUploaderContentType(file: DriveUploaderBlobLike, fallback?: string): string;
export declare function inferUploaderFileName(file: DriveUploaderBlobLike): string;
//# sourceMappingURL=uploadPlanner.d.ts.map