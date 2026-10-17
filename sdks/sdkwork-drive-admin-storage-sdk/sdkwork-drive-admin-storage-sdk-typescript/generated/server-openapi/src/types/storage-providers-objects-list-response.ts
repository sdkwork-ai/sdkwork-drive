import type { PageInfo } from './page-info';
import type { ProviderObject } from './provider-object';

export interface StorageProvidersObjectsListResponse {
  code: 0;
  data: unknown & { items: ProviderObject[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
