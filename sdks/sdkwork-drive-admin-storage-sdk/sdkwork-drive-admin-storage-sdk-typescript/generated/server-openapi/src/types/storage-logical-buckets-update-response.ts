import type { StorageLogicalBucket } from './storage-logical-bucket';

export interface StorageLogicalBucketsUpdateResponse {
  code: 0;
  data: unknown & { item: StorageLogicalBucket; };
  /** Server-owned request correlation id. */
  traceId: string;
}
