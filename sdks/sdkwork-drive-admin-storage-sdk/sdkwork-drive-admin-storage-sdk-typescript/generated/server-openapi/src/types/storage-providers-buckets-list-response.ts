import type { PageInfo } from './page-info';
import type { ProviderBucketListItem } from './provider-bucket-list-item';

export interface StorageProvidersBucketsListResponse {
  code: 0;
  data: unknown & { items: ProviderBucketListItem[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
