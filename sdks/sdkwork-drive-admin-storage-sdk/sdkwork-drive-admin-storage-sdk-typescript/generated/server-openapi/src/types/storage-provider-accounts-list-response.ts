import type { PageInfo } from './page-info';
import type { StorageProviderAccount } from './storage-provider-account';

export interface StorageProviderAccountsListResponse {
  code: 0;
  data: unknown & { items: StorageProviderAccount[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
