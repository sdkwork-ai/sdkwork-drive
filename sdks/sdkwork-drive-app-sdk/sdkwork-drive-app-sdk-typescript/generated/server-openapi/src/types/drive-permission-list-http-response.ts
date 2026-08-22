import type { DrivePermission } from './drive-permission';
import type { PageInfo } from './page-info';

export interface DrivePermissionListHttpResponse {
  code: 0;
  data: unknown & { items: DrivePermission[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
