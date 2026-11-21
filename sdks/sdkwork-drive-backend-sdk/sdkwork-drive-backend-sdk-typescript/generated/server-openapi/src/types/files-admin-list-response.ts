import type { DriveFile } from './drive-file';
import type { PageInfo } from './page-info';

export interface FilesAdminListResponse {
  code: 0;
  data: unknown & unknown & { items: DriveFile[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
