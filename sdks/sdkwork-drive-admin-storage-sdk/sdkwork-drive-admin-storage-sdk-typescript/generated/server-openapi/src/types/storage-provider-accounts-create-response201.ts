import type { StorageProviderAccount } from './storage-provider-account';

export interface StorageProviderAccountsCreateResponse201 {
  code: 0;
  data: unknown & { item: StorageProviderAccount; };
  /** Server-owned request correlation id. */
  traceId: string;
}
