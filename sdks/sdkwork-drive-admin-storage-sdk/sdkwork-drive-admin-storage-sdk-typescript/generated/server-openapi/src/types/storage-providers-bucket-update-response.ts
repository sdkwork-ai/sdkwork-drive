import type { ProviderBucketMutation } from './provider-bucket-mutation';

export interface StorageProvidersBucketUpdateResponse {
  code: 0;
  data: unknown & { item: ProviderBucketMutation; };
  /** Server-owned request correlation id. */
  traceId: string;
}
