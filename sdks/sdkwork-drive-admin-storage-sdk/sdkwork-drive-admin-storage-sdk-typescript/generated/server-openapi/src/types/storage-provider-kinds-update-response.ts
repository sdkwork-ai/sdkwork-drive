import type { StorageProviderKind } from './storage-provider-kind';

export interface StorageProviderKindsUpdateResponse {
  code: 0;
  data: unknown & { item: StorageProviderKind; };
  /** Server-owned request correlation id. */
  traceId: string;
}
