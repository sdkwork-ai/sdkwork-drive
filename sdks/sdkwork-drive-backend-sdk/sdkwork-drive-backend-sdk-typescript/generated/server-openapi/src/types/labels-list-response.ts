import type { DriveLabel } from './drive-label';
import type { PageInfo } from './page-info';

export interface LabelsListResponse {
  code: 0;
  data: unknown & unknown & { items: DriveLabel[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
