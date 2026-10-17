import type { OpenDownloadUrlResponse } from './open-download-url-response';

export interface OpenShareLinksDownloadUrlsCreateResponse201 {
  code: 0;
  data: unknown & unknown & { item: OpenDownloadUrlResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
