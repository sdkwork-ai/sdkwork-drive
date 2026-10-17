import type { PageInfo } from './page-info';
import type { StorageProviderAccountDefault } from './storage-provider-account-default';

export interface StorageProviderAccountDefaultsCreateResponse201 {
  code: 0;
  data: unknown & { items: StorageProviderAccountDefault[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
