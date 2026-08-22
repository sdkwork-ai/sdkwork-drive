import type { PresignedUploadPart } from './presigned-upload-part';

export interface PresignedUploadPartHttpResponse {
  code: 0;
  data: unknown & PresignedUploadPart;
  /** Server-owned request correlation id. */
  traceId: string;
}
