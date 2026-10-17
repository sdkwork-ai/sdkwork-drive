import type { ProviderObjectUploadPartGrants } from './provider-object-upload-part-grants';

export interface StorageProvidersObjectsMultipartUploadPartsPresignResponse {
  code: 0;
  data: unknown & { item: ProviderObjectUploadPartGrants; };
  /** Server-owned request correlation id. */
  traceId: string;
}
