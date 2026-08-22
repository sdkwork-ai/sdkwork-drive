import type { PrepareUploaderUploadResponse } from './prepare-uploader-upload-response';

export interface PrepareUploaderUploadHttpResponse {
  code: 0;
  data: unknown & PrepareUploaderUploadResponse;
  /** Server-owned request correlation id. */
  traceId: string;
}
