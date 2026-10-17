import type { ProviderBucket } from './provider-bucket';

export interface StorageProvidersBucketRetrieveResponse {
  code: 0;
  data: unknown & { item: ProviderBucket; };
  /** Server-owned request correlation id. */
  traceId: string;
}
