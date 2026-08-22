import type { CreateDownloadUrlResponse } from './create-download-url-response';

export interface CreateDownloadUrlHttpResponse {
  code: 0;
  data: unknown & CreateDownloadUrlResponse;
  /** Server-owned request correlation id. */
  traceId: string;
}
