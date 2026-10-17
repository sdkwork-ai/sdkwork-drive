import type { ProviderObject } from './provider-object';

export interface StorageProvidersObjectsMultipartUploadCompleteResponse {
  code: 0;
  data: unknown & { item: ProviderObject; };
  /** Server-owned request correlation id. */
  traceId: string;
}
