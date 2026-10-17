import type { PageInfo } from './page-info';
import type { StorageProvider } from './storage-provider';

export interface StorageProvidersListResponse {
  code: 0;
  data: unknown & { items: StorageProvider[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
