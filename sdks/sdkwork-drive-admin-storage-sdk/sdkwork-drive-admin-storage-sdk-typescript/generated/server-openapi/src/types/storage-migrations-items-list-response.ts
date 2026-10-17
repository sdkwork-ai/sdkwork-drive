import type { PageInfo } from './page-info';
import type { StorageMigrationItem } from './storage-migration-item';

export interface StorageMigrationsItemsListResponse {
  code: 0;
  data: unknown & { items: StorageMigrationItem[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
