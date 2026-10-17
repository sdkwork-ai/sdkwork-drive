import type { MaintenanceJob } from './maintenance-job';
import type { PageInfo } from './page-info';

export interface MaintenanceJobsListResponse {
  code: 0;
  data: unknown & unknown & { items: MaintenanceJob[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
