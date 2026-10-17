import type { PageInfo } from './page-info';
import type { SandboxGrant } from './sandbox-grant';

export interface SandboxGrantsListResponse {
  code: 0;
  data: unknown & unknown & { items: SandboxGrant[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
