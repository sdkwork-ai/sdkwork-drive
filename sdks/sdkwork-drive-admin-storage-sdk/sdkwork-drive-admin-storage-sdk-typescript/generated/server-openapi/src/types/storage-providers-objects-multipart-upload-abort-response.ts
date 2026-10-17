import type { ProviderObjectMutation } from './provider-object-mutation';

export interface StorageProvidersObjectsMultipartUploadAbortResponse {
  code: 0;
  data: unknown & { item: ProviderObjectMutation; };
  /** Server-owned request correlation id. */
  traceId: string;
}
