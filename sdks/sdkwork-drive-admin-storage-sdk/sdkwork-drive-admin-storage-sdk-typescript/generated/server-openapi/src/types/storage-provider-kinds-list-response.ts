import type { PageInfo } from './page-info';
import type { StorageProviderKind } from './storage-provider-kind';

export interface StorageProviderKindsListResponse {
  code: 0;
  data: unknown & { items: StorageProviderKind[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
