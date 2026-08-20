import type { DriveUploaderStateSnapshot, DriveUploaderStateStore } from "./types";
export declare class InMemoryUploaderStateStore implements DriveUploaderStateStore {
    private readonly snapshots;
    get(taskId: string): Promise<DriveUploaderStateSnapshot | undefined>;
    put(snapshot: DriveUploaderStateSnapshot): Promise<void>;
    clear(taskId: string): Promise<void>;
}
export declare function createInMemoryUploaderStateStore(): DriveUploaderStateStore;
//# sourceMappingURL=uploadStateStore.d.ts.map