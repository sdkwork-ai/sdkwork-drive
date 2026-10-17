import type { SandboxGrant } from './sandbox-grant';

export interface SandboxGrantsCreateResponse201 {
  code: 0;
  data: unknown & unknown & { item: SandboxGrant; };
  /** Server-owned request correlation id. */
  traceId: string;
}
