import type { SandboxGrant } from './sandbox-grant';

export interface SandboxGrantResourceHttpResponse {
  code: 0;
  data: unknown & { item: SandboxGrant; };
  /** Server-owned request correlation id. */
  traceId: string;
}
