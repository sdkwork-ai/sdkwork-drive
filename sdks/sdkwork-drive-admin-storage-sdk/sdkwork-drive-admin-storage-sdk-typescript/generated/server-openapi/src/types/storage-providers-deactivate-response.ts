import type { StorageProvider } from './storage-provider';

export interface StorageProvidersDeactivateResponse {
  code: 0;
  data: unknown & { item: StorageProvider; };
  /** Server-owned request correlation id. */
  traceId: string;
}
