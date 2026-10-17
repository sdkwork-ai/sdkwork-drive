import type { DownloadPackage } from './download-package';
import type { PageInfo } from './page-info';

export interface DownloadPackageListHttpResponse {
  code: 0;
  data: unknown & { items: DownloadPackage[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
