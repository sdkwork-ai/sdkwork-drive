import type { OpenDownloadUrlResponse } from './open-download-url-response';

export interface OpenDownloadUrlCreateResponse {
  code: 0;
  data: unknown & { item: OpenDownloadUrlResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
