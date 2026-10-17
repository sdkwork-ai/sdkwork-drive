import type { StorageMigration } from './storage-migration';

export interface StorageMigrationsCancelResponse {
  code: 0;
  data: unknown & { item: StorageMigration; };
  /** Server-owned request correlation id. */
  traceId: string;
}
