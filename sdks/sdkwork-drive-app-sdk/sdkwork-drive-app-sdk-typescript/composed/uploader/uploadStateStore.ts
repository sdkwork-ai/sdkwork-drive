import type { DriveUploaderStateSnapshot, DriveUploaderStateStore } from "./types";

export class InMemoryUploaderStateStore implements DriveUploaderStateStore {
  private readonly snapshots = new Map<string, DriveUploaderStateSnapshot>();

  async get(taskId: string): Promise<DriveUploaderStateSnapshot | undefined> {
    return this.snapshots.get(taskId);
  }

  async put(snapshot: DriveUploaderStateSnapshot): Promise<void> {
    this.snapshots.set(snapshot.taskId, {
      ...snapshot,
      uploadedParts: snapshot.uploadedParts.map((part) => ({ ...part })),
    });
  }

  async clear(taskId: string): Promise<void> {
    this.snapshots.delete(taskId);
  }
}

export function createInMemoryUploaderStateStore(): DriveUploaderStateStore {
  return new InMemoryUploaderStateStore();
}


