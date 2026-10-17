import type { DownloadPackage } from './download-package';
import type { PageInfo } from './page-info';

export interface DownloadPackagesListResponse {
  code: 0;
  data: unknown & unknown & { items: DownloadPackage[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
