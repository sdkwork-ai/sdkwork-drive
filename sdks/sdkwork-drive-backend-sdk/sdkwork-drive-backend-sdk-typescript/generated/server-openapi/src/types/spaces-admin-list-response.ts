import type { DriveSpace } from './drive-space';
import type { PageInfo } from './page-info';

export interface SpacesAdminListResponse {
  code: 0;
  data: unknown & unknown & { items: DriveSpace[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
