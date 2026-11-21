import type { StorageProviderInitialization } from './storage-provider-initialization';

export interface StorageProvidersInitializeResponse {
  code: 0;
  data: unknown & { item: StorageProviderInitialization; };
  /** Server-owned request correlation id. */
  traceId: string;
}
