import type { StorageMigration } from './storage-migration';

export interface StorageMigrationRunReport {
  migration: StorageMigration;
  copiedThisBatch: string;
  failedThisBatch: string;
  /** True once the run reached `succeeded` on this call. */
  completed: boolean;
}
