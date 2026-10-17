import type { StorageProviderCapabilities } from './storage-provider-capabilities';

export interface StorageProvidersCapabilitiesListResponse {
  code: 0;
  data: unknown & { item: StorageProviderCapabilities; };
  /** Server-owned request correlation id. */
  traceId: string;
}
