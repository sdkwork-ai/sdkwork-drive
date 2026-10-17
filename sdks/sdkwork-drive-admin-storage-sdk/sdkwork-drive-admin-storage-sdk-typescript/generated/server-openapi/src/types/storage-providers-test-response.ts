import type { TestStorageProviderResponse } from './test-storage-provider-response';

export interface StorageProvidersTestResponse {
  code: 0;
  data: unknown & { item: TestStorageProviderResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
