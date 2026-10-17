import type { StorageMigrationRunReport } from './storage-migration-run-report';

export interface StorageMigrationsRunResponse {
  code: 0;
  data: unknown & { item: StorageMigrationRunReport; };
  /** Server-owned request correlation id. */
  traceId: string;
}
