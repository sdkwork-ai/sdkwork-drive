import type { BusinessStorageBinding } from './business-storage-binding';

export interface StorageBusinessBindingsUpdateResponse {
  code: 0;
  data: unknown & { item: BusinessStorageBinding; };
  /** Server-owned request correlation id. */
  traceId: string;
}
