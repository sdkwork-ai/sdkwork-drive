export interface StorageMigration {
  id: string;
  name: string;
  sourceProviderId: string;
  targetProviderId: string;
  targetBucket?: string | null;
  status: 'pending' | 'running' | 'succeeded' | 'failed' | 'cancelled';
  /** When true, every migrated object is re-pointed at the target provider once the run completes without failures. When false the bytes land on the target but the registry keeps reading from the source. */
  applyBindingSwitch: boolean;
  /** Objects queued when the run was seeded. */
  objectsTotal: string;
  /** Objects copied and whose checksum was verified. */
  objectsCopied: string;
  /** Objects that failed verification. */
  objectsFailed: string;
  /** Objects still pending, derived from the queue. */
  objectsOutstanding: string;
  bytesCopied: string;
  progressRatio: number;
  failureMessage?: string | null;
  createdBy: string;
  updatedBy: string;
}
