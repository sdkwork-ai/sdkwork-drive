import type { PageInfo } from './page-info';
import type { StorageProviderKind } from './storage-provider-kind';

export interface StorageProviderKindsCreateResponse201 {
  code: 0;
  data: unknown & { items: StorageProviderKind[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
