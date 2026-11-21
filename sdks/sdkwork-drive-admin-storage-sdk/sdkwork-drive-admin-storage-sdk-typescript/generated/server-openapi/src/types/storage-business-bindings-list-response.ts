import type { BusinessStorageBinding } from './business-storage-binding';
import type { PageInfo } from './page-info';

export interface StorageBusinessBindingsListResponse {
  code: 0;
  data: unknown & { items: BusinessStorageBinding[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
