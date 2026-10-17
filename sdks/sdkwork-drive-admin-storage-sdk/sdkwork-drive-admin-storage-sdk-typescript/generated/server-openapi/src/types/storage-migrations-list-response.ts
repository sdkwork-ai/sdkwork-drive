import type { PageInfo } from './page-info';
import type { StorageMigration } from './storage-migration';

export interface StorageMigrationsListResponse {
  code: 0;
  data: unknown & { items: StorageMigration[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
