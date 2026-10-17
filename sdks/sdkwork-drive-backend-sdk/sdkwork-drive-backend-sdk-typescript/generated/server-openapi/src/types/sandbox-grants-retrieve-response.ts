import type { SandboxGrant } from './sandbox-grant';

export interface SandboxGrantsRetrieveResponse {
  code: 0;
  data: unknown & unknown & { item: SandboxGrant; };
  /** Server-owned request correlation id. */
  traceId: string;
}
