import type { ProviderObjectMultipartUpload } from './provider-object-multipart-upload';

export interface StorageProvidersObjectsMultipartUploadCreateResponse {
  code: 0;
  data: unknown & { item: ProviderObjectMultipartUpload; };
  /** Server-owned request correlation id. */
  traceId: string;
}
