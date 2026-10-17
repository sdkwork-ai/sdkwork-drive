import type { PageInfo } from './page-info';
import type { SandboxGrant } from './sandbox-grant';

export interface SandboxGrantListHttpResponse {
  code: 0;
  data: unknown & { items: SandboxGrant[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
