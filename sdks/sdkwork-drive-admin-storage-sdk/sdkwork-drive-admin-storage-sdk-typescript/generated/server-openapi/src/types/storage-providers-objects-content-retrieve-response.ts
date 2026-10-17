import type { ProviderObjectContent } from './provider-object-content';

export interface StorageProvidersObjectsContentRetrieveResponse {
  code: 0;
  data: unknown & { item: ProviderObjectContent; };
  /** Server-owned request correlation id. */
  traceId: string;
}
