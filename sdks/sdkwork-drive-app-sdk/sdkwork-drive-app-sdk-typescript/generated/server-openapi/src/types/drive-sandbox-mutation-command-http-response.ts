import type { DriveSandboxMutationCommandData } from './drive-sandbox-mutation-command-data';

export interface DriveSandboxMutationCommandHttpResponse {
  code: 0;
  data: unknown & DriveSandboxMutationCommandData;
  /** Server-owned request correlation id. */
  traceId: string;
}
