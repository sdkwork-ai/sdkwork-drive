import type { UploaderUploadPart } from './uploader-upload-part';

export interface UploaderUploadPartHttpResponse {
  code: 0;
  data: unknown & { item: UploaderUploadPart; };
  /** Server-owned request correlation id. */
  traceId: string;
}
