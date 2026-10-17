import type { ProviderObject } from './provider-object';

export interface StorageProvidersObjectsRetrieveResponse {
  code: 0;
  data: unknown & { item: ProviderObject; };
  /** Server-owned request correlation id. */
  traceId: string;
}
