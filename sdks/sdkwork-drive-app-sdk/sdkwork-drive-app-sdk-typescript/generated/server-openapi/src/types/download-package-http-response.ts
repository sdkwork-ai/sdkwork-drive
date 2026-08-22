import type { DownloadPackageResponse } from './download-package-response';

export interface DownloadPackageHttpResponse {
  code: 0;
  data: unknown & DownloadPackageResponse;
  /** Server-owned request correlation id. */
  traceId: string;
}
