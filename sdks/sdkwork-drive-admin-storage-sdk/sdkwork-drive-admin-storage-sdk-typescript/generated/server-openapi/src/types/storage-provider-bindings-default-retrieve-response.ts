import type { StorageProviderBinding } from './storage-provider-binding';

export interface StorageProviderBindingsDefaultRetrieveResponse {
  code: 0;
  data: unknown & { item: StorageProviderBinding; };
  /** Server-owned request correlation id. */
  traceId: string;
}
