import type { DriveSandboxEntryData } from './drive-sandbox-entry-data';

export interface DriveSandboxEntryHttpResponse {
  code: 0;
  data: unknown & DriveSandboxEntryData;
  /** Server-owned request correlation id. */
  traceId: string;
}
