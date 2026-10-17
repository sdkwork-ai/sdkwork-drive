import type { PageInfo } from './page-info';
import type { StorageProviderBinding } from './storage-provider-binding';

export interface StorageProviderBindingsListResponse {
  code: 0;
  data: unknown & { items: StorageProviderBinding[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
