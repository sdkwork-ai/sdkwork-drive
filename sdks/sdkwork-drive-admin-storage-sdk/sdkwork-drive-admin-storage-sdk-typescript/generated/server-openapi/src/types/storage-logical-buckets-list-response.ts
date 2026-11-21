import type { PageInfo } from './page-info';
import type { StorageLogicalBucket } from './storage-logical-bucket';

export interface StorageLogicalBucketsListResponse {
  code: 0;
  data: unknown & { items: StorageLogicalBucket[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
