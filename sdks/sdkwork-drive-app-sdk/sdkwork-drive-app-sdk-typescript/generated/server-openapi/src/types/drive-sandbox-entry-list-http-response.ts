import type { DriveSandboxEntryListData } from './drive-sandbox-entry-list-data';

export interface DriveSandboxEntryListHttpResponse {
  code: 0;
  data: unknown & DriveSandboxEntryListData;
  /** Server-owned request correlation id. */
  traceId: string;
}
