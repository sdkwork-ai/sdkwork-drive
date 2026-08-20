export class InMemoryUploaderStateStore {
    snapshots = new Map();
    async get(taskId) {
        return this.snapshots.get(taskId);
    }
    async put(snapshot) {
        this.snapshots.set(snapshot.taskId, {
            ...snapshot,
            uploadedParts: snapshot.uploadedParts.map((part) => ({ ...part })),
        });
    }
    async clear(taskId) {
        this.snapshots.delete(taskId);
    }
}
export function createInMemoryUploaderStateStore() {
    return new InMemoryUploaderStateStore();
}
//# sourceMappingURL=uploadStateStore.js.map